use serde::Serialize;
use troad_serde::{serde_bytes, var_int};

#[derive(Serialize)]
pub struct EncryptionRequest {
    pub server_id: String,
    #[serde(with = "serde_bytes")]
    pub public_key: Vec<u8>,
    #[serde(with = "serde_bytes")]
    pub verify_token: Vec<u8>,
}

#[derive(Serialize)]
pub struct LoginSuccess {
    pub uuid: String,
    pub username: String,
}

#[derive(Serialize)]
pub struct SetCompression {
    #[serde(with = "var_int")]
    pub threshold: usize,
}
