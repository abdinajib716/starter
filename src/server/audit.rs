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

pub async fn record(tx: &mut Transaction<'_, Postgres>, e: Entry<'_>) -> sqlx::Result<()> {
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
        e.before,
        e.after,
        e.ip.map(ipnet::IpNet::from),
        e.user_agent,
        e.success,
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}
