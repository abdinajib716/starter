//! Sessions (§22, M9.3–9.5) and TOTP two-factor (§21, M9.6–9.12).
use serde_json::json;
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use super::auth::{AuthError, LoginOk, finish_login, user_entry};
use crate::domain::security::{EnrollStart, SessionRow, TwoFaStatus};
use crate::domain::{AfterLogin, TwoFaMode};
use crate::server::audit::{self, Entry};
use crate::server::auth::{password, ratelimit, token, totp};
use crate::server::ctx::{Ctx, CurrentUser};
use crate::server::error::AppError;
use crate::server::modules::Modules;
use crate::server::settings::SecuritySettings;
use crate::server::{email, notify};

const MAX_2FA_FAILURES: i32 = 5;

fn app_secret() -> Result<String, AppError> {
    Ok(crate::server::state::get()
        .ok()
        .and_then(|st| st.config.app_secret.clone())
        .or_else(|| std::env::var("APP_SECRET").ok())
        .unwrap_or_else(|| "development-only-secret-change-me!".into()))
}

// ---------------------------------------------------------------- sessions

async fn rows(
    pool: &PgPool,
    user_id: Uuid,
    current: Option<Uuid>,
) -> Result<Vec<SessionRow>, AppError> {
    let rows = sqlx::query!(
        r#"SELECT id, device_label, user_agent, host(ip) AS ip, created_at, last_seen_at,
                  mfa_verified_at IS NOT NULL AS "mfa!"
           FROM sessions WHERE user_id = $1 AND kind = 'web' AND expires_at > now()
           ORDER BY last_seen_at DESC"#,
        user_id
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| SessionRow {
            id: r.id,
            device: r.device_label.unwrap_or_else(|| {
                crate::domain::security::device_label(r.user_agent.as_deref().unwrap_or(""))
            }),
            ip: r.ip,
            created_at: r.created_at,
            last_seen_at: r.last_seen_at,
            current: Some(r.id) == current,
            mfa: r.mfa,
        })
        .collect())
}

pub async fn my_sessions(pool: &PgPool, ctx: &Ctx) -> Result<Vec<SessionRow>, AppError> {
    let user = ctx.user.as_ref().ok_or(AppError::Unauthenticated)?;
    rows(pool, user.id, Some(user.session_id)).await
}

/// Sign out one of my other devices (`None` = every other device).
pub async fn revoke_mine(pool: &PgPool, ctx: &Ctx, id: Option<Uuid>) -> Result<u64, AppError> {
    let user = ctx.user.as_ref().ok_or(AppError::Unauthenticated)?;
    if id == Some(user.session_id) {
        return Err(AppError::Conflict("sessions.current"));
    }
    let mut tx = pool.begin().await?;
    let n = sqlx::query!(
        "DELETE FROM sessions WHERE user_id = $1 AND id <> $2 AND ($3::uuid IS NULL OR id = $3)",
        user.id,
        user.session_id,
        id
    )
    .execute(&mut *tx)
    .await?
    .rows_affected();
    audit::record(
        &mut tx,
        Entry {
            after: Some(json!({ "session": id, "count": n })),
            ..user_entry(
                ctx,
                user,
                if id.is_some() {
                    "session.revoke"
                } else {
                    "session.revoke_others"
                },
            )
        },
    )
    .await?;
    tx.commit().await?;
    Ok(n)
}

pub async fn user_sessions(
    pool: &PgPool,
    ctx: &Ctx,
    target: Uuid,
) -> Result<Vec<SessionRow>, AppError> {
    let actor = ctx.require("users.view")?;
    same_org(pool, actor, target).await?;
    rows(pool, target, None).await
}

pub async fn revoke_user_sessions(pool: &PgPool, ctx: &Ctx, target: Uuid) -> Result<u64, AppError> {
    let actor = ctx.require("users.revoke_sessions")?;
    same_org(pool, actor, target).await?;
    let mut tx = pool.begin().await?;
    let n = sqlx::query!(
        "DELETE FROM sessions WHERE user_id = $1 AND id <> $2",
        target,
        actor.session_id
    )
    .execute(&mut *tx)
    .await?
    .rows_affected();
    audit::record(
        &mut tx,
        Entry {
            entity_id: Some(target.to_string()),
            after: Some(json!({ "count": n })),
            ..user_entry(ctx, actor, "user.sessions_revoked")
        },
    )
    .await?;
    tx.commit().await?;
    Ok(n)
}

async fn same_org(pool: &PgPool, actor: &CurrentUser, target: Uuid) -> Result<(), AppError> {
    let ok = sqlx::query_scalar!(
        r#"SELECT EXISTS (SELECT 1 FROM users WHERE id = $1 AND org_id = $2) AS "e!""#,
        target,
        actor.org_id
    )
    .fetch_one(pool)
    .await?;
    if ok { Ok(()) } else { Err(AppError::NotFound) }
}

/// Expired sessions and spent one-time tokens (M9.5). Run hourly by `start_jobs`.
pub async fn purge_expired(pool: &PgPool) -> sqlx::Result<(u64, u64)> {
    let s = sqlx::query!("DELETE FROM sessions WHERE expires_at < now()")
        .execute(pool)
        .await?
        .rows_affected();
    let t = sqlx::query!(
        "DELETE FROM auth_tokens WHERE expires_at < now() - interval '1 day' OR used_at < now() - interval '1 day'"
    )
    .execute(pool)
    .await?
    .rows_affected();
    Ok((s, t))
}

// ---------------------------------------------------------------- 2FA status / enrolment

pub async fn status(pool: &PgPool, ctx: &Ctx) -> Result<TwoFaStatus, AppError> {
    let user = ctx.user.as_ref().ok_or(AppError::Unauthenticated)?;
    let left = sqlx::query_scalar!(
        r#"SELECT count(*) AS "n!" FROM recovery_codes WHERE user_id = $1 AND used_at IS NULL"#,
        user.id
    )
    .fetch_one(pool)
    .await?;
    Ok(TwoFaStatus {
        info: user.twofa_info(),
        available: Modules::get().twofa_mode() != TwoFaMode::Off,
        recovery_left: left,
    })
}

/// Step 1: a fresh secret (stored encrypted, unconfirmed) + QR code.
pub async fn begin_enroll(pool: &PgPool, ctx: &Ctx) -> Result<EnrollStart, AppError> {
    let user = ctx.user.as_ref().ok_or(AppError::Unauthenticated)?;
    if Modules::get().twofa_mode() == TwoFaMode::Off {
        return Err(AppError::Disabled("twofa.disabled"));
    }
    if user.twofa_enrolled {
        return Err(AppError::Conflict("twofa.already_enrolled"));
    }
    let secret = totp::new_secret();
    let enc = totp::encrypt(&app_secret()?, &secret)?;
    sqlx::query!(
        "INSERT INTO user_totp (user_id, secret_enc) VALUES ($1, $2)
         ON CONFLICT (user_id) DO UPDATE SET secret_enc = $2, confirmed_at = NULL, last_step = 0,
                                             created_at = now()",
        user.id,
        enc
    )
    .execute(pool)
    .await?;
    let url = totp::otpauth_url(&Modules::get().twofa_issuer(), &user.email, &secret);
    Ok(EnrollStart {
        secret: totp::base32(&secret),
        qr_svg: totp::qr_svg(&url)?,
    })
}

async fn store_recovery_codes(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    user_id: Uuid,
) -> sqlx::Result<Vec<String>> {
    let codes = totp::recovery_codes();
    sqlx::query!("DELETE FROM recovery_codes WHERE user_id = $1", user_id)
        .execute(&mut **tx)
        .await?;
    let hashes: Vec<Vec<u8>> = codes.iter().map(|c| totp::recovery_hash(c)).collect();
    sqlx::query!(
        "INSERT INTO recovery_codes (user_id, code_hash) SELECT $1, unnest($2::bytea[])",
        user_id,
        &hashes
    )
    .execute(&mut **tx)
    .await?;
    Ok(codes)
}

/// Check a TOTP code against the stored secret and burn its time step. `Ok(false)` = wrong.
async fn check_totp(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    user_id: Uuid,
    code: &str,
    confirmed: bool,
) -> Result<bool, AppError> {
    let Some(row) = sqlx::query!(
        "SELECT secret_enc, last_step FROM user_totp WHERE user_id = $1
           AND (confirmed_at IS NOT NULL) = $2 FOR UPDATE",
        user_id,
        confirmed
    )
    .fetch_optional(&mut **tx)
    .await?
    else {
        return Ok(false);
    };
    let secret = totp::decrypt(&app_secret()?, &row.secret_enc)?;
    match totp::verify(&secret, code, row.last_step, totp::now()) {
        Some(step) => {
            sqlx::query!(
                "UPDATE user_totp SET last_step = $2,
                        confirmed_at = COALESCE(confirmed_at, now()) WHERE user_id = $1",
                user_id,
                step
            )
            .execute(&mut **tx)
            .await?;
            Ok(true)
        }
        None => Ok(false),
    }
}

/// Use one recovery code. `Ok(false)` = unknown or already used.
async fn use_recovery(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    user_id: Uuid,
    code: &str,
) -> sqlx::Result<bool> {
    Ok(sqlx::query!(
        "UPDATE recovery_codes SET used_at = now()
         WHERE user_id = $1 AND code_hash = $2 AND used_at IS NULL",
        user_id,
        totp::recovery_hash(code)
    )
    .execute(&mut **tx)
    .await?
    .rows_affected()
        == 1)
}

async fn changed(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ctx: &Ctx,
    user_id: Uuid,
    state: &str,
) -> sqlx::Result<()> {
    notify::send(
        tx,
        ctx.org_id,
        notify::Target::User(user_id),
        "twofa_changed",
        json!({ "state": state }),
        Some("/profile?tab=security"),
    )
    .await?;
    email::queue_for_user(tx, user_id, "twofa_changed", json!({ "state": state })).await
}

/// Step 2: confirm with a code → 10 recovery codes (shown once); other devices signed out.
pub async fn confirm_enroll(pool: &PgPool, ctx: &Ctx, code: &str) -> Result<Vec<String>, AppError> {
    let user = ctx.user.as_ref().ok_or(AppError::Unauthenticated)?;
    if !ratelimit::hit(
        &format!("2fa-enroll:{}", user.id),
        10,
        std::time::Duration::from_secs(900),
    )
    .await
    {
        return Err(AppError::Conflict("auth.rate_limited"));
    }
    let mut tx = pool.begin().await?;
    if !check_totp(&mut tx, user.id, code, false).await? {
        return Err(AppError::Validation("twofa.invalid_code"));
    }
    let codes = store_recovery_codes(&mut tx, user.id).await?;
    sqlx::query!(
        "DELETE FROM sessions WHERE user_id = $1 AND id <> $2",
        user.id,
        user.session_id
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query!(
        "UPDATE sessions SET mfa_verified_at = now() WHERE id = $1",
        user.session_id
    )
    .execute(&mut *tx)
    .await?;
    audit::record(&mut tx, user_entry(ctx, user, "twofa.enabled")).await?;
    changed(&mut tx, ctx, user.id, "enabled").await?;
    tx.commit().await?;
    Ok(codes)
}

async fn verify_password(pool: &PgPool, user_id: Uuid, pw: String) -> Result<(), AppError> {
    let stored = sqlx::query_scalar!("SELECT password_hash FROM users WHERE id = $1", user_id)
        .fetch_one(pool)
        .await?;
    if password::verify(pw, Some(stored)).await {
        Ok(())
    } else {
        Err(AppError::Validation("auth.wrong_password"))
    }
}

/// Turn 2FA off (needs the password). Refused while the mode makes it mandatory for me.
pub async fn disable(pool: &PgPool, ctx: &Ctx, pw: String) -> Result<(), AppError> {
    let user = ctx.user.as_ref().ok_or(AppError::Unauthenticated)?;
    if user.twofa_info().required {
        return Err(AppError::Conflict("twofa.required"));
    }
    verify_password(pool, user.id, pw).await?;
    let mut tx = pool.begin().await?;
    remove(&mut tx, user.id).await?;
    audit::record(&mut tx, user_entry(ctx, user, "twofa.disabled")).await?;
    changed(&mut tx, ctx, user.id, "disabled").await?;
    tx.commit().await?;
    Ok(())
}

async fn remove(tx: &mut sqlx::Transaction<'_, sqlx::Postgres>, user_id: Uuid) -> sqlx::Result<()> {
    sqlx::query!("DELETE FROM user_totp WHERE user_id = $1", user_id)
        .execute(&mut **tx)
        .await?;
    sqlx::query!("DELETE FROM recovery_codes WHERE user_id = $1", user_id)
        .execute(&mut **tx)
        .await?;
    Ok(())
}

/// New set of recovery codes (the old ones stop working); needs a current TOTP code.
pub async fn regenerate_codes(
    pool: &PgPool,
    ctx: &Ctx,
    code: &str,
) -> Result<Vec<String>, AppError> {
    let user = ctx.user.as_ref().ok_or(AppError::Unauthenticated)?;
    let mut tx = pool.begin().await?;
    if !check_totp(&mut tx, user.id, code, true).await? {
        return Err(AppError::Validation("twofa.invalid_code"));
    }
    let codes = store_recovery_codes(&mut tx, user.id).await?;
    audit::record(&mut tx, user_entry(ctx, user, "twofa.recovery_regenerated")).await?;
    tx.commit().await?;
    Ok(codes)
}

/// Admin reset (`users.reset_2fa`, M9.10): needs the admin's own password; the user's sessions end.
pub async fn admin_reset(
    pool: &PgPool,
    ctx: &Ctx,
    target: Uuid,
    admin_pw: String,
) -> Result<(), AppError> {
    let actor = ctx.require("users.reset_2fa")?;
    same_org(pool, actor, target).await?;
    crate::server::rbac::not_self(actor, target)?;
    verify_password(pool, actor.id, admin_pw).await?;
    let mut tx = pool.begin().await?;
    remove(&mut tx, target).await?;
    sqlx::query!("DELETE FROM sessions WHERE user_id = $1", target)
        .execute(&mut *tx)
        .await?;
    audit::record(
        &mut tx,
        Entry {
            entity_id: Some(target.to_string()),
            ..user_entry(ctx, actor, "user.twofa_reset")
        },
    )
    .await?;
    changed(&mut tx, ctx, target, "reset by an administrator").await?;
    tx.commit().await?;
    Ok(())
}

// ---------------------------------------------------------------- login step 2

/// `/auth/2fa`: the pending session from the password step + a TOTP or recovery code.
/// Five wrong codes drop the pending session and lock the account like failed passwords.
pub async fn verify_login(
    pool: &PgPool,
    ctx: &Ctx,
    pending: &str,
    code: &str,
) -> Result<LoginOk, AuthError> {
    let Some(p) = sqlx::query!(
        r#"SELECT s.id, s.user_id, s.mfa_attempts, u.email::text AS "email!", u.must_change_pw
           FROM sessions s JOIN users u ON u.id = s.user_id
           WHERE s.token_hash = $1 AND s.kind = 'pending_2fa' AND s.expires_at > now()
             AND u.is_active AND u.deleted_at IS NULL"#,
        token::hash(pending)
    )
    .fetch_optional(pool)
    .await?
    else {
        return Err(AuthError::Expired2fa);
    };
    let s = SecuritySettings::load(pool).await?;
    let mut tx = pool.begin().await?;
    let code = code.trim();
    let digits = code.chars().filter(char::is_ascii_digit).count();
    let (ok, recovery) = if digits == 6 && code.len() <= 7 {
        (
            check_totp(&mut tx, p.user_id, code, true)
                .await
                .map_err(app)?,
            false,
        )
    } else {
        (use_recovery(&mut tx, p.user_id, code).await?, true)
    };
    let entry = |action, success| Entry {
        org_id: Some(ctx.org_id),
        actor_id: Some(p.user_id),
        actor_email: Some(p.email.as_str()),
        action,
        ip: ctx.meta.ip,
        user_agent: ctx.meta.user_agent.as_deref(),
        success,
        ..Default::default()
    };
    if !ok {
        let attempts = p.mfa_attempts + 1;
        audit::record(&mut tx, entry("auth.2fa_failed", false)).await?;
        if attempts >= MAX_2FA_FAILURES {
            sqlx::query!("DELETE FROM sessions WHERE id = $1", p.id)
                .execute(&mut *tx)
                .await?;
            sqlx::query!(
                "UPDATE users SET locked_until = now() + make_interval(mins => $2) WHERE id = $1",
                p.user_id,
                i32::try_from(s.lockout_minutes).unwrap_or(15)
            )
            .execute(&mut *tx)
            .await?;
            audit::record(
                &mut tx,
                Entry {
                    entity_type: Some("user"),
                    entity_id: Some(p.user_id.to_string()),
                    ..entry("auth.account_locked", false)
                },
            )
            .await?;
            notify::send(
                &mut tx,
                ctx.org_id,
                notify::Target::User(p.user_id),
                "account_locked",
                json!({}),
                None,
            )
            .await?;
            tx.commit().await?;
            return Err(AuthError::Expired2fa);
        }
        sqlx::query!(
            "UPDATE sessions SET mfa_attempts = $2 WHERE id = $1",
            p.id,
            attempts
        )
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        return Err(AuthError::Invalid2fa);
    }
    sqlx::query!("DELETE FROM sessions WHERE id = $1", p.id)
        .execute(&mut *tx)
        .await?;
    if recovery {
        audit::record(&mut tx, entry("auth.recovery_code_used", true)).await?;
    }
    let token = finish_login(&mut tx, ctx, p.user_id, &p.email, true, s.session_hours).await?;
    tx.commit().await?;
    Ok(LoginOk {
        token,
        max_age_secs: s.session_hours * 3600,
        next: if p.must_change_pw {
            AfterLogin::ChangePassword
        } else {
            AfterLogin::Home
        },
    })
}

fn app(e: AppError) -> AuthError {
    AuthError::Internal(anyhow::anyhow!("{e}"))
}

/// When this account must enrol by (for the 2FA-required banner); `None` when not required.
#[allow(dead_code)]
pub fn deadline(user: &CurrentUser) -> Option<OffsetDateTime> {
    user.twofa_info().deadline
}

/// Hourly maintenance jobs (session purge, notification retention).
pub fn start_jobs(pool: PgPool) {
    tokio::spawn(async move {
        let mut tick = tokio::time::interval(std::time::Duration::from_secs(3600));
        loop {
            tick.tick().await;
            match purge_expired(&pool).await {
                Ok((s, t)) if s + t > 0 => {
                    tracing::info!(sessions = s, tokens = t, "purged expired")
                }
                Ok(_) => {}
                Err(e) => tracing::error!("session purge: {e}"),
            }
            match crate::server::services::users::purge_expired(&pool).await {
                Ok(n) if n > 0 => {
                    tracing::info!(users = n, "purged users past deleted_retention_days")
                }
                Ok(_) => {}
                Err(e) => tracing::error!("user purge job: {e:#}"),
            }
            if let Err(e) = crate::server::auth::ratelimit::purge(&pool).await {
                tracing::error!("rate limit purge: {e}");
            }
            if let Err(e) = crate::server::services::notifications::retention(&pool).await {
                tracing::error!("notification retention: {e}");
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::auth::session;
    use crate::server::services::auth::{login, tests::setup};

    async fn signed_in(pool: &PgPool, email: &str) -> (Ctx, Uuid) {
        let (mut ctx, id) = setup(pool, email).await;
        let ok = login(pool, &ctx, email, "correct-horse-1".into())
            .await
            .unwrap();
        ctx.user = session::load(pool, &ok.token, 12).await.unwrap();
        (ctx, id)
    }

    fn code_for(secret_b32: &str, step_offset: i64) -> String {
        let raw = totp_rs::Secret::Encoded(secret_b32.to_string())
            .to_bytes()
            .unwrap();
        let t = totp_rs::TOTP::new(totp_rs::Algorithm::SHA1, 6, 1, 30, raw).unwrap();
        t.generate((totp::now() as i64 + step_offset * 30) as u64)
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn enrol_then_login_needs_second_step(pool: PgPool) {
        let (ctx, id) = signed_in(&pool, "tf@test.local").await;
        let start = begin_enroll(&pool, &ctx).await.unwrap();
        assert!(start.qr_svg.contains("<svg"));
        assert!(confirm_enroll(&pool, &ctx, "000000").await.is_err());
        let used = code_for(&start.secret, 0);
        let codes = confirm_enroll(&pool, &ctx, &used).await.unwrap();
        assert_eq!(codes.len(), 10);

        // Password step now ends in a pending session, invisible to `load`.
        let (anon, _) = (
            Ctx {
                user: None,
                ..ctx.clone()
            },
            id,
        );
        let ok = login(&pool, &anon, "tf@test.local", "correct-horse-1".into())
            .await
            .unwrap();
        assert_eq!(ok.next, AfterLogin::TwoFactor);
        assert!(session::load(&pool, &ok.token, 12).await.unwrap().is_none());

        // The code just used at enrolment is refused (replay); a recovery code works once.
        let replay = verify_login(&pool, &anon, &ok.token, &used).await;
        assert!(matches!(replay, Err(AuthError::Invalid2fa)));
        let done = verify_login(&pool, &anon, &ok.token, &codes[0].to_uppercase())
            .await
            .unwrap();
        let user = session::load(&pool, &done.token, 12)
            .await
            .unwrap()
            .unwrap();
        assert!(user.twofa_enrolled);

        let again = login(&pool, &anon, "tf@test.local", "correct-horse-1".into())
            .await
            .unwrap();
        assert!(matches!(
            verify_login(&pool, &anon, &again.token, &codes[0]).await,
            Err(AuthError::Invalid2fa)
        ));
        let n = sqlx::query_scalar!(
            "SELECT count(*) FROM audit_logs WHERE action = 'auth.recovery_code_used'"
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(n, Some(1));
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn five_wrong_codes_lock_the_account(pool: PgPool) {
        let (ctx, _) = signed_in(&pool, "lk@test.local").await;
        let start = begin_enroll(&pool, &ctx).await.unwrap();
        confirm_enroll(&pool, &ctx, &code_for(&start.secret, 0))
            .await
            .unwrap();
        let anon = Ctx {
            user: None,
            ..ctx.clone()
        };
        let ok = login(&pool, &anon, "lk@test.local", "correct-horse-1".into())
            .await
            .unwrap();
        for _ in 0..4 {
            assert!(matches!(
                verify_login(&pool, &anon, &ok.token, "111111").await,
                Err(AuthError::Invalid2fa)
            ));
        }
        assert!(matches!(
            verify_login(&pool, &anon, &ok.token, "111111").await,
            Err(AuthError::Expired2fa)
        ));
        let locked = sqlx::query_scalar!(
            r#"SELECT locked_until > now() AS "l!" FROM users WHERE email = 'lk@test.local'"#
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert!(locked);
        let unread = sqlx::query_scalar!(
            r#"SELECT count(*) AS "n!" FROM notifications WHERE kind = 'account_locked'"#
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(unread, 1);
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn sessions_list_and_revoke_others(pool: PgPool) {
        let (ctx, _) = signed_in(&pool, "ss@test.local").await;
        let anon = Ctx {
            user: None,
            ..ctx.clone()
        };
        login(&pool, &anon, "ss@test.local", "correct-horse-1".into())
            .await
            .unwrap();
        let list = my_sessions(&pool, &ctx).await.unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list.iter().filter(|s| s.current).count(), 1);
        let current = list.iter().find(|s| s.current).unwrap().id;
        assert!(revoke_mine(&pool, &ctx, Some(current)).await.is_err());
        assert_eq!(revoke_mine(&pool, &ctx, None).await.unwrap(), 1);
        assert_eq!(my_sessions(&pool, &ctx).await.unwrap().len(), 1);
    }
}
