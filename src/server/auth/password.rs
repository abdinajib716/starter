//! Argon2id hashing (§7, §13) and password policy (M1.18).
use std::sync::LazyLock;

use argon2::Argon2;
use argon2::password_hash::{PasswordHasher, PasswordVerifier, phc::PasswordHash};

/// Argon2id v19, m = 19 MiB, t = 2, p = 1 (the crate default = OWASP recommendation).
fn hasher() -> Argon2<'static> {
    Argon2::default()
}

/// Hash on a blocking thread: Argon2 is deliberately slow and would stall the async runtime.
pub async fn hash(password: String) -> anyhow::Result<String> {
    tokio::task::spawn_blocking(move || {
        hasher()
            .hash_password(password.as_bytes())
            .map(|h| h.to_string())
            .map_err(|e| anyhow::anyhow!("argon2 hash failed: {e}"))
    })
    .await?
}

/// A real hash of a random value, verified when the user does not exist so that
/// "unknown email" costs the same time as "wrong password" (§7 step 3).
static DUMMY_HASH: LazyLock<String> = LazyLock::new(|| {
    let mut secret = [0u8; 32];
    getrandom::fill(&mut secret).expect("OS RNG available");
    hasher()
        .hash_password(&secret)
        .expect("argon2 hash of dummy secret")
        .to_string()
});

/// Verify against `stored`, or against the dummy hash when `stored` is `None`.
/// Always performs one full Argon2 computation.
pub async fn verify(password: String, stored: Option<String>) -> bool {
    let known = stored.is_some();
    let hash = stored.unwrap_or_else(|| DUMMY_HASH.clone());
    let ok = tokio::task::spawn_blocking(move || match PasswordHash::new(&hash) {
        Ok(parsed) => hasher()
            .verify_password(password.as_bytes(), &parsed)
            .is_ok(),
        Err(_) => {
            // A corrupt stored hash must not become a fast path: burn the same time.
            let _ = PasswordHash::new(&DUMMY_HASH)
                .map(|d| hasher().verify_password(password.as_bytes(), &d));
            false
        }
    })
    .await
    .unwrap_or(false);
    known && ok
}

/// Small embedded deny-list; swap for a larger list (e.g. NCSC top 100k) if needed.
static COMMON: &str = include_str!("common_passwords.txt");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyError {
    Short,
    Long,
    Common,
}

impl PolicyError {
    /// Stable error code, translated on the client (§26 rule 2).
    pub fn code(self) -> &'static str {
        match self {
            Self::Short => "password.too_short",
            Self::Long => "password.too_long",
            Self::Common => "password.too_common",
        }
    }
}

/// Applied on create, reset and change (M1.18). Length counts characters, not bytes.
pub fn check_policy(password: &str, min_len: usize) -> Result<(), PolicyError> {
    let len = password.chars().count();
    if len < min_len {
        return Err(PolicyError::Short);
    }
    // Argon2 has no length limit, but unbounded input is a DoS vector.
    if len > 256 {
        return Err(PolicyError::Long);
    }
    let lower = password.to_lowercase();
    if COMMON.lines().any(|c| !c.is_empty() && c == lower) {
        return Err(PolicyError::Common);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn hash_then_verify() {
        let h = hash("correct horse battery".into()).await.unwrap();
        assert!(h.starts_with("$argon2id$v=19$m=19456,t=2,p=1$"));
        assert!(verify("correct horse battery".into(), Some(h.clone())).await);
        assert!(!verify("wrong".into(), Some(h)).await);
    }

    #[tokio::test]
    async fn unknown_user_never_verifies_but_runs_argon2() {
        // The dummy hash must be a parseable Argon2 hash, otherwise the
        // "user not found" path would skip the expensive computation.
        assert!(PasswordHash::new(&DUMMY_HASH).is_ok());
        assert!(!verify("anything".into(), None).await);
    }

    #[test]
    fn policy() {
        assert_eq!(check_policy("short", 10), Err(PolicyError::Short));
        assert_eq!(check_policy("Password123", 10), Err(PolicyError::Common));
        assert_eq!(check_policy(&"x".repeat(257), 10), Err(PolicyError::Long));
        assert_eq!(check_policy("blue-river-tractor", 10), Ok(()));
        // characters, not bytes: 10 Somali/Unicode chars pass
        assert_eq!(check_policy("ääääääääää", 10), Ok(()));
    }
}
