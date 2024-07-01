// use server::Server;

use serde::Deserialize;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};
use troad_protocol::{
    chat::{Chat, Color},
    handshake, login,
    net::Connection,
    status, State,
};
use troad_serde::{from_slice, to_vec_with_size, var_int};

// mod event;
// mod protocol;
// mod server;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("0.0.0.0:25565").await.unwrap();

    loop {
        let (mut stream, addr) = listener.accept().await.unwrap();
        println!("{addr}");

        tokio::spawn(async move {
            let mut connection = Connection::from(stream);
            let mut state = State::Handshaking;
            loop {
                match state {
                    State::Handshaking => {
                        let p = connection.recv::<handshake::ServerBound>().await.unwrap();
                        match p {
                            handshake::ServerBound::Handshake(handshake) => {
                                eprintln!("{}", handshake.server_address);

                                if state != State::Login && state != State::Status {
                                    connection
                                        .send(&login::ClientBound::Disconnect("kys".to_owned()))
                                        .await
                                        .unwrap();
                                }

                                state = handshake.next_state;
                            }

                            handshake::ServerBound::LegacyServerListPing => {}
                        }
                    }
                    _ => (),
                }
            }
        });
    }
}
