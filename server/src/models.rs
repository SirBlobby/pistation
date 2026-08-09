use serde::Serialize;
use serde_json::Value;
use sqlx::FromRow;

#[derive(FromRow)]
pub struct KioskRow {
    pub id: String,
    pub name: String,
    pub location: String,
    pub room_name: String,
    pub status: String,
    pub last_seen_at: Option<i64>,
    pub created_at: i64,
    pub metrics: Option<String>,
    pub metrics_at: Option<i64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KioskView {
    pub kiosk_id: String,
    pub name: String,
    pub location: String,
    pub room_name: String,
    pub status: String,
    pub last_seen_at: Option<i64>,
    pub created_at: i64,
    pub metrics: Option<Value>,
    pub metrics_at: Option<i64>,
}

impl From<KioskRow> for KioskView {
    fn from(row: KioskRow) -> Self {
        Self {
            kiosk_id: row.id,
            name: row.name,
            location: row.location,
            room_name: row.room_name,
            status: row.status,
            last_seen_at: row.last_seen_at,
            created_at: row.created_at,
            metrics: row.metrics.and_then(|raw| serde_json::from_str(&raw).ok()),
            metrics_at: row.metrics_at,
        }
    }
}

pub const KIOSK_COLUMNS: &str =
    "id, name, location, room_name, status, last_seen_at, created_at, metrics, metrics_at";

pub const OFFLINE_AFTER_MS: i64 = 90_000;

pub fn derive_status(last_seen_at: Option<i64>, now: i64) -> &'static str {
    match last_seen_at {
        Some(seen) if now - seen <= OFFLINE_AFTER_MS => "online",
        _ => "offline",
    }
}
