use serde::{Deserialize, Serialize};

pub mod handshake;
pub mod login;
pub mod status;

pub mod chat;

#[repr(i8)]
#[derive(Serialize, Deserialize, Copy, Clone)]
pub enum State {
    Game = -1,
    Initial = 0,
    Status = 1,
    Login = 2,
}
