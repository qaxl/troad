use serde::Deserialize;
use troad_serde::serde_bytes;

#[derive(Deserialize)]
pub struct LoginStart {
    pub name: String,
}

#[derive(Deserialize)]
pub struct EncryptionResponse {
    #[serde(with = "serde_bytes")]
    pub shared_secret: Vec<u8>,
    #[serde(with = "serde_bytes")]
    pub verify_token: Vec<u8>,
}
