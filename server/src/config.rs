use std::env;

#[derive(Clone)]
pub struct Config {
    pub bind_address: String,
    pub database_url: String,
    pub livekit_url: String,
    pub livekit_api_key: String,
    pub livekit_api_secret: String,
    pub session_secret: String,
    pub session_ttl_hours: i64,
    pub bootstrap_admin_email: Option<String>,
    pub bootstrap_admin_password: Option<String>,
    pub pin_rotation_seconds: i64,
    pub pin_grace_seconds: i64,
    pub public_web_url: String,
    pub cors_origins: Vec<String>,
    pub media_dir: String,
    pub package_dir: String,
    pub public_api_url: String,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        Ok(Self {
            bind_address: optional("BIND_ADDRESS", "0.0.0.0:8080"),
            database_url: optional("DATABASE_URL", "sqlite://data/pistation.db?mode=rwc"),
            livekit_url: required("LIVEKIT_URL")?,
            livekit_api_key: required("LIVEKIT_API_KEY")?,
            livekit_api_secret: required("LIVEKIT_API_SECRET")?,
            session_secret: required("SESSION_SECRET")?,
            session_ttl_hours: number("SESSION_TTL_HOURS", 12),
            bootstrap_admin_email: env::var("BOOTSTRAP_ADMIN_EMAIL").ok(),
            bootstrap_admin_password: env::var("BOOTSTRAP_ADMIN_PASSWORD").ok(),
            pin_rotation_seconds: number("PIN_ROTATION_SECONDS", 45).clamp(30, 60),
            pin_grace_seconds: number("PIN_GRACE_SECONDS", 15).clamp(0, 60),
            public_web_url: optional("PUBLIC_WEB_URL", "http://localhost:5173"),
            cors_origins: list("CORS_ORIGINS"),
            media_dir: optional("MEDIA_DIR", "data/media"),
            package_dir: optional("PACKAGE_DIR", "data/packages"),
            public_api_url: optional("PUBLIC_API_URL", ""),
        })
    }
}

fn required(key: &str) -> Result<String, String> {
    env::var(key).map_err(|_| format!("missing required environment variable {key}"))
}

fn optional(key: &str, fallback: &str) -> String {
    env::var(key).unwrap_or_else(|_| fallback.to_string())
}

fn number(key: &str, fallback: i64) -> i64 {
    env::var(key)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(fallback)
}

fn list(key: &str) -> Vec<String> {
    env::var(key)
        .unwrap_or_default()
        .split(',')
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect()
}
