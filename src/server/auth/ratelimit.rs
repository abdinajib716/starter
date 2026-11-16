//! In-memory fixed-window limiter for login (§7 step 1). Per instance only:
//! M14.2 replaces it with the Postgres limiter behind the same function signature.
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};

static HITS: LazyLock<Mutex<HashMap<String, (Instant, u32)>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Count one hit for `key`; `false` once more than `max` hits fall in the current window.
pub fn hit(key: &str, max: u32, window: Duration) -> bool {
    let now = Instant::now();
    let mut map = HITS.lock().unwrap_or_else(|p| p.into_inner());
    if map.len() > 10_000 {
        map.retain(|_, (start, _)| now.duration_since(*start) < window);
    }
    let entry = map.entry(key.to_owned()).or_insert((now, 0));
    if now.duration_since(entry.0) >= window {
        *entry = (now, 0);
    }
    entry.1 += 1;
    entry.1 <= max
}

pub fn reset(key: &str) {
    HITS.lock().unwrap_or_else(|p| p.into_inner()).remove(key);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_after_max_then_resets() {
        let w = Duration::from_secs(60);
        for _ in 0..3 {
            assert!(hit("t:a", 3, w));
        }
        assert!(!hit("t:a", 3, w));
        reset("t:a");
        assert!(hit("t:a", 3, w));
    }
}
