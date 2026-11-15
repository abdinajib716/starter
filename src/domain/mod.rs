//! Shared DTOs + validation; compiled for both the server and wasm (§31.2).
use serde::{Deserialize, Serialize};

/// `TENANCY_MODE` (§18). Install-time, read-only in the UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TenancyMode {
    #[default]
    Single,
    Multi,
}

impl std::fmt::Display for TenancyMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Single => "single",
            Self::Multi => "multi",
        })
    }
}

/// Public, non-sensitive server status (used by the home page and `/healthz`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServerInfo {
    pub version: String,
    pub tenancy: TenancyMode,
    pub db: String,
}
