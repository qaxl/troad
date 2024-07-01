use serde::{Deserialize, Serialize};

pub mod handshake;
pub mod login;
pub mod status;

pub mod chat;
pub mod net;

#[repr(i8)]
#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum State {
    Handshaking = 0,
    Status = 1,
    Login = 2,
    Game = -1,
}
