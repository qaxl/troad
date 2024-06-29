use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub enum ServerBound {
    Request,
    Ping(u64),
}

#[derive(Serialize)]
pub enum ClientBound {
    Response(String),
    Pong(u64),
}
