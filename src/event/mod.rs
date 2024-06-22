use std::io;

use serde::{Deserialize, Serialize};
use tokio::{io::AsyncWriteExt, net::TcpStream};
use uuid::Uuid;

use crate::{protocol::{packets::{ClientSettings, Description, Handshake, Header, HeldItemChange, JoinGame, LoginStart, LoginSuccess, Packet, Ping, PlayerAbilities, PlayerPos, PlayerPosLook, PlayersStatus, PluginMessage, ServerDifficulty, SpawnPosition, Status, StringPck, TimeUpdate, VersionInfo}, serde::{deserialize_from_slice, serialize, VarI32, VarInt}}, server::PeersMap};

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

#[repr(i32)]
pub enum State {
    Connected = 0,
    Status = 1,
    Login = 2,
    
    Invalid = -1,
    Play = -2,
}

impl From<i32> for State {
    fn from(value: i32) -> Self {
        match value {
            0 => Self::Connected,
            1 => Self::Status,
            2 => Self::Login,
            _ => Self::Invalid,
        }
    }
}

pub struct EventContext<'a> {
    pub peers: &'a PeersMap,

    pub state: &'a mut State,
    pub stream: &'a mut TcpStream,
    
    pub buf: &'a [u8],
    pub header: Header,
}

pub async fn handle_event<'a>(context: EventContext<'a>) -> Result<(), io::Error> {
    /// println!("{:?} {}", context.header, context.buf.len());

    match context.state {
        State::Connected => {
            match *context.header.id {
                HANDSHAKE => {
                    let handshake = deserialize_from_slice::<Handshake>(context.buf)?.1;
                    println!("{handshake:?}");

                    *context.state = (*handshake.next_state).into();
                },
                _ => (),
            }
        }

        State::Status => {
            match *context.header.id {
                STATUS => {
                    let status = Status { version: VersionInfo { name: String::from("Troad 1.8.x"), protocol: 47 }, players: PlayersStatus { max: 1, online: 0 }, description: Description { text: String::from("unfortunate") } };
                    let status = serde_json::to_string(&status)?;

                    println!("{status}");
                    let status = serialize(&StringPck { str: &status })?;
                    let status = serialize(&Packet { header: Header { len: VarInt::<i32>(status.len() as i32 + 1), id: VarInt::<i32>(0) }, packet: &status })?;


                    context.stream.write_all(&status).await?;
                }

                PING_PONG => {
                    let ping = deserialize_from_slice::<Ping>(context.buf)?.1;
                    let pong = serialize(&Packet { header: Header { len: VarInt::<i32>(9), id: VarInt::<i32>(PING_PONG) }, packet: ping })?;

                    context.stream.write_all(&pong).await?;
                },
                _ => (),
            }
        },

        State::Login => {
            match *context.header.id {
                LOGIN_START => {
                    let info = deserialize_from_slice::<LoginStart>(context.buf)?.1;
                    println!("{info:?}");

                    let info = serialize(&LoginSuccess { uuid: String::from("59e0bff5-3cdd-475a-a360-3ccf860b158b"), name: info.name })?;
                    let info = serialize(&Packet { header: Header { len: VarInt::<i32>(info.len() as i32 + 1), id: VarInt::<i32>(2) }, packet: info })?;

                    context.stream.write_all(&info).await?;
                    *context.state = State::Play;

                    let join = serialize(&JoinGame { entity_id: 12, game_mode: 0, dimension: 0, difficulty: 2, max_players: 100, level_type: String::from("flat"), reduced_debug_info: false })?;
                    let join = serialize(&Packet { header: Header { len: VarInt::<i32>(join.len() as i32 + 1), id: VarInt::<i32>(1) }, packet: join })?;

                    context.stream.write_all(&join).await?;

                    let time_update = serialize(&TimeUpdate { world_age: 0, time_of_day: 6000 /* % 24000 */ })?;
                    let time_update = serialize(&Packet { header: Header { len: VarInt::<i32>(time_update.len() as i32 + 1), id: VarInt::<i32>(0x03) }, packet: time_update })?;

                    context.stream.write_all(&time_update).await?;

                    let mc_brand = serialize(&PluginMessage { channel: String::from("MC|Brand"), data: String::from("troad") })?;
                    let mc_brand = serialize(&Packet { header: Header { len: VarInt::<i32>(mc_brand.len() as i32 + 1), id: VarInt::<i32>(0x3F) }, packet: mc_brand })?;

                    context.stream.write_all(&mc_brand).await?;

                    let mc_brand = serialize(&ServerDifficulty { difficulty: 2 })?;
                    let mc_brand = serialize(&Packet { header: Header { len: VarInt::<i32>(mc_brand.len() as i32 + 1), id: VarInt::<i32>(0x41) }, packet: mc_brand })?;

                    context.stream.write_all(&mc_brand).await?;

                    // let mc_brand = serialize(&SpawnPosition { location: 0 })?;
                    // let mc_brand = serialize(&Packet { header: Header { len: VarInt::<i32>(mc_brand.len() as i32 + 1), id: VarInt::<i32>(0x05) }, packet: mc_brand })?;

                    // context.stream.write_all(&mc_brand).await?;

                    // i still really don't know how these are encoded bro
                    let vec = vec![0x09, 0x05, 0xff, 0xff, 0xdd, 0x81, 0x00, 0x00, 0x00, 0xf5];
                    context.stream.write_all(&vec[..]).await?;

                    let mc_brand = serialize(&HeldItemChange { slot: 0 })?;
                    let mc_brand = serialize(&Packet { header: Header { len: VarInt::<i32>(mc_brand.len() as i32 + 1), id: VarInt::<i32>(0x09) }, packet: mc_brand })?;

                    context.stream.write_all(&mc_brand).await?;

                    let mc_brand = serialize(&PlayerPosLook { x: 0.0, y: 65.0, z: 0.0, yaw: 0.0, pitch: 0.0, on_ground: false })?;
                    let mc_brand = serialize(&Packet { header: Header { len: VarInt::<i32>(mc_brand.len() as i32 + 1), id: VarInt::<i32>(0x08) }, packet: mc_brand })?;

                    context.stream.write_all(&mc_brand).await?;

                    /*
                     */

                    let n = 16;
                    let total = n * 8192 + n * 2048 + n * 2048 + 256;

                    let s = context.stream;
                    let mut data = vec![0; total as usize];
                    #[derive(Serialize, Default)]
                    pub struct Chunk {
                        _ign0: i64,
                        _ign1: u8,
                        bit_field: u16,
                        size: VarInt<i32>,
                        data: Vec<u8>,
                    }

                    let y = 63;

                    for x in 0..15 {
                        for z in 0..15 {
                            let i = y << 8 | z << 4 | x;
                            let t = 35;
                            let d = (x + z);

                            data[2 * i] = ((t << 4) | d) as u8;
                            data[2 * i + 1] = ((t >> 4)) as u8;
                        }
                    }

                    for i in 0..n*2048 {
                        data[(n*8192 + i) as usize] = 0xFF;
                    }

                    for i in 0..n*2048 {
                        data[(n*8192 + n*2048 + i) as usize] = 0xFF;
                    }

                    let chunks = serialize(&Chunk { bit_field:0b1111111111111111, size: VarInt::<i32>(total as i32), data, ..Default::default() })?;
                    let chunks = serialize(&Packet { header: Header { len: VarInt::<i32>(chunks.len() as i32 + 1), id: VarInt::<i32>(0x21) }, packet: chunks })?;

                    s.write_all(&chunks[..]).await?;

                    #[derive(Serialize)]
                    pub struct SpawnPlayer {
                        entity_id: VarInt<i32>,
                        uuid: Uuid,
                        x: i32,
                        y: i32,
                        z: i32,
                        yaw: u8,
                        pitch: u8,
                        current_item: i16,
                        // TODO:
                        metadata: u8,
                    }

                    // let chunks = serialize(&SpawnPlayer { entity_id: VarInt::<i32>(12), uuid: Uuid::parse_str("59e0bff5-3cdd-475a-a360-3ccf860b158b").unwrap(), x: 0, y: 65, z: 0, yaw: 0, pitch: 50, current_item: 0, metadata: 0xFF })?;
                    // let chunks = serialize(&Packet { header: Header { len: VarInt::<i32>(chunks.len() as i32 + 1), id: VarInt::<i32>(0x0C) }, packet: chunks })?;

                    // s.write_all(&chunks[..]).await?;

                    // context.stream.write_all(&CHUNK_DATA).await?;
                }

                _ => (),
            }

            println!("peer wants to logon ({}/{:02x})", *context.header.id, *context.header.id);
        },

        State::Play => {
            match *context.header.id {
                0x15 => {
                    let settings = deserialize_from_slice::<ClientSettings>(context.buf)?;
                    println!("{settings:?}");
                }

                0x17 => {
                    let plugin_message = deserialize_from_slice::<PluginMessage<Vec<u8>>>(context.buf)?;
                    println!("{plugin_message:?}");
                }

                0x6 => {
                    let pos_and_look = deserialize_from_slice::<PlayerPosLook>(context.buf)?;
                    println!("{pos_and_look:?}");
                }

                0x4 => {
                    let pos = deserialize_from_slice::<PlayerPos>(context.buf)?;
                    println!("{pos:?}");
                }

                0x3 => {
                    // player wants player updates?
                }

                _ => println!("unhandled packet {:02x}", *context.header.id)
            }
        }

        State::Invalid => return Err(io::Error::other("peer state invalid!")),
    }

    Ok(())
}
