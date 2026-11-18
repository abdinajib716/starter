//! Realtime (§11, M8.1–8.5): Postgres `NOTIFY` → one in-process hub → Server-Sent Events.
//!
//! Triggers send table-level signals on `app_events` (no payloads); `notify` carries a user id
//! when that user got a notification. A listener task debounces table signals (~500 ms) and
//! broadcasts them; each SSE stream forwards only what its user may see.
use std::collections::BTreeSet;
use std::convert::Infallible;
use std::time::Duration;

use dioxus::server::axum::{
    Extension,
    http::StatusCode,
    response::{
        IntoResponse, Response,
        sse::{Event, KeepAlive, Sse},
    },
};
use futures_util::stream::{self, Stream};
use sqlx::PgPool;
use tokio::sync::broadcast;
use uuid::Uuid;

use super::ctx::Ctx;
use super::state::AppState;

#[derive(Debug, Clone)]
pub enum Signal {
    /// A table changed (`audit_logs`, `users`, `sessions`, `backups`, …).
    Table(String),
    /// This user's notifications changed.
    User(Uuid),
}

static HUB: std::sync::OnceLock<broadcast::Sender<Signal>> = std::sync::OnceLock::new();

fn hub() -> &'static broadcast::Sender<Signal> {
    HUB.get_or_init(|| broadcast::channel(256).0)
}

/// Listen with auto-reconnect; table signals are coalesced over 500 ms.
pub fn start(pool: PgPool) {
    tokio::spawn(async move {
        loop {
            if let Err(e) = run(&pool).await {
                tracing::warn!("realtime listener: {e}; reconnecting in 2 s");
            }
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    });
}

async fn run(pool: &PgPool) -> sqlx::Result<()> {
    let mut l = sqlx::postgres::PgListener::connect_with(pool).await?;
    l.listen_all(["app_events", "notify"]).await?;
    let mut pending: BTreeSet<String> = BTreeSet::new();
    loop {
        let wait = if pending.is_empty() {
            Duration::from_secs(3600)
        } else {
            Duration::from_millis(500)
        };
        match tokio::time::timeout(wait, l.recv()).await {
            Ok(Ok(n)) => match n.channel() {
                "notify" => {
                    if let Ok(id) = n.payload().parse() {
                        let _ = hub().send(Signal::User(id));
                    }
                }
                _ => {
                    let table = serde_json::from_str::<serde_json::Value>(n.payload())
                        .ok()
                        .and_then(|v| v["table"].as_str().map(str::to_owned));
                    if let Some(t) = table {
                        pending.insert(t);
                    }
                }
            },
            Ok(Err(e)) => return Err(e),
            Err(_) => {
                for t in std::mem::take(&mut pending) {
                    let _ = hub().send(Signal::Table(t));
                }
            }
        }
    }
}

/// Which client topic a table maps to, and the permission needed to receive it.
fn topics(table: &str) -> &'static [(&'static str, &'static str)] {
    match table {
        "audit_logs" => &[("audit", "audit.view"), ("dashboard", "dashboard.view")],
        "users" => &[("users", "users.view"), ("dashboard", "dashboard.view")],
        "sessions" => &[("dashboard", "dashboard.view")],
        "backups" => &[("backups", "backup.view"), ("dashboard", "dashboard.view")],
        "roles" => &[("roles", "roles.view")],
        _ => &[],
    }
}

struct StreamState {
    rx: broadcast::Receiver<Signal>,
    ctx: Ctx,
    pool: PgPool,
    queue: Vec<Event>,
    done: bool,
}

/// `GET /api/stream` (M8.2): `event: <topic>` messages; `session_ended` when the session goes
/// away (M8.3); keep-alive comment every 15 s.
pub async fn stream_route(
    Extension(ctx): Extension<Ctx>,
    Extension(state): Extension<AppState>,
) -> Response {
    if ctx.user.is_none() {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let st = StreamState {
        rx: hub().subscribe(),
        ctx,
        pool: state.pool.clone(),
        queue: vec![Event::default().event("ready").data("1")],
        done: false,
    };
    Sse::new(events(st))
        .keep_alive(KeepAlive::new().interval(Duration::from_secs(15)))
        .into_response()
}

fn events(st: StreamState) -> impl Stream<Item = Result<Event, Infallible>> {
    stream::unfold(st, |mut st| async move {
        loop {
            if let Some(e) = st.queue.pop() {
                return Some((Ok(e), st));
            }
            if st.done {
                return None;
            }
            let user = st.ctx.user.as_ref()?;
            match st.rx.recv().await {
                Ok(Signal::User(id)) if id == user.id => {
                    st.queue
                        .push(Event::default().event("notifications").data("1"));
                }
                Ok(Signal::User(_)) => {}
                Ok(Signal::Table(t)) => {
                    if t == "sessions" {
                        let alive = sqlx::query_scalar!(
                            r#"SELECT EXISTS (SELECT 1 FROM sessions WHERE id = $1 AND expires_at > now()) AS "e!""#,
                            user.session_id
                        )
                        .fetch_one(&st.pool)
                        .await
                        .unwrap_or(true);
                        if !alive {
                            st.queue
                                .push(Event::default().event("session_ended").data("1"));
                            st.done = true;
                            continue;
                        }
                    }
                    let mut seen = BTreeSet::new();
                    for (topic, perm) in topics(&t) {
                        if user.permissions.contains(*perm) && seen.insert(*topic) {
                            st.queue.push(Event::default().event(*topic).data("1"));
                        }
                    }
                }
                // Missed messages: tell the client to refetch everything once.
                Err(broadcast::error::RecvError::Lagged(_)) => {
                    st.queue.push(Event::default().event("resync").data("1"));
                }
                Err(broadcast::error::RecvError::Closed) => return None,
            }
        }
    })
}
