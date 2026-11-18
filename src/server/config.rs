//! 12-factor config from env (+ `.env` in dev), see `.env.example` and §5.
use std::path::PathBuf;

use anyhow::{Context, bail};
use figment::{Figment, providers::Env};
use serde::Deserialize;

use crate::domain::TenancyMode;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AppEnv {
    #[default]
    Development,
    Production,
}

/// No `Debug` derive on purpose: this holds secrets.
#[derive(Clone, Deserialize)]
pub struct Config {
    pub database_url: String,
    #[serde(default)]
    pub app_env: AppEnv,
    pub app_secret: Option<String>,
    #[serde(default)]
    pub cookie_secure: bool,
    #[serde(default = "default_backup_dir")]
    pub backup_dir: PathBuf,
    /// Uploaded files with the `local` driver (`FILES_DIR`, default `./data/files`).
    #[serde(default = "default_files_dir")]
    pub files_dir: PathBuf,
    pub seed_admin_email: Option<String>,
    pub seed_admin_password: Option<String>,
    #[serde(default)]
    pub tenancy_mode: TenancyMode,
    #[serde(default = "default_db_max_connections")]
    pub db_max_connections: u32,
    /// Header carrying the client IP, set by the trusted reverse proxy (e.g. `x-forwarded-for`).
    /// Unset in local dev: the client IP is then unknown and rate limits fall back to per-email keys.
    pub client_ip_header: Option<String>,
    /// Public origin, e.g. `https://app.example.com` (no path). Used by the CSRF origin check
    /// and, from M13, for links in emails (§20).
    pub app_url: Option<String>,
    /// Headless Chrome DevTools endpoint for PDF export (`docker compose up -d chrome`).
    #[serde(default = "default_chrome_url")]
    pub chrome_url: String,
}

fn default_files_dir() -> PathBuf {
    PathBuf::from("./data/files")
}

fn default_backup_dir() -> PathBuf {
    "./data/backups".into()
}
fn default_chrome_url() -> String {
    "http://127.0.0.1:9223".into()
}
fn default_db_max_connections() -> u32 {
    10
}

const KEYS: &[&str] = &[
    "DATABASE_URL",
    "APP_ENV",
    "APP_SECRET",
    "COOKIE_SECURE",
    "BACKUP_DIR",
    "SEED_ADMIN_EMAIL",
    "SEED_ADMIN_PASSWORD",
    "TENANCY_MODE",
    "DB_MAX_CONNECTIONS",
    "CLIENT_IP_HEADER",
    "APP_URL",
    "CHROME_URL",
];

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        // `.env` is optional (absent in containers); real env vars win.
        let _ = dotenvy::dotenv();
        let cfg: Config = Figment::new()
            .merge(Env::raw().only(KEYS))
            .extract()
            .context("invalid configuration (see .env.example)")?;
        cfg.validate()?;
        Ok(cfg)
    }

    /// `__Host-` prefix requires `Secure`, `Path=/` and no `Domain` (M1.17).
    pub fn session_cookie_name(&self) -> &'static str {
        if self.cookie_secure {
            "__Host-sid"
        } else {
            "sid"
        }
    }

    pub fn is_production(&self) -> bool {
        self.app_env == AppEnv::Production
    }

    fn validate(&self) -> anyhow::Result<()> {
        if self.is_production() {
            match &self.app_secret {
                Some(s) if s.len() >= 32 => {}
                _ => bail!("APP_SECRET must be set and at least 32 bytes in production"),
            }
            if !self.cookie_secure {
                bail!("COOKIE_SECURE must be true in production");
            }
            if self.app_url.is_none() {
                bail!("APP_URL must be set in production (CSRF origin check, email links)");
            }
        } else if self.app_secret.as_deref().is_none_or(|s| s.len() < 32) {
            tracing::warn!("APP_SECRET missing or < 32 bytes; acceptable in development only");
        }
        Ok(())
    }
}
