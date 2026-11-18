//! Notification DTOs (§23).
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NotificationRow {
    pub id: i64,
    /// Message key: `notif-<kind>` in `locales/*.ftl`, filled with `params`.
    pub kind: String,
    pub params: std::collections::HashMap<String, String>,
    pub link: Option<String>,
    pub read: bool,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct NotificationPage {
    pub rows: Vec<NotificationRow>,
    pub unread: i64,
    pub total: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NotificationPref {
    pub kind: String,
    pub in_app: bool,
    pub email: bool,
}
