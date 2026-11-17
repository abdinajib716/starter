//! System settings (§9.6, §17, §30.2): the code registry of keys (type, default, rules), the
//! idempotent seed, validation, and the in-memory cache every instance keeps in step through
//! `pg_notify('settings_changed')` (§28) with a 60 s reload as a safety net.
use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use sqlx::PgPool;

/// How a value is stored and checked.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Kind {
    Str,
    /// Inclusive range.
    Int(i64, i64),
    Bool,
    Enum(&'static [&'static str]),
}

impl Kind {
    pub fn name(self) -> &'static str {
        match self {
            Kind::Str => "string",
            Kind::Int(..) => "int",
            Kind::Bool => "bool",
            Kind::Enum(_) => "enum",
        }
    }
}

pub struct Def {
    pub key: &'static str,
    pub kind: Kind,
    pub default: fn() -> Value,
    pub category: &'static str,
    pub label: &'static str,
    pub description: &'static str,
}

macro_rules! def {
    ($key:literal, $kind:expr, $default:expr, $cat:literal, $label:literal, $desc:literal) => {
        Def {
            key: $key,
            kind: $kind,
            default: || json!($default),
            category: $cat,
            label: $label,
            description: $desc,
        }
    };
}

pub const TWOFA_MODES: &[&str] = &["off", "optional", "required_roles", "required_all"];

/// Every setting the app knows. Seeded at startup (existing values are kept); the System
/// Setting page (M7) edits them, grouped by `category`.
pub const REGISTRY: &[Def] = &[
    // §9.6
    def!(
        "app.name",
        Kind::Str,
        "Starter",
        "general",
        "Application name",
        "Shown in the sidebar, title and emails"
    ),
    def!(
        "app.timezone",
        Kind::Str,
        "UTC",
        "general",
        "Time zone",
        "Default time zone (IANA name) for dates"
    ),
    def!(
        "security.session_hours",
        Kind::Int(1, 720),
        12,
        "security",
        "Session length (hours)",
        "Sliding session lifetime"
    ),
    def!(
        "security.max_failed_attempts",
        Kind::Int(1, 100),
        5,
        "security",
        "Failed sign-ins before lockout",
        ""
    ),
    def!(
        "security.lockout_minutes",
        Kind::Int(1, 1440),
        15,
        "security",
        "Lockout (minutes)",
        ""
    ),
    def!(
        "security.password_min_length",
        Kind::Int(8, 128),
        10,
        "security",
        "Minimum password length",
        ""
    ),
    def!(
        "backup.retention_count",
        Kind::Int(1, 1000),
        14,
        "backup",
        "Backups to keep",
        ""
    ),
    def!(
        "backup.schedule_enabled",
        Kind::Bool,
        true,
        "backup",
        "Scheduled backups",
        ""
    ),
    def!(
        "backup.schedule_cron",
        Kind::Str,
        "0 0 2 * * *",
        "backup",
        "Backup schedule (cron)",
        "sec min hour day month weekday"
    ),
    // §17 master switches
    def!(
        "email.enabled",
        Kind::Bool,
        false,
        "email",
        "Send email",
        "Off: no email is sent; admins set temporary passwords"
    ),
    def!(
        "auth.password_reset_enabled",
        Kind::Bool,
        true,
        "security",
        "Forgot password",
        "Needs email"
    ),
    def!(
        "auth.invites_enabled",
        Kind::Bool,
        true,
        "security",
        "Email invitations",
        "Needs email"
    ),
    def!(
        "auth.email_verification",
        Kind::Bool,
        false,
        "security",
        "Verify email addresses",
        "Needs email"
    ),
    def!(
        "security.2fa_mode",
        Kind::Enum(TWOFA_MODES),
        "optional",
        "security",
        "Two-factor authentication",
        ""
    ),
    def!(
        "security.2fa_grace_days",
        Kind::Int(0, 90),
        7,
        "security",
        "2FA grace period (days)",
        "0 = enroll at next sign-in"
    ),
    def!(
        "api.public_enabled",
        Kind::Bool,
        false,
        "api",
        "Public API",
        "Off: /api/v1 returns 404"
    ),
    def!(
        "notifications.enabled",
        Kind::Bool,
        true,
        "general",
        "In-app notifications",
        ""
    ),
    def!(
        "i18n.enabled",
        Kind::Bool,
        false,
        "localization",
        "Multiple languages",
        "Off: one language, no switcher"
    ),
    def!(
        "storage.primary_driver",
        Kind::Enum(&["local", "r2", "gdrive"]),
        "local",
        "storage",
        "File storage",
        "Credentials come from the environment"
    ),
    def!(
        "backup.offsite_enabled",
        Kind::Bool,
        false,
        "backup",
        "Off-site backup copies",
        ""
    ),
    def!(
        "export.max_rows",
        Kind::Int(1, 1_000_000),
        50_000,
        "general",
        "CSV export row limit",
        ""
    ),
    def!(
        "bulk.max_items",
        Kind::Int(1, 100_000),
        1000,
        "general",
        "Bulk action limit",
        ""
    ),
    def!(
        "users.deleted_retention_days",
        Kind::Int(0, 3650),
        0,
        "general",
        "Keep deleted users (days)",
        "0 = forever"
    ),
    // §30.2
    def!(
        "email.from_name",
        Kind::Str,
        "Starter",
        "email",
        "From name",
        ""
    ),
    def!(
        "email.from_address",
        Kind::Str,
        "",
        "email",
        "From address",
        ""
    ),
    def!(
        "email.reply_to",
        Kind::Str,
        "",
        "email",
        "Reply-to address",
        ""
    ),
    def!(
        "security.login_alerts",
        Kind::Bool,
        true,
        "security",
        "New sign-in alerts",
        ""
    ),
    def!(
        "security.2fa_issuer",
        Kind::Str,
        "Starter",
        "security",
        "2FA issuer name",
        "Shown in authenticator apps"
    ),
    def!(
        "api.token_default_ttl_days",
        Kind::Int(1, 3650),
        90,
        "api",
        "Token lifetime (days)",
        ""
    ),
    def!(
        "api.max_tokens_per_user",
        Kind::Int(1, 100),
        10,
        "api",
        "Tokens per user",
        ""
    ),
    def!(
        "api.rate_limit_per_min",
        Kind::Int(1, 100_000),
        120,
        "api",
        "Requests per minute",
        ""
    ),
    def!(
        "api.cors_origins",
        Kind::Str,
        "",
        "api",
        "CORS origins",
        "Comma-separated"
    ),
    def!(
        "i18n.default_locale",
        Kind::Str,
        "en",
        "localization",
        "Default language",
        ""
    ),
    def!(
        "i18n.enabled_locales",
        Kind::Str,
        "en,so",
        "localization",
        "Enabled languages",
        "Comma-separated"
    ),
    def!(
        "notifications.retention_days",
        Kind::Int(1, 3650),
        30,
        "general",
        "Keep read notifications (days)",
        ""
    ),
    def!(
        "backup.encrypt_mode",
        Kind::Enum(&["off", "age", "symmetric"]),
        "off",
        "backup",
        "Backup encryption",
        ""
    ),
    def!(
        "backup.offsite_driver",
        Kind::Enum(&["r2", "gdrive"]),
        "r2",
        "backup",
        "Off-site target",
        ""
    ),
    def!(
        "backup.offsite_retention_count",
        Kind::Int(1, 1000),
        30,
        "backup",
        "Off-site copies to keep",
        ""
    ),
];

pub fn def(key: &str) -> Option<&'static Def> {
    REGISTRY.iter().find(|d| d.key == key)
}

/// Check a value against its key's type and range. Returns the code of the broken rule.
#[allow(dead_code)] // used from M5 (users) / M6 (roles) / M7 (settings)
pub fn validate(key: &str, v: &Value) -> Result<(), &'static str> {
    let d = def(key).ok_or("settings.unknown_key")?;
    let ok = match d.kind {
        Kind::Str => v.as_str().is_some_and(|s| s.len() <= 500),
        Kind::Int(lo, hi) => v.as_i64().is_some_and(|n| (lo..=hi).contains(&n)),
        Kind::Bool => v.is_boolean(),
        Kind::Enum(opts) => v.as_str().is_some_and(|s| opts.contains(&s)),
    };
    if ok {
        Ok(())
    } else {
        Err("settings.invalid_value")
    }
}

/// Insert missing keys with their defaults; refresh type/category/label/description of
/// existing ones without touching their values.
pub async fn seed(pool: &PgPool) -> sqlx::Result<()> {
    let mut tx = pool.begin().await?;
    for d in REGISTRY {
        sqlx::query!(
            r#"INSERT INTO settings (key, value, value_type, category, label, description)
               VALUES ($1, $2, $3, $4, $5, $6)
               ON CONFLICT (key) DO UPDATE
               SET value_type = EXCLUDED.value_type, category = EXCLUDED.category,
                   label = EXCLUDED.label, description = EXCLUDED.description"#,
            d.key,
            (d.default)(),
            d.kind.name(),
            d.category,
            d.label,
            d.description,
        )
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await
}

// ---------- cache ----------

/// Values of every setting, shared by the whole process.
pub struct Cache {
    values: RwLock<HashMap<String, Value>>,
    loaded: RwLock<Instant>,
}

const TTL: Duration = Duration::from_secs(60);
static CACHE: OnceLock<Cache> = OnceLock::new();

/// The process cache once [`start`] ran; `None` in unit tests that never start it.
pub fn cache() -> Option<&'static Cache> {
    CACHE.get()
}

impl Cache {
    pub fn get(&self, key: &str) -> Value {
        self.values
            .read()
            .ok()
            .and_then(|m| m.get(key).cloned())
            .or_else(|| def(key).map(|d| (d.default)()))
            .unwrap_or(Value::Null)
    }

    pub async fn reload(&self, pool: &PgPool) -> sqlx::Result<()> {
        let map = load_all(pool).await?;
        if let Ok(mut v) = self.values.write() {
            *v = map;
        }
        if let Ok(mut t) = self.loaded.write() {
            *t = Instant::now();
        }
        Ok(())
    }
}

async fn load_all(pool: &PgPool) -> sqlx::Result<HashMap<String, Value>> {
    Ok(sqlx::query!("SELECT key, value FROM settings")
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(|r| (r.key, r.value))
        .collect())
}

/// Load the cache and keep it fresh: reload on every `settings_changed` notification from any
/// instance, and at least every 60 s in case a notification was missed (reconnects).
pub async fn start(pool: &PgPool) -> anyhow::Result<()> {
    let cache = Cache {
        values: RwLock::new(load_all(pool).await?),
        loaded: RwLock::new(Instant::now()),
    };
    let _ = CACHE.set(cache);
    let pool = pool.clone();
    tokio::spawn(async move {
        loop {
            if let Err(e) = listen(&pool).await {
                tracing::warn!(error = %e, "settings listener stopped; retrying in 5 s");
            }
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    });
    Ok(())
}

async fn listen(pool: &PgPool) -> sqlx::Result<()> {
    let mut listener = sqlx::postgres::PgListener::connect_with(pool).await?;
    listener.listen("settings_changed").await?;
    let cache = CACHE.get().expect("cache set before listening");
    cache.reload(pool).await?; // catch changes made while disconnected
    loop {
        match tokio::time::timeout(TTL, listener.recv()).await {
            Ok(Ok(n)) => {
                tracing::debug!(key = n.payload(), "settings changed");
                if n.payload().starts_with("flag:") {
                    super::flags::reload(pool).await?;
                } else {
                    cache.reload(pool).await?;
                }
            }
            Ok(Err(e)) => return Err(e),
            Err(_) => {
                // TTL safety net
                cache.reload(pool).await?;
                super::flags::reload(pool).await?;
            }
        }
    }
}

/// Current value: from the cache when running, else straight from the database (tests,
/// startup). Falls back to the registry default.
pub async fn value(pool: &PgPool, key: &str) -> Value {
    if let Some(c) = cache() {
        return c.get(key);
    }
    sqlx::query_scalar!("SELECT value FROM settings WHERE key = $1", key)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
        .or_else(|| def(key).map(|d| (d.default)()))
        .unwrap_or(Value::Null)
}

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
        let d = Self::default();
        let int = |v: Value, fallback: i64| v.as_i64().unwrap_or(fallback);
        Ok(Self {
            session_hours: int(value(pool, "security.session_hours").await, d.session_hours),
            max_failed_attempts: i32::try_from(int(
                value(pool, "security.max_failed_attempts").await,
                d.max_failed_attempts.into(),
            ))
            .unwrap_or(d.max_failed_attempts),
            lockout_minutes: int(
                value(pool, "security.lockout_minutes").await,
                d.lockout_minutes,
            ),
            password_min_length: usize::try_from(int(
                value(pool, "security.password_min_length").await,
                d.password_min_length as i64,
            ))
            .unwrap_or(d.password_min_length),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_is_consistent() {
        let mut keys: Vec<_> = REGISTRY.iter().map(|d| d.key).collect();
        keys.sort();
        keys.dedup();
        assert_eq!(keys.len(), REGISTRY.len(), "duplicate setting key");
        for d in REGISTRY {
            assert_eq!(
                validate(d.key, &(d.default)()),
                Ok(()),
                "default of {}",
                d.key
            );
        }
    }

    #[test]
    fn validation_rules() {
        assert!(validate("security.session_hours", &json!(24)).is_ok());
        assert_eq!(
            validate("security.session_hours", &json!(0)),
            Err("settings.invalid_value")
        );
        assert_eq!(
            validate("security.session_hours", &json!("12")),
            Err("settings.invalid_value")
        );
        assert_eq!(
            validate("security.2fa_mode", &json!("sometimes")),
            Err("settings.invalid_value")
        );
        assert!(validate("email.enabled", &json!(true)).is_ok());
        assert_eq!(validate("nope.key", &json!(1)), Err("settings.unknown_key"));
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn seed_keeps_existing_values(pool: PgPool) -> sqlx::Result<()> {
        seed(&pool).await?;
        sqlx::query("UPDATE settings SET value = '24' WHERE key = 'security.session_hours'")
            .execute(&pool)
            .await?;
        seed(&pool).await?;
        let n: i64 = sqlx::query_scalar("SELECT count(*) FROM settings")
            .fetch_one(&pool)
            .await?;
        assert_eq!(n as usize, REGISTRY.len());
        assert_eq!(SecuritySettings::load(&pool).await?.session_hours, 24);
        Ok(())
    }

    /// The trigger announces every change on `settings_changed` (what other instances hear).
    #[sqlx::test(migrations = "./migrations")]
    async fn changes_are_announced(pool: PgPool) -> sqlx::Result<()> {
        seed(&pool).await?;
        let mut l = sqlx::postgres::PgListener::connect_with(&pool).await?;
        l.listen("settings_changed").await?;
        sqlx::query("UPDATE settings SET value = 'true' WHERE key = 'email.enabled'")
            .execute(&pool)
            .await?;
        let n = tokio::time::timeout(Duration::from_secs(5), l.recv())
            .await
            .expect("notification")?;
        assert_eq!(n.payload(), "email.enabled");
        Ok(())
    }
}
