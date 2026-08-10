pub mod admin;
pub mod install;
pub mod join;
pub mod kiosk;
pub mod media;
pub mod organization;
pub mod whiteboard;

pub use install::install_script;

use axum::routing::get;
use axum::{Json, Router};
use serde_json::{json, Value};

use crate::state::AppState;

/// Routes browsers call. These carry the configured origin allowlist.
pub fn api_router() -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
        .merge(join::router())
        .merge(organization::public_router())
        .merge(whiteboard::router())
        .nest(
            "/admin",
            admin::router()
                .merge(media::router())
                .merge(organization::admin_router()),
        )
}

/// Routes only kiosks call, mounted separately because they are not called from a web
/// origin. A Tauri webview reports `http://tauri.localhost` in production and whatever the
/// dev server uses otherwise, so an origin allowlist can only ever lock kiosks out. These
/// endpoints authenticate with a bearer token and never with a cookie, so an origin check
/// would add no protection anyway.
pub fn kiosk_router() -> Router<AppState> {
    kiosk::router()
}

async fn health() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}

pub async fn service_index() -> Json<Value> {
    Json(json!({
        "service": "pistation-server",
        "message": "This is the PiStation API. The web client is served separately, by default on port 3000.",
        "endpoints": {
            "health": "/api/health",
            "join": "POST /api/join",
            "admin": "POST /api/admin/login"
        }
    }))
}
