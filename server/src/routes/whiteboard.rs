use axum::body::Bytes;
use axum::extract::{DefaultBodyLimit, State};
use axum::http::HeaderMap;
use axum::routing::put;
use axum::{Json, Router};
use serde::Serialize;

use crate::auth::SessionIdentity;
use crate::error::AppResult;
use crate::routes::media::{store_image, MAX_UPLOAD_BYTES};
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/whiteboard/image", put(upload_image))
        .layer(DefaultBodyLimit::max(MAX_UPLOAD_BYTES))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct UploadResponse {
    image_url: String,
}

async fn upload_image(
    State(state): State<AppState>,
    session: SessionIdentity,
    headers: HeaderMap,
    body: Bytes,
) -> AppResult<Json<UploadResponse>> {
    let prefix = format!("wb-{}", session.kiosk_id);
    let image_url = store_image(&state, &headers, &body, &prefix).await?;
    Ok(Json(UploadResponse { image_url }))
}
