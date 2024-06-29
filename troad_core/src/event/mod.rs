use std::{
    io,
    net::SocketAddr,
    sync::{Arc, RwLock},
};

use aes::{
    cipher::{
        generic_array::GenericArray, AsyncStreamCipher, BlockCipher, BlockDecrypt, BlockEncrypt,
        BlockEncryptMut, KeyInit, KeyIvInit,
    },
    Aes128,
};
use bevy_ecs::{component::Component, entity::Entity, world::World};
use reqwest::StatusCode;
use rsa::{
    pkcs1::EncodeRsaPublicKey,
    pkcs1v15::{self, Pkcs1v15Encrypt},
    pkcs8::{der::Encode, LineEnding, SubjectPublicKeyInfo},
    traits::PaddingScheme,
    RsaPrivateKey, RsaPublicKey,
};
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use tokio::{io::AsyncWriteExt, net::TcpStream};
use troad_serde::{
    de::from_slice,
    ser::{to_vec, to_vec_with_size},
};
use uuid::Uuid;

use crate::{
    protocol::packets::{
        ClientSettings, Description, Handshake, Header, HeldItemChange, JoinGame, LoginStart,
        LoginSuccess, Packet, Ping, PlayerAbilities, PlayerPos, PlayerPosLook, PlayersStatus,
        PluginMessage, ServerDifficulty, SpawnPosition, Status, StringPck, TimeUpdate, VersionInfo,
    },
    server::{Cipher, PeersMap, TcpProtocolExt},
};

pub mod connected;
pub mod login;
pub mod status;

pub enum Event {
    Handshake,
    Ping,
}

// State::Connected
pub const HANDSHAKE: i32 = 0;
// State::Status
pub const STATUS: i32 = 0;
pub const PING_PONG: i32 = 1;
// State::Login
pub const LOGIN_START: i32 = 0;
pub const JOIN_GAME: i32 = 0x01;
pub const LOGIN_SUCCESS: i32 = 0x2;
pub const PLAYER_POSITION_AND_LOOK: i32 = 0x08;
pub const PLUGIN_MESSAGE_CLIENT_BOUND: i32 = 0x3F;
pub const ENCRYPTION_RESPONSE: i32 = 0x01;
pub const ENCRYPTION_REQUEST: i32 = 0x01;

#[repr(i32)]
pub enum State {
    Handshaking = -1,
    Play = 0,
    Status = 1,
    Login = 2,
}

impl From<i32> for State {
    fn from(value: i32) -> Self {
        match value {
            0 => Self::Play,
            1 => Self::Status,
            2 => Self::Login,
            _ => Self::Handshaking,
        }
    }
}

pub struct EventContext<'a> {
    pub peers: &'a PeersMap,

    pub state: &'a mut State,
    pub stream: &'a mut TcpStream,

    pub buf: &'a [u8],
    pub header: Header,

    pub world: Arc<RwLock<World>>,
    pub entity: &'a mut Option<Entity>,
    pub addr: &'a SocketAddr,

    pub encryption: &'a mut Encryption,
    pub verify_token: u128,
    pub shared_secret: &'a mut Vec<u8>,
    pub cipher: &'a mut Option<Cipher>,
}

pub struct Encryption {
    pub pub_key: RsaPublicKey,
    pub priv_key: RsaPrivateKey,
}

#[derive(Component, Clone)]
pub struct PlayerName {
    name: String,
    uuid: Uuid,
}

pub type Aes128Cfb8Enc = cfb8::Encryptor<aes::Aes128>;
pub type Aes128Cfb8Dec = cfb8::Decryptor<aes::Aes128>;

fn test() {
    let key = [0x42; 16];
    let iv = [0x24; 16];
    let plaintext = *b"hello world! this is my plaintext.";
    let ciphertext = "33b356ce9184290c4c8facc1c0b1f918d5475aeb75b88c161ca65bdf05c7137ff4b0";

    // encrypt/decrypt in-place
    let mut buf = (plaintext.to_vec());
    Aes128Cfb8Enc::new(&key.into(), &iv.into()).encrypt(&mut buf);
    println!("{:02x?}", buf);
    Aes128Cfb8Dec::new(&key.into(), &iv.into()).decrypt(&mut buf);
    println!("{}", String::from_utf8(buf).unwrap());

    // Initialize cipher
    let key = GenericArray::from([0u8; 16]);
    let mut block = GenericArray::from([42u8; 16]);
    let cipher = Aes128::new(&key);

    let block_copy = block.clone();

    // Encrypt block in-place
    cipher.encrypt_block(&mut block);

    // And decrypt it back
    cipher.decrypt_block(&mut block);
    assert_eq!(block, block_copy);

    // Implementation supports parallel block processing. Number of blocks
    // processed in parallel depends in general on hardware capabilities.
    // This is achieved by instruction-level parallelism (ILP) on a single
    // CPU core, which is differen from multi-threaded parallelism.
    let mut blocks = [block; 100];
    cipher.encrypt_blocks(&mut blocks);

    for block in blocks.iter_mut() {
        cipher.decrypt_block(block);
        assert_eq!(block, &block_copy);
    }

    // `decrypt_blocks` also supports parallel block processing.
    cipher.decrypt_blocks(&mut blocks);

    for block in blocks.iter_mut() {
        cipher.encrypt_block(block);
        assert_eq!(block, &block_copy);
    }
}

pub fn calc_hash(name: &str) -> String {
    let hash: [u8; 20] = Sha1::new().chain_update(name).finalize().into();
    hex_digest(hash)
}

pub fn hex_digest(mut hash: [u8; 20]) -> String {
    let negative = (hash[0] & 0x80) == 0x80;

    // Digest is 20 bytes, so 40 hex digits plus the minus sign if necessary.
    let mut hex = String::with_capacity(40 + negative as usize);
    if negative {
        hex.push('-');

        // two's complement
        let mut carry = true;
        for b in hash.iter_mut().rev() {
            (*b, carry) = (!*b).overflowing_add(carry as u8);
        }
    }
    hex.extend(
        hash.into_iter()
            // extract hex digits
            .flat_map(|x| [x >> 4, x & 0xf])
            // skip leading zeroes
            .skip_while(|&x| x == 0)
            .map(|x| char::from_digit(x as u32, 16).expect("x is always valid base16")),
    );
    hex
}

pub async fn handle_event<'a>(context: EventContext<'a>) -> Result<(), io::Error> {
    // println!("{:02x}", *context.header.id);

    match context.state {
        State::Handshaking => match *context.header.id {
            HANDSHAKE => {
                let handshake = from_slice::<Handshake>(context.buf)?.1;
                println!("{handshake:?}");

                *context.state = (*handshake.next_state).into();
            }
            _ => (),
        },

        State::Status => match *context.header.id {
            STATUS => {
                let status = Status {
                    version: VersionInfo {
                        name: String::from("Troad 1.8.x"),
                        protocol: 47,
                    },
                    players: PlayersStatus { max: 1, online: 0 },
                    description: Description {
                        text: String::from("unfortunate"),
                    },
                };

                let status = serde_json::to_string(&status)?;
                context.stream.send(STATUS, &status).await?;
            }

            PING_PONG => {
                let ping = from_slice::<Ping>(context.buf)?.1;
                context.stream.send(PING_PONG, &ping).await?;
            }
            _ => (),
        },

        State::Login => {
            match *context.header.id {
                LOGIN_START => {
                    let info = from_slice::<LoginStart>(context.buf)?.1;
                    let entity_id = {
                        let mut entity_world = context.world.write().unwrap();
                        let entity_world = entity_world.spawn(PlayerName {
                            name: info.name,
                            uuid: Uuid::nil(),
                        });
                        entity_world.id()
                    };

                    *context.entity = Some(entity_id);

                    // #[derive(Serialize, Debug)]
                    // pub struct EncryptionRequest {
                    //     server_id: String,
                    //     public_key_length: vsize,
                    //     public_key: Vec<u8>,
                    //     verify_token_length: vsize,
                    //     verify_token: Vec<u8>,
                    // }

                    // let mut pub_key = vec![];
                    // SubjectPublicKeyInfo::from_key(context.encryption.pub_key.clone())
                    //     .unwrap()
                    //     .encode_to_vec(&mut pub_key)
                    //     .unwrap();

                    // let p = EncryptionRequest {
                    //     server_id: String::default(),
                    //     public_key_length: VarInt::<usize>(pub_key.len()),
                    //     public_key: pub_key.into(),
                    //     verify_token_length: VarInt::<usize>(16),
                    //     verify_token: context.verify_token.to_be_bytes().into(),
                    // };
                    // context.stream.send(ENCRYPTION_REQUEST, &p).await?;

                    // println!("{p:02x?}");
                }

                ENCRYPTION_RESPONSE => {
                    // #[derive(Deserialize, Debug)]
                    // pub struct EncryptionResponse {
                    //     shared_secret: SizedVec,
                    //     verify_token: SizedVec,
                    // }

                    // let response = deserialize_from_slice::<EncryptionResponse>(context.buf)?.1;
                    // println!("{response:?}");

                    // let ss = response.shared_secret.1;
                    // let vt = response.verify_token.1;
                    // let ss = context
                    //     .encryption
                    //     .priv_key
                    //     .decrypt(Pkcs1v15Encrypt, &ss[..])
                    //     .unwrap();
                    // let vt = context
                    //     .encryption
                    //     .priv_key
                    //     .decrypt(Pkcs1v15Encrypt, &vt[..])
                    //     .unwrap();

                    // if vt == context.verify_token.to_be_bytes() {
                    //     println!("They're same!");
                    //     *context.cipher = Some(Cipher::new(&ss));
                    // }

                    // let player = {
                    //     let world = context.world.read().unwrap();
                    //     world
                    //         .get::<PlayerName>(context.entity.unwrap())
                    //         .unwrap()
                    //         .clone()
                    // };

                    // let mut pub_key = vec![];
                    // SubjectPublicKeyInfo::from_key(context.encryption.pub_key.clone())
                    //     .unwrap()
                    //     .encode_to_vec(&mut pub_key)
                    //     .unwrap();

                    // let result = calc_hash(&player.name);
                    // println!("{} {} {}", result, calc_hash("jeb_"), calc_hash("Joilpa"));
                    // println!("{}", player.name);

                    // *context.shared_secret = ss.clone();

                    // let hash: [u8; 20] = Sha1::new()
                    //     .chain_update(&ss)
                    //     .chain_update(pub_key)
                    //     .finalize()
                    //     .into();
                    // let hash = hex_digest(hash);

                    // let resp = reqwest::get(format!("https://sessionserver.mojang.com/session/minecraft/hasJoined?username={}&serverId={}", player.name.to_ascii_lowercase(), hash)).await.unwrap();
                    // if resp.status() != StatusCode::OK {
                    //     // TODO: real dc
                    //     context.stream.shutdown().await?;
                    //     return Ok(());
                    // }

                    // #[derive(Deserialize)]
                    // pub struct Response {
                    //     id: Uuid,
                    // }

                    // let resp =
                    //     serde_json::from_str::<Response>(&resp.text().await.unwrap()).unwrap();
                    // println!("{} {}", resp.id.to_string(), player.name);

                    // println!("{} SHARED SECRET IS {:02x?}", resp.id.to_string(), ss);

                    // if let Some(cipher) = context.cipher {
                    //     context
                    //         .stream
                    //         .send_enc(
                    //             cipher,
                    //             LOGIN_SUCCESS,
                    //             &LoginSuccess {
                    //                 uuid: resp.id.to_string(),
                    //                 name: player.name,
                    //             },
                    //         )
                    //         .await?;

                    //     *context.state = State::Play;

                    //     // context.stream.shutdown().await?;
                    //     let entity_id = context.entity.unwrap().index() as i32;

                    //     // return Ok(());

                    //     context
                    //         .stream
                    //         .send_enc(
                    //             cipher,
                    //             PLUGIN_MESSAGE_CLIENT_BOUND,
                    //             &PluginMessage {
                    //                 channel: String::from("MC|Brand"),
                    //                 data: String::from("troad"),
                    //             },
                    //         )
                    //         .await?;

                    //     context
                    //         .stream
                    //         .send_enc(
                    //             cipher,
                    //             JOIN_GAME,
                    //             &JoinGame {
                    //                 entity_id,
                    //                 game_mode: 1,
                    //                 dimension: 0,
                    //                 difficulty: 2,
                    //                 max_players: 10,
                    //                 level_type: String::from("flat"),
                    //                 reduced_debug_info: false,
                    //             },
                    //         )
                    //         .await?;

                    //     context
                    //         .stream
                    //         .send_enc(
                    //             cipher,
                    //             PLUGIN_MESSAGE_CLIENT_BOUND,
                    //             &PluginMessage {
                    //                 channel: String::from("MC|Brand"),
                    //                 data: b"troad",
                    //             },
                    //         )
                    //         .await?;

                    //     context
                    //         .stream
                    //         .send_enc(
                    //             cipher,
                    //             PLAYER_POSITION_AND_LOOK,
                    //             &PlayerPosLook {
                    //                 x: 0.0,
                    //                 y: 65.0,
                    //                 z: 0.0,
                    //                 yaw: 0.0,
                    //                 pitch: 0.0,
                    //                 on_ground: false,
                    //             },
                    //         )
                    //         .await?;

                    //     let n = 16;
                    //     let total = n * 8192 + n * 2048 + n * 2048 + 256;

                    //     let s = context.stream;
                    //     let mut data = vec![0; total as usize];
                    //     #[derive(Serialize, Default)]
                    //     pub struct ChunkBulk {
                    //         skylight_sent: bool,
                    //         chunk_col_count: vsize,
                    //         metas: Vec<ChunkMeta>,
                    //         datas: Vec<Vec<u8>>,
                    //     }

                    //     #[derive(Serialize)]
                    //     pub struct ChunkMeta {
                    //         x: i32,
                    //         z: i32,
                    //         prim_bit_mask: u16,
                    //     }

                    //     let y = 63;

                    //     for y in 0..=63 {
                    //         for x in 0..=15 {
                    //             for z in 0..=15 {
                    //                 let i = y << 8 | z << 4 | x;
                    //                 let t = 35; // 35;
                    //                 let d = x + z;

                    //                 // data[2 * i] = (t & 0xFF) as u8;
                    //                 // data[2 * i + 1] = (t >> 8) as u8;
                    //                 data[2 * i] = ((t << 4) | d) as u8;
                    //                 data[2 * i + 1] = (t >> 4) as u8;
                    //             }
                    //         }
                    //     }

                    //     for i in 0..n * 2048 {
                    //         data[(n * 8192 + i) as usize] = 0xFF;
                    //     }

                    //     for i in 0..n * 2048 {
                    //         data[(n * 8192 + n * 2048 + i) as usize] = 0xFF;
                    //     }

                    //     let mut bulk = ChunkBulk {
                    //         skylight_sent: true,
                    //         chunk_col_count: VarInt::<usize>(9),
                    //         metas: Vec::new(),
                    //         datas: Vec::new(),
                    //     };
                    //     for x in -1..=1 {
                    //         for z in -1..=1 {
                    //             bulk.metas.push(ChunkMeta {
                    //                 x,
                    //                 z,
                    //                 prim_bit_mask: u16::MAX,
                    //             });
                    //             bulk.datas.push(data.clone());
                    //         }
                    //     }
                    //     s.send_enc(cipher, 0x26, &bulk).await?;

                    // s.send_enc(
                    //     &ss,
                    //     0x39,
                    //     &PlayerAbilities {
                    //         flags: 0x08 | 0x02,
                    //         flying_speed: 0.5,
                    //         fov_modifier: 1.0,
                    //     },
                    // )
                    // .await?;

                    // let mut thyself = None;
                    // let players = {
                    //     let mut vec = Vec::new();
                    //     let world = context.world.read().unwrap();
                    //     for ent in world.iter_entities() {
                    //         let player_info = ent.get::<PlayerName>().unwrap();
                    //         if ent.id() == context.entity.unwrap() {
                    //             thyself = Some(player_info.clone());
                    //         }

                    //         vec.push((ent.id(), player_info.clone()));
                    //     }
                    //     vec
                    // };

                    // #[derive(Serialize)]
                    // pub struct PlayerListItem {
                    //     id: v32,
                    //     action: v32,
                    //     num_players: vsize,
                    //     players: Vec<Player>,
                    // }

                    // #[derive(Serialize, Clone)]
                    // pub struct Player {
                    //     uuid: Uuid,

                    //     // for 0, add -> player
                    //     name: String,
                    //     num_of_props: vsize,
                    //     properties: Vec<Property>,
                    //     game_mode: v32,
                    //     ping: v32,
                    //     has_display_name: bool,
                    //     display_name: Option<String>,
                    // }

                    // #[derive(Serialize, Clone)]
                    // pub struct Property {
                    //     name: String,
                    //     value: String,
                    //     is_signed: bool,
                    //     signature: String,
                    // }

                    // let mut pli = PlayerListItem {
                    //     id: VarInt::<i32>(0x38),
                    //     action: VarInt::<i32>(0),
                    //     num_players: VarInt::<usize>(players.len()),
                    //     players: Vec::new(),
                    // };

                    // for player in players {
                    //     pli.players.push(Player {
                    //         uuid: player.1.uuid,
                    //         name: player.1.name,
                    //         num_of_props: VarInt::<usize>(0),
                    //         properties: Vec::new(),
                    //         game_mode: VarInt::<i32>(0),
                    //         ping: VarInt::<i32>(0),
                    //         has_display_name: false,
                    //         display_name: None,
                    //     });
                    // }

                    // let packet = serialize_with_size(&pli)?;
                    // let packet: Arc<[u8]> = (&packet[..]).into();
                    // println!("{:02x?}", packet);

                    // s.write_all(&packet.clone()).await.unwrap();
                    // let thyself = thyself.unwrap();

                    // pli.num_players = VarInt::<usize>(1);
                    // let mut ply = None;
                    // for p in pli.players {
                    //     if p.uuid == thyself.uuid {
                    //         ply = Some(p.clone());
                    //     }
                    // }
                    // let ply = ply.unwrap();
                    // pli.players = vec![ply];

                    // let packet = serialize_with_size(&pli)?;
                    // let packet: Arc<[u8]> = (&packet[..]).into();

                    // for peer in context.peers.iter() {
                    //     if peer.key() != context.addr {
                    //         peer.send(packet.clone()).await.unwrap();
                    //     }
                    // }

                    // #[derive(Serialize, Debug)]
                    // pub struct SpawnPlayer {
                    //     id: v32,
                    //     eid: v32,
                    //     uuid: Uuid,
                    //     x: i32,
                    //     y: i32,
                    //     z: i32,
                    //     yaw: u8,
                    //     pitch: u8,
                    //     current_item: u16,
                    //     metadata: Vec<u8>,
                    // }

                    // // First send the new player to old clients
                    // let spawn = SpawnPlayer {
                    //     id: VarInt::<i32>(0x0C),
                    //     eid: VarInt::<i32>(context.entity.unwrap().index() as i32),
                    //     uuid: thyself.uuid,
                    //     x: 0,
                    //     y: 65 * 32,
                    //     z: 0,
                    //     yaw: 0,
                    //     pitch: 0,
                    //     current_item: 0,
                    //     metadata: vec![6 | (3 << 5), 0, 0, 0xA0, 0x41, 0x7F],
                    // };
                    // println!("{:?}", spawn);
                    // let spawn = serialize_with_size(&spawn)?;
                    // let spawn: Arc<[u8]> = spawn.into();
                    // println!("{:02x?}", spawn);
                    // for peer in context.peers.iter() {
                    //     if peer.key() != context.addr {
                    //         peer.send(spawn.clone()).await.unwrap();
                    //     }
                    // }

                    // s.write_all(&chunks[..]).await?;

                    // #[derive(Serialize)]
                    // pub struct SpawnPlayer {
                    //     entity_id: VarInt<i32>,
                    //     uuid: Uuid,
                    //     x: i32,
                    //     y: i32,
                    //     z: i32,
                    //     yaw: u8,
                    //     pitch: u8,
                    //     current_item: i16,
                    //     // TODO:
                    //     metadata: u8,
                    // }

                    // let chunks = serialize_to_vec(&SpawnPlayer { entity_id: VarInt::<i32>(12), uuid: Uuid::parse_str("59e0bff5-3cdd-475a-a360-3ccf860b158b").unwrap(), x: 0, y: 65, z: 0, yaw: 0, pitch: 50, current_item: 0, metadata: 0xFF })?;
                    // let chunks = serialize_to_vec(&Packet { header: Header { len: VarInt::<i32>(chunks.len() as i32 + 1), id: VarInt::<i32>(0x0C) }, packet: chunks })?;

                    // s.write_all(&chunks[..]).await?;

                    // context.stream.write_all(&CHUNK_DATA).await?;
                    // }
                }

                _ => (),
            }

            println!(
                "peer wants to logon ({}/{:02x})",
                *context.header.id, *context.header.id
            );
        }

        State::Play => {
            match *context.header.id {
                0x00 => {}

                0x01 => {
                    // let packet = {
                    //     let world = context.world.read().unwrap();
                    //     let player = world.get::<PlayerName>(context.entity.unwrap()).unwrap();

                    //     let message = deserialize_from_slice::<String>(context.buf)?.1;
                    //     println!("User {} sent a message {}", player.name, message);

                    //     #[derive(Serialize)]
                    //     pub struct ChatMessage {
                    //         id: v32,
                    //         json: String,
                    //         position: u8,
                    //     }

                    //     let message = serialize_with_size(&ChatMessage { id: VarInt::<i32>(0x02), json: format!("{{\"text\": \"<{}> \", \"bold\": true, \"extra\": [{{\"text\": \"{}\", \"bold\": false}}]}}", player.name, message), position: 0 }).unwrap();
                    //     Into::<Arc<[u8]>>::into(&message[..])
                    // };
                    // for peer in context.peers.iter() {
                    //     peer.send(packet.clone()).await.unwrap();
                    // }
                }

                0x15 => {
                    let settings = from_slice::<ClientSettings>(context.buf)?;
                    println!("{settings:?}");
                }

                0x17 => {
                    let plugin_message = from_slice::<PluginMessage<Vec<u8>>>(
                        &context.buf[..*context.header.len as usize - 1],
                    )?;
                    println!("{plugin_message:?}");
                }

                0x6 => {
                    let pos_and_look = from_slice::<PlayerPosLook>(context.buf)?;
                    // println!("{pos_and_look:?}");
                }

                0x4 => {
                    let pos = from_slice::<PlayerPos>(context.buf)?;
                    // println!("{pos:?}");
                }

                0x3 => {
                    // player wants player updates?
                    // whatever we respond w keep alive bro
                    // if let Some(cipher) = context.cipher {
                    //     context
                    //         .stream
                    //         .send_enc(cipher, 0x0, &VarInt::<u32>(rand::random()))
                    //         .await?;
                    // }
                }

                0x05 => {
                    // player look
                }

                _ => println!("unhandled packet {:02x}", *context.header.id),
            }
        } // State::Invalid => return Err(io::Error::other("peer state invalid!")),
    }

    Ok(())
}
