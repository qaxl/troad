use std::{
    io,
    sync::{Arc, RwLock},
};

use bevy_ecs::{component::Component, world::World};
use serde::{Deserialize, Serialize};
use tokio::{io::AsyncWriteExt, net::TcpStream};
use uuid::Uuid;

use crate::{
    protocol::{
        packets::{
            ClientSettings, Description, Handshake, Header, HeldItemChange, JoinGame, LoginStart,
            LoginSuccess, Packet, Ping, PlayerAbilities, PlayerPos, PlayerPosLook, PlayersStatus,
            PluginMessage, ServerDifficulty, SpawnPosition, Status, StringPck, TimeUpdate,
            VersionInfo,
        },
        serde::{deserialize_from_slice, serialize_to_vec, VarInt},
    },
    server::{PeersMap, TcpProtocolExt},
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
}

#[derive(Component)]
pub struct PlayerName {
    name: String,
    uuid: Uuid,
}

pub async fn handle_event<'a>(context: EventContext<'a>) -> Result<(), io::Error> {
    println!("{:02x}", *context.header.id);

    match context.state {
        State::Handshaking => match *context.header.id {
            HANDSHAKE => {
                let handshake = deserialize_from_slice::<Handshake>(context.buf)?.1;
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
                let ping = deserialize_from_slice::<Ping>(context.buf)?.1;
                context.stream.send(PING_PONG, &ping).await?;
            }
            _ => (),
        },

        State::Login => {
            match *context.header.id {
                LOGIN_START => {
                    let info = deserialize_from_slice::<LoginStart>(context.buf)?.1;
                    let uuid = Uuid::new_v4();

                    context
                        .stream
                        .send(
                            LOGIN_SUCCESS,
                            &LoginSuccess {
                                uuid: uuid.to_string(),
                                name: info.name.clone(),
                            },
                        )
                        .await?;

                    *context.state = State::Play;

                    let entity_id = {
                        let mut entity_world = context.world.write().unwrap();
                        let entity_world = entity_world.spawn(PlayerName {
                            name: info.name,
                            uuid,
                        });
                        entity_world.id().index() as i32
                    };

                    context
                        .stream
                        .send(
                            JOIN_GAME,
                            &JoinGame {
                                entity_id,
                                game_mode: 0,
                                dimension: 0,
                                difficulty: 2,
                                max_players: 100,
                                level_type: String::from("flat"),
                                reduced_debug_info: false,
                            },
                        )
                        .await?;

                    context
                        .stream
                        .send(
                            PLUGIN_MESSAGE_CLIENT_BOUND,
                            &PluginMessage {
                                channel: String::from("MC|Brand"),
                                data: String::from("troad"),
                            },
                        )
                        .await?;

                    context
                        .stream
                        .send(
                            PLAYER_POSITION_AND_LOOK,
                            &PlayerPosLook {
                                x: 0.0,
                                y: 65.0,
                                z: 0.0,
                                yaw: 0.0,
                                pitch: 0.0,
                                on_ground: false,
                            },
                        )
                        .await?;

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
                0x15 => {
                    let settings = deserialize_from_slice::<ClientSettings>(context.buf)?;
                    println!("{settings:?}");
                }

                0x17 => {
                    let plugin_message = deserialize_from_slice::<PluginMessage<Vec<u8>>>(
                        &context.buf[..*context.header.len as usize - 1],
                    )?;
                    println!("{plugin_message:?}");
                }

                0x6 => {
                    let pos_and_look = deserialize_from_slice::<PlayerPosLook>(context.buf)?;
                    // println!("{pos_and_look:?}");
                }

                0x4 => {
                    let pos = deserialize_from_slice::<PlayerPos>(context.buf)?;
                    // println!("{pos:?}");
                }

                0x3 => {
                    // player wants player updates?
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
