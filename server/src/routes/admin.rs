use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, post, put};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::auth::{
    generate_opaque_token, hash_opaque_token, mint_admin_token, verify_password, AdminIdentity,
};
use crate::clock::now_ms;
use crate::error::{AppError, AppResult};
use crate::layout;
use crate::models::{derive_status, KioskRow, KioskView, KIOSK_COLUMNS};
use crate::pins;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/login", post(login))
        .route("/kiosks", get(list_kiosks).post(create_kiosk))
        .route(
            "/kiosks/{kiosk_id}",
            get(kiosk_detail).patch(update_kiosk).delete(delete_kiosk),
        )
        .route("/kiosks/{kiosk_id}/layout", put(update_layout))
        .route("/kiosks/{kiosk_id}/enrollment", post(rotate_enrollment))
}

#[derive(Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LoginResponse {
    access_token: String,
    email: String,
    expires_at: i64,
}

async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> AppResult<Json<LoginResponse>> {
    let email = body.email.trim().to_lowercase();

    let row: Option<(String, String)> =
        sqlx::query_as("SELECT id, password_hash FROM admins WHERE email = ?")
            .bind(&email)
            .fetch_optional(&state.db)
            .await?;

    let (admin_id, password_hash) =
        row.ok_or_else(|| AppError::Unauthorized("invalid email or password".into()))?;

    if !verify_password(&body.password, &password_hash) {
        return Err(AppError::Unauthorized("invalid email or password".into()));
    }

    let (access_token, expires_at) = mint_admin_token(
        &state.config.session_secret,
        &admin_id,
        &email,
        state.config.session_ttl_hours,
    )?;

    Ok(Json(LoginResponse {
        access_token,
        email,
        expires_at: expires_at * 1000,
    }))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct KioskListResponse {
    kiosks: Vec<KioskView>,
}

async fn list_kiosks(
    State(state): State<AppState>,
    _admin: AdminIdentity,
) -> AppResult<Json<KioskListResponse>> {
    let rows: Vec<KioskRow> =
        sqlx::query_as(&format!("SELECT {KIOSK_COLUMNS} FROM kiosks ORDER BY created_at"))
            .fetch_all(&state.db)
            .await?;

    let now = now_ms();
    let kiosks = rows
        .into_iter()
        .map(|row| {
            let mut view = KioskView::from(row);
            view.status = derive_status(view.last_seen_at, now).to_string();
            view
        })
        .collect();

    Ok(Json(KioskListResponse { kiosks }))
}

#[derive(Deserialize)]
struct CreateKioskRequest {
    name: String,
    #[serde(default)]
    location: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CreateKioskResponse {
    kiosk: KioskView,
    enrollment_token: String,
}

async fn create_kiosk(
    State(state): State<AppState>,
    _admin: AdminIdentity,
    Json(body): Json<CreateKioskRequest>,
) -> AppResult<Json<CreateKioskResponse>> {
    let name = body.name.trim();
    if name.is_empty() {
        return Err(AppError::BadRequest("kiosk name is required".into()));
    }

    let kiosk_id = Uuid::new_v4().to_string();
    let room_name = format!("kiosk-{}", &kiosk_id[..8]);
    let enrollment_token = generate_opaque_token();
    let created_at = now_ms();

    sqlx::query(
        "INSERT INTO kiosks (id, name, location, room_name, enrollment_token_hash, status, created_at)
         VALUES (?, ?, ?, ?, ?, 'offline', ?)",
    )
    .bind(&kiosk_id)
    .bind(name)
    .bind(body.location.trim())
    .bind(&room_name)
    .bind(hash_opaque_token(&enrollment_token))
    .bind(created_at)
    .execute(&state.db)
    .await?;

    layout::save_layout(&state.db, &kiosk_id, &layout::default_layout(&kiosk_id)).await?;
    pins::ensure_pin(
        &state.db,
        &kiosk_id,
        state.config.pin_rotation_seconds,
        state.config.pin_grace_seconds,
    )
    .await?;

    Ok(Json(CreateKioskResponse {
        kiosk: KioskView {
            kiosk_id,
            name: name.to_string(),
            location: body.location.trim().to_string(),
            room_name,
            status: "offline".to_string(),
            last_seen_at: None,
            created_at,
            metrics: None,
            metrics_at: None,
        },
        enrollment_token,
    }))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct KioskDetailResponse {
    kiosk: KioskView,
    layout: Value,
    current_pin: Option<String>,
}

async fn kiosk_detail(
    State(state): State<AppState>,
    _admin: AdminIdentity,
    Path(kiosk_id): Path<String>,
) -> AppResult<Json<KioskDetailResponse>> {
    let row: Option<KioskRow> =
        sqlx::query_as(&format!("SELECT {KIOSK_COLUMNS} FROM kiosks WHERE id = ?"))
            .bind(&kiosk_id)
            .fetch_optional(&state.db)
            .await?;

    let row = row.ok_or_else(|| AppError::NotFound("kiosk not found".into()))?;
    let mut kiosk = KioskView::from(row);
    kiosk.status = derive_status(kiosk.last_seen_at, now_ms()).to_string();

    let layout = layout::load_layout(&state.db, &kiosk_id).await?;
    let current_pin = pins::current_pin(&state.db, &kiosk_id)
        .await?
        .map(|issued| issued.pin);

    Ok(Json(KioskDetailResponse {
        kiosk,
        layout,
        current_pin,
    }))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateKioskRequest {
    name: String,
    #[serde(default)]
    location: String,
}

async fn update_kiosk(
    State(state): State<AppState>,
    _admin: AdminIdentity,
    Path(kiosk_id): Path<String>,
    Json(body): Json<UpdateKioskRequest>,
) -> AppResult<Json<KioskView>> {
    let name = body.name.trim();
    if name.is_empty() {
        return Err(AppError::BadRequest("kiosk name is required".into()));
    }

    let result = sqlx::query("UPDATE kiosks SET name = ?, location = ? WHERE id = ?")
        .bind(name)
        .bind(body.location.trim())
        .bind(&kiosk_id)
        .execute(&state.db)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("kiosk not found".into()));
    }

    let row: KioskRow = sqlx::query_as(&format!("SELECT {KIOSK_COLUMNS} FROM kiosks WHERE id = ?"))
        .bind(&kiosk_id)
        .fetch_one(&state.db)
        .await?;

    let mut kiosk = KioskView::from(row);
    kiosk.status = derive_status(kiosk.last_seen_at, now_ms()).to_string();

    Ok(Json(kiosk))
}

#[derive(Deserialize)]
struct UpdateLayoutRequest {
    layout: Value,
}

async fn update_layout(
    State(state): State<AppState>,
    _admin: AdminIdentity,
    Path(kiosk_id): Path<String>,
    Json(body): Json<UpdateLayoutRequest>,
) -> AppResult<Json<Value>> {
    let exists: Option<(String,)> = sqlx::query_as("SELECT id FROM kiosks WHERE id = ?")
        .bind(&kiosk_id)
        .fetch_optional(&state.db)
        .await?;

    if exists.is_none() {
        return Err(AppError::NotFound("kiosk not found".into()));
    }

    layout::save_layout(&state.db, &kiosk_id, &body.layout).await?;
    let stored = layout::load_layout(&state.db, &kiosk_id).await?;
    Ok(Json(stored))
}

async fn delete_kiosk(
    State(state): State<AppState>,
    _admin: AdminIdentity,
    Path(kiosk_id): Path<String>,
) -> AppResult<StatusCode> {
    let result = sqlx::query("DELETE FROM kiosks WHERE id = ?")
        .bind(&kiosk_id)
        .execute(&state.db)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("kiosk not found".into()));
    }

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EnrollmentResponse {
    enrollment_token: String,
}

async fn rotate_enrollment(
    State(state): State<AppState>,
    _admin: AdminIdentity,
    Path(kiosk_id): Path<String>,
) -> AppResult<Json<EnrollmentResponse>> {
    let enrollment_token = generate_opaque_token();

    let result = sqlx::query(
        "UPDATE kiosks SET enrollment_token_hash = ?, kiosk_token_hash = NULL WHERE id = ?",
    )
    .bind(hash_opaque_token(&enrollment_token))
    .bind(&kiosk_id)
    .execute(&state.db)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("kiosk not found".into()));
    }

    Ok(Json(EnrollmentResponse { enrollment_token }))
}
