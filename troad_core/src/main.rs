// use server::Server;

use std::{
    process::Command,
    sync::{Arc, Mutex},
    time::Duration,
};

use tokio::net::TcpListener;
use troad_auth::session_server;
use troad_crypto::{
    rsa::{DecryptRsa, RsaKeyPool},
    sha1::sha1_notchian_hexdigest_arr,
};
use troad_protocol::{
    chat::{Chat, Color},
    game, handshake,
    login::{
        self,
        client_bound::{EncryptionRequest, LoginSuccess},
    },
    net::Connection,
    status, State,
};

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("0.0.0.0:25565").await.unwrap();
    let key_pool = Arc::new(Mutex::new(RsaKeyPool::new(1024)));

    // Have some keys in the pool...
    RsaKeyPool::replenish(key_pool.clone(), Some(16));

    {
        let key_pool = key_pool.clone();
        tokio::task::spawn_blocking(move || {
            // {
            //     let key_pool = key_pool.clone();

            //     for _ in 0..10 {
            //         let key_pool = key_pool.clone();
            //         tokio::task::spawn_blocking(move || {
            //             for _ in 0..100 {
            //                 RsaKeyPool::replenish(key_pool.clone(), Some(16));
            //             }
            //         });
            //     }
            // }

            loop {
                let fullness = {
                    let key_pool = key_pool.lock().unwrap();
                    key_pool.fullness()
                };

                if fullness < 0.5 {
                    // dumb syntax, cuz it locks itself twice.
                    RsaKeyPool::replenish(key_pool.clone(), None);
                } else {
                    std::thread::sleep(Duration::from_secs(60));
                }
            }
        });
    }

    loop {
        let (stream, addr) = listener.accept().await.unwrap();
        println!("connection made: {addr}");

        let key_pool = key_pool.clone();
        let mut key = None;
        let mut player_name = String::new();
        let verify_token = rand::random::<u128>().to_be_bytes().to_vec();

        tokio::spawn(async move {
            let mut connection = Connection::from(stream);
            let mut state = State::Handshaking;
            loop {
                match state {
                    State::Handshaking => {
                        let p = connection.recv::<handshake::ServerBound>().await.unwrap();
                        match p {
                            handshake::ServerBound::Handshake(handshake) => {
                                if handshake.next_state == State::Status
                                    || handshake.next_state == State::Login
                                {
                                    state = handshake.next_state;
                                } else {
                                    return;
                                }
                            }

                            handshake::ServerBound::LegacyServerListPing => {
                                return;
                            }
                        }
                    }

                    State::Status => {
                        let p = connection.recv::<status::ServerBound>().await.unwrap();
                        match p {
                            status::ServerBound::Request => {
                                connection.send(&status::ClientBound::Response(
                                    format!("{{\"version\":{{\"name\":\"Troad 1.8.x\",\"protocol\":47}},\"players\":{{\"online\":{},\"max\":{}}},\"description\":{}}}", 0, 0, 
                                        Chat::new()
                                        .text("Running on version ")
                                        .text(&
                                            String::from_utf8_lossy(
                                                &Command::new("git")
                                                .arg("rev-parse")
                                                .arg("--short")
                                                .arg("HEAD")
                                                .output()
                                                .unwrap()
                                                .stdout
                                            )
                                            .lines()
                                            .next()
                                            .unwrap())
                                        .bold()
                                        .color(Color::Cyan)
                                        .text("!\nLocal test server.")
                                        .finish())
                                    )
                                ).await.unwrap();
                            }

                            status::ServerBound::Ping(payload) => {
                                connection
                                    .send(&status::ClientBound::Pong(payload))
                                    .await
                                    .unwrap();
                                return;
                            }
                        }
                    }

                    State::Login => {
                        let p = connection.recv::<login::ServerBound>().await.unwrap();
                        match p {
                            login::ServerBound::LoginStart(info) => {
                                key = Some(key_pool.lock().unwrap().pop());
                                let key = unsafe { key.clone().unwrap_unchecked() };
                                player_name = info.name;

                                // connection.set_compression(Some(256)).await.unwrap();
                                connection
                                    .send(&login::ClientBound::EncryptionRequest(
                                        EncryptionRequest {
                                            server_id: String::default(),
                                            public_key: key.2,
                                            verify_token: verify_token.clone(),
                                        },
                                    ))
                                    .await
                                    .unwrap();
                            }
                            login::ServerBound::EncryptionResponse(res) => {
                                let key = key.clone().unwrap();

                                println!("{:02x?}\n{:02x?}", res.shared_secret, res.verify_token);

                                let vt = key.1.decrypt_ct(&res.verify_token);
                                if vt != verify_token {
                                    eprintln!("Verify token doesn't match ({vt:02x?} != {verify_token:02x?}");
                                    return;
                                }

                                println!("A");
                                let ss = key.1.decrypt_ct(&res.shared_secret);
                                println!("A");
                                connection.enable_encryption(&ss).unwrap();

                                let auth = session_server::authenticate_player(
                                    &player_name,
                                    &sha1_notchian_hexdigest_arr(&[&ss, &key.2]),
                                    None,
                                )
                                .await;
                                match auth {
                                    Ok(s) => {
                                        connection
                                            .send(&login::ClientBound::LoginSuccess(LoginSuccess {
                                                uuid: s.id.to_string(),
                                                username: s.name,
                                            }))
                                            .await
                                            .unwrap();
                                        connection
                                            .send(&game::ClientBound::JoinGame {
                                                entity_id: rand::random::<i32>(),
                                                game_mode: 1,
                                                dimension: 0,
                                                difficulty: 0,
                                                max_players: 10,
                                                level_type: String::from("flat"),
                                                reduced_debug_info: false,
                                            })
                                            .await
                                            .unwrap();
                                        connection
                                            .send(&game::ClientBound::PlayerLookAndPosition {
                                                x: 0.0,
                                                y: 0.0,
                                                z: 1.0,
                                                yaw: 0.0,
                                                pitch: 0.0,
                                                flags: 0,
                                            })
                                            .await
                                            .unwrap();

                                        connection
                                            .send(&game::ClientBound::ChatMessage {
                                                json: Chat::new()
                                                    .text("")
                                                    .text(&player_name)
                                                    .bold()
                                                    .color(Color::Blue)
                                                    .text(" joined!")
                                                    .finish(),
                                                position: 0,
                                            })
                                            .await
                                            .unwrap();

                                        state = State::Game;

                                        // println!("Successful authentication!");
                                    }

                                    Err(e) => {
                                        match e {
                                            session_server::Error::Unauthorized => {
                                                eprintln!("Unauthorized player tried to connect from {addr}!");
                                                return;
                                            }
                                            session_server::Error::ReqwestError(e) => {
                                                eprintln!("Minecraft auth server errored out: {e}");
                                                connection
                                                    .send(&game::ClientBound::Disconnect(
                                                        Chat::new()
                                                            .text("Authentication failure!")
                                                            .finish(),
                                                    ))
                                                    .await
                                                    .unwrap();
                                            }
                                            session_server::Error::WeirdResponse => {
                                                eprintln!("Minecraft auth server is down?");
                                                connection.send(&game::ClientBound::Disconnect(Chat::new().text("Authentication server is unavailable.").finish())).await.unwrap();
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    State::Game => {
                        let p = connection.recv::<game::ServerBound>().await.unwrap();

                        match &p {
                            game::ServerBound::KeepAlive(_) => (),
                            game::ServerBound::ClientSettings {
                                locale,
                                view_distance,
                                chat_mode,
                                chat_colors,
                                displayed_skin_parts,
                            } => {
                                println!("{p:?}");
                            }
                            // game::ServerBound::Unhandled => eprintln!("UNHANDLED PACKET!"),
                            _ => eprintln!("unhandled packet: {p:?}"),
                        }
                    }
                }
            }
        });
    }
}
