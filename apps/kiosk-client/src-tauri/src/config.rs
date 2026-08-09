use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

const CONFIG_FILE_NAME: &str = "kiosk.json";
const PROVISION_PATHS: [&str; 2] = ["/boot/firmware/pistation.json", "/boot/pistation.json"];

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KioskConfig {
    #[serde(default)]
    pub server_url: String,
    #[serde(default)]
    pub enrollment_token: String,
    #[serde(default)]
    pub kiosk_token: String,
    #[serde(default)]
    pub kiosk_id: String,
    #[serde(default)]
    pub room_name: String,
    #[serde(default)]
    pub livekit_url: String,
    #[serde(default)]
    pub join_url: String,
}

fn config_path(app: &AppHandle) -> Result<PathBuf, String> {
    let directory = app
        .path()
        .app_config_dir()
        .map_err(|error| format!("no config directory: {error}"))?;

    fs::create_dir_all(&directory).map_err(|error| format!("cannot create config dir: {error}"))?;
    Ok(directory.join(CONFIG_FILE_NAME))
}

fn read_provisioned() -> Option<KioskConfig> {
    for path in PROVISION_PATHS {
        let Ok(contents) = fs::read_to_string(path) else {
            continue;
        };
        if let Ok(config) = serde_json::from_str::<KioskConfig>(&contents) {
            return Some(config);
        }
    }
    None
}

fn env_value(key: &str) -> Option<String> {
    std::env::var(key).ok().filter(|value| !value.is_empty())
}

/// Works out the effective configuration.
///
/// Addresses come from, in order of increasing authority: what was saved last, the file on
/// the boot partition, then the environment. Someone who edits the boot file or sets a
/// variable is stating where this screen should point, and that has to beat whatever was
/// saved on a previous run, otherwise a kiosk can never be moved to a new server.
///
/// Credentials work the other way. The kiosk token is earned at enrolment and is only
/// discarded when it is provably useless, which is when the server address changes.
fn merge_provisioned(mut config: KioskConfig) -> KioskConfig {
    let provisioned = read_provisioned();

    let mut server_url = config.server_url.clone();
    let mut join_url = config.join_url.clone();
    let mut enrollment_token = config.enrollment_token.clone();

    if let Some(provisioned) = provisioned {
        if !provisioned.server_url.is_empty() {
            server_url = provisioned.server_url;
        }
        if !provisioned.join_url.is_empty() {
            join_url = provisioned.join_url;
        }
        if !provisioned.enrollment_token.is_empty() {
            enrollment_token = provisioned.enrollment_token;
        }
    }

    if let Some(value) = env_value("PISTATION_SERVER_URL") {
        server_url = value;
    }
    if let Some(value) = env_value("PISTATION_JOIN_URL") {
        join_url = value;
    }
    if let Some(value) = env_value("PISTATION_ENROLLMENT_TOKEN") {
        enrollment_token = value;
    }

    // A token issued by one server means nothing to another, so pointing somewhere new
    // has to force a fresh enrolment rather than looping on rejected requests.
    let is_moving = !config.server_url.is_empty() && config.server_url != server_url;
    if is_moving {
        eprintln!(
            "[pistation] server changed from {} to {}, re-enrolling",
            config.server_url, server_url
        );
        config.kiosk_token.clear();
        config.kiosk_id.clear();
    }

    config.server_url = server_url;
    config.join_url = join_url;

    // An enrolment token is only of use while there is no kiosk token to replace it.
    config.enrollment_token = if config.kiosk_token.is_empty() {
        enrollment_token
    } else {
        String::new()
    };

    config
}

#[tauri::command]
pub fn load_config(app: AppHandle) -> Result<KioskConfig, String> {
    let path = config_path(&app)?;

    let stored = fs::read_to_string(&path)
        .ok()
        .and_then(|contents| serde_json::from_str::<KioskConfig>(&contents).ok())
        .unwrap_or_default();

    Ok(merge_provisioned(stored))
}

#[tauri::command]
pub fn save_config(app: AppHandle, config: KioskConfig) -> Result<(), String> {
    let path = config_path(&app)?;
    let contents = serde_json::to_string_pretty(&config)
        .map_err(|error| format!("cannot serialize config: {error}"))?;

    fs::write(&path, contents).map_err(|error| format!("cannot write config: {error}"))
}

#[tauri::command]
pub fn hardware_id() -> String {
    for path in ["/etc/machine-id", "/var/lib/dbus/machine-id"] {
        if let Ok(contents) = fs::read_to_string(path) {
            let trimmed = contents.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }
    }

    hostname()
}

fn hostname() -> String {
    fs::read_to_string("/etc/hostname")
        .map(|value| value.trim().to_string())
        .unwrap_or_else(|_| "unknown-kiosk".to_string())
}
