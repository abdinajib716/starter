//! Dashboard DTOs (§10): one struct, one round trip.
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct DashboardData {
    pub total_users: i64,
    /// Users created in the last 30 days (the delta shown under the total).
    pub new_users_30d: i64,
    pub active_30d: i64,
    pub active_prev_30d: i64,
    pub online_now: i64,
    pub logins_today: i64,
    pub logins_yesterday: i64,
    pub failed_24h: i64,
    pub failed_prev_24h: i64,
    pub last_backup: Option<BackupInfo>,
    /// Last 14 days, oldest first: (ISO date, logins, failed logins).
    pub logins_14d: Vec<(String, i64, i64)>,
    /// Last 7 days: (module, audit rows), most active first.
    pub activity_by_module: Vec<(String, i64)>,
    /// (role name, active users).
    pub users_by_role: Vec<(String, i64)>,
    /// Last 12 weeks, oldest first: (ISO date of the Monday, new users).
    pub new_users_12w: Vec<(String, i64)>,
    pub recent: Vec<ActivityItem>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BackupInfo {
    pub status: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActivityItem {
    pub id: i64,
    #[serde(with = "time::serde::rfc3339")]
    pub at: OffsetDateTime,
    pub actor_email: Option<String>,
    pub action: String,
    pub success: bool,
}
