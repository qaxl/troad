use serde::Serialize;
use troad_serde::{serde_bytes, var_int};

#[derive(Serialize)]
pub struct EncryptionRequest {
    server_id: String,
    #[serde(with = "serde_bytes")]
    public_key: Vec<u8>,
    #[serde(with = "serde_bytes")]
    verify_token: Vec<u8>,
}

#[derive(Serialize)]
pub struct LoginSuccess {
    uuid: String,
    username: String,
}

#[derive(Serialize)]
pub struct SetCompression {
    #[serde(with = "var_int")]
    threshold: usize,
}
