//! `features/users/data/{data.ts,schema.ts,users.ts}`: the user shape, status badge colours,
//! roles and the 500 faker-generated demo users (same seed as the reference). Real users
//! replace this in M5.
use std::sync::LazyLock;

use serde::{Deserialize, Serialize};

use crate::components::ui_extra::icon::IconType;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: String,
    pub first_name: String,
    pub last_name: String,
    pub username: String,
    pub email: String,
    pub phone_number: String,
    pub status: String,
    pub role: String,
}

pub static USERS: LazyLock<Vec<User>> =
    LazyLock::new(|| serde_json::from_str(include_str!("users.json")).expect("users.json"));

/// `callTypes`: badge colours per status.
pub fn status_class(status: &str) -> &'static str {
    match status {
        "active" => "bg-teal-100/30 text-teal-900 dark:text-teal-200 border-teal-200",
        "inactive" => "bg-neutral-300/40 border-neutral-300",
        "invited" => "bg-sky-200/40 text-sky-900 dark:text-sky-100 border-sky-300",
        "suspended" => {
            "bg-destructive/10 dark:bg-destructive/50 text-destructive dark:text-primary border-destructive/10"
        }
        _ => "",
    }
}

/// (value, i18n key)
pub const STATUSES: [(&str, &str); 4] = [
    ("active", "user-status-active"),
    ("inactive", "user-status-inactive"),
    ("invited", "user-status-invited"),
    ("suspended", "user-status-suspended"),
];

pub const ROLES: [(&str, &str, IconType); 4] = [
    ("superadmin", "user-role-superadmin", IconType::Shield),
    ("admin", "user-role-admin", IconType::UserCheck),
    ("manager", "user-role-manager", IconType::Users),
    ("cashier", "user-role-cashier", IconType::CreditCard),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_users_load() {
        assert_eq!(USERS.len(), 500);
        assert!(
            USERS
                .iter()
                .all(|u| STATUSES.iter().any(|s| s.0 == u.status))
        );
        assert!(USERS.iter().all(|u| ROLES.iter().any(|r| r.0 == u.role)));
    }
}
