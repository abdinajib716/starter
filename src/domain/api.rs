//! Public API token DTOs (§19, M15.4).
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TokenRow {
    pub id: Uuid,
    pub name: String,
    pub prefix: String,
    pub scopes: Vec<String>,
    pub owner: String,
    pub expires_at: Option<OffsetDateTime>,
    pub last_used_at: Option<OffsetDateTime>,
    pub revoked: bool,
    pub created_at: OffsetDateTime,
}

/// Shown once after creation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewToken {
    pub id: Uuid,
    pub token: String,
}
