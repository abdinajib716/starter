//! Login, logout, change password (§7, §9.1).
use std::time::Duration;

use sqlx::PgPool;

use crate::domain::AfterLogin;
use crate::server::audit::{self, Entry};
use crate::server::auth::{password, ratelimit, session};
use crate::server::ctx::{Ctx, CurrentUser};
use crate::server::settings::SecuritySettings;

#[derive(Debug)]
pub enum AuthError {
    /// Wrong email/password, locked, inactive: one generic answer (no enumeration).
    InvalidCredentials,
    RateLimited,
    /// Change password: current password wrong.
    WrongPassword,
    /// Second step: wrong code.
    Invalid2fa,
    /// Second step: the pending sign-in expired or was dropped after too many tries.
    Expired2fa,
    Policy(password::PolicyError),
    Internal(anyhow::Error),
}

impl From<sqlx::Error> for AuthError {
    fn from(e: sqlx::Error) -> Self {
        Self::Internal(e.into())
    }
}

impl From<anyhow::Error> for AuthError {
    fn from(e: anyhow::Error) -> Self {
        Self::Internal(e)
    }
}

pub struct LoginOk {
    pub token: String,
    pub max_age_secs: i64,
    pub next: AfterLogin,
}

const WINDOW: Duration = Duration::from_secs(15 * 60);

pub async fn login(
    pool: &PgPool,
    ctx: &Ctx,
    email: &str,
    password_in: String,
) -> Result<LoginOk, AuthError> {
    let email = email.trim().to_lowercase();
    let ip = ctx.meta.ip;
    let ua = ctx.meta.user_agent.as_deref();
    let s = SecuritySettings::load(pool).await?;

    // 1. Rate limit: per IP+email (5 / 15 min), per email (20), per IP (50).
    let k_pair = format!(
        "login:{}:{email}",
        ip.map(|i| i.to_string()).unwrap_or_default()
    );
    let pair_ok = ratelimit::hit(&k_pair, 5, WINDOW).await;
    let email_ok = ratelimit::hit(&format!("login-email:{email}"), 20, WINDOW).await;
    let ip_ok = match ip {
        Some(i) => ratelimit::hit(&format!("login-ip:{i}"), 50, WINDOW).await,
        None => true,
    };
    let allowed = pair_ok && email_ok && ip_ok;
    if !allowed {
        return Err(AuthError::RateLimited);
    }

    // 2. Load the user (non-deleted, this org).
    let user = sqlx::query!(
        r#"SELECT id, password_hash, is_active, must_change_pw, failed_attempts,
                  COALESCE(locked_until > now(), false) AS "locked!",
                  EXISTS (SELECT 1 FROM user_totp t WHERE t.user_id = users.id
                          AND t.confirmed_at IS NOT NULL) AS "enrolled!"
           FROM users WHERE org_id = $1 AND email = $2::text::citext AND deleted_at IS NULL"#,
        ctx.org_id,
        email,
    )
    .fetch_optional(pool)
    .await?;

    // 3–4. Always one Argon2 verification, whatever the outcome (constant time).
    let locked = user.as_ref().is_some_and(|u| u.locked);
    let stored = user
        .as_ref()
        .filter(|_| !locked)
        .map(|u| u.password_hash.clone());
    let password_ok = password::verify(password_in, stored).await;

    let mut tx = pool.begin().await?;
    let fail = |reason: &'static str| Entry {
        org_id: Some(ctx.org_id),
        actor_id: user.as_ref().map(|u| u.id),
        actor_email: Some(email.as_str()),
        action: "auth.login_failed",
        after: Some(serde_json::json!({ "reason": reason })),
        ip,
        user_agent: ua,
        success: false,
        ..Default::default()
    };

    let Some(u) = user.as_ref() else {
        audit::record(&mut tx, fail("unknown_email")).await?;
        tx.commit().await?;
        return Err(AuthError::InvalidCredentials);
    };
    if locked {
        audit::record(&mut tx, fail("locked")).await?;
        tx.commit().await?;
        return Err(AuthError::InvalidCredentials);
    }
    if !password_ok {
        // 4. Count the failure; lock at the threshold and start a fresh count after it.
        let attempts = u.failed_attempts + 1;
        let lock = attempts >= s.max_failed_attempts;
        sqlx::query!(
            r#"UPDATE users SET
                 failed_attempts = CASE WHEN $2 THEN 0 ELSE $3 END,
                 locked_until = CASE WHEN $2 THEN now() + make_interval(mins => $4) ELSE locked_until END
               WHERE id = $1"#,
            u.id,
            lock,
            attempts,
            i32::try_from(s.lockout_minutes).unwrap_or(15),
        )
        .execute(&mut *tx)
        .await?;
        audit::record(&mut tx, fail("bad_password")).await?;
        if lock {
            // M1.19: tell the user (in-app; by email when the module is on).
            crate::server::notify::send(
                &mut tx,
                ctx.org_id,
                crate::server::notify::Target::User(u.id),
                "account_locked",
                serde_json::json!({ "minutes": s.lockout_minutes }),
                None,
            )
            .await?;
            audit::record(
                &mut tx,
                Entry {
                    action: "auth.account_locked",
                    entity_type: Some("user"),
                    entity_id: Some(u.id.to_string()),
                    ..fail("threshold")
                },
            )
            .await?;
        }
        tx.commit().await?;
        return Err(AuthError::InvalidCredentials);
    }
    // 5. Correct password but deactivated.
    if !u.is_active {
        audit::record(&mut tx, fail("inactive")).await?;
        tx.commit().await?;
        return Err(AuthError::InvalidCredentials);
    }

    // 6. Second factor (§21): password ok, the code comes next on `/auth/2fa`.
    if u.enrolled
        && crate::server::modules::Modules::get().twofa_mode() != crate::domain::TwoFaMode::Off
    {
        let token =
            session::create_kind(&mut tx, u.id, ip, ua, session::Kind::Pending2fa, 5).await?;
        tx.commit().await?;
        ratelimit::reset(&k_pair).await;
        return Ok(LoginOk {
            token,
            max_age_secs: 300,
            next: AfterLogin::TwoFactor,
        });
    }

    // 7. Success: reset counters, open a session, audit.
    let token = finish_login(&mut tx, ctx, u.id, &email, false, s.session_hours).await?;
    tx.commit().await?;
    ratelimit::reset(&k_pair).await;

    Ok(LoginOk {
        token,
        max_age_secs: s.session_hours * 3600,
        next: if u.must_change_pw {
            AfterLogin::ChangePassword
        } else {
            AfterLogin::Home
        },
    })
}

/// The last step of every successful sign-in (password only, or after the 2FA code):
/// reset counters, open the `web` session, audit, flag a new device (§22, M9.13).
pub async fn finish_login(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ctx: &Ctx,
    user_id: uuid::Uuid,
    email: &str,
    mfa: bool,
    session_hours: i64,
) -> Result<String, AuthError> {
    let (ip, ua) = (ctx.meta.ip, ctx.meta.user_agent.as_deref());
    sqlx::query!(
        "UPDATE users SET failed_attempts = 0, locked_until = NULL, last_login_at = now()
         WHERE id = $1",
        user_id
    )
    .execute(&mut **tx)
    .await?;
    // New device = no session in the last 90 days from the same browser + OS.
    let device = crate::domain::security::device_label(ua.unwrap_or(""));
    let known = sqlx::query_scalar!(
        r#"SELECT EXISTS (SELECT 1 FROM audit_logs WHERE actor_id = $1 AND action = 'auth.login'
                          AND after->>'device' = $2 AND at > now() - interval '90 days') AS "e!""#,
        user_id,
        device
    )
    .fetch_one(&mut **tx)
    .await?;
    let first = !sqlx::query_scalar!(
        r#"SELECT EXISTS (SELECT 1 FROM audit_logs WHERE actor_id = $1 AND action = 'auth.login') AS "e!""#,
        user_id
    )
    .fetch_one(&mut **tx)
    .await?;
    let token = session::create_kind(
        tx,
        user_id,
        ip,
        ua,
        session::Kind::Web { mfa },
        session_hours * 60,
    )
    .await?;
    let entry = |action| Entry {
        org_id: Some(ctx.org_id),
        actor_id: Some(user_id),
        actor_email: Some(email),
        action,
        after: Some(serde_json::json!({ "device": device, "mfa": mfa })),
        ip,
        user_agent: ua,
        success: true,
        ..Default::default()
    };
    audit::record(tx, entry("auth.login")).await?;
    if !known && !first {
        audit::record(tx, entry("auth.new_device")).await?;
        crate::server::notify::new_device(tx, ctx.org_id, user_id, &device, ip).await?;
    }
    Ok(token)
}

pub async fn logout(pool: &PgPool, ctx: &Ctx, user: &CurrentUser) -> Result<(), AuthError> {
    let mut tx = pool.begin().await?;
    sqlx::query!("DELETE FROM sessions WHERE id = $1", user.session_id)
        .execute(&mut *tx)
        .await?;
    audit::record(&mut tx, user_entry(ctx, user, "auth.logout")).await?;
    tx.commit().await?;
    Ok(())
}

/// Verify the current password, set the new one, end every *other* session (§7).
pub async fn change_password(
    pool: &PgPool,
    ctx: &Ctx,
    user: &CurrentUser,
    current: String,
    new: String,
) -> Result<(), AuthError> {
    let s = SecuritySettings::load(pool).await?;
    password::check_policy(&new, s.password_min_length).map_err(AuthError::Policy)?;
    if !ratelimit::hit(&format!("chpw:{}", user.id), 10, WINDOW).await {
        return Err(AuthError::RateLimited);
    }
    let stored = sqlx::query_scalar!("SELECT password_hash FROM users WHERE id = $1", user.id)
        .fetch_one(pool)
        .await?;
    if !password::verify(current, Some(stored)).await {
        return Err(AuthError::WrongPassword);
    }
    let hash = password::hash(new).await?;

    let mut tx = pool.begin().await?;
    sqlx::query!(
        "UPDATE users SET password_hash = $2, must_change_pw = false, updated_at = now()
         WHERE id = $1",
        user.id,
        hash
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query!(
        "DELETE FROM sessions WHERE user_id = $1 AND id <> $2",
        user.id,
        user.session_id
    )
    .execute(&mut *tx)
    .await?;
    audit::record(&mut tx, user_entry(ctx, user, "profile.password_change")).await?;
    crate::server::notify::send(
        &mut tx,
        ctx.org_id,
        crate::server::notify::Target::User(user.id),
        "password_changed",
        serde_json::json!({}),
        None,
    )
    .await?;
    crate::server::email::queue_for_user(
        &mut tx,
        user.id,
        "password_changed",
        serde_json::json!({}),
    )
    .await?;
    tx.commit().await?;
    Ok(())
}

/// Own name (§9.1 `update_profile`); session only, audited with before/after.
pub async fn update_profile(
    pool: &PgPool,
    ctx: &Ctx,
    user: &CurrentUser,
    full_name: String,
    timezone: String,
) -> Result<(), crate::server::error::AppError> {
    use crate::server::error::AppError;
    let name = full_name.trim();
    // Empty = follow the organization's time zone.
    let tz = Some(timezone.trim().to_string()).filter(|t| !t.is_empty());
    if let Some(t) = &tz
        && !crate::util::time::valid_zone(t)
    {
        return Err(AppError::Validation("settings.timezone_invalid"));
    }
    if name.is_empty() {
        return Err(AppError::Validation("users.name_required"));
    }
    if name.chars().count() > crate::domain::users::NAME_MAX {
        return Err(AppError::Validation("users.name_too_long"));
    }
    let mut tx = pool.begin().await?;
    sqlx::query!(
        "UPDATE users SET full_name = $2, timezone = $3, updated_at = now() WHERE id = $1",
        user.id,
        name,
        tz
    )
    .execute(&mut *tx)
    .await?;
    audit::record(
        &mut tx,
        Entry {
            before: Some(
                serde_json::json!({ "full_name": user.full_name, "timezone": user.timezone }),
            ),
            after: Some(serde_json::json!({ "full_name": name, "timezone": tz })),
            ..user_entry(ctx, user, "profile.update")
        },
    )
    .await?;
    tx.commit().await?;
    Ok(())
}

/// Own language (M3.8); must be one of the enabled locales.
pub async fn set_locale(
    pool: &PgPool,
    ctx: &Ctx,
    user: &CurrentUser,
    locale: Option<String>,
) -> Result<(), crate::server::error::AppError> {
    let (enabled, _) = crate::server::modules::Modules::get().locales();
    if let Some(l) = &locale
        && !enabled.contains(l)
    {
        return Err(crate::server::error::AppError::Validation(
            "i18n.locale_not_enabled",
        ));
    }
    let mut tx = pool.begin().await?;
    sqlx::query!(
        "UPDATE users SET locale = $2, updated_at = now() WHERE id = $1",
        user.id,
        locale
    )
    .execute(&mut *tx)
    .await?;
    audit::record(
        &mut tx,
        Entry {
            before: Some(serde_json::json!({ "locale": user.locale })),
            after: Some(serde_json::json!({ "locale": locale })),
            ..user_entry(ctx, user, "profile.locale")
        },
    )
    .await?;
    tx.commit().await?;
    Ok(())
}

pub(crate) fn user_entry<'a>(ctx: &'a Ctx, user: &'a CurrentUser, action: &'a str) -> Entry<'a> {
    Entry {
        org_id: Some(ctx.org_id),
        actor_id: Some(user.id),
        actor_email: Some(user.email.as_str()),
        action,
        entity_type: Some("user"),
        entity_id: Some(user.id.to_string()),
        ip: ctx.meta.ip,
        user_agent: ctx.meta.user_agent.as_deref(),
        success: true,
        ..Default::default()
    }
}

/// Internal → client error: a stable code + HTTP status, never internal details (§13).
impl From<AuthError> for dioxus::prelude::ServerFnError {
    fn from(e: AuthError) -> Self {
        use crate::domain::errors::coded;
        match e {
            AuthError::InvalidCredentials => coded(401, "auth.invalid_credentials"),
            AuthError::RateLimited => coded(429, "auth.rate_limited"),
            AuthError::WrongPassword => coded(400, "auth.wrong_password"),
            AuthError::Invalid2fa => coded(400, "twofa.invalid_code"),
            AuthError::Expired2fa => coded(401, "twofa.expired"),
            AuthError::Policy(p) => coded(400, p.code()),
            AuthError::Internal(err) => {
                tracing::error!(error = ?err, "auth internal error");
                coded(500, "internal")
            }
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::server::{request::RequestMeta, seed};
    use uuid::Uuid;

    /// Default org + Super Admin role + one user with password `correct-horse-1`.
    pub(crate) async fn setup(pool: &PgPool, email: &str) -> (Ctx, Uuid) {
        let org_id = seed::ensure_default_org(pool).await.unwrap();
        let role_id = seed::ensure_super_admin_role(pool).await.unwrap();
        let hash = password::hash("correct-horse-1".into()).await.unwrap();
        let user_id = sqlx::query_scalar!(
            r#"INSERT INTO users (org_id, email, full_name, password_hash, role_id)
               VALUES ($1, $2::text::citext, 'Test User', $3, $4) RETURNING id"#,
            org_id,
            email,
            hash,
            role_id
        )
        .fetch_one(pool)
        .await
        .unwrap();
        let ctx = Ctx {
            org_id,
            user: None,
            request_id: None,
            meta: RequestMeta::default(),
        };
        (ctx, user_id)
    }

    async fn audit_actions(pool: &PgPool) -> Vec<String> {
        sqlx::query_scalar!("SELECT action FROM audit_logs ORDER BY id")
            .fetch_all(pool)
            .await
            .unwrap()
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn login_success_creates_session_and_audits(pool: PgPool) {
        let (ctx, _) = setup(&pool, "ok@test.local").await;
        let ok = login(&pool, &ctx, "OK@test.local ", "correct-horse-1".into())
            .await
            .unwrap();
        assert_eq!(ok.next, AfterLogin::Home);
        let user = session::load(&pool, &ok.token, 12).await.unwrap().unwrap();
        assert_eq!(user.email, "ok@test.local");
        assert!(user.is_super_admin);
        assert_eq!(audit_actions(&pool).await, ["auth.login"]);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn lockout_after_max_failures_even_with_correct_password(pool: PgPool) {
        let (ctx, _) = setup(&pool, "lock@test.local").await;
        for _ in 0..5 {
            let r = login(&pool, &ctx, "lock@test.local", "wrong".into()).await;
            assert!(matches!(r, Err(AuthError::InvalidCredentials)));
        }
        // Rate limiter (5 per IP+email) would also block now: reset it to test the lock itself.
        ratelimit::reset("login::lock@test.local").await;
        let r = login(&pool, &ctx, "lock@test.local", "correct-horse-1".into()).await;
        assert!(matches!(r, Err(AuthError::InvalidCredentials)));
        let actions = audit_actions(&pool).await;
        assert!(actions.contains(&"auth.account_locked".to_string()));
        assert_eq!(actions.last().unwrap(), "auth.login_failed");
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn inactive_user_cannot_log_in(pool: PgPool) {
        let (ctx, id) = setup(&pool, "off@test.local").await;
        sqlx::query!("UPDATE users SET is_active = false WHERE id = $1", id)
            .execute(&pool)
            .await
            .unwrap();
        let r = login(&pool, &ctx, "off@test.local", "correct-horse-1".into()).await;
        assert!(matches!(r, Err(AuthError::InvalidCredentials)));
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn unknown_email_is_generic_and_audited(pool: PgPool) {
        let (ctx, _) = setup(&pool, "real@test.local").await;
        let r = login(&pool, &ctx, "ghost@test.local", "whatever".into()).await;
        assert!(matches!(r, Err(AuthError::InvalidCredentials)));
        assert_eq!(audit_actions(&pool).await, ["auth.login_failed"]);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn expired_session_is_rejected(pool: PgPool) {
        let (ctx, _) = setup(&pool, "exp@test.local").await;
        let ok = login(&pool, &ctx, "exp@test.local", "correct-horse-1".into())
            .await
            .unwrap();
        sqlx::query!("UPDATE sessions SET expires_at = now() - interval '1 second'")
            .execute(&pool)
            .await
            .unwrap();
        assert!(session::load(&pool, &ok.token, 12).await.unwrap().is_none());
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn change_password_ends_other_sessions(pool: PgPool) {
        let (ctx, _) = setup(&pool, "cp@test.local").await;
        let a = login(&pool, &ctx, "cp@test.local", "correct-horse-1".into())
            .await
            .unwrap();
        let b = login(&pool, &ctx, "cp@test.local", "correct-horse-1".into())
            .await
            .unwrap();
        let me = session::load(&pool, &a.token, 12).await.unwrap().unwrap();
        let wrong =
            change_password(&pool, &ctx, &me, "nope".into(), "blue-river-tractor".into()).await;
        assert!(matches!(wrong, Err(AuthError::WrongPassword)));
        let common = change_password(
            &pool,
            &ctx,
            &me,
            "correct-horse-1".into(),
            "password123".into(),
        )
        .await;
        assert!(matches!(
            common,
            Err(AuthError::Policy(password::PolicyError::Common))
        ));
        change_password(
            &pool,
            &ctx,
            &me,
            "correct-horse-1".into(),
            "blue-river-tractor".into(),
        )
        .await
        .unwrap();
        assert!(session::load(&pool, &a.token, 12).await.unwrap().is_some());
        assert!(session::load(&pool, &b.token, 12).await.unwrap().is_none());
    }
}
