//! Dashboard (§10): every number is a real query. The queries run concurrently
//! (`tokio::try_join!`) and the result is cached ~5 s per organization, so many open
//! dashboards refreshing at once cost one set of queries.
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::dashboard::{ActivityItem, BackupInfo, DashboardData};
use crate::server::ctx::Ctx;
use crate::server::error::AppError;

const CACHE_TTL: Duration = Duration::from_secs(5);
static CACHE: Mutex<Option<HashMap<Uuid, (Instant, DashboardData)>>> = Mutex::new(None);

pub async fn get(pool: &PgPool, ctx: &Ctx) -> Result<DashboardData, AppError> {
    ctx.require("dashboard.view")?;
    if let Some(hit) = CACHE
        .lock()
        .ok()
        .and_then(|c| c.as_ref().and_then(|m| m.get(&ctx.org_id).cloned()))
        .filter(|(at, _)| at.elapsed() < CACHE_TTL)
    {
        return Ok(hit.1);
    }
    let data = load(pool, ctx.org_id).await?;
    if let Ok(mut c) = CACHE.lock() {
        c.get_or_insert_with(HashMap::new)
            .insert(ctx.org_id, (Instant::now(), data.clone()));
    }
    Ok(data)
}

pub async fn load(pool: &PgPool, org: Uuid) -> Result<DashboardData, AppError> {
    let counts = sqlx::query!(
        r#"SELECT
             count(*) FILTER (WHERE deleted_at IS NULL) AS "total!",
             count(*) FILTER (WHERE deleted_at IS NULL AND created_at > now() - interval '30 days') AS "new30!",
             count(*) FILTER (WHERE deleted_at IS NULL AND last_login_at > now() - interval '30 days') AS "active30!",
             count(*) FILTER (WHERE deleted_at IS NULL AND last_login_at > now() - interval '60 days'
                              AND last_login_at <= now() - interval '30 days') AS "active_prev!"
           FROM users WHERE org_id = $1"#,
        org
    )
    .fetch_one(pool);
    let online = sqlx::query_scalar!(
        r#"SELECT count(DISTINCT s.user_id) AS "n!" FROM sessions s JOIN users u ON u.id = s.user_id
           WHERE u.org_id = $1 AND s.kind = 'web' AND s.expires_at > now()
             AND s.last_seen_at > now() - interval '5 minutes'"#,
        org
    )
    .fetch_one(pool);
    let logins = sqlx::query!(
        r#"SELECT
             count(*) FILTER (WHERE action = 'auth.login' AND at >= date_trunc('day', now())) AS "today!",
             count(*) FILTER (WHERE action = 'auth.login' AND at >= date_trunc('day', now()) - interval '1 day'
                              AND at < date_trunc('day', now())) AS "yesterday!",
             count(*) FILTER (WHERE action = 'auth.login_failed' AND at > now() - interval '24 hours') AS "failed!",
             count(*) FILTER (WHERE action = 'auth.login_failed' AND at > now() - interval '48 hours'
                              AND at <= now() - interval '24 hours') AS "failed_prev!"
           FROM audit_logs
           WHERE (org_id = $1 OR org_id IS NULL) AND at > now() - interval '2 days'
             AND action IN ('auth.login', 'auth.login_failed')"#,
        org
    )
    .fetch_one(pool);
    let backup =
        sqlx::query!("SELECT status, created_at FROM backups ORDER BY created_at DESC LIMIT 1")
            .fetch_optional(pool);
    let daily = sqlx::query!(
        r#"SELECT to_char(d, 'YYYY-MM-DD') AS "day!",
                  count(a.id) FILTER (WHERE a.action = 'auth.login') AS "ok!",
                  count(a.id) FILTER (WHERE a.action = 'auth.login_failed') AS "failed!"
           FROM generate_series(date_trunc('day', now()) - interval '13 days', date_trunc('day', now()),
                                interval '1 day') AS d
           LEFT JOIN audit_logs a
             ON a.at >= d AND a.at < d + interval '1 day'
            AND a.action IN ('auth.login', 'auth.login_failed')
            AND (a.org_id = $1 OR a.org_id IS NULL)
           GROUP BY d ORDER BY d"#,
        org
    )
    .fetch_all(pool);
    let modules = sqlx::query!(
        r#"SELECT split_part(action, '.', 1) AS "module!", count(*) AS "n!" FROM audit_logs
           WHERE (org_id = $1 OR org_id IS NULL) AND at > now() - interval '7 days'
           GROUP BY 1 ORDER BY 2 DESC, 1"#,
        org
    )
    .fetch_all(pool);
    let roles = sqlx::query!(
        r#"SELECT r.name, count(u.id) AS "n!" FROM roles r
           LEFT JOIN users u ON u.role_id = r.id AND u.org_id = $1 AND u.deleted_at IS NULL
           WHERE r.org_id = $1 OR r.org_id IS NULL
           GROUP BY r.id, r.name ORDER BY 2 DESC, r.name"#,
        org
    )
    .fetch_all(pool);
    let weekly = sqlx::query!(
        r#"SELECT to_char(w, 'YYYY-MM-DD') AS "week!", count(u.id) AS "n!"
           FROM generate_series(date_trunc('week', now()) - interval '11 weeks', date_trunc('week', now()),
                                interval '1 week') AS w
           LEFT JOIN users u ON u.org_id = $1 AND u.created_at >= w AND u.created_at < w + interval '1 week'
           GROUP BY w ORDER BY w"#,
        org
    )
    .fetch_all(pool);
    let recent = sqlx::query_as!(
        ActivityItem,
        r#"SELECT id, at, actor_email, action, success FROM audit_logs
           WHERE org_id = $1 OR org_id IS NULL ORDER BY at DESC, id DESC LIMIT 10"#,
        org
    )
    .fetch_all(pool);

    let (counts, online, logins, backup, daily, modules, roles, weekly, recent) = tokio::try_join!(
        counts, online, logins, backup, daily, modules, roles, weekly, recent
    )?;

    Ok(DashboardData {
        total_users: counts.total,
        new_users_30d: counts.new30,
        active_30d: counts.active30,
        active_prev_30d: counts.active_prev,
        online_now: online,
        logins_today: logins.today,
        logins_yesterday: logins.yesterday,
        failed_24h: logins.failed,
        failed_prev_24h: logins.failed_prev,
        last_backup: backup.map(|b| BackupInfo {
            status: b.status,
            created_at: b.created_at,
        }),
        logins_14d: daily.into_iter().map(|r| (r.day, r.ok, r.failed)).collect(),
        activity_by_module: modules.into_iter().map(|r| (r.module, r.n)).collect(),
        users_by_role: roles.into_iter().map(|r| (r.name, r.n)).collect(),
        new_users_12w: weekly.into_iter().map(|r| (r.week, r.n)).collect(),
        recent,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::seed;

    /// "Done when every number matches a hand-run SQL query": a known data set, then
    /// the struct is compared field by field with what was inserted.
    #[sqlx::test(migrations = "./migrations")]
    async fn numbers_match_the_data(pool: PgPool) {
        let org = seed::ensure_default_org(&pool).await.unwrap();
        let role = seed::ensure_super_admin_role(&pool).await.unwrap();
        for (i, (created_days, login_days)) in [(1, Some(0)), (40, Some(45)), (100, None)]
            .iter()
            .enumerate()
        {
            sqlx::query(
                "INSERT INTO users (org_id, email, full_name, password_hash, role_id, created_at, last_login_at)
                 VALUES ($1, $2, 'U', 'x', $3, now() - make_interval(days => $4),
                         now() - make_interval(days => $5))",
            )
            .bind(org)
            .bind(format!("u{i}@test.local"))
            .bind(role)
            .bind(*created_days)
            .bind(login_days.unwrap_or(9999))
            .execute(&pool)
            .await
            .unwrap();
        }
        for (action, hours_ago) in [
            ("auth.login", 0),
            ("auth.login", 0),
            ("auth.login_failed", 1),
            ("auth.login_failed", 30),
            ("user.create", 2),
        ] {
            sqlx::query("INSERT INTO audit_logs (org_id, action, at) VALUES ($1, $2, now() - make_interval(hours => $3))")
                .bind(org)
                .bind(action)
                .bind(hours_ago)
                .execute(&pool)
                .await
                .unwrap();
        }
        let d = load(&pool, org).await.unwrap();
        assert_eq!((d.total_users, d.new_users_30d), (3, 1));
        assert_eq!((d.active_30d, d.active_prev_30d), (1, 1));
        assert_eq!(d.logins_today, 2);
        assert_eq!((d.failed_24h, d.failed_prev_24h), (1, 1));
        assert_eq!(d.logins_14d.len(), 14);
        assert_eq!(d.logins_14d.last().unwrap().1, 2);
        assert_eq!(d.logins_14d.iter().map(|x| x.2).sum::<i64>(), 2);
        assert_eq!(d.new_users_12w.len(), 12);
        assert!(d.activity_by_module.contains(&("auth".to_string(), 4)));
        assert_eq!(d.users_by_role, vec![(seed::SUPER_ADMIN.to_string(), 3)]);
        assert_eq!(d.recent.len(), 5);
        assert!(d.last_backup.is_none());
    }
}
