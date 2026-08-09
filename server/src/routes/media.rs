use axum::body::Bytes;
use axum::extract::{DefaultBodyLimit, Path, State};
use axum::http::HeaderMap;
use axum::routing::put;
use axum::{Json, Router};
use serde::Serialize;
use uuid::Uuid;

use crate::auth::AdminIdentity;
use crate::error::{AppError, AppResult};
use crate::state::AppState;

pub const MAX_UPLOAD_BYTES: usize = 8 * 1024 * 1024;

const ALLOWED_TYPES: [(&str, &str); 4] = [
    ("image/png", "png"),
    ("image/jpeg", "jpg"),
    ("image/webp", "webp"),
    ("image/gif", "gif"),
];

pub const MAX_PACKAGE_BYTES: usize = 256 * 1024 * 1024;

/// Validates an uploaded image and writes it under the media directory, returning the path
/// clients should store. Shared by kiosk wallpapers and the organisation logo.
pub async fn store_image(
    state: &AppState,
    headers: &HeaderMap,
    body: &Bytes,
    prefix: &str,
) -> AppResult<String> {
    if body.is_empty() {
        return Err(AppError::BadRequest("the uploaded file was empty".into()));
    }

    if body.len() > MAX_UPLOAD_BYTES {
        return Err(AppError::BadRequest(
            "images must be 8 MB or smaller".into(),
        ));
    }

    let content_type = headers
        .get(axum::http::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(|value| {
            value
                .split(';')
                .next()
                .unwrap_or(value)
                .trim()
                .to_lowercase()
        })
        .unwrap_or_default();

    let extension = ALLOWED_TYPES
        .iter()
        .find(|(mime, _)| *mime == content_type)
        .map(|(_, extension)| *extension)
        .ok_or_else(|| AppError::BadRequest("images must be a PNG, JPEG, WEBP or GIF".into()))?;

    let directory = std::path::Path::new(&state.config.media_dir);
    tokio::fs::create_dir_all(directory)
        .await
        .map_err(|error| AppError::Internal(format!("cannot create media directory: {error}")))?;

    let file_name = format!("{prefix}-{}.{extension}", Uuid::new_v4());
    tokio::fs::write(directory.join(&file_name), body)
        .await
        .map_err(|error| AppError::Internal(format!("cannot write image: {error}")))?;

    Ok(format!("/media/{file_name}"))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/kiosks/{kiosk_id}/wallpaper", put(upload_wallpaper))
        .route("/organization/logo", put(upload_logo))
        .layer(DefaultBodyLimit::max(MAX_UPLOAD_BYTES))
        .merge(
            Router::new()
                .route("/packages/kiosk", put(upload_package))
                .layer(DefaultBodyLimit::max(MAX_PACKAGE_BYTES)),
        )
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PackageResponse {
    download_url: String,
    size_bytes: usize,
}

async fn upload_package(
    State(state): State<AppState>,
    _admin: AdminIdentity,
    body: Bytes,
) -> AppResult<Json<PackageResponse>> {
    if body.is_empty() {
        return Err(AppError::BadRequest("the uploaded file was empty".into()));
    }

    if !body.starts_with(b"!<arch>\ndebian-binary") {
        return Err(AppError::BadRequest(
            "that does not look like a .deb package".into(),
        ));
    }

    let directory = std::path::Path::new(&state.config.package_dir);
    tokio::fs::create_dir_all(directory)
        .await
        .map_err(|error| AppError::Internal(format!("cannot create package directory: {error}")))?;

    tokio::fs::write(directory.join("pistation-kiosk.deb"), &body)
        .await
        .map_err(|error| AppError::Internal(format!("cannot write package: {error}")))?;

    Ok(Json(PackageResponse {
        download_url: "/downloads/pistation-kiosk.deb".to_string(),
        size_bytes: body.len(),
    }))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct UploadResponse {
    image_url: String,
}

async fn upload_wallpaper(
    State(state): State<AppState>,
    _admin: AdminIdentity,
    Path(kiosk_id): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> AppResult<Json<UploadResponse>> {
    let exists: Option<(String,)> = sqlx::query_as("SELECT id FROM kiosks WHERE id = ?")
        .bind(&kiosk_id)
        .fetch_optional(&state.db)
        .await?;

    if exists.is_none() {
        return Err(AppError::NotFound("kiosk not found".into()));
    }

    let image_url = store_image(&state, &headers, &body, &kiosk_id).await?;
    Ok(Json(UploadResponse { image_url }))
}

async fn upload_logo(
    State(state): State<AppState>,
    _admin: AdminIdentity,
    headers: HeaderMap,
    body: Bytes,
) -> AppResult<Json<UploadResponse>> {
    let image_url = store_image(&state, &headers, &body, "logo").await?;
    Ok(Json(UploadResponse { image_url }))
}
