use serde::Deserialize;
use troad_serde::var_int;

use crate::State;

#[derive(Deserialize, Debug)]
pub struct Handshake {
    #[serde(with = "var_int")]
    pub protocol_version: u32,

    pub server_address: String,
    pub server_port: u16,

    // #[serde(with = "var_int")]
    // This is realistically a var int, but it doesn't matter as it never exceeds 7F
    pub next_state: State,
}

#[derive(Deserialize)]
pub struct LegacyServerListPing {
    pub payload: u8,
}
