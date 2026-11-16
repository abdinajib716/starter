//! Typed reads of the `settings` table (§9.6). M1 reads directly with defaults;
//! M2.8/M2.9 add the seeds and the NOTIFY-invalidated cache.
use std::collections::HashMap;

use serde_json::Value;
use sqlx::PgPool;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SecuritySettings {
    pub session_hours: i64,
    pub max_failed_attempts: i32,
    pub lockout_minutes: i64,
    pub password_min_length: usize,
}

impl Default for SecuritySettings {
    fn default() -> Self {
        Self {
            session_hours: 12,
            max_failed_attempts: 5,
            lockout_minutes: 15,
            password_min_length: 10,
        }
    }
}

impl SecuritySettings {
    pub async fn load(pool: &PgPool) -> sqlx::Result<Self> {
        let rows = sqlx::query!(
            "SELECT key, value FROM settings WHERE key = ANY($1)",
            &[
                "security.session_hours",
                "security.max_failed_attempts",
                "security.lockout_minutes",
                "security.password_min_length",
            ][..] as &[&str]
        )
        .fetch_all(pool)
        .await?;
        let map: HashMap<String, Value> = rows.into_iter().map(|r| (r.key, r.value)).collect();
        let int = |key: &str| map.get(key).and_then(Value::as_i64);
        let d = Self::default();
        Ok(Self {
            session_hours: int("security.session_hours").unwrap_or(d.session_hours),
            max_failed_attempts: int("security.max_failed_attempts")
                .and_then(|v| i32::try_from(v).ok())
                .unwrap_or(d.max_failed_attempts),
            lockout_minutes: int("security.lockout_minutes").unwrap_or(d.lockout_minutes),
            password_min_length: int("security.password_min_length")
                .and_then(|v| usize::try_from(v).ok())
                .unwrap_or(d.password_min_length),
        })
    }
}
