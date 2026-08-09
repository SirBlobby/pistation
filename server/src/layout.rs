use serde_json::{json, Value};
use sqlx::SqlitePool;

use crate::clock::now_ms;
use crate::error::AppResult;

pub fn default_background() -> Value {
    json!({
        "imageUrl": "",
        "images": [],
        "rotationSeconds": 60,
        "fit": "cover",
        "dim": 0.35
    })
}

/// Layouts saved before wallpaper rotation existed carry a single `imageUrl`. Fold it into
/// the list so clients only ever have to read `images`.
fn migrate_background(background: Option<&mut Value>) {
    let Some(Value::Object(background)) = background else {
        return;
    };

    let has_images = background
        .get("images")
        .and_then(|images| images.as_array())
        .map(|images| !images.is_empty())
        .unwrap_or(false);

    if has_images {
        return;
    }

    let legacy = background
        .get("imageUrl")
        .and_then(|url| url.as_str())
        .unwrap_or_default()
        .to_string();

    if !legacy.is_empty() {
        background.insert("images".into(), json!([legacy]));
    }
}

pub fn default_widget_style() -> Value {
    json!({
        "padding": 5,
        "align": "start",
        "verticalAlign": "center",
        "backgroundColor": "",
        "opacity": null
    })
}

pub fn default_night_mode() -> Value {
    json!({
        "enabled": false,
        "startTime": "22:00",
        "endTime": "06:30",
        "timeZone": "",
        "showPin": true,
        "brightness": 0.45
    })
}

pub fn default_layout(kiosk_id: &str) -> Value {
    json!({
        "kioskId": kiosk_id,
        "backgroundColor": "#0b0d10",
        "foregroundColor": "#f4f6f8",
        "widgetOpacity": 0.5,
        "background": default_background(),
        "nightMode": default_night_mode(),
        "widgets": [
            {
                "widgetId": "clock",
                "kind": "clock",
                "placement": { "column": 1, "row": 1, "columnSpan": 5, "rowSpan": 2 },
                "settings": { "timeZone": "UTC", "showSeconds": false, "showDate": true, "hour12": true },
                "style": default_widget_style(),
                "enabled": true
            },
            {
                "widgetId": "weather",
                "kind": "weather",
                "placement": { "column": 8, "row": 1, "columnSpan": 5, "rowSpan": 2 },
                "settings": {
                    "latitude": 38.8304,
                    "longitude": -77.3078,
                    "locationLabel": "Fairfax",
                    "units": "imperial"
                },
                "enabled": true
            },
            {
                "widgetId": "pin",
                "kind": "pin",
                "placement": { "column": 1, "row": 4, "columnSpan": 12, "rowSpan": 4 },
                "settings": { "label": "Join at", "showJoinUrl": true },
                "enabled": true
            }
        ],
        "customDefinitions": [],
        "updatedAt": now_ms()
    })
}

fn apply_defaults(kiosk_id: &str, mut layout: Value) -> Value {
    let Some(object) = layout.as_object_mut() else {
        return default_layout(kiosk_id);
    };

    object.entry("kioskId").or_insert_with(|| json!(kiosk_id));
    object
        .entry("backgroundColor")
        .or_insert_with(|| json!("#0b0d10"));
    object
        .entry("foregroundColor")
        .or_insert_with(|| json!("#f4f6f8"));
    object.entry("widgetOpacity").or_insert_with(|| json!(0.5));
    object
        .entry("background")
        .or_insert_with(default_background);
    object.entry("nightMode").or_insert_with(default_night_mode);
    object.entry("widgets").or_insert_with(|| json!([]));
    object
        .entry("customDefinitions")
        .or_insert_with(|| json!([]));
    object.entry("updatedAt").or_insert_with(|| json!(now_ms()));

    fill_nested_defaults(object.get_mut("background"), default_background());
    migrate_background(object.get_mut("background"));
    fill_nested_defaults(object.get_mut("nightMode"), default_night_mode());
    fill_widget_defaults(object.get_mut("widgets"));

    layout
}

fn fill_widget_defaults(widgets: Option<&mut Value>) {
    let Some(Value::Array(widgets)) = widgets else {
        return;
    };

    for widget in widgets.iter_mut() {
        let Some(object) = widget.as_object_mut() else {
            continue;
        };

        object.entry("style").or_insert_with(default_widget_style);
        fill_nested_defaults(object.get_mut("style"), default_widget_style());
    }
}

fn fill_nested_defaults(target: Option<&mut Value>, defaults: Value) {
    let (Some(Value::Object(target)), Value::Object(defaults)) = (target, defaults) else {
        return;
    };

    for (key, value) in defaults {
        target.entry(key).or_insert(value);
    }
}

pub async fn load_layout(db: &SqlitePool, kiosk_id: &str) -> AppResult<Value> {
    let row: Option<(String,)> =
        sqlx::query_as("SELECT data FROM kiosk_layouts WHERE kiosk_id = ?")
            .bind(kiosk_id)
            .fetch_optional(db)
            .await?;

    let stored = match row {
        Some((data,)) => serde_json::from_str(&data).unwrap_or_else(|_| default_layout(kiosk_id)),
        None => default_layout(kiosk_id),
    };

    Ok(apply_defaults(kiosk_id, stored))
}

pub async fn save_layout(db: &SqlitePool, kiosk_id: &str, layout: &Value) -> AppResult<()> {
    let now = now_ms();
    let mut stored = apply_defaults(kiosk_id, layout.clone());
    stored["kioskId"] = json!(kiosk_id);
    stored["updatedAt"] = json!(now);

    sqlx::query(
        "INSERT INTO kiosk_layouts (kiosk_id, data, updated_at) VALUES (?, ?, ?)
         ON CONFLICT(kiosk_id) DO UPDATE SET data = excluded.data, updated_at = excluded.updated_at",
    )
    .bind(kiosk_id)
    .bind(stored.to_string())
    .bind(now)
    .execute(db)
    .await?;

    Ok(())
}
