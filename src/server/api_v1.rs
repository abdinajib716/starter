//! Public REST API v1 (§19, M15). JSON over `/api/v1`, for mobile apps and integrations.
//!
//! - Off unless `api.public_enabled` (then every route answers 404) and built only with the
//!   `public-api` Cargo feature.
//! - Auth: `Authorization: Bearer <token>`. Personal tokens `stk_…` (Profile → API tokens;
//!   permissions = token scopes ∩ owner's permissions) or mobile access tokens `sat_…`
//!   (15 min) from `POST /auth/login`, renewed with a rotating refresh token (30 days).
//! - Errors: RFC 9457 `application/problem+json` with a stable `code`.
//! - Lists: `?limit=&cursor=` (opaque cursor, `next_cursor` in the answer).
//! - `Idempotency-Key` on POST/PATCH/DELETE replays the first answer for 24 h.
//! - CORS: only `api.cors_origins`. Rate limit `api.rate_limit_per_min` per token, with
//!   `Retry-After`.
//! - Every call goes through the same services (permission check + audit) as the web app.
use std::collections::HashSet;

use base64::Engine;
use dioxus::server::axum::{
    Extension, Json, Router,
    body::{Body, Bytes},
    extract::{Path, Query, Request},
    http::{HeaderMap, HeaderValue, Method, StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::Deserialize;
use serde_json::{Value, json};
use uuid::Uuid;

use super::auth::{session, token};
use super::ctx::{Ctx, CurrentUser};
use super::error::AppError;
use super::modules::Modules;
use super::state::AppState;
use crate::domain::users::{BulkUserAction, UserInput, UserQuery};

const ACCESS_MINUTES: i64 = 15;
const REFRESH_DAYS: i64 = 30;

// ---------------------------------------------------------------- errors

pub struct Problem {
    status: StatusCode,
    code: String,
    detail: Option<String>,
    retry_after: Option<u64>,
}

impl Problem {
    fn new(status: StatusCode, code: &str) -> Self {
        Self {
            status,
            code: code.into(),
            detail: None,
            retry_after: None,
        }
    }
}

impl From<AppError> for Problem {
    fn from(e: AppError) -> Self {
        let status = match &e {
            AppError::Unauthenticated => StatusCode::UNAUTHORIZED,
            AppError::Forbidden => StatusCode::FORBIDDEN,
            AppError::NotFound | AppError::Disabled(_) => StatusCode::NOT_FOUND,
            AppError::Validation(_) => StatusCode::UNPROCESSABLE_ENTITY,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::Internal(err) => {
                tracing::error!(error = ?err, "api internal error");
                StatusCode::INTERNAL_SERVER_ERROR
            }
        };
        Self::new(status, e.code())
    }
}

impl From<sqlx::Error> for Problem {
    fn from(e: sqlx::Error) -> Self {
        AppError::from(e).into()
    }
}

impl IntoResponse for Problem {
    fn into_response(self) -> Response {
        let body = json!({
            "type": format!("https://docs.invalid/errors/{}", self.code),
            "title": self.status.canonical_reason().unwrap_or("Error"),
            "status": self.status.as_u16(),
            "code": self.code,
            "detail": self.detail,
        });
        let mut r = (self.status, body.to_string()).into_response();
        r.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/problem+json"),
        );
        if let Some(s) = self.retry_after {
            r.headers_mut()
                .insert(header::RETRY_AFTER, HeaderValue::from(s));
        }
        r
    }
}

type ApiResult<T> = Result<T, Problem>;

// ---------------------------------------------------------------- router + middleware

pub fn router() -> Router {
    Router::new()
        .route("/api/v1/auth/login", post(login))
        .route("/api/v1/auth/refresh", post(refresh))
        .route("/api/v1/auth/logout", post(logout))
        .route("/api/v1/me", get(me))
        .route("/api/v1/users", get(list_users).post(create_user))
        .route(
            "/api/v1/users/{id}",
            get(get_user).patch(update_user).delete(delete_user),
        )
        .route("/api/v1/users/{id}/actions/{action}", post(user_action))
        .route("/api/v1/roles", get(list_roles))
        .route("/api/v1/audit", get(list_audit))
        .route("/api/v1/backups", get(list_backups).post(create_backup))
        .route("/api/v1/notifications", get(list_notifications))
        .route("/api/v1/notifications/read", post(read_notifications))
        .route("/api/v1/openapi.json", get(openapi))
        .route("/api/docs", get(docs))
        .layer(dioxus::server::axum::middleware::from_fn(layer))
}

fn cors_origin(headers: &HeaderMap) -> Option<HeaderValue> {
    let origin = headers.get(header::ORIGIN)?.to_str().ok()?;
    let allowed = super::settings::cache()
        .map(|c| c.get("api.cors_origins"))
        .and_then(|v| v.as_str().map(str::to_owned))
        .unwrap_or_default();
    allowed
        .split(',')
        .map(str::trim)
        .any(|a| !a.is_empty() && (a == "*" || a.eq_ignore_ascii_case(origin)))
        .then(|| HeaderValue::from_str(origin).ok())
        .flatten()
}

/// Module switch, CORS, bearer auth, rate limit, idempotency.
async fn layer(Extension(state): Extension<AppState>, mut req: Request, next: Next) -> Response {
    if !Modules::get().public_api() {
        return Problem::new(StatusCode::NOT_FOUND, "not_found").into_response();
    }
    let origin = cors_origin(req.headers());
    if req.method() == Method::OPTIONS {
        let mut r = StatusCode::NO_CONTENT.into_response();
        if let Some(o) = origin {
            let h = r.headers_mut();
            h.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, o);
            h.insert(
                header::ACCESS_CONTROL_ALLOW_METHODS,
                HeaderValue::from_static("GET, POST, PATCH, DELETE"),
            );
            h.insert(
                header::ACCESS_CONTROL_ALLOW_HEADERS,
                HeaderValue::from_static("authorization, content-type, idempotency-key"),
            );
            h.insert(
                header::ACCESS_CONTROL_MAX_AGE,
                HeaderValue::from_static("600"),
            );
        }
        return r;
    }
    let path = req.uri().path().to_string();
    let public = matches!(
        path.as_str(),
        "/api/v1/auth/login" | "/api/v1/auth/refresh" | "/api/v1/openapi.json" | "/api/docs"
    );

    // Bearer → CurrentUser (personal token or mobile access token).
    let bearer = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(str::trim)
        .map(str::to_owned);
    let mut user: Option<CurrentUser> = None;
    if let Some(raw) = &bearer {
        user = match authenticate(&state.pool, raw).await {
            Ok(u) => u,
            Err(e) => return e.into_response(),
        };
    }
    if !public && user.is_none() {
        return Problem::new(StatusCode::UNAUTHORIZED, "auth.unauthenticated").into_response();
    }
    // Rate limit per token (or per IP for the public auth endpoints).
    let per_min = super::settings::cache()
        .and_then(|c| c.get("api.rate_limit_per_min").as_u64())
        .unwrap_or(120) as u32;
    let ctx = req.extensions().get::<Ctx>().cloned();
    let rl_key = match (&user, &ctx) {
        (Some(u), _) => format!("api:{}", u.session_id),
        (None, Some(c)) => format!(
            "api-ip:{}",
            c.meta.ip.map(|i| i.to_string()).unwrap_or_default()
        ),
        _ => "api:anon".into(),
    };
    if !super::auth::ratelimit::hit(&rl_key, per_min, std::time::Duration::from_secs(60)).await {
        let mut p = Problem::new(StatusCode::TOO_MANY_REQUESTS, "auth.rate_limited");
        p.retry_after = Some(60);
        return p.into_response();
    }
    if let Some(mut c) = ctx {
        c.user = user.clone();
        req.extensions_mut().insert(c);
    }

    // Idempotency for state-changing calls with a key.
    let idem = req
        .headers()
        .get("idempotency-key")
        .and_then(|v| v.to_str().ok())
        .filter(|k| !k.is_empty() && k.len() <= 200)
        .map(str::to_owned);
    let method = req.method().clone();
    let mutating = matches!(method, Method::POST | Method::PATCH | Method::DELETE);
    if let (Some(key), Some(u), true) = (&idem, &user, mutating)
        && let Ok(Some(prev)) = sqlx::query!(
            "SELECT method, path, status, body FROM idempotency_keys
             WHERE user_id = $1 AND key = $2 AND created_at > now() - interval '24 hours'",
            u.id,
            key
        )
        .fetch_optional(&state.pool)
        .await
    {
        if prev.method != method.as_str() || prev.path != path {
            return Problem::new(
                StatusCode::UNPROCESSABLE_ENTITY,
                "api.idempotency_key_reused",
            )
            .into_response();
        }
        let mut r = Response::new(Body::from(prev.body));
        *r.status_mut() = StatusCode::from_u16(prev.status as u16).unwrap_or(StatusCode::OK);
        r.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );
        r.headers_mut()
            .insert("idempotent-replayed", HeaderValue::from_static("true"));
        return with_cors(r, origin);
    }

    let resp = next.run(req).await;
    let resp = match (idem, user, mutating) {
        (Some(key), Some(u), true) if resp.status().is_success() => {
            let (parts, body) = resp.into_parts();
            let bytes = dioxus::server::axum::body::to_bytes(body, 1024 * 1024)
                .await
                .unwrap_or_default();
            let _ = sqlx::query!(
                "INSERT INTO idempotency_keys (user_id, key, method, path, status, body)
                 VALUES ($1, $2, $3, $4, $5, $6) ON CONFLICT DO NOTHING",
                u.id,
                key,
                method.as_str(),
                path,
                i32::from(parts.status.as_u16()),
                bytes.to_vec()
            )
            .execute(&state.pool)
            .await;
            Response::from_parts(parts, Body::from(bytes))
        }
        (_, _, _) => resp,
    };
    with_cors(resp, origin)
}

fn with_cors(mut r: Response, origin: Option<HeaderValue>) -> Response {
    if let Some(o) = origin {
        r.headers_mut()
            .insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, o);
        r.headers_mut()
            .insert(header::VARY, HeaderValue::from_static("Origin"));
    }
    r
}

async fn authenticate(pool: &sqlx::PgPool, raw: &str) -> ApiResult<Option<CurrentUser>> {
    let unauth = || Problem::new(StatusCode::UNAUTHORIZED, "auth.unauthenticated");
    if raw.starts_with("sat_") {
        return Ok(Some(
            session::load_kind(pool, raw, "api_access", None)
                .await?
                .ok_or_else(unauth)?,
        ));
    }
    if raw.starts_with("stk_") {
        let mut u = session::load_kind(pool, raw, "api_token", None)
            .await?
            .ok_or_else(unauth)?;
        let t = sqlx::query!(
            "UPDATE api_tokens SET last_used_at = now() WHERE token_hash = $1 AND revoked_at IS NULL
             RETURNING scopes",
            token::hash(raw)
        )
        .fetch_optional(pool)
        .await?
        .ok_or_else(unauth)?;
        let scopes: HashSet<String> = t.scopes.into_iter().collect();
        u.permissions.retain(|p| scopes.contains(p));
        return Ok(Some(u));
    }
    Err(unauth())
}

fn ctx_of(ctx: &Ctx) -> ApiResult<&CurrentUser> {
    ctx.user
        .as_ref()
        .ok_or_else(|| Problem::new(StatusCode::UNAUTHORIZED, "auth.unauthenticated"))
}

fn pool() -> ApiResult<&'static sqlx::PgPool> {
    super::state::get()
        .map(|s| &s.pool)
        .map_err(|_| Problem::new(StatusCode::SERVICE_UNAVAILABLE, "unavailable"))
}

// ---------------------------------------------------------------- cursors

fn encode_cursor(v: &Value) -> String {
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(v.to_string())
}

fn decode_cursor(c: &Option<String>) -> ApiResult<Option<Value>> {
    match c {
        None => Ok(None),
        Some(c) => base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(c)
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .map(Some)
            .ok_or_else(|| Problem::new(StatusCode::BAD_REQUEST, "api.bad_cursor")),
    }
}

#[derive(Deserialize, Default)]
struct ListParams {
    limit: Option<u32>,
    cursor: Option<String>,
    q: Option<String>,
}

// ---------------------------------------------------------------- auth (mobile)

#[derive(Deserialize)]
struct LoginBody {
    email: String,
    password: String,
    /// TOTP or recovery code when the account has 2FA.
    code: Option<String>,
}

async fn issue_pair(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    ctx: &Ctx,
    user_id: Uuid,
    mfa: bool,
) -> ApiResult<Value> {
    let (ip, ua) = (ctx.meta.ip, ctx.meta.user_agent.as_deref());
    let access = format!("sat_{}", token::generate());
    let refresh = format!("srt_{}", token::generate());
    for (raw, kind, mins) in [
        (&access, "api_access", ACCESS_MINUTES),
        (&refresh, "api_refresh", REFRESH_DAYS * 24 * 60),
    ] {
        sqlx::query!(
            r#"INSERT INTO sessions (token_hash, user_id, ip, user_agent, device_label, kind, mfa_verified_at, expires_at)
               VALUES ($1, $2, $3, $4, $5, $6, CASE WHEN $7 THEN now() END, now() + make_interval(mins => $8))"#,
            token::hash(raw),
            user_id,
            ip.map(ipnet::IpNet::from),
            ua,
            crate::domain::security::device_label(ua.unwrap_or("")),
            kind,
            mfa,
            i32::try_from(mins).unwrap_or(15)
        )
        .execute(&mut **tx)
        .await?;
    }
    Ok(
        json!({ "access_token": access, "token_type": "Bearer", "expires_in": ACCESS_MINUTES * 60, "refresh_token": refresh }),
    )
}

async fn login(Extension(ctx): Extension<Ctx>, Json(b): Json<LoginBody>) -> ApiResult<Json<Value>> {
    use super::services::auth::{AuthError, login as web_login};
    let pool = pool()?;
    // Same checks as the web (rate limit, lockout, audit); its session is swapped for tokens.
    let ok = web_login(pool, &ctx, &b.email, b.password)
        .await
        .map_err(|e| match e {
            AuthError::RateLimited => {
                Problem::new(StatusCode::TOO_MANY_REQUESTS, "auth.rate_limited")
            }
            _ => Problem::new(StatusCode::UNAUTHORIZED, "auth.invalid_credentials"),
        })?;
    let (user_id, mfa) = if ok.next == crate::domain::AfterLogin::TwoFactor {
        let Some(code) = b.code.filter(|c| !c.trim().is_empty()) else {
            sqlx::query!(
                "DELETE FROM sessions WHERE token_hash = $1",
                token::hash(&ok.token)
            )
            .execute(pool)
            .await?;
            return Err(Problem::new(
                StatusCode::UNAUTHORIZED,
                "twofa.code_required",
            ));
        };
        let done = super::services::security::verify_login(pool, &ctx, &ok.token, &code)
            .await
            .map_err(|_| Problem::new(StatusCode::UNAUTHORIZED, "twofa.invalid_code"))?;
        let u = session::load(pool, &done.token, 12)
            .await?
            .ok_or_else(|| Problem::new(StatusCode::UNAUTHORIZED, "auth.unauthenticated"))?;
        sqlx::query!(
            "DELETE FROM sessions WHERE token_hash = $1",
            token::hash(&done.token)
        )
        .execute(pool)
        .await?;
        (u.id, true)
    } else {
        let u = session::load(pool, &ok.token, 12)
            .await?
            .ok_or_else(|| Problem::new(StatusCode::UNAUTHORIZED, "auth.unauthenticated"))?;
        sqlx::query!(
            "DELETE FROM sessions WHERE token_hash = $1",
            token::hash(&ok.token)
        )
        .execute(pool)
        .await?;
        (u.id, false)
    };
    let mut tx = pool.begin().await?;
    let pair = issue_pair(&mut tx, &ctx, user_id, mfa).await?;
    tx.commit().await?;
    Ok(Json(pair))
}

#[derive(Deserialize)]
struct RefreshBody {
    refresh_token: String,
}

/// Rotating refresh: the old refresh token stops working; reuse of a spent one fails.
async fn refresh(
    Extension(ctx): Extension<Ctx>,
    Json(b): Json<RefreshBody>,
) -> ApiResult<Json<Value>> {
    let pool = pool()?;
    let mut tx = pool.begin().await?;
    let row = sqlx::query!(
        r#"DELETE FROM sessions s USING users u
           WHERE s.token_hash = $1 AND s.kind = 'api_refresh' AND s.expires_at > now()
             AND u.id = s.user_id AND u.is_active AND u.deleted_at IS NULL
           RETURNING s.user_id, s.mfa_verified_at IS NOT NULL AS "mfa!""#,
        token::hash(&b.refresh_token)
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| Problem::new(StatusCode::UNAUTHORIZED, "auth.unauthenticated"))?;
    let pair = issue_pair(&mut tx, &ctx, row.user_id, row.mfa).await?;
    tx.commit().await?;
    Ok(Json(pair))
}

async fn logout(
    Extension(ctx): Extension<Ctx>,
    headers: HeaderMap,
    body: Bytes,
) -> ApiResult<StatusCode> {
    let pool = pool()?;
    ctx_of(&ctx)?;
    let access = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .unwrap_or("");
    sqlx::query!(
        "DELETE FROM sessions WHERE token_hash = $1 AND kind = 'api_access'",
        token::hash(access.trim())
    )
    .execute(pool)
    .await?;
    if let Ok(b) = serde_json::from_slice::<RefreshBody>(&body) {
        sqlx::query!(
            "DELETE FROM sessions WHERE token_hash = $1 AND kind = 'api_refresh'",
            token::hash(&b.refresh_token)
        )
        .execute(pool)
        .await?;
    }
    Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------- resources

async fn me(Extension(ctx): Extension<Ctx>) -> ApiResult<Json<Value>> {
    let u = ctx_of(&ctx)?;
    let mut perms: Vec<&String> = u.permissions.iter().collect();
    perms.sort();
    Ok(Json(json!({
        "id": u.id, "email": u.email, "full_name": u.full_name, "role": u.role_name,
        "permissions": perms, "timezone": u.timezone, "locale": u.locale,
        "two_factor": u.twofa_enrolled,
    })))
}

async fn list_users(
    Extension(ctx): Extension<Ctx>,
    Query(p): Query<ListParams>,
) -> ApiResult<Json<Value>> {
    let limit = p.limit.unwrap_or(20).clamp(1, 100);
    // Zero-based page inside the opaque cursor.
    let page = decode_cursor(&p.cursor)?
        .and_then(|v| v["page"].as_u64())
        .unwrap_or(0) as u32;
    // The list service pages by 10/20/30/40/50; map the limit onto the nearest allowed size.
    let size = [10, 20, 30, 40, 50]
        .into_iter()
        .find(|s| *s >= limit)
        .unwrap_or(50);
    let q = UserQuery {
        q: p.q.unwrap_or_default(),
        page,
        page_size: size,
        ..Default::default()
    };
    let r = super::services::users::list(pool()?, &ctx, &q).await?;
    let next = (u64::from(page + 1) * u64::from(size) < r.total)
        .then(|| encode_cursor(&json!({ "page": page + 1 })));
    Ok(Json(
        json!({ "data": r.rows, "total": r.total, "next_cursor": next }),
    ))
}

async fn get_user(Extension(ctx): Extension<Ctx>, Path(id): Path<Uuid>) -> ApiResult<Json<Value>> {
    let d = super::services::users::detail(pool()?, &ctx, id).await?;
    Ok(Json(
        json!({ "data": d.user, "two_factor": d.twofa_enrolled, "active_sessions": d.active_sessions }),
    ))
}

async fn create_user(
    Extension(ctx): Extension<Ctx>,
    Json(b): Json<UserInput>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    let id = super::services::users::create(pool()?, &ctx, b).await?;
    Ok((StatusCode::CREATED, Json(json!({ "id": id }))))
}

async fn update_user(
    Extension(ctx): Extension<Ctx>,
    Path(id): Path<Uuid>,
    Json(b): Json<UserInput>,
) -> ApiResult<StatusCode> {
    super::services::users::update(pool()?, &ctx, id, b).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn delete_user(
    Extension(ctx): Extension<Ctx>,
    Path(id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    super::services::users::apply(pool()?, &ctx, BulkUserAction::Delete, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn user_action(
    Extension(ctx): Extension<Ctx>,
    Path((id, action)): Path<(Uuid, String)>,
) -> ApiResult<StatusCode> {
    let a = match action.as_str() {
        "activate" => BulkUserAction::Activate,
        "deactivate" => BulkUserAction::Deactivate,
        "restore" => BulkUserAction::Restore,
        _ => return Err(Problem::new(StatusCode::NOT_FOUND, "not_found")),
    };
    super::services::users::apply(pool()?, &ctx, a, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn list_roles(Extension(ctx): Extension<Ctx>) -> ApiResult<Json<Value>> {
    Ok(Json(
        json!({ "data": super::services::roles::list(pool()?, &ctx).await? }),
    ))
}

async fn list_audit(
    Extension(ctx): Extension<Ctx>,
    Query(p): Query<ListParams>,
) -> ApiResult<Json<Value>> {
    let before = decode_cursor(&p.cursor)?.and_then(|v| serde_json::from_value(v).ok());
    let q = crate::domain::audit::AuditQuery {
        q: p.q.unwrap_or_default(),
        limit: p.limit.unwrap_or(50).clamp(1, 200),
        before,
        ..Default::default()
    };
    let r = super::services::audit_log::list(pool()?, &ctx, &q).await?;
    let next = r
        .next
        .map(|c| encode_cursor(&serde_json::to_value(c).unwrap_or_default()));
    Ok(Json(json!({ "data": r.rows, "next_cursor": next })))
}

async fn list_backups(Extension(ctx): Extension<Ctx>) -> ApiResult<Json<Value>> {
    Ok(Json(
        json!({ "data": super::services::backups::list(pool()?, &ctx).await? }),
    ))
}

async fn create_backup(Extension(ctx): Extension<Ctx>) -> ApiResult<(StatusCode, Json<Value>)> {
    let id = super::services::backups::create(pool()?, &ctx).await?;
    Ok((
        StatusCode::ACCEPTED,
        Json(json!({ "id": id, "status": "running" })),
    ))
}

async fn list_notifications(
    Extension(ctx): Extension<Ctx>,
    Query(p): Query<ListParams>,
) -> ApiResult<Json<Value>> {
    let offset = decode_cursor(&p.cursor)?
        .and_then(|v| v["offset"].as_i64())
        .unwrap_or(0);
    let limit = i64::from(p.limit.unwrap_or(20).clamp(1, 100));
    let r = super::services::notifications::list(pool()?, &ctx, false, limit, offset).await?;
    let next =
        (offset + limit < r.total).then(|| encode_cursor(&json!({ "offset": offset + limit })));
    Ok(Json(
        json!({ "data": r.rows, "unread": r.unread, "next_cursor": next }),
    ))
}

#[derive(Deserialize)]
struct ReadBody {
    ids: Option<Vec<i64>>,
}

async fn read_notifications(
    Extension(ctx): Extension<Ctx>,
    Json(b): Json<ReadBody>,
) -> ApiResult<StatusCode> {
    super::services::notifications::mark_read(pool()?, &ctx, b.ids).await?;
    Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------- personal tokens (used by the web UI)

/// Create a personal token: returned once (`stk_…`), stored as SHA-256. Scopes are limited to
/// the owner's own permissions; count and lifetime follow `api.max_tokens_per_user` and
/// `api.token_default_ttl_days`.
pub async fn create_token(
    pool: &sqlx::PgPool,
    ctx: &Ctx,
    name: String,
    scopes: Vec<String>,
    days: Option<i64>,
) -> Result<crate::domain::api::NewToken, AppError> {
    let user = ctx.require("api_keys.create")?;
    if !Modules::get().public_api() {
        return Err(AppError::Disabled("api.disabled"));
    }
    let name = name.trim().to_string();
    if name.is_empty() || name.chars().count() > 60 {
        return Err(AppError::Validation("api.token_name"));
    }
    let scopes: Vec<String> = scopes
        .into_iter()
        .filter(|s| user.permissions.contains(s))
        .collect();
    if scopes.is_empty() {
        return Err(AppError::Validation("api.token_scopes"));
    }
    let cache = super::settings::cache();
    let max = cache
        .map(|c| c.get("api.max_tokens_per_user").as_i64().unwrap_or(10))
        .unwrap_or(10);
    let ttl = days
        .unwrap_or_else(|| {
            cache
                .map(|c| c.get("api.token_default_ttl_days").as_i64().unwrap_or(90))
                .unwrap_or(90)
        })
        .clamp(1, 3650);
    let mut tx = pool.begin().await?;
    let n = sqlx::query_scalar!(
        r#"SELECT count(*) AS "n!" FROM api_tokens WHERE user_id = $1 AND revoked_at IS NULL AND (expires_at IS NULL OR expires_at > now())"#,
        user.id
    )
    .fetch_one(&mut *tx)
    .await?;
    if n >= max {
        return Err(AppError::Conflict("api.too_many_tokens"));
    }
    let raw = format!("stk_{}", token::generate());
    let prefix: String = raw.chars().take(10).collect();
    let id = sqlx::query_scalar!(
        "INSERT INTO api_tokens (org_id, user_id, name, prefix, token_hash, scopes, expires_at)
         VALUES ($1, $2, $3, $4, $5, $6, now() + make_interval(days => $7)) RETURNING id",
        ctx.org_id,
        user.id,
        name,
        prefix,
        token::hash(&raw),
        &scopes,
        i32::try_from(ttl).unwrap_or(90)
    )
    .fetch_one(&mut *tx)
    .await?;
    // A matching session row lets `session::load_kind` resolve the token like any login.
    sqlx::query!(
        "INSERT INTO sessions (token_hash, user_id, kind, device_label, expires_at)
         VALUES ($1, $2, 'api_token', $3, now() + make_interval(days => $4))",
        token::hash(&raw),
        user.id,
        format!("API token: {name}"),
        i32::try_from(ttl).unwrap_or(90)
    )
    .execute(&mut *tx)
    .await?;
    super::audit::record(
        &mut tx,
        super::audit::Entry {
            entity_type: Some("api_token"),
            entity_id: Some(id.to_string()),
            after: Some(json!({ "name": name, "scopes": scopes, "days": ttl })),
            ..super::services::auth::user_entry(ctx, user, "api_token.create")
        },
    )
    .await?;
    tx.commit().await?;
    Ok(crate::domain::api::NewToken { id, token: raw })
}

/// Own tokens, or (`all` + `api_keys.view_all`) every token of the organization.
pub async fn list_tokens(
    pool: &sqlx::PgPool,
    ctx: &Ctx,
    all: bool,
) -> Result<Vec<crate::domain::api::TokenRow>, AppError> {
    let user = if all {
        ctx.require("api_keys.view_all")?
    } else {
        ctx.user.as_ref().ok_or(AppError::Unauthenticated)?
    };
    Ok(sqlx::query!(
        r#"SELECT t.id, t.name, t.prefix, t.scopes, t.expires_at, t.last_used_at, t.revoked_at, t.created_at,
                  u.email::text AS "owner!"
           FROM api_tokens t JOIN users u ON u.id = t.user_id
           WHERE t.org_id = $1 AND ($2 OR t.user_id = $3)
           ORDER BY t.created_at DESC"#,
        ctx.org_id,
        all,
        user.id
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|r| crate::domain::api::TokenRow {
        id: r.id,
        name: r.name,
        prefix: r.prefix,
        scopes: r.scopes,
        owner: r.owner,
        expires_at: r.expires_at,
        last_used_at: r.last_used_at,
        revoked: r.revoked_at.is_some(),
        created_at: r.created_at,
    })
    .collect())
}

/// Revoke own token, or any token with `api_keys.revoke_all`.
pub async fn revoke_token(pool: &sqlx::PgPool, ctx: &Ctx, id: Uuid) -> Result<(), AppError> {
    let user = ctx.user.as_ref().ok_or(AppError::Unauthenticated)?;
    let any = user.permissions.contains("api_keys.revoke_all");
    let mut tx = pool.begin().await?;
    let hash = sqlx::query_scalar!(
        "UPDATE api_tokens SET revoked_at = now() WHERE id = $1 AND org_id = $2 AND ($3 OR user_id = $4)
           AND revoked_at IS NULL RETURNING token_hash",
        id,
        ctx.org_id,
        any,
        user.id
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(AppError::NotFound)?;
    sqlx::query!("DELETE FROM sessions WHERE token_hash = $1", hash)
        .execute(&mut *tx)
        .await?;
    super::audit::record(
        &mut tx,
        super::audit::Entry {
            entity_type: Some("api_token"),
            entity_id: Some(id.to_string()),
            ..super::services::auth::user_entry(ctx, user, "api_token.revoke")
        },
    )
    .await?;
    tx.commit().await?;
    Ok(())
}

// ---------------------------------------------------------------- OpenAPI + docs

async fn openapi() -> Json<Value> {
    Json(spec())
}

async fn docs() -> Response {
    let html = r#"<!doctype html><html><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>API v1</title></head><body><script id="api-reference" data-url="/api/v1/openapi.json"></script><script src="https://cdn.jsdelivr.net/npm/@scalar/api-reference@1.25.11"></script></body></html>"#;
    let mut r = Response::new(Body::from(html));
    r.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/html; charset=utf-8"),
    );
    // The docs page loads Scalar from a CDN: relax the app's strict CSP for this one page.
    r.headers_mut().insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static("default-src 'self'; script-src 'self' https://cdn.jsdelivr.net 'unsafe-inline'; style-src 'self' 'unsafe-inline' https://cdn.jsdelivr.net https://fonts.googleapis.com; font-src https://fonts.gstatic.com data:; img-src 'self' data: https:; connect-src 'self'"),
    );
    r
}

/// The OpenAPI 3.1 document (kept next to the handlers it describes).
pub fn spec() -> Value {
    let list = |tag: &str, summary: &str, perm: &str| {
        json!({ "tags": [tag], "summary": summary, "description": format!("Needs `{perm}`."),
        "parameters": [ {"$ref": "#/components/parameters/limit"}, {"$ref": "#/components/parameters/cursor"}, {"name": "q", "in": "query", "schema": {"type": "string"}} ],
        "responses": { "200": { "description": "A page", "content": { "application/json": { "schema": { "$ref": "#/components/schemas/Page" } } } }, "default": { "$ref": "#/components/responses/Problem" } } })
    };
    let simple = |tag: &str, summary: &str, perm: &str, ok: &str| {
        json!({ "tags": [tag], "summary": summary, "description": format!("Needs `{perm}`."),
        "responses": { ok: { "description": "OK" }, "default": { "$ref": "#/components/responses/Problem" } } })
    };
    let id = json!([{ "name": "id", "in": "path", "required": true, "schema": { "type": "string", "format": "uuid" } }]);
    let user_body = json!({ "required": true, "content": { "application/json": { "schema": { "$ref": "#/components/schemas/UserInput" } } } });
    let mut user_get = simple("Users", "Get a user", "users.view", "200");
    user_get["parameters"] = id.clone();
    let mut user_patch = simple("Users", "Update a user", "users.update", "204");
    user_patch["parameters"] = id.clone();
    user_patch["requestBody"] = user_body.clone();
    let mut user_del = simple("Users", "Delete a user (soft)", "users.delete", "204");
    user_del["parameters"] = id.clone();
    let mut user_post = simple("Users", "Create a user", "users.create", "201");
    user_post["requestBody"] = user_body;
    let mut action = simple(
        "Users",
        "activate | deactivate | restore",
        "users.update / users.restore",
        "204",
    );
    action["parameters"] = json!([id[0], { "name": "action", "in": "path", "required": true, "schema": { "enum": ["activate", "deactivate", "restore"] } }]);
    json!({
        "openapi": "3.1.0",
        "info": { "title": format!("{} API", Modules::get().app_name()), "version": "1",
                  "description": "Bearer tokens: personal `stk_…` (Profile → API tokens) or `sat_…` from /auth/login. Errors are RFC 9457 problem+json with a stable `code`. Send `Idempotency-Key` on POST/PATCH/DELETE to make retries safe." },
        "servers": [{ "url": "/api/v1" }],
        "security": [{ "bearer": [] }],
        "paths": {
            "/auth/login": { "post": { "tags": ["Auth"], "summary": "Email + password (+ 2FA code) → access + refresh token", "security": [],
                "requestBody": { "required": true, "content": { "application/json": { "schema": { "type": "object", "required": ["email", "password"], "properties": { "email": {"type": "string"}, "password": {"type": "string"}, "code": {"type": "string", "description": "TOTP or recovery code when 2FA is on"} } } } } },
                "responses": { "200": { "description": "Tokens", "content": { "application/json": { "schema": { "$ref": "#/components/schemas/Tokens" } } } }, "default": { "$ref": "#/components/responses/Problem" } } } },
            "/auth/refresh": { "post": { "tags": ["Auth"], "summary": "Rotate the refresh token", "security": [],
                "requestBody": { "required": true, "content": { "application/json": { "schema": { "type": "object", "required": ["refresh_token"], "properties": { "refresh_token": {"type": "string"} } } } } },
                "responses": { "200": { "description": "Tokens", "content": { "application/json": { "schema": { "$ref": "#/components/schemas/Tokens" } } } }, "default": { "$ref": "#/components/responses/Problem" } } } },
            "/auth/logout": { "post": simple("Auth", "End this access token (and the refresh token in the body)", "a token", "204") },
            "/me": { "get": simple("Me", "The token's user and effective permissions", "a token", "200") },
            "/users": { "get": list("Users", "List users", "users.view"), "post": user_post },
            "/users/{id}": { "get": user_get, "patch": user_patch, "delete": user_del },
            "/users/{id}/actions/{action}": { "post": action },
            "/roles": { "get": simple("Roles", "List roles", "roles.view", "200") },
            "/audit": { "get": list("Audit", "Audit log, newest first", "audit.view") },
            "/backups": { "get": simple("Backups", "List backups", "backup.view", "200"), "post": simple("Backups", "Start a backup", "backup.create", "202") },
            "/notifications": { "get": list("Notifications", "My notifications", "a token") },
            "/notifications/read": { "post": simple("Notifications", "Mark read (`ids`, or all when omitted)", "a token", "204") }
        },
        "components": {
            "securitySchemes": { "bearer": { "type": "http", "scheme": "bearer" } },
            "parameters": {
                "limit": { "name": "limit", "in": "query", "schema": { "type": "integer", "minimum": 1, "maximum": 100 } },
                "cursor": { "name": "cursor", "in": "query", "schema": { "type": "string" }, "description": "`next_cursor` from the previous page" }
            },
            "responses": { "Problem": { "description": "RFC 9457 problem", "content": { "application/problem+json": { "schema": { "$ref": "#/components/schemas/Problem" } } } } },
            "schemas": {
                "Problem": { "type": "object", "properties": { "type": {"type": "string"}, "title": {"type": "string"}, "status": {"type": "integer"}, "code": {"type": "string"}, "detail": {"type": ["string", "null"]} } },
                "Page": { "type": "object", "properties": { "data": {"type": "array", "items": {}}, "next_cursor": {"type": ["string", "null"]} } },
                "Tokens": { "type": "object", "properties": { "access_token": {"type": "string"}, "token_type": {"const": "Bearer"}, "expires_in": {"type": "integer"}, "refresh_token": {"type": "string"} } },
                "UserInput": { "type": "object", "required": ["full_name", "email", "role_id", "password"], "properties": { "full_name": {"type": "string"}, "email": {"type": "string"}, "role_id": {"type": "string", "format": "uuid"}, "password": {"type": "string", "description": "Create: temporary password. Update: empty keeps it."} } }
            }
        }
    })
}
