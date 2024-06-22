use std::sync::Arc;

use num::pow;
use server::Server;

mod server;
mod protocol;
mod event;

#[tokio::main]
async fn main() {
    Server::new().await.run().await;
}
