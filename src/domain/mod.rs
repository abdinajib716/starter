//! Shared DTOs + validation; compiled for both the server and wasm (§31.2).
use serde::{Deserialize, Serialize};

/// `TENANCY_MODE` (§18). Install-time, read-only in the UI.
#[cfg_attr(not(feature = "server"), allow(dead_code))]
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

/// Public, non-sensitive server status (`/api/system/info`).
#[cfg_attr(not(feature = "server"), allow(dead_code))]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServerInfo {
    pub version: String,
    pub tenancy: TenancyMode,
    pub db: String,
}

/// Where the client goes after a successful login (§7 step 8).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AfterLogin {
    Home,
    /// `must_change_pw`: the profile page with the change-password form.
    ChangePassword,
}

/// The signed-in user as the UI sees it (`current_user()`, §8). No secrets.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: uuid::Uuid,
    pub email: String,
    pub full_name: String,
    pub role_name: String,
    pub is_super_admin: bool,
    pub permissions: Vec<String>,
    pub must_change_pw: bool,
}

impl UserInfo {
    /// Cosmetic check for hiding UI (§7); the server re-checks in every server fn.
    #[allow(dead_code)] // used by PermissionGate (M2.6)
    pub fn can(&self, permission: &str) -> bool {
        self.is_super_admin || self.permissions.iter().any(|p| p == permission)
    }

    pub fn initials(&self) -> String {
        self.full_name
            .split_whitespace()
            .filter_map(|w| w.chars().next())
            .take(2)
            .collect::<String>()
            .to_uppercase()
    }
}

/// Stable error codes in `ServerFnError::ServerError { message }` (§26 rule 2): the client
/// translates them, the server never sends sentences.
pub mod errors {
    use dioxus::prelude::ServerFnError;

    #[cfg_attr(not(feature = "server"), allow(dead_code))]
    pub fn coded(status: u16, code: &str) -> ServerFnError {
        ServerFnError::ServerError {
            message: code.to_string(),
            code: status,
            details: None,
        }
    }

    /// Fluent message id for an error: `auth.invalid_credentials` → `err-auth-invalid-credentials`.
    /// Unknown or transport errors map to `err-generic`.
    pub fn i18n_key(err: &ServerFnError) -> String {
        match err {
            ServerFnError::ServerError { message, .. }
                if !message.is_empty()
                    && message
                        .chars()
                        .all(|c| c.is_ascii_lowercase() || c == '.' || c == '_') =>
            {
                format!("err-{}", message.replace(['.', '_'], "-"))
            }
            _ => "err-generic".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::errors::{coded, i18n_key};
    use dioxus::prelude::ServerFnError;

    // lib/handle-server-error.test.ts: unknown errors fall back to the generic message; a
    // recognised server error is shown with its own text (here: its Fluent key).
    #[test]
    fn unknown_errors_map_to_generic() {
        assert_eq!(
            i18n_key(&ServerFnError::new("Some English sentence")),
            "err-generic"
        );
        let raw = ServerFnError::ServerError {
            message: String::new(),
            code: 500,
            details: None,
        };
        assert_eq!(i18n_key(&raw), "err-generic");
    }

    #[test]
    fn coded_errors_map_to_their_key() {
        assert_eq!(
            i18n_key(&coded(401, "auth.invalid_credentials")),
            "err-auth-invalid-credentials"
        );
    }
}
