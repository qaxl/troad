use server::Server;

mod event;
mod protocol;
mod server;

#[tokio::main]
async fn main() {
    Server::new().await.run().await;
}
