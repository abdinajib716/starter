//! Audit writer (§9.5). Always called inside the transaction of the change it records,
//! so the change and its audit row commit or roll back together. Full page: M4.
use std::net::IpAddr;

use serde_json::Value;
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

#[derive(Debug, Default)]
pub struct Entry<'a> {
    pub org_id: Option<Uuid>,
    pub actor_id: Option<Uuid>,
    pub actor_email: Option<&'a str>,
    pub action: &'a str,
    pub entity_type: Option<&'a str>,
    pub entity_id: Option<String>,
    pub before: Option<Value>,
    pub after: Option<Value>,
    pub ip: Option<IpAddr>,
    pub user_agent: Option<&'a str>,
    pub success: bool,
}

/// Keys whose values never reach the audit log (§9.5, M4.2), matched case-insensitively as
/// substrings: `password`, `new_password`, `token_hash`, `api_secret`, …
const SECRET_KEYS: &[&str] = &["password", "token", "secret", "hash", "otp", "recovery"];

/// Replace secret values anywhere in a JSON document with `"***"`.
pub fn mask(v: Value) -> Value {
    match v {
        Value::Object(map) => Value::Object(
            map.into_iter()
                .map(|(k, v)| {
                    let lk = k.to_lowercase();
                    if SECRET_KEYS.iter().any(|s| lk.contains(s)) {
                        (k, Value::String("***".into()))
                    } else {
                        (k, mask(v))
                    }
                })
                .collect(),
        ),
        Value::Array(a) => Value::Array(a.into_iter().map(mask).collect()),
        other => other,
    }
}

pub async fn record(tx: &mut Transaction<'_, Postgres>, e: Entry<'_>) -> sqlx::Result<()> {
    let (before, after) = (e.before.map(mask), e.after.map(mask));
    sqlx::query!(
        r#"INSERT INTO audit_logs (org_id, actor_id, actor_email, action, entity_type, entity_id,
                                   before, after, ip, user_agent, success, channel)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, 'web')"#,
        e.org_id,
        e.actor_id,
        e.actor_email,
        e.action,
        e.entity_type,
        e.entity_id,
        before,
        after,
        e.ip.map(ipnet::IpNet::from),
        e.user_agent,
        e.success,
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::mask;
    use serde_json::json;

    #[test]
    fn secrets_are_masked_at_any_depth() {
        let v = json!({ "email": "a@b.c", "password": "x", "nested": { "TokenHash": "y", "ok": 1 },
                        "list": [{ "api_secret": "z" }] });
        assert_eq!(
            mask(v),
            json!({ "email": "a@b.c", "password": "***", "nested": { "TokenHash": "***", "ok": 1 },
                    "list": [{ "api_secret": "***" }] })
        );
    }
}
