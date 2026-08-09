use std::path::Path;

use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::auth::hash_password;
use crate::clock::now_ms;
use crate::config::Config;
use crate::error::{AppError, AppResult};

pub async fn connect_database(config: &Config) -> AppResult<SqlitePool> {
    ensure_database_directory(&config.database_url)?;

    let pool = SqlitePoolOptions::new()
        .max_connections(8)
        .connect(&config.database_url)
        .await?;

    sqlx::query("PRAGMA journal_mode = WAL")
        .execute(&pool)
        .await?;
    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(&pool)
        .await?;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(|error| AppError::Internal(format!("migration failed: {error}")))?;

    Ok(pool)
}

pub async fn ensure_bootstrap_admin(pool: &SqlitePool, config: &Config) -> AppResult<()> {
    let (Some(email), Some(password)) = (
        config.bootstrap_admin_email.as_ref(),
        config.bootstrap_admin_password.as_ref(),
    ) else {
        return Ok(());
    };

    let email = email.trim().to_lowercase();
    if email.is_empty() || password.is_empty() {
        return Ok(());
    }

    let existing: Option<(String,)> = sqlx::query_as("SELECT id FROM admins WHERE email = ?")
        .bind(&email)
        .fetch_optional(pool)
        .await?;

    if existing.is_some() {
        return Ok(());
    }

    sqlx::query("INSERT INTO admins (id, email, password_hash, created_at) VALUES (?, ?, ?, ?)")
        .bind(Uuid::new_v4().to_string())
        .bind(&email)
        .bind(hash_password(password)?)
        .bind(now_ms())
        .execute(pool)
        .await?;

    tracing::info!(%email, "created bootstrap admin account");
    Ok(())
}

fn ensure_database_directory(database_url: &str) -> AppResult<()> {
    let without_scheme = database_url
        .strip_prefix("sqlite://")
        .or_else(|| database_url.strip_prefix("sqlite:"))
        .unwrap_or(database_url);

    let path = without_scheme.split('?').next().unwrap_or(without_scheme);
    if path.is_empty() || path == ":memory:" {
        return Ok(());
    }

    if let Some(parent) = Path::new(path).parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(|error| {
                AppError::Internal(format!("could not create database directory: {error}"))
            })?;
        }
    }

    Ok(())
}
