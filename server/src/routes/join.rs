use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::clock::{now_ms, seconds_to_ms};
use crate::error::{AppError, AppResult};
use crate::livekit::{mint_access_token, TokenRequest};
use crate::pins;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/join", post(join))
        .route("/session/refresh", post(refresh))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct JoinRequest {
    pin: String,
    #[serde(default)]
    display_name: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct JoinResponse {
    session_id: String,
    room_name: String,
    kiosk_name: String,
    livekit_url: String,
    access_token: String,
    participant_id: String,
    display_name: String,
    role: String,
    expires_at: i64,
}

async fn join(
    State(state): State<AppState>,
    Json(body): Json<JoinRequest>,
) -> AppResult<Json<JoinResponse>> {
    let pin = body.pin.trim().replace(' ', "");
    if pin.len() != 6 || !pin.chars().all(|character| character.is_ascii_digit()) {
        return Err(AppError::BadRequest("pin must be six digits".into()));
    }

    let kiosk_id = pins::resolve_pin(&state.db, &pin)
        .await?
        .ok_or_else(|| AppError::NotFound("that pin is not valid right now".into()))?;

    let (kiosk_name, room_name): (String, String) =
        sqlx::query_as("SELECT name, room_name FROM kiosks WHERE id = ?")
            .bind(&kiosk_id)
            .fetch_optional(&state.db)
            .await?
            .ok_or_else(|| AppError::NotFound("kiosk no longer exists".into()))?;

    let display_name = sanitize_display_name(&body.display_name)?;
    let role = "presenter".to_string();

    let session_id = Uuid::new_v4().to_string();
    let participant_id = format!("web-{session_id}");
    let created_at = now_ms();
    let expires_at = created_at + seconds_to_ms(state.config.session_ttl_hours * 3600);

    sqlx::query(
        "INSERT INTO sessions (id, kiosk_id, participant_id, display_name, role, created_at, expires_at, revoked)
         VALUES (?, ?, ?, ?, ?, ?, ?, 0)",
    )
    .bind(&session_id)
    .bind(&kiosk_id)
    .bind(&participant_id)
    .bind(&display_name)
    .bind(&role)
    .bind(created_at)
    .bind(expires_at)
    .execute(&state.db)
    .await?;

    let access_token = mint_for_session(&state, &room_name, &participant_id, &display_name, &role)?;

    Ok(Json(JoinResponse {
        session_id,
        room_name,
        kiosk_name,
        livekit_url: state.config.livekit_url.clone(),
        access_token,
        participant_id,
        display_name,
        role,
        expires_at,
    }))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RefreshRequest {
    session_id: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RefreshResponse {
    room_name: String,
    livekit_url: String,
    access_token: String,
    participant_id: String,
    display_name: String,
    role: String,
    expires_at: i64,
}

async fn refresh(
    State(state): State<AppState>,
    Json(body): Json<RefreshRequest>,
) -> AppResult<Json<RefreshResponse>> {
    let row: Option<(String, String, String, i64, String)> = sqlx::query_as(
        "SELECT sessions.participant_id, sessions.display_name, sessions.role,
                sessions.expires_at, kiosks.room_name
         FROM sessions JOIN kiosks ON kiosks.id = sessions.kiosk_id
         WHERE sessions.id = ? AND sessions.revoked = 0",
    )
    .bind(&body.session_id)
    .fetch_optional(&state.db)
    .await?;

    let (participant_id, display_name, role, expires_at, room_name) =
        row.ok_or_else(|| AppError::Unauthorized("unknown session".into()))?;

    if expires_at <= now_ms() {
        return Err(AppError::Unauthorized("session expired".into()));
    }

    let access_token = mint_for_session(&state, &room_name, &participant_id, &display_name, &role)?;

    Ok(Json(RefreshResponse {
        room_name,
        livekit_url: state.config.livekit_url.clone(),
        access_token,
        participant_id,
        display_name,
        role,
        expires_at,
    }))
}

fn mint_for_session(
    state: &AppState,
    room_name: &str,
    participant_id: &str,
    display_name: &str,
    role: &str,
) -> AppResult<String> {
    // Everyone who joins may publish. Only one screen share runs at a time, and that is
    // enforced by the clients against the room's live tracks rather than by the token.
    let can_publish = true;
    let (token, _) = mint_access_token(
        &state.config.livekit_api_key,
        &state.config.livekit_api_secret,
        TokenRequest {
            room_name,
            identity: participant_id,
            display_name,
            role,
            can_publish,
            room_admin: can_publish,
            ttl_seconds: state.config.session_ttl_hours * 3600,
        },
    )?;
    Ok(token)
}

fn sanitize_display_name(raw: &str) -> AppResult<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(AppError::BadRequest("please enter your name".into()));
    }
    Ok(trimmed.chars().take(32).collect())
}
