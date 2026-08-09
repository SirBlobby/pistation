use axum::extract::State;
use axum::routing::{get, put};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::SqlitePool;

use crate::auth::AdminIdentity;
use crate::clock::now_ms;
use crate::error::AppResult;
use crate::state::AppState;

const SETTINGS_KEY: &str = "organization";

pub fn public_router() -> Router<AppState> {
    Router::new().route("/organization", get(read_public))
}

pub fn admin_router() -> Router<AppState> {
    Router::new().route("/organization", put(update))
}

fn defaults() -> Value {
    json!({
        "name": "PiStation",
        "headline": "Any screen becomes a shared screen.",
        "description": "Type the code shown on screen to present, draw on what is being shown, or open a whiteboard together. No accounts, no installs, and nothing leaves the network it runs on.",
        "logoUrl": "",
        "accentColor": "#4f7cff",
        "joinLabel": "Enter the code on screen",
        "footerNote": "",
        "showSourceLink": true,
        "landingMode": "full",
        "theme": {
            "surface0": "#0b0d10",
            "surface1": "#14181d",
            "surface2": "#1c2229",
            "surface3": "#262e37",
            "ink0": "#f4f6f8",
            "ink1": "#a8b3c0",
            "ink2": "#6b7885"
        },
        "links": [],
        "updatedAt": 0
    })
}

fn apply_defaults(mut branding: Value) -> Value {
    let Some(object) = branding.as_object_mut() else {
        return defaults();
    };

    if let Value::Object(fallback) = defaults() {
        for (key, value) in fallback {
            object.entry(key).or_insert(value);
        }
    }

    // The palette is nested, so a stored branding written before a colour existed still
    // needs that one filling in rather than the whole object being replaced.
    if let (Some(Value::Object(theme)), Value::Object(defaults)) =
        (object.get_mut("theme"), defaults()["theme"].clone())
    {
        for (key, value) in defaults {
            theme.entry(key).or_insert(value);
        }
    }

    branding
}

pub async fn load(db: &SqlitePool) -> AppResult<Value> {
    let row: Option<(String,)> = sqlx::query_as("SELECT value FROM settings WHERE key = ?")
        .bind(SETTINGS_KEY)
        .fetch_optional(db)
        .await?;

    let stored = row
        .and_then(|(value,)| serde_json::from_str(&value).ok())
        .unwrap_or_else(defaults);

    Ok(apply_defaults(stored))
}

async fn read_public(State(state): State<AppState>) -> AppResult<Json<Value>> {
    Ok(Json(load(&state.db).await?))
}

#[derive(Deserialize)]
struct UpdateRequest {
    branding: Value,
}

async fn update(
    State(state): State<AppState>,
    _admin: AdminIdentity,
    Json(body): Json<UpdateRequest>,
) -> AppResult<Json<Value>> {
    let now = now_ms();
    let mut branding = apply_defaults(body.branding);
    branding["updatedAt"] = json!(now);

    sqlx::query(
        "INSERT INTO settings (key, value, updated_at) VALUES (?, ?, ?)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
    )
    .bind(SETTINGS_KEY)
    .bind(branding.to_string())
    .bind(now)
    .execute(&state.db)
    .await?;

    Ok(Json(branding))
}
