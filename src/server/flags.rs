//! Feature flags (§27): declared in code, synced into `feature_flags` at startup, cached in
//! memory (reloaded on `settings_changed`, like settings), evaluated per user with a stable
//! bucket so the same user always gets the same answer.
use std::collections::HashMap;
use std::sync::RwLock;

use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use super::ctx::{Ctx, CurrentUser};
use super::error::AppError;

/// (key, description, default enabled). Removing a flag here deletes its rows at startup.
pub const REGISTRY: &[(&str, &str, bool)] = &[(
    "dashboard_activity",
    "Activity tab on the dashboard (audit by module, users by role, weekly sign-ups)",
    true,
)];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FlagRow {
    pub key: String,
    pub org_id: Option<Uuid>,
    pub enabled: bool,
    pub rollout_percent: i32,
    pub allowed_role_ids: Vec<Uuid>,
    pub allowed_user_ids: Vec<Uuid>,
}

static CACHE: RwLock<Option<Vec<FlagRow>>> = RwLock::new(None);

/// Insert missing global rows (with the code default), refresh descriptions, delete flags
/// no longer declared.
pub async fn sync(pool: &PgPool) -> sqlx::Result<()> {
    let mut tx = pool.begin().await?;
    let keys: Vec<String> = REGISTRY.iter().map(|(k, _, _)| (*k).to_string()).collect();
    for (key, desc, on) in REGISTRY {
        let exists: bool = sqlx::query_scalar!(
            r#"SELECT EXISTS (SELECT 1 FROM feature_flags WHERE key = $1 AND org_id IS NULL) AS "e!""#,
            key
        )
        .fetch_one(&mut *tx)
        .await?;
        if exists {
            sqlx::query!(
                "UPDATE feature_flags SET description = $2 WHERE key = $1",
                key,
                desc
            )
            .execute(&mut *tx)
            .await?;
        } else {
            sqlx::query!(
                "INSERT INTO feature_flags (key, description, enabled) VALUES ($1, $2, $3)",
                key,
                desc,
                on
            )
            .execute(&mut *tx)
            .await?;
        }
    }
    sqlx::query!("DELETE FROM feature_flags WHERE NOT (key = ANY($1))", &keys)
        .execute(&mut *tx)
        .await?;
    tx.commit().await
}

pub async fn reload(pool: &PgPool) -> sqlx::Result<()> {
    let rows = load(pool).await?;
    if let Ok(mut c) = CACHE.write() {
        *c = Some(rows);
    }
    Ok(())
}

async fn load(pool: &PgPool) -> sqlx::Result<Vec<FlagRow>> {
    sqlx::query_as!(
        FlagRow,
        "SELECT key, org_id, enabled, rollout_percent, allowed_role_ids, allowed_user_ids FROM feature_flags"
    )
    .fetch_all(pool)
    .await
}

/// Stable 0–99 bucket for (user, flag): FNV-1a over the bytes, so it never changes between
/// releases (std's hasher may).
pub fn bucket(user: Uuid, flag: &str) -> i32 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in user.as_bytes().iter().chain(flag.as_bytes()) {
        h ^= u64::from(*b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    (h % 100) as i32
}

/// The org row wins over the global one.
fn pick<'a>(rows: &'a [FlagRow], key: &str, org: Uuid) -> Option<&'a FlagRow> {
    rows.iter()
        .find(|f| f.key == key && f.org_id == Some(org))
        .or_else(|| rows.iter().find(|f| f.key == key && f.org_id.is_none()))
}

pub fn evaluate(rows: &[FlagRow], key: &str, user: &CurrentUser) -> bool {
    let Some(f) = pick(rows, key, user.org_id) else {
        return false;
    };
    f.enabled
        && (f.allowed_role_ids.is_empty() || f.allowed_role_ids.contains(&user.role_id))
        && (f.allowed_user_ids.contains(&user.id) || bucket(user.id, key) < f.rollout_percent)
}

/// From the cache (registry default before it loads).
pub fn is_enabled(key: &str, user: &CurrentUser) -> bool {
    match CACHE.read().ok().and_then(|c| c.clone()) {
        Some(rows) => evaluate(&rows, key, user),
        None => REGISTRY.iter().any(|(k, _, on)| *k == key && *on),
    }
}

/// Every declared flag's value for this user (sent to the UI with `current_user()`).
pub fn for_user(user: &CurrentUser) -> HashMap<String, bool> {
    REGISTRY
        .iter()
        .map(|(k, _, _)| ((*k).to_string(), is_enabled(k, user)))
        .collect()
}

/// Server side gate: `require_flag(&ctx, "new_reports")?`. Off → 404, as if it didn't exist.
#[allow(dead_code)] // first feature behind a server-side flag will call it
pub fn require_flag(ctx: &Ctx, key: &str) -> Result<(), AppError> {
    let user = ctx.user.as_ref().ok_or(AppError::Unauthenticated)?;
    if is_enabled(key, user) {
        Ok(())
    } else {
        Err(AppError::Disabled("flags.off"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn user(role: Uuid) -> CurrentUser {
        CurrentUser {
            id: Uuid::new_v4(),
            session_id: Uuid::new_v4(),
            org_id: Uuid::nil(),
            email: "a@b.c".into(),
            full_name: "A".into(),
            role_id: role,
            role_name: "R".into(),
            is_super_admin: false,
            permissions: Default::default(),
            must_change_pw: false,
            locale: None,
        }
    }

    fn row(enabled: bool, pct: i32) -> FlagRow {
        FlagRow {
            key: "x".into(),
            org_id: None,
            enabled,
            rollout_percent: pct,
            allowed_role_ids: vec![],
            allowed_user_ids: vec![],
        }
    }

    #[test]
    fn bucket_is_stable_and_spread() {
        let u = Uuid::from_u128(42);
        assert_eq!(bucket(u, "x"), bucket(u, "x"));
        let hits = (0..1000u128)
            .filter(|i| bucket(Uuid::from_u128(*i), "x") < 30)
            .count();
        assert!((200..400).contains(&hits), "about 30% of users: {hits}");
    }

    #[test]
    fn evaluation_rules() {
        let role = Uuid::new_v4();
        let u = user(role);
        assert!(!evaluate(&[row(false, 100)], "x", &u));
        assert!(evaluate(&[row(true, 100)], "x", &u));
        assert!(!evaluate(&[row(true, 0)], "x", &u));
        let mut beta = row(true, 0);
        beta.allowed_user_ids = vec![u.id];
        assert!(evaluate(&[beta], "x", &u), "beta users always get it");
        let mut other_role = row(true, 100);
        other_role.allowed_role_ids = vec![Uuid::new_v4()];
        assert!(!evaluate(&[other_role], "x", &u));
        let mut org = row(false, 100);
        org.org_id = Some(Uuid::nil());
        assert!(
            !evaluate(&[row(true, 100), org], "x", &u),
            "org row overrides global"
        );
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn sync_declares_and_cleans(pool: PgPool) -> sqlx::Result<()> {
        sqlx::query("INSERT INTO feature_flags (key) VALUES ('old_flag')")
            .execute(&pool)
            .await?;
        sync(&pool).await?;
        sync(&pool).await?;
        let keys: Vec<String> = sqlx::query_scalar("SELECT key FROM feature_flags ORDER BY key")
            .fetch_all(&pool)
            .await?;
        assert_eq!(
            keys,
            REGISTRY.iter().map(|r| r.0.to_string()).collect::<Vec<_>>()
        );
        Ok(())
    }
}
