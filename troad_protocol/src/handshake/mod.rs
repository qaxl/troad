use serde::Deserialize;

pub mod server_bound;

#[derive(Deserialize, Debug)]
pub enum ServerBound {
    Handshake(server_bound::Handshake),

    // #[serde(other)]
    // LegacyServerListPing,
}
