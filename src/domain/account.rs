//! Email delivery log DTOs (§20, M13.7).
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmailLogRow {
    pub id: i64,
    pub to: String,
    pub template: String,
    /// pending | sent | failed | delivered | bounced | complained | suppressed
    pub status: String,
    pub attempts: i32,
    pub error: Option<String>,
    pub created_at: OffsetDateTime,
    pub sent_at: Option<OffsetDateTime>,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct EmailLogPage {
    pub rows: Vec<EmailLogRow>,
    pub total: i64,
}
