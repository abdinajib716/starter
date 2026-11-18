//! System Setting (§9.6, §17, §27): list grouped settings with secret status, update them with
//! type + per-key + dependency validation (audited, broadcast to all instances), and the
//! feature-flag admin.
use std::collections::HashMap;

use serde_json::{Value, json};
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::settings::{FlagItem, FlagUpdate, SettingItem, SettingsView};
use crate::server::audit::{self, Entry};
use crate::server::ctx::Ctx;
use crate::server::error::AppError;
use crate::server::settings::{self as reg, Kind};

/// Secrets are read from the environment only; the page shows configured ✓ / missing ✗.
pub const SECRETS: &[&str] = &[
    "RESEND_API_KEY",
    "BACKUP_PUBLIC_KEY",
    "BACKUP_ENCRYPTION_KEY",
    "R2_ACCOUNT_ID",
    "R2_ACCESS_KEY_ID",
    "R2_SECRET_ACCESS_KEY",
    "R2_BUCKET",
    "GDRIVE_SERVICE_ACCOUNT_JSON",
    "GDRIVE_FOLDER_ID",
];

pub fn secret_set(name: &str) -> bool {
    std::env::var(name).is_ok_and(|v| !v.trim().is_empty())
}

pub async fn view(pool: &PgPool, ctx: &Ctx) -> Result<SettingsView, AppError> {
    ctx.require("settings.view")?;
    let rows = sqlx::query!(
        r#"SELECT s.key, s.value, s.updated_at, u.email::text AS updated_by
           FROM settings s LEFT JOIN users u ON u.id = s.updated_by"#
    )
    .fetch_all(pool)
    .await?;
    let by_key: HashMap<String, _> = rows.into_iter().map(|r| (r.key.clone(), r)).collect();
    let items = reg::REGISTRY
        .iter()
        .map(|d| {
            let row = by_key.get(d.key);
            let (options, min, max) = match d.kind {
                Kind::Enum(o) => (o.iter().map(|s| s.to_string()).collect(), None, None),
                Kind::Int(lo, hi) => (vec![], Some(lo), Some(hi)),
                _ => (vec![], None, None),
            };
            SettingItem {
                key: d.key.into(),
                value: row
                    .map(|r| r.value.to_string())
                    .unwrap_or_else(|| (d.default)().to_string()),
                kind: d.kind.name().into(),
                options,
                min,
                max,
                category: d.category.into(),
                label: d.label.into(),
                description: d.description.into(),
                updated_at: row.map(|r| r.updated_at),
                updated_by: row.and_then(|r| r.updated_by.clone()),
            }
        })
        .collect();
    let st = crate::server::state::get().map_err(|e| anyhow::anyhow!(e.to_string()))?;
    Ok(SettingsView {
        items,
        secrets: SECRETS
            .iter()
            .map(|s| (s.to_string(), secret_set(s)))
            .collect(),
        tenancy: st.config.tenancy_mode,
        version: env!("CARGO_PKG_VERSION").into(),
    })
}

/// §17 dependency rules plus per-key rules, over the merged (current + changed) values.
/// `secret` tells whether an environment secret is present. Returns the broken rule's code.
pub fn check(
    merged: &HashMap<String, Value>,
    changed: &[String],
    secret: &dyn Fn(&str) -> bool,
) -> Result<(), &'static str> {
    let b = |k: &str| merged.get(k).and_then(Value::as_bool).unwrap_or(false);
    let s = |k: &str| {
        merged
            .get(k)
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string()
    };
    let touched = |k: &str| changed.iter().any(|c| c == k);

    if b("email.enabled") && !secret("RESEND_API_KEY") {
        return Err("settings.email_needs_key");
    }
    if touched("security.2fa_mode") && s("security.2fa_mode").starts_with("required") {
        // Self-lockout guard (M9.12): the acting admin must have 2FA enrolled. `secret` also
        // answers the pseudo-key `@actor_2fa`.
        if !secret("@actor_2fa") {
            return Err("settings.2fa_actor_not_enrolled");
        }
    }
    if b("api.public_enabled") && !b("email.enabled") {
        return Err("settings.api_needs_email");
    }
    let r2 = || {
        [
            "R2_ACCOUNT_ID",
            "R2_ACCESS_KEY_ID",
            "R2_SECRET_ACCESS_KEY",
            "R2_BUCKET",
        ]
        .iter()
        .all(|k| secret(k))
    };
    let gdrive = || {
        ["GDRIVE_SERVICE_ACCOUNT_JSON", "GDRIVE_FOLDER_ID"]
            .iter()
            .all(|k| secret(k))
    };
    let driver_ok = |d: &str| match d {
        "r2" => r2(),
        "gdrive" => gdrive(),
        _ => true,
    };
    if !driver_ok(&s("storage.primary_driver")) {
        return Err("settings.storage_not_configured");
    }
    if b("backup.offsite_enabled")
        && (!secret("BACKUP_PUBLIC_KEY") || !driver_ok(&s("backup.offsite_driver")))
    {
        return Err("settings.offsite_not_configured");
    }
    match s("backup.encrypt_mode").as_str() {
        "age" if !secret("BACKUP_PUBLIC_KEY") => return Err("settings.encryption_key_missing"),
        "symmetric" if !secret("BACKUP_ENCRYPTION_KEY") => {
            return Err("settings.encryption_key_missing");
        }
        _ => {}
    }
    let locales: Vec<String> = s("i18n.enabled_locales")
        .split(',')
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();
    if locales.is_empty()
        || !locales
            .iter()
            .all(|l| crate::i18n::SUPPORTED.contains(&l.as_str()))
    {
        return Err("settings.locales_unsupported");
    }
    if !locales.contains(&s("i18n.default_locale")) {
        return Err("settings.default_locale_not_enabled");
    }
    if !crate::util::time::valid_zone(&s("app.timezone")) {
        return Err("settings.timezone_invalid");
    }
    if s("backup.schedule_cron").split_whitespace().count() != 6 {
        return Err("settings.cron_invalid");
    }
    Ok(())
}

/// Validate and save `changes` (key → JSON text). One transaction, one audit row with the
/// before/after of the changed keys; the trigger tells every instance to reload.
pub async fn update(
    pool: &PgPool,
    ctx: &Ctx,
    changes: Vec<(String, String)>,
) -> Result<usize, AppError> {
    let actor = ctx.require("settings.update")?;
    let mut parsed: Vec<(String, Value)> = Vec::new();
    for (k, raw) in changes {
        let v: Value = serde_json::from_str(&raw)
            .map_err(|_| AppError::Validation("settings.invalid_value"))?;
        reg::validate(&k, &v).map_err(AppError::Validation)?;
        parsed.push((k, v));
    }
    let mut tx = pool.begin().await?;
    let current: HashMap<String, Value> =
        sqlx::query!("SELECT key, value FROM settings FOR UPDATE")
            .fetch_all(&mut *tx)
            .await?
            .into_iter()
            .map(|r| (r.key, r.value))
            .collect();
    let mut merged = current.clone();
    let mut changed = vec![];
    for (k, v) in &parsed {
        if current.get(k) != Some(v) {
            changed.push(k.clone());
        }
        merged.insert(k.clone(), v.clone());
    }
    if changed.is_empty() {
        return Ok(0);
    }
    let enrolled = actor.twofa_enrolled;
    let known = move |n: &str| {
        if n == "@actor_2fa" {
            enrolled
        } else {
            secret_set(n)
        }
    };
    check(&merged, &changed, &known).map_err(AppError::Conflict)?;
    for k in &changed {
        sqlx::query!(
            "UPDATE settings SET value = $2, updated_by = $3, updated_at = now() WHERE key = $1",
            k,
            merged[k],
            actor.id
        )
        .execute(&mut *tx)
        .await?;
    }
    let before: serde_json::Map<String, Value> = changed
        .iter()
        .map(|k| (k.clone(), current.get(k).cloned().unwrap_or(Value::Null)))
        .collect();
    let after: serde_json::Map<String, Value> = changed
        .iter()
        .map(|k| (k.clone(), merged[k].clone()))
        .collect();
    audit::record(
        &mut tx,
        Entry {
            org_id: Some(ctx.org_id),
            actor_id: Some(actor.id),
            actor_email: Some(actor.email.as_str()),
            action: "settings.update",
            entity_type: Some("settings"),
            before: Some(Value::Object(before)),
            after: Some(Value::Object(after)),
            ip: ctx.meta.ip,
            user_agent: ctx.meta.user_agent.as_deref(),
            success: true,
            ..Default::default()
        },
    )
    .await?;
    tx.commit().await?;
    // This instance applies it now; the others on the NOTIFY.
    if let Some(c) = reg::cache() {
        c.reload(pool).await?;
    }
    Ok(changed.len())
}

// ---------- feature flags (§27) ----------

pub async fn flags(pool: &PgPool, ctx: &Ctx) -> Result<Vec<FlagItem>, AppError> {
    ctx.require("flags.view")?;
    let rows = sqlx::query!(
        r#"SELECT f.key, f.description, f.enabled, f.rollout_percent, f.allowed_role_ids,
                  f.allowed_user_ids, f.updated_at, u.email::text AS updated_by,
                  (f.enabled AND f.rollout_percent = 100 AND cardinality(f.allowed_role_ids) = 0
                   AND f.updated_at < now() - interval '30 days') AS "stale!"
           FROM feature_flags f LEFT JOIN users u ON u.id = f.updated_by
           WHERE f.org_id IS NULL ORDER BY f.key"#
    )
    .fetch_all(pool)
    .await?;
    let mut out = vec![];
    for r in rows {
        let beta_emails = sqlx::query_scalar!(
            r#"SELECT email::text AS "e!" FROM users WHERE id = ANY($1) ORDER BY 1"#,
            &r.allowed_user_ids
        )
        .fetch_all(pool)
        .await?;
        out.push(FlagItem {
            key: r.key,
            description: r.description,
            enabled: r.enabled,
            rollout_percent: r.rollout_percent,
            allowed_role_ids: r.allowed_role_ids,
            beta_emails,
            updated_at: r.updated_at,
            updated_by: r.updated_by,
            stale: r.stale,
        });
    }
    Ok(out)
}

pub async fn update_flag(pool: &PgPool, ctx: &Ctx, f: FlagUpdate) -> Result<(), AppError> {
    let actor = ctx.require("flags.update")?;
    if !(0..=100).contains(&f.rollout_percent) {
        return Err(AppError::Validation("flags.percent_range"));
    }
    let mut tx = pool.begin().await?;
    let before = sqlx::query!(
        "SELECT enabled, rollout_percent, allowed_role_ids, allowed_user_ids FROM feature_flags
         WHERE key = $1 AND org_id IS NULL FOR UPDATE",
        f.key
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(AppError::NotFound)?;
    let emails: Vec<String> = f
        .beta_emails
        .iter()
        .map(|e| e.trim().to_lowercase())
        .filter(|e| !e.is_empty())
        .collect();
    let user_ids: Vec<Uuid> = sqlx::query_scalar!(
        "SELECT id FROM users WHERE org_id = $1 AND deleted_at IS NULL AND email = ANY($2::text[]::citext[])",
        ctx.org_id,
        &emails
    )
    .fetch_all(&mut *tx)
    .await?;
    if user_ids.len() != emails.len() {
        return Err(AppError::Validation("flags.unknown_user"));
    }
    sqlx::query!(
        "UPDATE feature_flags SET enabled = $2, rollout_percent = $3, allowed_role_ids = $4,
                allowed_user_ids = $5, updated_by = $6, updated_at = now()
         WHERE key = $1 AND org_id IS NULL",
        f.key,
        f.enabled,
        f.rollout_percent,
        &f.allowed_role_ids,
        &user_ids,
        actor.id
    )
    .execute(&mut *tx)
    .await?;
    audit::record(
        &mut tx,
        Entry {
            org_id: Some(ctx.org_id),
            actor_id: Some(actor.id),
            actor_email: Some(actor.email.as_str()),
            action: "flag.update",
            entity_type: Some("feature_flag"),
            entity_id: Some(f.key.clone()),
            before: Some(json!({ "enabled": before.enabled, "rollout_percent": before.rollout_percent,
                                 "roles": before.allowed_role_ids, "beta_users": before.allowed_user_ids.len() })),
            after: Some(json!({ "enabled": f.enabled, "rollout_percent": f.rollout_percent,
                                "roles": f.allowed_role_ids, "beta_users": emails })),
            ip: ctx.meta.ip,
            user_agent: ctx.meta.user_agent.as_deref(),
            success: true,
        },
    )
    .await?;
    tx.commit().await?;
    crate::server::flags::reload(pool).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn defaults() -> HashMap<String, Value> {
        reg::REGISTRY
            .iter()
            .map(|d| (d.key.to_string(), (d.default)()))
            .collect()
    }

    #[test]
    fn defaults_pass_the_rules() {
        assert_eq!(check(&defaults(), &[], &|_| false), Ok(()));
    }

    #[test]
    fn dependency_rules() {
        let none = |_: &str| false;
        let all = |_: &str| true;
        let with = |k: &str, v: Value| {
            let mut m = defaults();
            m.insert(k.into(), v);
            m
        };
        assert_eq!(
            check(&with("email.enabled", json!(true)), &[], &none),
            Err("settings.email_needs_key")
        );
        assert_eq!(
            check(&with("email.enabled", json!(true)), &[], &all),
            Ok(())
        );
        assert_eq!(
            check(&with("api.public_enabled", json!(true)), &[], &all),
            Err("settings.api_needs_email")
        );
        assert_eq!(
            check(
                &with("security.2fa_mode", json!("required_all")),
                &["security.2fa_mode".into()],
                &none
            ),
            Err("settings.2fa_actor_not_enrolled")
        );
        assert_eq!(
            check(
                &with("security.2fa_mode", json!("required_all")),
                &["security.2fa_mode".into()],
                &all
            ),
            Ok(())
        );
        assert_eq!(
            check(&with("storage.primary_driver", json!("r2")), &[], &none),
            Err("settings.storage_not_configured")
        );
        assert_eq!(
            check(&with("backup.offsite_enabled", json!(true)), &[], &none),
            Err("settings.offsite_not_configured")
        );
        assert_eq!(
            check(&with("i18n.enabled_locales", json!("en,xx")), &[], &none),
            Err("settings.locales_unsupported")
        );
        assert_eq!(
            check(&with("i18n.default_locale", json!("so")), &[], &none),
            Ok(())
        );
        assert_eq!(
            check(&with("app.timezone", json!("Africa/Mogadishu")), &[], &none),
            Ok(())
        );
        assert_eq!(
            check(&with("app.timezone", json!("drop table")), &[], &none),
            Err("settings.timezone_invalid")
        );
        assert_eq!(
            check(
                &with("backup.schedule_cron", json!("0 2 * * *")),
                &[],
                &none
            ),
            Err("settings.cron_invalid")
        );
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn update_is_validated_audited_and_applied(pool: PgPool) {
        crate::server::rbac::sync_permissions(&pool).await.unwrap();
        reg::seed(&pool).await.unwrap();
        let org = crate::server::seed::ensure_default_org(&pool)
            .await
            .unwrap();
        let role = crate::server::seed::ensure_super_admin_role(&pool)
            .await
            .unwrap();
        let me: Uuid = sqlx::query_scalar("INSERT INTO users (org_id, email, full_name, password_hash, role_id) VALUES ($1,'a@test.local','A','x',$2) RETURNING id")
            .bind(org)
            .bind(role)
            .fetch_one(&pool)
            .await
            .unwrap();
        let ctx = Ctx {
            org_id: org,
            user: Some(crate::server::ctx::CurrentUser {
                id: me,
                session_id: Uuid::new_v4(),
                org_id: org,
                email: "a@test.local".into(),
                full_name: "A".into(),
                role_id: role,
                role_name: "Super Admin".into(),
                is_super_admin: true,
                permissions: crate::server::rbac::all_keys(),
                must_change_pw: false,
                locale: None,
                timezone: None,
                twofa_enrolled: false,
                role_require_2fa: false,
                twofa_since: time::OffsetDateTime::UNIX_EPOCH,
            }),
            request_id: None,
            meta: Default::default(),
        };
        let bad = update(
            &pool,
            &ctx,
            vec![("security.session_hours".into(), "0".into())],
        )
        .await;
        assert!(matches!(
            bad,
            Err(AppError::Validation("settings.invalid_value"))
        ));
        let n = update(
            &pool,
            &ctx,
            vec![
                ("security.session_hours".into(), "24".into()),
                ("app.name".into(), "\"Starter\"".into()),
            ],
        )
        .await
        .unwrap();
        assert_eq!(n, 1, "unchanged keys are not written");
        assert_eq!(
            reg::SecuritySettings::load(&pool)
                .await
                .unwrap()
                .session_hours,
            24
        );
        let after: Value =
            sqlx::query_scalar("SELECT after FROM audit_logs WHERE action = 'settings.update'")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(after, json!({ "security.session_hours": 24 }));
    }
}
