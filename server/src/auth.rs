use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::error::{AppError, AppResult};
use crate::state::AppState;

pub fn hash_password(password: &str) -> AppResult<String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|error| AppError::Internal(format!("password hashing failed: {error}")))
}

pub fn verify_password(password: &str, stored_hash: &str) -> bool {
    let Ok(parsed) = PasswordHash::new(stored_hash) else {
        return false;
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok()
}

pub fn generate_opaque_token() -> String {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    hex::encode(bytes)
}

pub fn hash_opaque_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}

#[derive(Serialize, Deserialize)]
struct AdminClaims {
    sub: String,
    email: String,
    exp: i64,
}

pub fn mint_admin_token(
    secret: &str,
    admin_id: &str,
    email: &str,
    ttl_hours: i64,
) -> AppResult<(String, i64)> {
    let expires_at = Utc::now().timestamp() + ttl_hours * 3600;
    let claims = AdminClaims {
        sub: admin_id.to_string(),
        email: email.to_string(),
        exp: expires_at,
    };
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )?;
    Ok((token, expires_at))
}

pub struct AdminIdentity {
    pub admin_id: String,
    pub email: String,
}

pub struct KioskIdentity {
    pub kiosk_id: String,
    pub room_name: String,
}

impl FromRequestParts<AppState> for AdminIdentity {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let token = bearer_token(parts)?;
        let data = decode::<AdminClaims>(
            &token,
            &DecodingKey::from_secret(state.config.session_secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|_| AppError::Unauthorized("invalid or expired admin session".into()))?;

        Ok(AdminIdentity {
            admin_id: data.claims.sub,
            email: data.claims.email,
        })
    }
}

impl FromRequestParts<AppState> for KioskIdentity {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let token = bearer_token(parts)?;
        let token_hash = hash_opaque_token(&token);

        let row: Option<(String, String)> =
            sqlx::query_as("SELECT id, room_name FROM kiosks WHERE kiosk_token_hash = ?")
                .bind(&token_hash)
                .fetch_optional(&state.db)
                .await?;

        let (kiosk_id, room_name) =
            row.ok_or_else(|| AppError::Unauthorized("unknown kiosk token".into()))?;

        Ok(KioskIdentity {
            kiosk_id,
            room_name,
        })
    }
}

fn bearer_token(parts: &Parts) -> AppResult<String> {
    let header = parts
        .headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| AppError::Unauthorized("missing authorization header".into()))?;

    header
        .strip_prefix("Bearer ")
        .map(|token| token.trim().to_string())
        .filter(|token| !token.is_empty())
        .ok_or_else(|| AppError::Unauthorized("malformed authorization header".into()))
}
