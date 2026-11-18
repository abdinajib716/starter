//! Binary commands (M14.7): `startup [serve | migrate | seed | create-admin <email> <name> |
//! reset-2fa <email> | backup-verify | help]`. No argument = `serve`.
use anyhow::{Context, bail};

use super::{config::Config, db, rbac, seed, settings};

pub const HELP: &str = "usage: startup [command]
  serve                       run the web server (default)
  migrate                     apply database migrations and exit
  seed                        default organization, permissions, settings, Super Admin role, first admin
  create-admin <email> <name> new Super Admin; the password is read from ADMIN_PASSWORD
  reset-2fa <email>           remove a user's two-factor authentication and sign them out
  backup-verify               check every completed backup file against its size and SHA-256
  backup-keygen               print a new age key pair for BACKUP_PUBLIC_KEY (keep the private key offline)
  health                      exit 0 when http://127.0.0.1:$PORT/readyz answers 200 (container healthcheck)
  help                        this text";

/// `Some(code)` = a command ran and the process should exit; `None` = serve.
pub fn run() -> Option<i32> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let cmd = args.first().map(String::as_str).unwrap_or("serve");
    if cmd == "serve" {
        return None;
    }
    if matches!(cmd, "help" | "--help" | "-h") {
        println!("{HELP}");
        return Some(0);
    }
    if cmd == "health" {
        use std::io::{Read, Write};
        let port = std::env::var("PORT").unwrap_or_else(|_| "8080".into());
        let ok = std::net::TcpStream::connect(format!("127.0.0.1:{port}"))
            .and_then(|mut s| {
                s.set_read_timeout(Some(std::time::Duration::from_secs(3)))?;
                s.write_all(b"GET /readyz HTTP/1.0\r\nHost: localhost\r\n\r\n")?;
                let mut buf = [0u8; 16];
                let n = s.read(&mut buf)?;
                Ok(String::from_utf8_lossy(&buf[..n]).contains(" 200"))
            })
            .unwrap_or(false);
        return Some(if ok { 0 } else { 1 });
    }
    if cmd == "backup-keygen" {
        let id = age::x25519::Identity::generate();
        println!("BACKUP_PUBLIC_KEY={}", id.to_public());
        println!(
            "# private key: store it OFF this server (password manager / safe). Needed to restore."
        );
        println!(
            "{}",
            age::secrecy::ExposeSecret::expose_secret(&id.to_string())
        );
        return Some(0);
    }
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("tokio runtime");
    match rt.block_on(command(cmd, &args[1..])) {
        Ok(msg) => {
            println!("{msg}");
            Some(0)
        }
        Err(e) => {
            eprintln!("error: {e:#}");
            Some(1)
        }
    }
}

async fn command(cmd: &str, args: &[String]) -> anyhow::Result<String> {
    let _ = dotenvy::dotenv();
    let cfg = Config::load()?;
    let pool = db::connect(&cfg).await?;
    match cmd {
        "migrate" => {
            db::migrate(&pool).await?;
            Ok("migrations applied".into())
        }
        "seed" => {
            db::migrate(&pool).await?;
            let org = seed::ensure_default_org(&pool).await?;
            rbac::sync_permissions(&pool).await?;
            settings::seed(&pool).await?;
            seed::ensure_super_admin_role(&pool).await?;
            seed::ensure_first_admin(&pool, &cfg, org).await?;
            Ok("seed done".into())
        }
        "create-admin" => {
            let (Some(email), Some(name)) = (args.first(), args.get(1)) else {
                bail!("usage: create-admin <email> <name>  (password in ADMIN_PASSWORD)");
            };
            let pw = std::env::var("ADMIN_PASSWORD").context("set ADMIN_PASSWORD")?;
            super::auth::password::check_policy(&pw, 8).map_err(|p| anyhow::anyhow!(p.code()))?;
            let org = seed::ensure_default_org(&pool).await?;
            let role = seed::ensure_super_admin_role(&pool).await?;
            let hash = super::auth::password::hash(pw).await?;
            sqlx::query!(
                r#"INSERT INTO users (org_id, email, full_name, password_hash, role_id, must_change_pw)
                   VALUES ($1, $2::text::citext, $3, $4, $5, true)"#,
                org,
                email.trim().to_lowercase(),
                name,
                hash,
                role
            )
            .execute(&pool)
            .await
            .context("could not create the user (email already used?)")?;
            Ok(format!(
                "created Super Admin {email} (must change password at first sign-in)"
            ))
        }
        "reset-2fa" => {
            let email = args.first().context("usage: reset-2fa <email>")?;
            let id = sqlx::query_scalar!(
                "SELECT id FROM users WHERE email = $1::text::citext AND deleted_at IS NULL",
                email
            )
            .fetch_optional(&pool)
            .await?
            .context("no such user")?;
            let mut tx = pool.begin().await?;
            sqlx::query!("DELETE FROM user_totp WHERE user_id = $1", id)
                .execute(&mut *tx)
                .await?;
            sqlx::query!("DELETE FROM recovery_codes WHERE user_id = $1", id)
                .execute(&mut *tx)
                .await?;
            sqlx::query!("DELETE FROM sessions WHERE user_id = $1", id)
                .execute(&mut *tx)
                .await?;
            super::audit::record(
                &mut tx,
                super::audit::Entry {
                    action: "user.twofa_reset",
                    entity_type: Some("user"),
                    entity_id: Some(id.to_string()),
                    after: Some(serde_json::json!({ "via": "cli" })),
                    success: true,
                    ..Default::default()
                },
            )
            .await?;
            tx.commit().await?;
            Ok(format!("two-factor removed for {email}"))
        }
        "backup-verify" => {
            let report = super::services::backups::verify_all(&pool, &cfg).await?;
            Ok(report)
        }
        other => bail!("unknown command `{other}`\n{HELP}"),
    }
}
