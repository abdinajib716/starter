//! One error type for services (§13). Each variant maps to an HTTP status and a stable code
//! (`users.last_super_admin`); the client translates the code (§26 rule 2). Internal details
//! stay in the `tracing` log, never in the response.
use dioxus::prelude::ServerFnError;

use crate::domain::errors::coded;

#[derive(Debug)]
pub enum AppError {
    Unauthenticated,
    Forbidden,
    NotFound,
    /// Bad input; the code names the rule (`users.email_invalid`).
    Validation(&'static str),
    /// Valid input that clashes with the current state (`users.email_taken`).
    Conflict(&'static str),
    /// A module switched off in System Settings (`email.disabled`).
    Disabled(&'static str),
    Internal(anyhow::Error),
}

impl AppError {
    /// The stable code the client sees (also used for bulk skip reasons).
    pub fn code(&self) -> &'static str {
        match self {
            Self::Unauthenticated => "auth.unauthenticated",
            Self::Forbidden => "auth.forbidden",
            Self::NotFound => "not_found",
            Self::Validation(c) | Self::Conflict(c) | Self::Disabled(c) => c,
            Self::Internal(_) => "internal",
        }
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unauthenticated => f.write_str("not signed in"),
            Self::Forbidden => f.write_str("permission denied"),
            Self::NotFound => f.write_str("not found"),
            Self::Validation(c) => write!(f, "validation: {c}"),
            Self::Conflict(c) => write!(f, "conflict: {c}"),
            Self::Disabled(c) => write!(f, "module disabled: {c}"),
            Self::Internal(e) => write!(f, "internal: {e}"),
        }
    }
}

impl From<anyhow::Error> for AppError {
    fn from(e: anyhow::Error) -> Self {
        Self::Internal(e)
    }
}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        Self::Internal(e.into())
    }
}

impl From<AppError> for ServerFnError {
    fn from(e: AppError) -> Self {
        match e {
            AppError::Unauthenticated => coded(401, "auth.unauthenticated"),
            AppError::Forbidden => coded(403, "auth.forbidden"),
            AppError::NotFound => coded(404, "not_found"),
            AppError::Validation(c) => coded(400, c),
            AppError::Conflict(c) => coded(409, c),
            AppError::Disabled(c) => coded(404, c),
            AppError::Internal(err) => {
                tracing::error!(error = ?err, "internal error");
                coded(500, "internal")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::errors::i18n_key;

    #[test]
    fn codes_reach_the_client_details_do_not() {
        let e: ServerFnError = AppError::Conflict("users.last_super_admin").into();
        assert_eq!(i18n_key(&e), "err-users-last-super-admin");
        let e: ServerFnError = AppError::Internal(anyhow::anyhow!("db password=secret")).into();
        assert_eq!(i18n_key(&e), "err-internal");
        assert!(!format!("{e:?}").contains("secret"));
    }
}
