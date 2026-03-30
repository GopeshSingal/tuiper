use std::time::{SystemTime, UNIX_EPOCH};

pub const MULTIPLAYER_GRACE_PERIOD_SECS: u64 = 5;

pub fn now_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
