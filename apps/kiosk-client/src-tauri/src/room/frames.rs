use std::sync::Mutex;
use std::time::{Duration, Instant};

use image::codecs::jpeg::JpegEncoder;
use image::ExtendedColorType;
use tauri::ipc::{Channel, InvokeResponseBody};

/// Where encoded frames go. The webview opens this channel once and keeps it for the life
/// of the app, so video never has to travel as an event payload.
static SINK: Mutex<Option<Channel<InvokeResponseBody>>> = Mutex::new(None);
static LAST_SENT: Mutex<Option<Instant>> = Mutex::new(None);

const DEFAULT_MAX_WIDTH: u32 = 1920;
const DEFAULT_FPS: u32 = 60;
const DEFAULT_QUALITY: u8 = 85;

/// Read once at startup. A Pi will not keep up with the desktop defaults, so these are
/// tunable without a rebuild: PISTATION_VIDEO_MAX_WIDTH, _FPS and _QUALITY.
pub struct VideoSettings {
    pub max_width: u32,
    pub frame_interval: Duration,
    pub quality: u8,
}

fn settings() -> &'static VideoSettings {
    static SETTINGS: std::sync::OnceLock<VideoSettings> = std::sync::OnceLock::new();

    SETTINGS.get_or_init(|| {
        let max_width = env_number("PISTATION_VIDEO_MAX_WIDTH", DEFAULT_MAX_WIDTH as u64) as u32;
        let fps = (env_number("PISTATION_VIDEO_FPS", DEFAULT_FPS as u64) as u32).clamp(1, 120);
        let quality = env_number("PISTATION_VIDEO_QUALITY", DEFAULT_QUALITY as u64).clamp(1, 100);

        VideoSettings {
            max_width: max_width.max(160),
            frame_interval: Duration::from_secs_f64(1.0 / fps as f64),
            quality: quality as u8,
        }
    })
}

fn env_number(key: &str, fallback: u64) -> u64 {
    std::env::var(key)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(fallback)
}

pub fn set_sink(channel: Option<Channel<InvokeResponseBody>>) {
    if let Ok(mut guard) = SINK.lock() {
        *guard = channel;
    }
    if let Ok(mut guard) = LAST_SENT.lock() {
        *guard = None;
    }
}

pub fn has_sink() -> bool {
    SINK.lock().map(|guard| guard.is_some()).unwrap_or(false)
}

/// Returns the size this frame should be encoded at, or None when it should be dropped
/// because the previous one went out too recently.
pub fn next_target(width: u32, height: u32) -> Option<(u32, u32)> {
    if width == 0 || height == 0 || !has_sink() {
        return None;
    }

    let config = settings();
    let mut guard = LAST_SENT.lock().ok()?;

    if let Some(last) = *guard {
        if last.elapsed() < config.frame_interval {
            return None;
        }
    }

    *guard = Some(Instant::now());

    if width <= config.max_width {
        return Some((width, height));
    }

    let scaled_height = ((height as f32) * (config.max_width as f32 / width as f32)).round() as u32;
    Some((config.max_width, scaled_height.max(1)))
}

/// Packs RGB into `rgb` from an R,G,B,A source and encodes into `encoded`.
///
/// Both buffers are owned by the caller and reused across frames. At sixty frames a second
/// the allocations alone were costing more than the encode.
pub fn send(source: &[u8], rgb: &mut Vec<u8>, encoded: &mut Vec<u8>, width: u32, height: u32) {
    let pixels = (width as usize) * (height as usize);
    rgb.clear();
    rgb.reserve(pixels * 3);

    for pixel in source.chunks_exact(4) {
        rgb.extend_from_slice(&pixel[..3]);
    }

    encoded.clear();
    let mut encoder = JpegEncoder::new_with_quality(&mut *encoded, settings().quality);

    if let Err(error) = encoder.encode(rgb, width, height, ExtendedColorType::Rgb8) {
        report_once(&format!("jpeg encode failed: {error}"));
        return;
    }

    let Ok(guard) = SINK.lock() else {
        return;
    };

    if let Some(channel) = guard.as_ref() {
        if let Err(error) = channel.send(InvokeResponseBody::Raw(std::mem::take(encoded))) {
            report_once(&format!("could not push a frame to the webview: {error}"));
        }
    }
}

/// Frames arrive many times a second, so a failure that repeats must not flood the log.
fn report_once(message: &str) {
    static REPORTED: Mutex<Option<String>> = Mutex::new(None);

    if let Ok(mut guard) = REPORTED.lock() {
        if guard.as_deref() == Some(message) {
            return;
        }
        *guard = Some(message.to_string());
    }

    eprintln!("[pistation] {message}");
}
