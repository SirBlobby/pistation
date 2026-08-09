use chrono::Utc;
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::Serialize;

use crate::error::AppResult;

#[derive(Serialize)]
struct VideoGrant {
    room: String,
    #[serde(rename = "roomJoin")]
    room_join: bool,
    #[serde(rename = "canPublish")]
    can_publish: bool,
    #[serde(rename = "canSubscribe")]
    can_subscribe: bool,
    #[serde(rename = "canPublishData")]
    can_publish_data: bool,
    #[serde(rename = "canUpdateOwnMetadata")]
    can_update_own_metadata: bool,
    #[serde(rename = "roomAdmin")]
    room_admin: bool,
}

#[derive(Serialize)]
struct AccessTokenClaims {
    iss: String,
    sub: String,
    nbf: i64,
    exp: i64,
    name: String,
    metadata: String,
    video: VideoGrant,
}

pub struct TokenRequest<'a> {
    pub room_name: &'a str,
    pub identity: &'a str,
    pub display_name: &'a str,
    pub role: &'a str,
    pub can_publish: bool,
    pub room_admin: bool,
    pub ttl_seconds: i64,
}

pub fn mint_access_token(
    api_key: &str,
    api_secret: &str,
    request: TokenRequest<'_>,
) -> AppResult<(String, i64)> {
    let issued_at = Utc::now().timestamp();
    let expires_at = issued_at + request.ttl_seconds;

    let claims = AccessTokenClaims {
        iss: api_key.to_string(),
        sub: request.identity.to_string(),
        nbf: issued_at - 10,
        exp: expires_at,
        name: request.display_name.to_string(),
        metadata: serde_json::json!({ "role": request.role }).to_string(),
        video: VideoGrant {
            room: request.room_name.to_string(),
            room_join: true,
            can_publish: request.can_publish,
            can_subscribe: true,
            can_publish_data: true,
            can_update_own_metadata: true,
            room_admin: request.room_admin,
        },
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(api_secret.as_bytes()),
    )?;

    Ok((token, expires_at))
}
