use serde::Deserialize;
use troad_serde::var_int;

use crate::State;

#[derive(Deserialize)]
pub struct Handshake {
    #[serde(with = "var_int")]
    protocol_version: u32,

    server_address: String,
    server_port: u16,

    // #[serde(with = "var_int")]
    // This is realistically a var int, but it doesn't matter as it never exceeds 7F
    next_state: State,
}

#[derive(Deserialize)]
pub struct LegacyServerListPing {
    payload: u8,
}
