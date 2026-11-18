//! Email-based account flows (§20, M13.5): forgot / reset password, invitations, email
//! verification, test email and delivery log (M13.7). One-time links carry a random token;
//! only its SHA-256 is stored, it works once and expires.
use serde_json::json;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use super::auth::user_entry;
use crate::domain::account::{EmailLogPage, EmailLogRow};
use crate::domain::users::UserInput;
use crate::server::audit::{self, Entry};
use crate::server::auth::{password, ratelimit, token};
use crate::server::ctx::Ctx;
use crate::server::email;
use crate::server::error::AppError;
use crate::server::modules::Modules;
use crate::server::settings::SecuritySettings;

const RESET_MINUTES: i64 = 30;
const INVITE_HOURS: i64 = 72;
const VERIFY_HOURS: i64 = 48;

fn base_url() -> String {
    crate::server::state::get()
        .ok()
        .and_then(|s| s.config.app_url.clone())
        .unwrap_or_else(|| "http://127.0.0.1:8080".into())
        .trim_end_matches('/')
        .to_string()
}

/// New one-time token for `purpose`; earlier unused ones for the same purpose stop working.
async fn new_token(
    tx: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
    purpose: &str,
    minutes: i64,
) -> sqlx::Result<String> {
    sqlx::query!(
        "UPDATE auth_tokens SET used_at = now() WHERE user_id = $1 AND purpose = $2 AND used_at IS NULL",
        user_id,
        purpose
    )
    .execute(&mut **tx)
    .await?;
    let raw = token::generate();
    sqlx::query!(
        "INSERT INTO auth_tokens (token_hash, user_id, purpose, expires_at)
         VALUES ($1, $2, $3, now() + make_interval(mins => $4))",
        token::hash(&raw),
        user_id,
        purpose,
        i32::try_from(minutes).unwrap_or(60)
    )
    .execute(&mut **tx)
    .await?;
    Ok(raw)
}

/// Consume a token: the user it belongs to, or `auth.link_invalid`.
async fn take_token(
    tx: &mut Transaction<'_, Postgres>,
    raw: &str,
    purpose: &str,
) -> Result<(Uuid, Uuid, String), AppError> {
    let r = sqlx::query!(
        r#"UPDATE auth_tokens t SET used_at = now()
           FROM users u
           WHERE t.token_hash = $1 AND t.purpose = $2 AND t.used_at IS NULL
             AND t.expires_at > now() AND u.id = t.user_id AND u.deleted_at IS NULL
           RETURNING u.id, u.org_id, u.email::text AS "email!""#,
        token::hash(raw.trim()),
        purpose
    )
    .fetch_optional(&mut **tx)
    .await?
    .ok_or(AppError::Validation("auth.link_invalid"))?;
    Ok((r.id, r.org_id, r.email))
}

fn system_entry<'a>(
    org: Uuid,
    user: Uuid,
    email: &'a str,
    action: &'a str,
    ctx: &'a Ctx,
) -> Entry<'a> {
    Entry {
        org_id: Some(org),
        actor_id: Some(user),
        actor_email: Some(email),
        action,
        entity_type: Some("user"),
        entity_id: Some(user.to_string()),
        ip: ctx.meta.ip,
        user_agent: ctx.meta.user_agent.as_deref(),
        success: true,
        ..Default::default()
    }
}

/// Public. Always answers OK (no account enumeration); rate limited per IP and email.
pub async fn forgot_password(pool: &PgPool, ctx: &Ctx, email_in: &str) -> Result<(), AppError> {
    if !Modules::get().password_reset() {
        return Err(AppError::Disabled("auth.reset_disabled"));
    }
    let email_in = email_in.trim().to_lowercase();
    let ip = ctx.meta.ip.map(|i| i.to_string()).unwrap_or_default();
    let window = std::time::Duration::from_secs(3600);
    if !ratelimit::hit(&format!("forgot-ip:{ip}"), 20, window).await
        || !ratelimit::hit(&format!("forgot:{email_in}"), 3, window).await
    {
        return Ok(());
    }
    let Some(u) = sqlx::query!(
        "SELECT id FROM users WHERE org_id = $1 AND email = $2::text::citext
           AND deleted_at IS NULL AND is_active",
        ctx.org_id,
        email_in
    )
    .fetch_optional(pool)
    .await?
    else {
        return Ok(());
    };
    let mut tx = pool.begin().await?;
    let raw = new_token(&mut tx, u.id, "password_reset", RESET_MINUTES).await?;
    let link = format!("{}/auth/reset-password?token={raw}", base_url());
    email::queue_for_user(
        &mut tx,
        u.id,
        "password_reset",
        json!({ "link": link, "minutes": RESET_MINUTES }),
    )
    .await?;
    audit::record(
        &mut tx,
        system_entry(
            ctx.org_id,
            u.id,
            &email_in,
            "auth.password_reset_requested",
            ctx,
        ),
    )
    .await?;
    tx.commit().await?;
    Ok(())
}

/// Public: set a new password with a reset or invite link. Every session of the user ends;
/// an invite also confirms the email address.
pub async fn set_password_with_token(
    pool: &PgPool,
    ctx: &Ctx,
    raw: &str,
    purpose: &str,
    new_password: String,
) -> Result<(), AppError> {
    if !matches!(purpose, "password_reset" | "invite") {
        return Err(AppError::Validation("auth.link_invalid"));
    }
    let min = SecuritySettings::load(pool).await?.password_min_length;
    password::check_policy(&new_password, min).map_err(|p| AppError::Validation(p.code()))?;
    let hash = password::hash(new_password).await?;
    let mut tx = pool.begin().await?;
    let (id, org, email_addr) = take_token(&mut tx, raw, purpose).await?;
    sqlx::query!(
        "UPDATE users SET password_hash = $2, must_change_pw = false, failed_attempts = 0,
                locked_until = NULL, updated_at = now(),
                email_verified_at = CASE WHEN $3 THEN COALESCE(email_verified_at, now()) ELSE email_verified_at END
         WHERE id = $1",
        id,
        hash,
        purpose == "invite"
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query!("DELETE FROM sessions WHERE user_id = $1", id)
        .execute(&mut *tx)
        .await?;
    let action = if purpose == "invite" {
        "auth.invite_accepted"
    } else {
        "auth.password_reset"
    };
    audit::record(&mut tx, system_entry(org, id, &email_addr, action, ctx)).await?;
    if purpose == "password_reset" {
        email::queue_for_user(&mut tx, id, "password_changed", json!({})).await?;
    }
    tx.commit().await?;
    Ok(())
}

/// `users.create` + invites on: the user sets their own password from the email link.
pub async fn invite(pool: &PgPool, ctx: &Ctx, input: UserInput) -> Result<Uuid, AppError> {
    let actor = ctx.require("users.create")?;
    if !Modules::get().invites() {
        return Err(AppError::Disabled("auth.invites_disabled"));
    }
    // A random password nobody knows; the invite link replaces it.
    let input = UserInput {
        password: format!("{}A1!", token::generate()),
        ..input
    };
    let id = super::users::create(pool, ctx, input).await?;
    let mut tx = pool.begin().await?;
    sqlx::query!("UPDATE users SET must_change_pw = false WHERE id = $1", id)
        .execute(&mut *tx)
        .await?;
    let raw = new_token(&mut tx, id, "invite", INVITE_HOURS * 60).await?;
    let link = format!("{}/auth/accept-invite?token={raw}", base_url());
    email::queue_for_user(
        &mut tx,
        id,
        "invite",
        json!({ "link": link, "hours": INVITE_HOURS, "inviter": actor.full_name }),
    )
    .await?;
    audit::record(
        &mut tx,
        Entry {
            entity_id: Some(id.to_string()),
            ..user_entry(ctx, actor, "user.invite")
        },
    )
    .await?;
    tx.commit().await?;
    Ok(id)
}

/// Own account: send a verification link to my address.
pub async fn send_verification(pool: &PgPool, ctx: &Ctx) -> Result<(), AppError> {
    let user = ctx.user.as_ref().ok_or(AppError::Unauthenticated)?;
    if !Modules::get().email_verification() {
        return Err(AppError::Disabled("auth.verification_disabled"));
    }
    if !ratelimit::hit(
        &format!("verify:{}", user.id),
        3,
        std::time::Duration::from_secs(3600),
    )
    .await
    {
        return Err(AppError::Conflict("auth.rate_limited"));
    }
    let mut tx = pool.begin().await?;
    let raw = new_token(&mut tx, user.id, "verify_email", VERIFY_HOURS * 60).await?;
    let link = format!("{}/auth/verify-email?token={raw}", base_url());
    email::queue_for_user(
        &mut tx,
        user.id,
        "verify_email",
        json!({ "link": link, "hours": VERIFY_HOURS }),
    )
    .await?;
    tx.commit().await?;
    Ok(())
}

/// Public: confirm an email address from the link.
pub async fn verify_email(pool: &PgPool, ctx: &Ctx, raw: &str) -> Result<(), AppError> {
    let mut tx = pool.begin().await?;
    let (id, org, email_addr) = take_token(&mut tx, raw, "verify_email").await?;
    sqlx::query!(
        "UPDATE users SET email_verified_at = now() WHERE id = $1",
        id
    )
    .execute(&mut *tx)
    .await?;
    audit::record(
        &mut tx,
        system_entry(org, id, &email_addr, "auth.email_verified", ctx),
    )
    .await?;
    tx.commit().await?;
    Ok(())
}

/// `email.send_test`: queue the test template to an address and send it now (M13.7).
pub async fn send_test(pool: &PgPool, ctx: &Ctx, to: String) -> Result<(), AppError> {
    let actor = ctx.require("email.send_test")?;
    if !crate::domain::valid_email(&to) {
        return Err(AppError::Validation("users.email_invalid"));
    }
    let rendered =
        email::render("test", &json!({}), &Modules::get().app_name()).ok_or(AppError::NotFound)?;
    let result = email::send_resend(to.trim(), &rendered).await;
    let mut tx = pool.begin().await?;
    sqlx::query!(
        r#"INSERT INTO email_outbox (org_id, to_email, template, status, attempts, sent_at, provider_id, last_error)
           VALUES ($1, $2::text::citext, 'test', $3, 1, CASE WHEN $4 THEN now() END, $5, $6)"#,
        ctx.org_id,
        to.trim(),
        if result.is_ok() { "sent" } else { "failed" },
        result.is_ok(),
        result.as_ref().ok(),
        result.as_ref().err().map(|e| e.to_string())
    )
    .execute(&mut *tx)
    .await?;
    audit::record(
        &mut tx,
        Entry {
            after: Some(json!({ "to": to, "ok": result.is_ok() })),
            success: result.is_ok(),
            ..user_entry(ctx, actor, "email.test")
        },
    )
    .await?;
    tx.commit().await?;
    match result {
        Ok(_) => Ok(()),
        Err(e) => {
            tracing::warn!("test email failed: {e}");
            Err(AppError::Conflict("email.send_failed"))
        }
    }
}

/// `email.view_log`: latest outbox rows, newest first.
pub async fn log(pool: &PgPool, ctx: &Ctx, offset: i64) -> Result<EmailLogPage, AppError> {
    ctx.require("email.view_log")?;
    let rows = sqlx::query!(
        r#"SELECT id, to_email::text AS "to!", template, status, attempts, last_error, created_at, sent_at
           FROM email_outbox WHERE org_id = $1 OR org_id IS NULL
           ORDER BY id DESC LIMIT 25 OFFSET $2"#,
        ctx.org_id,
        offset.max(0)
    )
    .fetch_all(pool)
    .await?;
    let total = sqlx::query_scalar!(
        r#"SELECT count(*) AS "n!" FROM email_outbox WHERE org_id = $1 OR org_id IS NULL"#,
        ctx.org_id
    )
    .fetch_one(pool)
    .await?;
    Ok(EmailLogPage {
        rows: rows
            .into_iter()
            .map(|r| EmailLogRow {
                id: r.id,
                to: r.to,
                template: r.template,
                status: r.status,
                attempts: r.attempts,
                error: r.last_error,
                created_at: r.created_at,
                sent_at: r.sent_at,
            })
            .collect(),
        total,
    })
}

/// Whether "Send test email" succeeded in the last 24 h (settings dependency rule, §17).
pub async fn tested_recently(pool: &PgPool) -> sqlx::Result<bool> {
    sqlx::query_scalar!(
        r#"SELECT EXISTS (SELECT 1 FROM email_outbox WHERE template = 'test' AND status IN ('sent','delivered')
                          AND created_at > now() - interval '24 hours') AS "e!""#
    )
    .fetch_one(pool)
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::services::auth::{login, tests::setup};

    #[sqlx::test(migrations = "./migrations")]
    async fn reset_link_works_once_and_ends_sessions(pool: PgPool) {
        let (ctx, id) = setup(&pool, "rs@test.local").await;
        login(&pool, &ctx, "rs@test.local", "correct-horse-1".into())
            .await
            .unwrap();
        let mut tx = pool.begin().await.unwrap();
        let raw = new_token(&mut tx, id, "password_reset", 30).await.unwrap();
        tx.commit().await.unwrap();
        // Wrong purpose and a bad token are refused.
        assert!(
            set_password_with_token(&pool, &ctx, &raw, "invite", "brand-new-pass-9".into())
                .await
                .is_err()
        );
        set_password_with_token(
            &pool,
            &ctx,
            &raw,
            "password_reset",
            "brand-new-pass-9".into(),
        )
        .await
        .unwrap();
        assert!(
            set_password_with_token(&pool, &ctx, &raw, "password_reset", "another-pass-9".into())
                .await
                .is_err()
        );
        let sessions = sqlx::query_scalar!(
            r#"SELECT count(*) AS "n!" FROM sessions WHERE user_id = $1"#,
            id
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(sessions, 0);
        assert!(
            login(&pool, &ctx, "rs@test.local", "brand-new-pass-9".into())
                .await
                .is_ok()
        );
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn forgot_password_is_off_without_email(pool: PgPool) {
        let (ctx, _) = setup(&pool, "fp@test.local").await;
        assert!(matches!(
            forgot_password(&pool, &ctx, "fp@test.local").await,
            Err(AppError::Disabled(_))
        ));
    }

    #[test]
    fn queue_renders_link_param() {
        let r = email::render(
            "password_reset",
            &json!({"name": "A", "link": "https://x/r?token=t", "minutes": 30}),
            "S",
        )
        .unwrap();
        assert!(r.html.contains("token=t"));
    }
}
