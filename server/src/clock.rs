use chrono::Utc;

pub fn now_ms() -> i64 {
    Utc::now().timestamp_millis()
}

pub fn seconds_to_ms(seconds: i64) -> i64 {
    seconds * 1000
}
