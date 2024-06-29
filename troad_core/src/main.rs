// use server::Server;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};
use troad_protocol::{
    chat::{Chat, Color},
    login,
};
use troad_serde::to_vec_with_size;

mod event;
mod protocol;
mod server;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("0.0.0.0:25565").await.unwrap();

    loop {
        let (mut stream, addr) = listener.accept().await.unwrap();
        println!("{addr}");

        tokio::spawn(async move {
            let s = Chat::new()
                .text("Hello, world! ")
                .text("I am inside you. ")
                .color(Color::White)
                .reset()
                .text("ALALALALALAL")
                .color(Color::Red)
                .click_url("https://google.com")
                .underlined()
                .finish();
            let p = to_vec_with_size(&login::ClientBound::Disconnect(s.clone())).unwrap();

            println!("{s}");
            println!("{p:02x?}");

            let mut buf = [0; 1024];
            stream.read(&mut buf).await.unwrap();

            println!("{buf:02x?}");

            stream.write_all(&p).await.unwrap();

            loop {
                let mut buf = [0; 1024];
                if stream.read(&mut buf).await.unwrap() == 0 {
                    return;
                }
                println!("{buf:02x?}");
            }
        });
    }
}
