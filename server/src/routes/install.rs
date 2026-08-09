use axum::extract::State;
use axum::http::header::{HeaderMap, CONTENT_TYPE};
use axum::http::HeaderValue;
use axum::response::{IntoResponse, Response};

use crate::state::AppState;

const INSTALL_SCRIPT: &str = include_str!("../../assets/install.sh");
const SERVER_URL_PLACEHOLDER: &str = "__PISTATION_SERVER_URL__";

pub async fn install_script(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let server_url = resolve_server_url(&state, &headers);
    let body = INSTALL_SCRIPT.replace(SERVER_URL_PLACEHOLDER, &server_url);

    let mut response = body.into_response();
    response.headers_mut().insert(
        CONTENT_TYPE,
        HeaderValue::from_static("text/x-shellscript; charset=utf-8"),
    );
    response
}

fn resolve_server_url(state: &AppState, headers: &HeaderMap) -> String {
    let host = header_value(headers, "x-forwarded-host").or_else(|| header_value(headers, "host"));

    if let Some(host) = host {
        let scheme =
            header_value(headers, "x-forwarded-proto").unwrap_or_else(|| "http".to_string());
        return format!("{scheme}://{host}");
    }

    if !state.config.public_api_url.is_empty() {
        return state
            .config
            .public_api_url
            .trim_end_matches('/')
            .to_string();
    }

    "http://localhost:8080".to_string()
}

fn header_value(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(|value| value.split(',').next().unwrap_or(value).trim().to_string())
        .filter(|value| !value.is_empty())
}
