//! Transactional email via Resend (§20, M13): an outbox written inside the business
//! transaction, a worker that sends with back-off, templates with an `en` fallback, and the
//! delivery webhook (Svix-signed).
use std::time::Duration;

use serde_json::{Value, json};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use super::modules::Modules;
use super::settings;

const TEMPLATES: &[(&str, &str)] = &[
    ("invite", include_str!("../../templates/email/invite.txt")),
    (
        "password_reset",
        include_str!("../../templates/email/password_reset.txt"),
    ),
    (
        "verify_email",
        include_str!("../../templates/email/verify_email.txt"),
    ),
    (
        "new_login_alert",
        include_str!("../../templates/email/new_login_alert.txt"),
    ),
    (
        "password_changed",
        include_str!("../../templates/email/password_changed.txt"),
    ),
    (
        "twofa_changed",
        include_str!("../../templates/email/twofa_changed.txt"),
    ),
    (
        "backup_failed",
        include_str!("../../templates/email/backup_failed.txt"),
    ),
    ("test", include_str!("../../templates/email/test.txt")),
];

/// Back-off after attempt n (1-based); after the last one the email is `failed`.
const BACKOFF_MINS: [i32; 4] = [1, 5, 30, 120];
const MAX_ATTEMPTS: i32 = 5;

/// Which notification kind (Profile → Notifications) controls a template; `None` = always sent.
fn pref_kind(template: &str) -> Option<&'static str> {
    match template {
        "new_login_alert" => Some("new_device"),
        "password_changed" => Some("password_changed"),
        "twofa_changed" => Some("twofa_changed"),
        "backup_failed" => Some("backup_failed"),
        _ => None,
    }
}

pub struct Rendered {
    pub subject: String,
    pub text: String,
    pub html: String,
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Fill `{{key}}` from `params` (+ `app_name`). Templates are plain text: the first line is
/// `Subject: …`; the HTML version is generated (paragraphs, links become buttons).
pub fn render(template: &str, params: &Value, app_name: &str) -> Option<Rendered> {
    let src = TEMPLATES.iter().find(|(n, _)| *n == template)?.1;
    let fill = |s: &str| {
        let mut out = s.replace("{{app_name}}", app_name);
        if let Some(map) = params.as_object() {
            for (k, v) in map {
                let v = v
                    .as_str()
                    .map(str::to_owned)
                    .unwrap_or_else(|| v.to_string());
                out = out.replace(&format!("{{{{{k}}}}}"), &v);
            }
        }
        out
    };
    let (first, body) = src.split_once('\n')?;
    let subject = fill(first.trim_start_matches("Subject:").trim());
    let text = fill(body.trim());
    let paras: String = text
        .split("\n\n")
        .map(|p| {
            let p = p.trim();
            if p.starts_with("http://") || p.starts_with("https://") {
                format!(
                    r#"<p style="margin:24px 0"><a href="{0}" style="background:#18181b;color:#fff;padding:10px 18px;border-radius:6px;text-decoration:none;display:inline-block">{1}</a></p><p style="font-size:12px;color:#71717a;word-break:break-all">{0}</p>"#,
                    esc(p),
                    esc(&subject)
                )
            } else {
                format!("<p>{}</p>", esc(p).replace('\n', "<br>"))
            }
        })
        .collect();
    let html = format!(
        r#"<!doctype html><html><body style="margin:0;background:#f4f4f5;font-family:-apple-system,Segoe UI,Roboto,sans-serif;color:#18181b"><div style="max-width:520px;margin:32px auto;background:#fff;border:1px solid #e4e4e7;border-radius:10px;padding:28px;font-size:14px;line-height:1.6"><div style="font-weight:600;font-size:16px;margin-bottom:16px">{}</div>{paras}</div></body></html>"#,
        esc(app_name)
    );
    Some(Rendered {
        subject,
        text,
        html,
    })
}

/// Queue an email inside `tx`. A suppressed address (bounce / complaint) is recorded as
/// `suppressed` and never sent.
pub async fn queue(
    tx: &mut Transaction<'_, Postgres>,
    org_id: Option<Uuid>,
    to: &str,
    template: &str,
    locale: &str,
    params: Value,
) -> sqlx::Result<()> {
    sqlx::query!(
        r#"INSERT INTO email_outbox (org_id, to_email, template, locale, params, status)
           VALUES ($1, $2::text::citext, $3, $4, $5,
                   CASE WHEN EXISTS (SELECT 1 FROM email_suppressions WHERE email = $2::text::citext)
                        THEN 'suppressed' ELSE 'pending' END)"#,
        org_id,
        to,
        template,
        locale,
        params
    )
    .execute(&mut **tx)
    .await?;
    sqlx::query!("SELECT pg_notify('email_outbox', '')")
        .execute(&mut **tx)
        .await?;
    Ok(())
}

/// Queue a template for a user (their address, name, language), if the email module is on and
/// the user hasn't switched this kind off.
pub async fn queue_for_user(
    tx: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
    template: &str,
    mut params: Value,
) -> sqlx::Result<()> {
    if !Modules::get().email_enabled() {
        return Ok(());
    }
    let kind = pref_kind(template).unwrap_or("");
    let Some(u) = sqlx::query!(
        r#"SELECT u.org_id, u.email::text AS "email!", u.full_name, u.locale,
                  COALESCE((SELECT p.email FROM notification_prefs p
                            WHERE p.user_id = u.id AND p.kind = $2), true) AS "wants!"
           FROM users u WHERE u.id = $1 AND u.deleted_at IS NULL"#,
        user_id,
        kind
    )
    .fetch_optional(&mut **tx)
    .await?
    else {
        return Ok(());
    };
    if !u.wants {
        return Ok(());
    }
    if let Some(m) = params.as_object_mut() {
        m.entry("name").or_insert(json!(u.full_name));
    }
    let locale = u.locale.unwrap_or_else(|| "en".into());
    queue(tx, Some(u.org_id), &u.email, template, &locale, params).await
}

fn app_name() -> String {
    settings::cache()
        .map(|c| c.get("app.name"))
        .and_then(|v| v.as_str().map(str::to_owned))
        .unwrap_or_else(|| "Starter".into())
}

fn setting(key: &str) -> String {
    settings::cache()
        .map(|c| c.get(key))
        .and_then(|v| v.as_str().map(str::to_owned))
        .unwrap_or_default()
}

/// Send one rendered email through Resend; returns the provider id.
pub async fn send_resend(to: &str, r: &Rendered) -> anyhow::Result<String> {
    let key = std::env::var("RESEND_API_KEY")
        .ok()
        .filter(|k| !k.trim().is_empty())
        .ok_or_else(|| anyhow::anyhow!("RESEND_API_KEY is not set"))?;
    let from_addr = setting("email.from_address");
    if from_addr.is_empty() {
        anyhow::bail!("email.from_address is not set");
    }
    let from_name = setting("email.from_name");
    let from = if from_name.is_empty() {
        from_addr
    } else {
        format!("{from_name} <{from_addr}>")
    };
    let mut body =
        json!({ "from": from, "to": [to], "subject": r.subject, "html": r.html, "text": r.text });
    let reply = setting("email.reply_to");
    if !reply.is_empty() {
        body["reply_to"] = json!(reply);
    }
    let base = std::env::var("RESEND_API_URL").unwrap_or_else(|_| "https://api.resend.com".into());
    let res = client()
        .post(format!("{base}/emails"))
        .bearer_auth(key)
        .json(&body)
        .send()
        .await?;
    let status = res.status();
    let v: Value = res.json().await.unwrap_or(Value::Null);
    if !status.is_success() {
        anyhow::bail!(
            "Resend {status}: {}",
            v["message"].as_str().unwrap_or("request failed")
        );
    }
    Ok(v["id"].as_str().unwrap_or_default().to_string())
}

fn client() -> &'static reqwest::Client {
    static C: std::sync::OnceLock<reqwest::Client> = std::sync::OnceLock::new();
    C.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
            .expect("HTTP client")
    })
}

/// Send what is due. Rows are claimed with `FOR UPDATE SKIP LOCKED`, so several app servers
/// can run the worker without sending twice.
pub async fn send_due(pool: &PgPool) -> anyhow::Result<usize> {
    let mut tx = pool.begin().await?;
    let rows = sqlx::query!(
        r#"SELECT id, org_id, to_email::text AS "to!", template, params, attempts
           FROM email_outbox WHERE status = 'pending' AND next_attempt_at <= now()
           ORDER BY id LIMIT 10 FOR UPDATE SKIP LOCKED"#
    )
    .fetch_all(&mut *tx)
    .await?;
    let n = rows.len();
    let app = app_name();
    for r in rows {
        let result = match render(&r.template, &r.params, &app) {
            Some(rendered) => send_resend(&r.to, &rendered).await,
            None => Err(anyhow::anyhow!("unknown template {}", r.template)),
        };
        match result {
            Ok(id) => {
                // Links carry one-time tokens: don't keep them once sent.
                sqlx::query!(
                    "UPDATE email_outbox SET status = 'sent', sent_at = now(), provider_id = $2,
                            attempts = attempts + 1, last_error = NULL, params = params - 'link'
                     WHERE id = $1",
                    r.id,
                    id
                )
                .execute(&mut *tx)
                .await?;
            }
            Err(e) => {
                let attempts = r.attempts + 1;
                let failed = attempts >= MAX_ATTEMPTS;
                let wait = BACKOFF_MINS[usize::try_from(attempts - 1).unwrap_or(0).min(3)];
                tracing::warn!(id = r.id, attempts, "email send failed: {e}");
                sqlx::query!(
                    "UPDATE email_outbox SET attempts = $2, last_error = $3,
                            status = CASE WHEN $4 THEN 'failed' ELSE 'pending' END,
                            next_attempt_at = now() + make_interval(mins => $5)
                     WHERE id = $1",
                    r.id,
                    attempts,
                    e.to_string(),
                    failed,
                    wait
                )
                .execute(&mut *tx)
                .await?;
                if failed && let Some(org) = r.org_id {
                    super::notify::send(
                        &mut tx,
                        org,
                        super::notify::Target::Permission("email.view_log"),
                        "email_failed",
                        json!({ "to": r.to, "template": r.template }),
                        Some("/system-settings?tab=email"),
                    )
                    .await?;
                }
            }
        }
    }
    tx.commit().await?;
    Ok(n)
}

/// Background worker: wakes on `NOTIFY email_outbox` or every 30 s.
pub fn start_worker(pool: PgPool) {
    tokio::spawn(async move {
        let mut listener = sqlx::postgres::PgListener::connect_with(&pool).await.ok();
        if let Some(l) = listener.as_mut() {
            let _ = l.listen("email_outbox").await;
        }
        loop {
            if Modules::get().email_enabled() {
                loop {
                    match send_due(&pool).await {
                        Ok(10) => continue,
                        Ok(_) => break,
                        Err(e) => {
                            tracing::error!("email worker: {e:#}");
                            break;
                        }
                    }
                }
            }
            match listener.as_mut() {
                Some(l) => {
                    let _ = tokio::time::timeout(Duration::from_secs(30), l.recv()).await;
                }
                None => tokio::time::sleep(Duration::from_secs(30)).await,
            }
        }
    });
}

/// Svix signature check for `POST /api/webhooks/resend` (M13.6): `v1,<base64 HMAC-SHA256 of
/// "{id}.{timestamp}.{body}">` with the base64 secret after `whsec_`; 5-minute tolerance.
pub fn verify_webhook(secret: &str, id: &str, ts: &str, body: &[u8], sigs: &str, now: i64) -> bool {
    use base64::{Engine, engine::general_purpose::STANDARD};
    use hmac::{Hmac, Mac};
    let Ok(t) = ts.parse::<i64>() else {
        return false;
    };
    if (now - t).abs() > 300 {
        return false;
    }
    let Ok(key) = STANDARD.decode(secret.trim_start_matches("whsec_")) else {
        return false;
    };
    let Ok(mut mac) = Hmac::<sha2::Sha256>::new_from_slice(&key) else {
        return false;
    };
    mac.update(id.as_bytes());
    mac.update(b".");
    mac.update(ts.as_bytes());
    mac.update(b".");
    mac.update(body);
    let expected = STANDARD.encode(mac.finalize().into_bytes());
    sigs.split(' ')
        .filter_map(|s| s.strip_prefix("v1,"))
        .any(|s| s.len() == expected.len() && s.bytes().zip(expected.bytes()).all(|(a, b)| a == b))
}

/// Apply a Resend event: delivered / bounced / complained (the last two suppress the address).
pub async fn apply_event(pool: &PgPool, event: &Value) -> sqlx::Result<()> {
    let kind = event["type"].as_str().unwrap_or("");
    let id = event["data"]["email_id"].as_str().unwrap_or("");
    let status = match kind {
        "email.delivered" => "delivered",
        "email.bounced" => "bounced",
        "email.complained" => "complained",
        _ => return Ok(()),
    };
    let mut tx = pool.begin().await?;
    let to = sqlx::query_scalar!(
        r#"UPDATE email_outbox SET status = $2 WHERE provider_id = $1 RETURNING to_email::text AS "t!""#,
        id,
        status
    )
    .fetch_optional(&mut *tx)
    .await?;
    if let Some(to) = to
        && status != "delivered"
    {
        sqlx::query!(
            "INSERT INTO email_suppressions (email, reason) VALUES ($1::text::citext, $2)
             ON CONFLICT (email) DO NOTHING",
            to,
            status
        )
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_every_template() {
        for (name, _) in TEMPLATES {
            let r = render(
                name,
                &json!({"name": "Ali", "link": "https://x.test/a?t=1"}),
                "Acme",
            )
            .unwrap();
            assert!(!r.subject.contains("{{app_name}}"), "{name}");
            assert!(r.html.contains("Acme"));
        }
        let r = render(
            "password_reset",
            &json!({"name": "<b>", "link": "https://x.test/r", "minutes": 30}),
            "A",
        )
        .unwrap();
        assert!(r.text.contains("30 minutes"));
        assert!(r.html.contains("&lt;b&gt;") && r.html.contains(r#"href="https://x.test/r""#));
    }

    #[test]
    fn svix_signature() {
        use base64::{Engine, engine::general_purpose::STANDARD};
        use hmac::{Hmac, Mac};
        let secret = format!("whsec_{}", STANDARD.encode(b"0123456789abcdef"));
        let mut mac = Hmac::<sha2::Sha256>::new_from_slice(b"0123456789abcdef").unwrap();
        mac.update(b"msg_1.1700000000.{}");
        let sig = format!("v1,{}", STANDARD.encode(mac.finalize().into_bytes()));
        assert!(verify_webhook(
            &secret,
            "msg_1",
            "1700000000",
            b"{}",
            &sig,
            1_700_000_010
        ));
        assert!(!verify_webhook(
            &secret,
            "msg_1",
            "1700000000",
            b"{x}",
            &sig,
            1_700_000_010
        ));
        assert!(!verify_webhook(
            &secret,
            "msg_1",
            "1700000000",
            b"{}",
            &sig,
            1_700_001_000
        ));
    }
}
