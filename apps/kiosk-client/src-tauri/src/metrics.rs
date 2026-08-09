use std::fs;
use std::sync::Mutex;

use serde::Serialize;

/// Previous CPU sample, so usage can be reported as a delta between heartbeats rather
/// than as the meaningless since-boot average.
static LAST_CPU_SAMPLE: Mutex<Option<CpuSample>> = Mutex::new(None);

#[derive(Clone, Copy)]
struct CpuSample {
    total: u64,
    idle: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WifiMetrics {
    pub interface: String,
    pub link_quality: f32,
    pub signal_dbm: f32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Metrics {
    pub cpu_percent: Option<f32>,
    pub memory_used_bytes: u64,
    pub memory_total_bytes: u64,
    pub uptime_seconds: u64,
    pub temperature_celsius: Option<f32>,
    pub wifi: Option<WifiMetrics>,
}

#[tauri::command]
pub fn collect_metrics() -> Metrics {
    let memory = read_memory().unwrap_or((0, 0));

    Metrics {
        cpu_percent: read_cpu_percent(),
        memory_used_bytes: memory.0,
        memory_total_bytes: memory.1,
        uptime_seconds: read_uptime().unwrap_or(0),
        temperature_celsius: read_temperature(),
        wifi: read_wifi(),
    }
}

fn read_cpu_percent() -> Option<f32> {
    let contents = fs::read_to_string("/proc/stat").ok()?;
    let line = contents.lines().next()?;
    if !line.starts_with("cpu ") {
        return None;
    }

    let values: Vec<u64> = line
        .split_whitespace()
        .skip(1)
        .filter_map(|value| value.parse().ok())
        .collect();

    if values.len() < 5 {
        return None;
    }

    let total: u64 = values.iter().sum();
    let idle = values[3] + values[4];
    let sample = CpuSample { total, idle };

    let mut guard = LAST_CPU_SAMPLE.lock().ok()?;
    let previous = guard.replace(sample);

    let previous = previous?;
    let total_delta = total.checked_sub(previous.total)?;
    let idle_delta = idle.checked_sub(previous.idle)?;

    if total_delta == 0 {
        return None;
    }

    let busy = total_delta.saturating_sub(idle_delta) as f32;
    Some((busy / total_delta as f32) * 100.0)
}

fn read_memory() -> Option<(u64, u64)> {
    let contents = fs::read_to_string("/proc/meminfo").ok()?;
    let mut total_kb = 0u64;
    let mut available_kb = 0u64;

    for line in contents.lines() {
        let mut parts = line.split_whitespace();
        let key = parts.next()?;
        let value: u64 = parts.next().and_then(|value| value.parse().ok())?;

        match key {
            "MemTotal:" => total_kb = value,
            "MemAvailable:" => available_kb = value,
            _ => {}
        }

        if total_kb > 0 && available_kb > 0 {
            break;
        }
    }

    if total_kb == 0 {
        return None;
    }

    let used_kb = total_kb.saturating_sub(available_kb);
    Some((used_kb * 1024, total_kb * 1024))
}

fn read_uptime() -> Option<u64> {
    let contents = fs::read_to_string("/proc/uptime").ok()?;
    let seconds: f64 = contents.split_whitespace().next()?.parse().ok()?;
    Some(seconds as u64)
}

fn read_temperature() -> Option<f32> {
    let raw = fs::read_to_string("/sys/class/thermal/thermal_zone0/temp").ok()?;
    let millidegrees: f32 = raw.trim().parse().ok()?;
    Some(millidegrees / 1000.0)
}

fn read_wifi() -> Option<WifiMetrics> {
    let contents = fs::read_to_string("/proc/net/wireless").ok()?;

    for line in contents.lines().skip(2) {
        let Some((interface, rest)) = line.split_once(':') else {
            continue;
        };

        let values: Vec<&str> = rest.split_whitespace().collect();
        if values.len() < 3 {
            continue;
        }

        let link_quality = parse_wireless_number(values[1])?;
        let signal_dbm = parse_wireless_number(values[2])?;

        return Some(WifiMetrics {
            interface: interface.trim().to_string(),
            link_quality,
            signal_dbm,
        });
    }

    None
}

fn parse_wireless_number(raw: &str) -> Option<f32> {
    raw.trim_end_matches('.').parse().ok()
}
