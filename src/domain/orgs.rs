//! Organizations (multi-tenant mode, §18, M16.3).
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrgRow {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub is_default: bool,
    pub is_active: bool,
    pub users: i64,
    pub max_users: Option<i32>,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewOrg {
    pub name: String,
    pub slug: String,
    pub admin_name: String,
    pub admin_email: String,
    /// Temporary password for the first administrator (must change it at first sign-in).
    pub admin_password: String,
    pub max_users: Option<i32>,
}

/// `acme`, `north-branch`: lowercase letters, digits, dashes; 2–40 chars; not reserved.
#[cfg_attr(not(feature = "server"), allow(dead_code))]
pub fn valid_slug(s: &str) -> bool {
    let reserved = ["www", "api", "app", "admin", "static", "assets", "mail"];
    (2..=40).contains(&s.len())
        && s.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        && !s.starts_with('-')
        && !s.ends_with('-')
        && !reserved.contains(&s)
}

#[cfg(test)]
mod tests {
    #[test]
    fn slugs() {
        assert!(super::valid_slug("acme"));
        assert!(super::valid_slug("north-2"));
        assert!(!super::valid_slug("Acme"));
        assert!(!super::valid_slug("-x"));
        assert!(!super::valid_slug("www"));
        assert!(!super::valid_slug("a.b"));
    }
}
