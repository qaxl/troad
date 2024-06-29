use serde::Deserialize;

pub mod server_bound;

#[derive(Deserialize)]
pub enum ServerBound {
    Handshake(server_bound::Handshake),

    #[serde(other)]
    LegacyServerListPing,
}
