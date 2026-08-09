use rand::Rng;
use serde::Serialize;
use sqlx::SqlitePool;

use crate::clock::{now_ms, seconds_to_ms};
use crate::error::{AppError, AppResult};
use crate::state::AppState;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IssuedPin {
    pub pin: String,
    pub issued_at: i64,
    pub expires_at: i64,
}

pub async fn issue_pin(
    db: &SqlitePool,
    kiosk_id: &str,
    rotation_seconds: i64,
    grace_seconds: i64,
) -> AppResult<IssuedPin> {
    let issued_at = now_ms();
    let expires_at = issued_at + seconds_to_ms(rotation_seconds + grace_seconds);
    let pin = allocate_unique_pin(db, issued_at).await?;

    sqlx::query(
        "INSERT INTO kiosk_pins (pin, kiosk_id, issued_at, expires_at) VALUES (?, ?, ?, ?)",
    )
    .bind(&pin)
    .bind(kiosk_id)
    .bind(issued_at)
    .bind(expires_at)
    .execute(db)
    .await?;

    Ok(IssuedPin {
        pin,
        issued_at,
        expires_at,
    })
}

pub async fn current_pin(db: &SqlitePool, kiosk_id: &str) -> AppResult<Option<IssuedPin>> {
    let row: Option<(String, i64, i64)> = sqlx::query_as(
        "SELECT pin, issued_at, expires_at FROM kiosk_pins
         WHERE kiosk_id = ? AND expires_at > ?
         ORDER BY issued_at DESC LIMIT 1",
    )
    .bind(kiosk_id)
    .bind(now_ms())
    .fetch_optional(db)
    .await?;

    Ok(row.map(|(pin, issued_at, expires_at)| IssuedPin {
        pin,
        issued_at,
        expires_at,
    }))
}

pub async fn ensure_pin(
    db: &SqlitePool,
    kiosk_id: &str,
    rotation_seconds: i64,
    grace_seconds: i64,
) -> AppResult<IssuedPin> {
    match current_pin(db, kiosk_id).await? {
        Some(pin) => Ok(pin),
        None => issue_pin(db, kiosk_id, rotation_seconds, grace_seconds).await,
    }
}

pub async fn resolve_pin(db: &SqlitePool, pin: &str) -> AppResult<Option<String>> {
    let row: Option<(String,)> =
        sqlx::query_as("SELECT kiosk_id FROM kiosk_pins WHERE pin = ? AND expires_at > ?")
            .bind(pin)
            .bind(now_ms())
            .fetch_optional(db)
            .await?;

    Ok(row.map(|(kiosk_id,)| kiosk_id))
}

pub async fn purge_expired(db: &SqlitePool) -> AppResult<()> {
    sqlx::query("DELETE FROM kiosk_pins WHERE expires_at <= ?")
        .bind(now_ms())
        .execute(db)
        .await?;
    Ok(())
}

async fn allocate_unique_pin(db: &SqlitePool, now: i64) -> AppResult<String> {
    for _ in 0..32 {
        let candidate = format!("{:06}", rand::thread_rng().gen_range(0..1_000_000));
        let taken: Option<(String,)> =
            sqlx::query_as("SELECT pin FROM kiosk_pins WHERE pin = ? AND expires_at > ?")
                .bind(&candidate)
                .bind(now)
                .fetch_optional(db)
                .await?;
        if taken.is_none() {
            return Ok(candidate);
        }
    }
    Err(AppError::Internal("could not allocate a unique pin".into()))
}

pub fn spawn_rotation_task(state: AppState) {
    tokio::spawn(async move {
        let rotation = state.config.pin_rotation_seconds;
        let grace = state.config.pin_grace_seconds;
        let mut ticker =
            tokio::time::interval(std::time::Duration::from_secs(rotation.max(1) as u64));
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

        loop {
            ticker.tick().await;
            if let Err(error) = rotate_all(&state, rotation, grace).await {
                tracing::error!(%error, "pin rotation cycle failed");
            }
        }
    });
}

async fn rotate_all(state: &AppState, rotation: i64, grace: i64) -> AppResult<()> {
    purge_expired(&state.db).await?;

    let kiosks: Vec<(String,)> = sqlx::query_as("SELECT id FROM kiosks")
        .fetch_all(&state.db)
        .await?;

    for (kiosk_id,) in kiosks {
        issue_pin(&state.db, &kiosk_id, rotation, grace).await?;
    }

    Ok(())
}
