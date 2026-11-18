//! Users module DTOs (§9.2, §25), shared by the server functions and the Users page.
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

/// Derived from the user row, newest rule first: a deleted user is `Deleted`; a locked one
/// `Suspended` (too many failed sign-ins); `Inactive` when switched off; `Invited` until the
/// first sign-in; otherwise `Active`. Same four states as the reference UI, plus `Deleted`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UserStatus {
    Active,
    Inactive,
    Invited,
    Suspended,
    Deleted,
}

impl UserStatus {
    pub const FILTERABLE: [UserStatus; 4] =
        [Self::Active, Self::Inactive, Self::Invited, Self::Suspended];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Inactive => "inactive",
            Self::Invited => "invited",
            Self::Suspended => "suspended",
            Self::Deleted => "deleted",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        [
            Self::Active,
            Self::Inactive,
            Self::Invited,
            Self::Suspended,
            Self::Deleted,
        ]
        .into_iter()
        .find(|v| v.as_str() == s)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserRow {
    pub id: Uuid,
    pub email: String,
    pub full_name: String,
    pub role_id: Uuid,
    pub role_name: String,
    pub status: UserStatus,
    pub must_change_pw: bool,
    #[serde(with = "time::serde::rfc3339::option")]
    pub last_login_at: Option<OffsetDateTime>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339::option")]
    pub deleted_at: Option<OffsetDateTime>,
    pub deleted_by_email: Option<String>,
}

/// Sortable columns (server whitelist).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UserSort {
    Name,
    Email,
    LastLogin,
    Created,
}

impl UserSort {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "name" | "fullName" => Some(Self::Name),
            "email" => Some(Self::Email),
            "lastLogin" => Some(Self::LastLogin),
            "created" => Some(Self::Created),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct UserQuery {
    /// Matches name or email (case-insensitive).
    pub q: String,
    pub statuses: Vec<UserStatus>,
    pub role_ids: Vec<Uuid>,
    /// (column, descending). Default: newest first.
    pub sort: Option<(UserSort, bool)>,
    /// 0-based.
    pub page: u32,
    pub page_size: u32,
    /// The "Deleted" tab: soft-deleted users only.
    pub deleted: bool,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct UserPage {
    pub rows: Vec<UserRow>,
    pub total: u64,
    /// Facet counts for the filter menus (each ignores its own filter, like TanStack).
    pub status_counts: Vec<(UserStatus, u64)>,
    pub role_counts: Vec<(Uuid, u64)>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RoleOption {
    pub id: Uuid,
    pub name: String,
    pub is_system: bool,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct UserInput {
    pub full_name: String,
    pub email: String,
    pub role_id: Uuid,
    /// Create: the temporary password (user must change it). Edit: empty = unchanged.
    pub password: String,
}

/// What a bulk action did (§25.1): done ids, and skipped ones with the reason code.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct BulkResult {
    pub done: u32,
    pub skipped: Vec<(Uuid, String)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BulkUserAction {
    Activate,
    Deactivate,
    Delete,
    Restore,
}

pub const NAME_MAX: usize = 120;

/// Client- and server-side checks of the user form. Returns `(field, code)` pairs.
pub fn validate_input(
    i: &UserInput,
    creating: bool,
    min_pw: usize,
) -> Vec<(&'static str, &'static str)> {
    let mut e = vec![];
    let name = i.full_name.trim();
    if name.is_empty() {
        e.push(("full_name", "users.name_required"));
    } else if name.chars().count() > NAME_MAX {
        e.push(("full_name", "users.name_too_long"));
    }
    if !crate::domain::valid_email(&i.email) {
        e.push(("email", "users.email_invalid"));
    }
    if i.role_id.is_nil() {
        e.push(("role_id", "users.role_required"));
    }
    let pw = i.password.chars().count();
    if creating && pw == 0 {
        e.push(("password", "users.password_required"));
    } else if pw > 0 && pw < min_pw {
        e.push(("password", "password.too_short"));
    }
    e
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn statuses_round_trip() {
        for s in [
            UserStatus::Active,
            UserStatus::Inactive,
            UserStatus::Invited,
            UserStatus::Suspended,
            UserStatus::Deleted,
        ] {
            assert_eq!(UserStatus::parse(s.as_str()), Some(s));
        }
        assert_eq!(UserStatus::parse("x"), None);
    }

    #[test]
    fn form_validation() {
        let ok = UserInput {
            full_name: "Ann".into(),
            email: "a@b.co".into(),
            role_id: Uuid::new_v4(),
            password: "long-enough-1".into(),
        };
        assert!(validate_input(&ok, true, 10).is_empty());
        let bad = UserInput {
            full_name: " ".into(),
            email: "nope".into(),
            role_id: Uuid::nil(),
            password: String::new(),
        };
        let codes: Vec<_> = validate_input(&bad, true, 10)
            .into_iter()
            .map(|(_, c)| c)
            .collect();
        assert_eq!(
            codes,
            [
                "users.name_required",
                "users.email_invalid",
                "users.role_required",
                "users.password_required"
            ]
        );
        // editing: empty password = unchanged
        let edit = UserInput {
            password: String::new(),
            ..ok
        };
        assert!(validate_input(&edit, false, 10).is_empty());
    }
}

/// User details page (`/users/:id`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserDetail {
    pub user: UserRow,
    /// Own time zone (`None` = organization default).
    pub timezone: Option<String>,
    pub failed_attempts: i32,
    #[serde(with = "time::serde::rfc3339::option")]
    pub locked_until: Option<OffsetDateTime>,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
    /// Signed-in sessions that have not expired.
    pub active_sessions: i64,
    pub twofa_enrolled: bool,
    /// Latest audit entries by or about this user (newest first, max 20).
    pub activity: Vec<crate::domain::audit::AuditRow>,
}
