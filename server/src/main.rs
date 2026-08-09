mod auth;
mod bootstrap;
mod clock;
mod config;
mod error;
mod layout;
mod livekit;
mod models;
mod pins;
mod routes;
mod state;

use axum::http::{HeaderValue, Method};
use axum::Router;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

use crate::config::Config;
use crate::state::AppState;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "pistation_server=info,tower_http=warn".into()),
        )
        .init();

    let config = match Config::from_env() {
        Ok(config) => config,
        Err(message) => {
            eprintln!("configuration error: {message}");
            std::process::exit(1);
        }
    };

    let pool = match bootstrap::connect_database(&config).await {
        Ok(pool) => pool,
        Err(error) => {
            eprintln!("database startup failed: {error}");
            std::process::exit(1);
        }
    };

    if let Err(error) = bootstrap::ensure_bootstrap_admin(&pool, &config).await {
        eprintln!("admin bootstrap failed: {error}");
        std::process::exit(1);
    }

    let bind_address = config.bind_address.clone();
    let cors = build_cors(&config);
    let state = AppState::new(config, pool);

    pins::spawn_rotation_task(state.clone());

    for directory in [&state.config.media_dir, &state.config.package_dir] {
        if let Err(error) = std::fs::create_dir_all(directory) {
            eprintln!("could not create directory {directory}: {error}");
            std::process::exit(1);
        }
    }

    let kiosk_cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_origin(Any);

    let app = Router::new()
        .route("/", axum::routing::get(routes::service_index))
        .route("/install.sh", axum::routing::get(routes::install_script))
        .nest("/api/kiosk", routes::kiosk_router().layer(kiosk_cors))
        .nest("/api", routes::api_router().layer(cors))
        .nest_service("/media", ServeDir::new(state.config.media_dir.clone()))
        .nest_service(
            "/downloads",
            ServeDir::new(state.config.package_dir.clone()),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let listener = match tokio::net::TcpListener::bind(&bind_address).await {
        Ok(listener) => listener,
        Err(error) => {
            eprintln!("could not bind {bind_address}: {error}");
            std::process::exit(1);
        }
    };

    tracing::info!(%bind_address, "pistation server listening");

    if let Err(error) = axum::serve(listener, app).await {
        eprintln!("server stopped: {error}");
        std::process::exit(1);
    }
}

fn build_cors(config: &Config) -> CorsLayer {
    let base = CorsLayer::new()
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers(Any);

    if config.cors_origins.is_empty() {
        return base.allow_origin(Any);
    }

    let origins: Vec<HeaderValue> = config
        .cors_origins
        .iter()
        .filter_map(|origin| origin.parse().ok())
        .collect();

    base.allow_origin(origins)
}
