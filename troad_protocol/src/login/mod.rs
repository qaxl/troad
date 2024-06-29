use serde::{Deserialize, Serialize};

pub mod client_bound;
pub mod server_bound;

#[derive(Deserialize)]
pub enum ServerBound {
    LoginStart(server_bound::LoginStart),
    EncryptionResponse(server_bound::EncryptionResponse),
}

#[derive(Serialize)]
pub enum ClientBound {
    // TODO: Chat
    Disconnect(String),

    EncryptionRequest(client_bound::EncryptionRequest),
    LoginSuccess(client_bound::LoginSuccess),
    SetCompression(client_bound::SetCompression),
}
