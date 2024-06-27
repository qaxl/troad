use serde::{Deserialize, Serialize};
use troad_serde::var_int::VarInt;

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Header {
    pub len: VarInt,
    pub id: VarInt,
}

#[deprecated = "serialize with serialize_with_size instead"]
#[derive(Serialize)]
pub struct Packet<T> {
    pub header: Header,
    pub packet: T,
}

// LOGIN
#[derive(Deserialize, Debug)]
pub struct Handshake {
    pub version: VarInt,
    pub address: String,
    pub port: u16,
    pub next_state: VarInt, // TODO: enum...
}

// STATUS RESPONSE
// TODO: server icon
#[derive(Serialize)]
pub struct Status {
    pub version: VersionInfo,
    pub players: PlayersStatus,
    pub description: Description,
}

#[derive(Serialize)]
pub struct VersionInfo {
    pub name: String,
    pub protocol: u64,
}

#[derive(Serialize)]
pub struct PlayersStatus {
    pub max: u64,
    pub online: u64,
    // TODO: ???
    // sample: Option<()>,
}

// TODO: implement text components...
#[derive(Serialize)]
pub struct Description {
    pub text: String,
}

// PING REQUEST/RESPONSE
#[derive(Serialize, Deserialize)]
pub struct Ping {
    pub value: i64,
}

#[derive(Serialize)]
pub struct StringPck<'a> {
    pub str: &'a str,
}

#[derive(Deserialize, Debug)]
pub struct LoginStart {
    pub name: String,
    // Not on 1.8.9 bro
    // pub uuid: [u64; 2],
}

#[derive(Serialize)]
pub struct LoginSuccess {
    pub uuid: String,
    pub name: String,
}

// State::Play
#[derive(Serialize)]
pub struct JoinGame {
    pub entity_id: i32,
    // 0 -> surv, 1 -> creative, 2 -> adv, 3 -> spec, bit 3 (0x8) -> hardcore
    pub game_mode: u8,
    // -1 -> nether, 0 -> overworld, 1 -> the end
    pub dimension: i8,
    // {0, 1, 2, 3} -> {peaceful, easy, normal, hard}
    pub difficulty: u8,
    // only used to draw the player list
    pub max_players: u8,
    // level type
    // TODO: String or &'a str?
    pub level_type: String,
    // show less or more debug info
    pub reduced_debug_info: bool,
}

#[derive(Deserialize, Debug)]
pub struct ClientSettings {
    pub locale: String,
    pub view_distance: i8,
    pub chat_mode: i8,
    pub chat_colors: bool,
    // TODO: bitfield...
    /*
        Displayed Skin Parts flags:

        Bit 0 (0x01): Cape enabled
        Bit 1 (0x02): Jacket enabled
        Bit 2 (0x04): Left Sleeve enabled
        Bit 3 (0x08): Right Sleeve enabled
        Bit 4 (0x10): Left Pants Leg enabled
        Bit 5 (0x20): Right Pants Leg enabled
        Bit 6 (0x40): Hat enabled

    The most significant bit (bit 7, 0x80) appears to be unused.  */
    pub displayed_skin_parts: i8,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PluginMessage<T> {
    pub channel: String,
    pub data: T,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PlayerPosLook {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
    pub on_ground: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PlayerPos {
    x: f64,
    y: f64,
    z: f64,
    on_ground: bool,
}

#[derive(Serialize, Deserialize)]
pub struct TimeUpdate {
    pub world_age: i64,
    pub time_of_day: i64,
}

#[derive(Serialize, Deserialize)]
pub struct ServerDifficulty {
    pub difficulty: u8,
}

// 26 + 26 + 12
#[derive(Serialize, Deserialize)]
pub struct SpawnPosition {
    pub location: u64,
}

#[derive(Serialize, Deserialize)]
pub struct PlayerAbilities {
    pub flags: i8,
    pub flying_speed: f32,
    pub fov_modifier: f32,
}

#[derive(Serialize, Deserialize)]
pub struct HeldItemChange {
    pub slot: i8,
}
