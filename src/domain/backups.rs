//! Backup DTOs (§9.6.1).
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BackupRow {
    pub id: Uuid,
    pub filename: String,
    pub size_bytes: i64,
    pub sha256: Option<String>,
    /// `running` | `completed` | `failed`
    pub status: String,
    /// `manual` | `scheduled` | `pre_restore`
    pub kind: String,
    pub error: Option<String>,
    pub created_by: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339::option")]
    pub finished_at: Option<OffsetDateTime>,
    pub encrypted: bool,
    /// Off-site copy: `None` (off) | `pending` | `done` | `failed`.
    pub offsite: Option<String>,
    pub offsite_error: Option<String>,
    #[serde(with = "time::serde::rfc3339::option")]
    pub verified_at: Option<OffsetDateTime>,
    pub verify_error: Option<String>,
}

/// Restore request (M12.7/8).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RestoreRequest {
    pub id: Uuid,
    /// Must equal the file name (typed confirmation).
    pub confirm: String,
    /// `primary` (local file) or `offsite` (download the copy).
    pub source: String,
    /// Private key / passphrase for an encrypted backup; used in memory only, never stored.
    pub secret: Option<String>,
    /// The acting admin's password (re-authentication).
    pub password: String,
}

/// "1.4 MB"
pub fn human_size(bytes: i64) -> String {
    let b = bytes.max(0) as f64;
    let units = ["B", "KB", "MB", "GB", "TB"];
    let mut v = b;
    let mut i = 0;
    while v >= 1024.0 && i < units.len() - 1 {
        v /= 1024.0;
        i += 1;
    }
    if i == 0 {
        format!("{} B", bytes.max(0))
    } else {
        format!("{v:.1} {}", units[i])
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn sizes() {
        assert_eq!(super::human_size(512), "512 B");
        assert_eq!(super::human_size(1536), "1.5 KB");
        assert_eq!(super::human_size(5 * 1024 * 1024), "5.0 MB");
    }
}
