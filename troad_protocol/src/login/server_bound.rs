use serde::Deserialize;
use troad_serde::serde_bytes;

#[derive(Deserialize)]
pub struct LoginStart {
    name: String,
}

#[derive(Deserialize)]
pub struct EncryptionResponse {
    #[serde(with = "serde_bytes")]
    shared_secret: Vec<u8>,
    #[serde(with = "serde_bytes")]
    verify_token: Vec<u8>,
}
