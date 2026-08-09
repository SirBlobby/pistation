use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::Value;


use crate::auth::{generate_opaque_token, hash_opaque_token, KioskIdentity};
use crate::clock::now_ms;
use crate::error::{AppError, AppResult};
use crate::layout;
use crate::livekit::{mint_access_token, TokenRequest};
use crate::pins;
use crate::state::AppState;

const KIOSK_TOKEN_TTL_SECONDS: i64 = 12 * 3600;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/register", post(register))
        .route("/pin", get(pin))
        .route("/session", get(session))
        .route("/layout", get(kiosk_layout))
        .route("/heartbeat", post(heartbeat))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RegisterRequest {
    enrollment_token: String,
    hardware_id: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RegisterResponse {
    kiosk_id: String,
    kiosk_token: String,
    room_name: String,
    livekit_url: String,
    rotation_seconds: i64,
}

async fn register(
    State(state): State<AppState>,
    Json(body): Json<RegisterRequest>,
) -> AppResult<Json<RegisterResponse>> {
    let enrollment_hash = hash_opaque_token(body.enrollment_token.trim());

    let row: Option<(String, String)> =
        sqlx::query_as("SELECT id, room_name FROM kiosks WHERE enrollment_token_hash = ?")
            .bind(&enrollment_hash)
            .fetch_optional(&state.db)
            .await?;

    let (kiosk_id, room_name) =
        row.ok_or_else(|| AppError::Unauthorized("invalid enrollment token".into()))?;

    let kiosk_token = generate_opaque_token();

    // The name belongs to whoever created the kiosk in the admin panel. Enrolling a Pi
    // must never rename it.
    sqlx::query(
        "UPDATE kiosks
         SET kiosk_token_hash = ?, enrollment_token_hash = NULL, hardware_id = ?,
             status = 'online', last_seen_at = ?
         WHERE id = ?",
    )
    .bind(hash_opaque_token(&kiosk_token))
    .bind(body.hardware_id.trim())
    .bind(now_ms())
    .bind(&kiosk_id)
    .execute(&state.db)
    .await?;

    pins::ensure_pin(
        &state.db,
        &kiosk_id,
        state.config.pin_rotation_seconds,
        state.config.pin_grace_seconds,
    )
    .await?;

    Ok(Json(RegisterResponse {
        kiosk_id,
        kiosk_token,
        room_name,
        livekit_url: state.config.livekit_url.clone(),
        rotation_seconds: state.config.pin_rotation_seconds,
    }))
}

async fn pin(
    State(state): State<AppState>,
    identity: KioskIdentity,
) -> AppResult<Json<pins::IssuedPin>> {
    let issued = pins::ensure_pin(
        &state.db,
        &identity.kiosk_id,
        state.config.pin_rotation_seconds,
        state.config.pin_grace_seconds,
    )
    .await?;

    Ok(Json(issued))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SessionResponse {
    room_name: String,
    livekit_url: String,
    access_token: String,
    participant_id: String,
}

async fn session(
    State(state): State<AppState>,
    identity: KioskIdentity,
) -> AppResult<Json<SessionResponse>> {
    let participant_id = format!("kiosk-{}", identity.kiosk_id);

    let (access_token, _) = mint_access_token(
        &state.config.livekit_api_key,
        &state.config.livekit_api_secret,
        TokenRequest {
            room_name: &identity.room_name,
            identity: &participant_id,
            display_name: "Kiosk",
            role: "kiosk",
            can_publish: false,
            room_admin: true,
            ttl_seconds: KIOSK_TOKEN_TTL_SECONDS,
        },
    )?;

    Ok(Json(SessionResponse {
        room_name: identity.room_name,
        livekit_url: state.config.livekit_url.clone(),
        access_token,
        participant_id,
    }))
}

async fn kiosk_layout(
    State(state): State<AppState>,
    identity: KioskIdentity,
) -> AppResult<Json<Value>> {
    let layout = layout::load_layout(&state.db, &identity.kiosk_id).await?;
    Ok(Json(layout))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct HeartbeatRequest {
    #[serde(default)]
    metrics: Option<Value>,
}

async fn heartbeat(
    State(state): State<AppState>,
    identity: KioskIdentity,
    body: Option<Json<HeartbeatRequest>>,
) -> AppResult<StatusCode> {
    let now = now_ms();
    let metrics = body.and_then(|Json(body)| body.metrics);

    match metrics {
        Some(metrics) => {
            sqlx::query(
                "UPDATE kiosks
                 SET last_seen_at = ?, status = 'online', metrics = ?, metrics_at = ?
                 WHERE id = ?",
            )
            .bind(now)
            .bind(metrics.to_string())
            .bind(now)
            .bind(&identity.kiosk_id)
            .execute(&state.db)
            .await?;
        }
        None => {
            sqlx::query("UPDATE kiosks SET last_seen_at = ?, status = 'online' WHERE id = ?")
                .bind(now)
                .bind(&identity.kiosk_id)
                .execute(&state.db)
                .await?;
        }
    }

    Ok(StatusCode::NO_CONTENT)
}
