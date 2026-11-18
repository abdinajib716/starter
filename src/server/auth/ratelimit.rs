//! Fixed-window rate limiter (§7 step 1, M14.2). With the app state up it counts in Postgres
//! (`rate_limits`, shared by every instance); without it (unit tests, CLI) in memory.
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};

static HITS: LazyLock<Mutex<HashMap<String, (Instant, u32)>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn pool() -> Option<sqlx::PgPool> {
    crate::server::state::get().ok().map(|s| s.pool.clone())
}

/// Count one hit for `key`; `false` once more than `max` hits fall in the current window.
/// A database error fails open (logged): the limiter must never lock everyone out.
pub async fn hit(key: &str, max: u32, window: Duration) -> bool {
    if let Some(pool) = pool() {
        let secs = window.as_secs_f64();
        let n = sqlx::query_scalar!(
            r#"INSERT INTO rate_limits (key, window_start, count) VALUES ($1, now(), 1)
               ON CONFLICT (key) DO UPDATE SET
                 count = CASE WHEN rate_limits.window_start < now() - make_interval(secs => $2)
                              THEN 1 ELSE rate_limits.count + 1 END,
                 window_start = CASE WHEN rate_limits.window_start < now() - make_interval(secs => $2)
                                     THEN now() ELSE rate_limits.window_start END
               RETURNING count"#,
            key,
            secs
        )
        .fetch_one(&pool)
        .await;
        return match n {
            Ok(n) => u32::try_from(n).unwrap_or(u32::MAX) <= max,
            Err(e) => {
                tracing::error!("rate limiter: {e}");
                true
            }
        };
    }
    memory_hit(key, max, window)
}

fn memory_hit(key: &str, max: u32, window: Duration) -> bool {
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

pub async fn reset(key: &str) {
    if let Some(pool) = pool() {
        let _ = sqlx::query!("DELETE FROM rate_limits WHERE key = $1", key)
            .execute(&pool)
            .await;
    }
    HITS.lock().unwrap_or_else(|p| p.into_inner()).remove(key);
}

/// Drop windows older than a day (hourly job).
pub async fn purge(pool: &sqlx::PgPool) -> sqlx::Result<u64> {
    Ok(
        sqlx::query!("DELETE FROM rate_limits WHERE window_start < now() - interval '1 day'")
            .execute(pool)
            .await?
            .rows_affected(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn blocks_after_max_then_resets() {
        let w = Duration::from_secs(60);
        for _ in 0..3 {
            assert!(hit("t:a", 3, w).await);
        }
        assert!(!hit("t:a", 3, w).await);
        reset("t:a").await;
        assert!(hit("t:a", 3, w).await);
    }
}
