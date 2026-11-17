//! System Setting DTOs (§9.6, §17, §27).
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

use super::TenancyMode;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SettingItem {
    pub key: String,
    /// JSON text of the value (`"UTC"`, `12`, `true`).
    pub value: String,
    /// `string` | `int` | `bool` | `enum`
    pub kind: String,
    pub options: Vec<String>,
    pub min: Option<i64>,
    pub max: Option<i64>,
    pub category: String,
    pub label: String,
    pub description: String,
    #[serde(with = "time::serde::rfc3339::option")]
    pub updated_at: Option<OffsetDateTime>,
    pub updated_by: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct SettingsView {
    pub items: Vec<SettingItem>,
    /// Secret name → configured in the environment (never the value, §17).
    pub secrets: Vec<(String, bool)>,
    pub tenancy: TenancyMode,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FlagItem {
    pub key: String,
    pub description: String,
    pub enabled: bool,
    pub rollout_percent: i32,
    pub allowed_role_ids: Vec<Uuid>,
    pub beta_emails: Vec<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
    pub updated_by: Option<String>,
    /// On for everyone for 30+ days: time to remove it from the code (§27 hygiene).
    pub stale: bool,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct FlagUpdate {
    pub key: String,
    pub enabled: bool,
    pub rollout_percent: i32,
    pub allowed_role_ids: Vec<Uuid>,
    pub beta_emails: Vec<String>,
}

/// Tabs of the System Setting page and the categories they show (§30.3).
pub const TABS: &[(&str, &[&str])] = &[
    ("general", &["general"]),
    ("security", &["security"]),
    ("email", &["email"]),
    ("storage", &["storage", "backup"]),
    ("api", &["api"]),
    ("localization", &["localization"]),
];

/// The master switch shown at the top of a tab (§30.3).
pub fn master_switch(tab: &str) -> Option<&'static str> {
    match tab {
        "email" => Some("email.enabled"),
        "api" => Some("api.public_enabled"),
        "localization" => Some("i18n.enabled"),
        "storage" => Some("backup.schedule_enabled"),
        _ => None,
    }
}
