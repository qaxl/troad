use reqwest::StatusCode;
use serde::Deserialize;
use uuid::Uuid;

pub enum PlayerAuthenticationStatus {
    Authenticated(String),
    Unauthenticated,
    ServerError,
}

#[derive(Deserialize)]
pub struct AuthenticationResponse {
    id: Uuid,
    name: String,

    // Player's skin blob.
    properties: Vec<AuthenticationProperties>,
}

#[derive(Deserialize)]
pub struct AuthenticationProperties {
    name: String,
    value: String,
    signature: String,
}

pub async fn authenticate_player(
    username: &str,
    server_id: &str,
    ip: Option<&str>,
) -> Result<AuthenticationResponse, Error> {
    let request = reqwest::get(format!(
        "https://sessionserver.mojang.com/session/minecraft/hasJoined?username={}&serverId={}{}",
        username,
        server_id,
        if let Some(ip) = ip {
            format!("&ip={}", ip)
        } else {
            "".to_owned()
        }
    ))
    .await?;

    match request.error_for_status() {
        Ok(request) => match request.status() {
            StatusCode::OK => {
                let body = request.text().await?;
                Ok(serde_json::from_str(&body).expect("invalid json response"))
            }

            StatusCode::NO_CONTENT => Err(Error::Unauthorized),

            _ => Err(Error::WeirdResponse),
        },

        Err(e) => Err(e.into()),
    }
}

pub enum Error {
    ReqwestError(reqwest::Error),
    Unauthorized,
    WeirdResponse,
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Error::ReqwestError(value)
    }
}
