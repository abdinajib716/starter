# Rust Fullstack Startup Starter — Architecture & Blueprint

Leptos (SSR + hydrate) · Axum · SQLx · PostgreSQL (Docker) · Rust/UI (Tailwind)

> Status: design document only. Nothing here has been compiled. Crate versions are the lines current as of Nov 2026 — confirm with `cargo add` / `cargo search` when you start.
>
> **Revision 3 (framework change):** UI framework is now **Dioxus 0.7 fullstack**, not Leptos. Everything backend-side (Postgres, SQLx, Axum, auth, RBAC, audit, backups, email, 2FA, storage, CI/CD) is unchanged. Wherever §1–30 say "Leptos", "`cargo-leptos`", "`leptos_i18n`", `Resource`/`Suspense`, read the Dioxus equivalent in §31. Rust/UI components are Dioxus components.
>
> **Revision 2:** §1–16 are the core. §17–30 add optional/controllable modules (tenancy mode, public API, Resend email, 2FA, sessions, notifications, storage + encrypted off-site backups, table features, i18n, feature flags, multi-instance safety, CI/CD). Where §17–30 change an earlier schema line, the earlier line has already been edited in place, so the SQL in §6 is the current version. §30 lists every change in one place and replaces the build order in §14.

---

## 1. Goal & Scope

A reusable **single-app starter** (one Rust workspace, one deployable binary + Postgres) that every new startup project clones. It ships with real working logic, not mock UI:

| Area | What is real |
|---|---|
| Auth | Login/logout, Argon2id passwords, DB-backed sessions, lockout, rate limit |
| Dashboard | KPI cards + charts computed from real tables, live-updated |
| Access Control → System Setting | Typed key/value settings stored in DB, cached, audited |
| Access Control → User | CRUD, activate/deactivate, assign role, reset password |
| Access Control → Role | CRUD, assign permissions, system roles protected |
| Access Control → Permission | Catalog of permissions (seeded from code), read + describe |
| Access Control → Audit | Append-only log, filter/search/export, live feed |
| Access Control → Backup | `pg_dump` create / download / restore / retention / schedule |
| Avatar menu | Profile (edit name, avatar, change password), Logout |
| Sidebar | Items shown/hidden by the user's permissions |

---

## 2. Framework Decisions

| Layer | Choice | Why |
|---|---|---|
| UI components | **Rust/UI** (rust-ui.com) | **Now Dioxus-only** (Dioxus 0.7 + Tailwind v4): copy-paste components, ~90 of them incl. `sidenav`, `charts`, `data_table`, `dropdown_menu`, `avatar`, `dialog`, `table`, `tabs`, `toast`. Dictates the frontend framework. *(Revision 3: originally Leptos — see §31.)* |
| Frontend + SSR | **Dioxus 0.7 fullstack** (stable Rust, SSR + hydration) | Built on Axum; **server functions** replace a hand-written REST layer; custom Axum routes (SSE, downloads, webhooks) can be added to the same server. |
| HTTP server | **Axum** via `leptos_axum` | Tower middleware ecosystem, first-class Leptos integration. |
| Database access | **SQLx** (Postgres, compile-time-checked queries, `sqlx migrate`) | Plain SQL, async, no ORM magic; fits audit/RBAC queries and aggregates for KPIs. |
| Database | **PostgreSQL 17** in Docker | Also gives `LISTEN/NOTIFY` for realtime. |
| Password hashing | `argon2` (Argon2id) | Current best practice. |
| Sessions | Server-side sessions table + opaque token in `HttpOnly` cookie | Revocable instantly (deactivate user = kill sessions), no JWT pitfalls. |
| Realtime | **SSE** (Axum `Sse`) fed by Postgres `LISTEN/NOTIFY` | Dashboard/audit are server → client only; SSE is simpler and more robust than WebSockets. |
| Background jobs | `tokio` tasks + `tokio-cron-scheduler` | Scheduled backups, session cleanup, retention. No extra infrastructure. |
| Validation | `validator` (shared types compile for both server and WASM) | Same rules on client and server. |
| Config | `figment` or `config` + `.env` | 12-factor settings. |
| Logging | `tracing` + `tracing-subscriber` | Request ids, structured logs. |
| Charts | Small Rust/Leptos **SVG chart components** (line, bar, donut) | Zero JS dependency, SSR-friendly. Fallback: Chart.js/ECharts via `wasm-bindgen` if you need more chart types. Check rust-ui.com/docs for its own chart component first and use it if it fits. |
| Build tool | **`dx`** (Dioxus CLI) + Tailwind v4 | Official Dioxus pipeline (`dx serve`, `dx bundle`). Replaces `cargo-leptos`. |

Rejected alternatives (for the record):
- **Leptos** — was the first choice, dropped in Revision 3 because Rust/UI's maintained components are now Dioxus-only (the Leptos registry no longer serves components).
- **Yew/Trunk + separate API** — two projects, hand-written DTOs/REST, no SSR.
- **Next.js + Rust API** — valid (it is your Police MIS stack) but not a *Rust fullstack* starter and does not use Rust/UI.
- **SeaORM/Diesel** — heavier; RBAC and KPI queries are easier as raw SQL.
- **JWT** — can't revoke on user deactivation without a denylist; sessions table is simpler.

---

## 3. System Architecture

```
┌──────────────────────────── Browser ────────────────────────────┐
│  Leptos WASM (hydrated)  ──  Rust/UI components + Tailwind      │
│   • Router, Layout (Sidebar, Topbar, Avatar menu)               │
│   • Pages call #[server] functions (HTTP POST)                  │
│   • EventSource(/api/stream) for realtime                       │
└──────────────┬───────────────────────────────▲──────────────────┘
               │ cookie: sid (HttpOnly)        │ SSE events
┌──────────────▼───────────────────────────────┴──────────────────┐
│                         Axum server                              │
│  Middleware: trace → request-id → session loader → CSRF/Origin  │
│              → rate-limit (login)                                │
│  ┌────────────┐ ┌──────────────┐ ┌───────────────┐ ┌──────────┐ │
│  │ Leptos SSR │ │ Server fns   │ │ /api/stream   │ │ /api/    │ │
│  │ renderer   │ │ (business    │ │ SSE hub       │ │ backups/ │ │
│  │            │ │  logic)      │ │ (broadcast)   │ │ download │ │
│  └────────────┘ └──────┬───────┘ └──────▲────────┘ └──────────┘ │
│        Services: auth · users · roles · permissions · audit ·    │
│                  settings · backup · metrics                     │
│  Background: scheduler (backup, cleanup) · PG listener task      │
└──────────────┬───────────────────────────────▲──────────────────┘
               │ SQLx pool                      │ LISTEN app_events
┌──────────────▼───────────────────────────────┴──────────────────┐
│ PostgreSQL (Docker)   tables + triggers (NOTIFY) + backup volume │
└──────────────────────────────────────────────────────────────────┘
```

Request rule: **every server function** does `session → permission check → business logic → audit write → return`. No logic lives in components.

---

## 4. Repository Layout

Single crate with `ssr` / `hydrate` features (the cargo-leptos default). Simplest for a starter.

```
starter/
├─ Cargo.toml
├─ rust-toolchain.toml            # channel = "nightly"
├─ ui_config.toml                 # Rust/UI CLI config
├─ tailwind.css  (input)  / style/ (output)
├─ .env.example
├─ Dockerfile
├─ docker-compose.yml
├─ migrations/
│   ├─ 0001_init_auth.sql
│   ├─ 0002_rbac.sql
│   ├─ 0003_audit.sql
│   ├─ 0004_settings.sql
│   ├─ 0005_backups.sql
│   └─ 0006_notify_triggers.sql
├─ public/                        # static assets, avatars default
└─ src/
   ├─ main.rs                     # server entry (ssr)
   ├─ lib.rs                      # hydrate entry + app module
   ├─ app.rs                      # Router, routes, shell
   ├─ components/
   │   ├─ ui/                     # Rust/UI components added via `ui add`
   │   ├─ layout/                 # app_shell, sidebar, topbar, avatar_menu
   │   ├─ charts/                 # line_chart, bar_chart, donut, sparkline
   │   └─ shared/                 # data_table, confirm_dialog, permission_gate
   ├─ pages/
   │   ├─ login.rs
   │   ├─ dashboard.rs
   │   ├─ profile.rs
   │   └─ access_control/ { settings.rs users.rs roles.rs permissions.rs audit.rs backup.rs }
   ├─ api/                        # #[server] functions (thin) — one file per module
   ├─ domain/                     # shared DTOs + validation (compiled for WASM too)
   └─ server/                     # cfg(feature="ssr") only
       ├─ state.rs                # AppState { pool, config, hub, settings_cache }
       ├─ config.rs
       ├─ db.rs                   # pool + migrate + seed
       ├─ auth/ { password.rs session.rs middleware.rs ratelimit.rs }
       ├─ rbac.rs                 # permission registry + checks
       ├─ services/ { users roles audit settings backup metrics }.rs
       ├─ realtime.rs             # PG LISTEN → broadcast → SSE handler
       ├─ jobs.rs                 # scheduler
       └─ seed.rs                 # permissions, roles, first super admin
```

---

## 5. Docker

### docker-compose.yml (development: DB only; app runs with `cargo leptos watch`)

```yaml
services:
  db:
    image: postgres:17
    environment:
      POSTGRES_USER: starter
      POSTGRES_PASSWORD: starter
      POSTGRES_DB: starter
    ports: ["5432:5432"]
    volumes: [pgdata:/var/lib/postgresql/data]
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U starter"]
      interval: 5s
      retries: 10
volumes:
  pgdata:
```

### Production add-on (`docker-compose.prod.yml`)

- `app` service built from the Dockerfile, `depends_on: db (healthy)`, env from `.env`.
- Volume `backups:/data/backups` mounted into **app**.
- Multi-stage Dockerfile: builder (nightly + cargo-leptos + wasm32 target + node for Tailwind) → runtime `debian:bookworm-slim` that **installs `postgresql-client-17`** (needed for `pg_dump`/`pg_restore` — version must match the server).
- Reverse proxy (Caddy/Nginx) for TLS in front.

### .env.example

```
DATABASE_URL=postgres://starter:starter@localhost:5432/starter
APP_SECRET=change-me-32-bytes-min
SITE_ADDR=0.0.0.0:3000
COOKIE_SECURE=false            # true in production
BACKUP_DIR=./data/backups
SEED_ADMIN_EMAIL=admin@example.com
SEED_ADMIN_PASSWORD=ChangeMe!234
```

---

## 6. Database Schema (PostgreSQL)

Use `uuid` primary keys (`gen_random_uuid()`), `timestamptz`, `citext` for emails.

```sql
-- 0001_init_auth.sql
CREATE EXTENSION IF NOT EXISTS citext;
CREATE EXTENSION IF NOT EXISTS pgcrypto;

-- Always created, even in single-tenant mode (one default row). See §18.
CREATE TABLE organizations (
  id         uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  name       text NOT NULL,
  slug       citext NOT NULL UNIQUE,            -- subdomain / URL key
  is_default boolean NOT NULL DEFAULT false,    -- the single org in single mode
  is_active  boolean NOT NULL DEFAULT true,
  created_at timestamptz NOT NULL DEFAULT now()
);
CREATE UNIQUE INDEX one_default_org ON organizations(is_default) WHERE is_default;

CREATE TABLE roles (
  id          uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  org_id      uuid REFERENCES organizations(id),  -- NULL = platform role (Super Admin)
  name        text NOT NULL,                      -- unique per org, see index below
  description text NOT NULL DEFAULT '',
  require_2fa boolean NOT NULL DEFAULT false,     -- used when security.2fa_mode = required_roles (§21)
  is_system   boolean NOT NULL DEFAULT false,   -- cannot be deleted/renamed
  created_at  timestamptz NOT NULL DEFAULT now(),
  updated_at  timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE users (
  id               uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  org_id           uuid NOT NULL REFERENCES organizations(id),
  email            citext NOT NULL,               -- unique among non-deleted rows, see index below
  full_name        text NOT NULL,
  password_hash    text NOT NULL,
  role_id          uuid NOT NULL REFERENCES roles(id),
  avatar_key       text,                          -- storage key, see §24
  locale           text,                          -- NULL = org/app default, see §26
  timezone         text,                          -- NULL = app.timezone setting
  email_verified_at timestamptz,
  deleted_by       uuid,
  is_active        boolean NOT NULL DEFAULT true,
  must_change_pw   boolean NOT NULL DEFAULT false,
  failed_attempts  int NOT NULL DEFAULT 0,
  locked_until     timestamptz,
  last_login_at    timestamptz,
  created_at       timestamptz NOT NULL DEFAULT now(),
  updated_at       timestamptz NOT NULL DEFAULT now(),
  deleted_at       timestamptz                    -- soft delete
);
CREATE INDEX users_role_idx ON users(role_id);
CREATE INDEX users_org_idx  ON users(org_id);
-- soft delete frees the email for re-use; restore must re-check this (§25)
CREATE UNIQUE INDEX users_email_active ON users(org_id, email) WHERE deleted_at IS NULL;
-- role names unique per org; NULL org (platform roles) share one namespace
CREATE UNIQUE INDEX roles_name_per_org ON roles(coalesce(org_id, '00000000-0000-0000-0000-000000000000'::uuid), name);

CREATE TABLE sessions (
  id           uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  token_hash   bytea NOT NULL UNIQUE,            -- SHA-256 of the cookie token
  user_id      uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  ip           inet,
  user_agent   text,
  created_at   timestamptz NOT NULL DEFAULT now(),
  last_seen_at timestamptz NOT NULL DEFAULT now(),
  expires_at   timestamptz NOT NULL,
  device_label text,                             -- "Chrome on Windows", parsed from user_agent (§22)
  mfa_verified_at timestamptz,                   -- NULL = password only (§21)
  kind         text NOT NULL DEFAULT 'web'       -- 'web' | 'pending_2fa' | 'api_refresh'
);
CREATE INDEX sessions_user_idx ON sessions(user_id);
CREATE INDEX sessions_exp_idx  ON sessions(expires_at);

-- 0002_rbac.sql
CREATE TABLE permissions (
  key         text PRIMARY KEY,                  -- e.g. 'users.create'
  module      text NOT NULL,                     -- 'users'
  action      text NOT NULL,                     -- 'create'
  description text NOT NULL
);

CREATE TABLE role_permissions (
  role_id        uuid NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
  permission_key text NOT NULL REFERENCES permissions(key) ON DELETE CASCADE,
  PRIMARY KEY (role_id, permission_key)
);

-- 0003_audit.sql
CREATE TABLE audit_logs (
  id           bigserial PRIMARY KEY,
  at           timestamptz NOT NULL DEFAULT now(),
  org_id       uuid REFERENCES organizations(id),
  actor_id     uuid REFERENCES users(id) ON DELETE SET NULL,
  channel      text NOT NULL DEFAULT 'web',      -- 'web' | 'api' | 'system'
  actor_email  text,                             -- denormalized: survives user deletion
  action       text NOT NULL,                    -- 'user.create', 'auth.login_failed'
  entity_type  text,
  entity_id    text,
  before       jsonb,
  after        jsonb,
  ip           inet,
  user_agent   text,
  success      boolean NOT NULL DEFAULT true
);
CREATE INDEX audit_at_idx     ON audit_logs(at DESC);
CREATE INDEX audit_action_idx ON audit_logs(action);
CREATE INDEX audit_actor_idx  ON audit_logs(actor_id);

-- append-only: block UPDATE/DELETE
CREATE FUNCTION audit_immutable() RETURNS trigger AS $$
BEGIN RAISE EXCEPTION 'audit_logs is append-only'; END $$ LANGUAGE plpgsql;
CREATE TRIGGER audit_no_update BEFORE UPDATE OR DELETE ON audit_logs
  FOR EACH ROW EXECUTE FUNCTION audit_immutable();
-- (retention purge, if ever needed, runs as a separate DB role / TRUNCATE partition)

-- 0004_settings.sql
CREATE TABLE settings (
  key         text PRIMARY KEY,                  -- 'app.name', 'security.session_hours'
  value       jsonb NOT NULL,
  value_type  text NOT NULL,                     -- 'string'|'int'|'bool'|'enum'
  category    text NOT NULL,                     -- 'general'|'security'|'backup'
  label       text NOT NULL,
  description text NOT NULL DEFAULT '',
  updated_by  uuid REFERENCES users(id),
  updated_at  timestamptz NOT NULL DEFAULT now()
);

-- 0005_backups.sql
CREATE TABLE backups (
  id          uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  filename    text NOT NULL UNIQUE,
  size_bytes  bigint NOT NULL DEFAULT 0,
  sha256      text,
  status      text NOT NULL,                     -- 'running'|'completed'|'failed'
  kind        text NOT NULL,                     -- 'manual'|'scheduled'|'pre_restore'
  error       text,
  created_by  uuid REFERENCES users(id),
  created_at  timestamptz NOT NULL DEFAULT now(),
  finished_at timestamptz
);

-- 0006_notify_triggers.sql  (realtime)
CREATE FUNCTION notify_event() RETURNS trigger AS $$
BEGIN
  PERFORM pg_notify('app_events', json_build_object(
      'table', TG_TABLE_NAME, 'op', TG_OP)::text);
  RETURN NULL;
END $$ LANGUAGE plpgsql;
CREATE TRIGGER audit_notify    AFTER INSERT ON audit_logs FOR EACH STATEMENT EXECUTE FUNCTION notify_event();
CREATE TRIGGER users_notify    AFTER INSERT OR UPDATE OR DELETE ON users    FOR EACH STATEMENT EXECUTE FUNCTION notify_event();
CREATE TRIGGER sessions_notify AFTER INSERT OR DELETE ON sessions           FOR EACH STATEMENT EXECUTE FUNCTION notify_event();
CREATE TRIGGER backups_notify  AFTER INSERT OR UPDATE ON backups            FOR EACH STATEMENT EXECUTE FUNCTION notify_event();
```

Because the dashboard KPIs come from `users`, `sessions`, `audit_logs`, `backups`, the starter has **real data from day one** with no demo tables. A project later adds its own domain tables and extra KPI queries.

---

## 7. Authentication Logic

**Login (`login` server fn)**
1. Rate limit by IP + email (e.g. 5 attempts / 15 min, in-memory `governor`, or DB counter).
2. Load user by email where `deleted_at IS NULL`.
3. If `locked_until > now()` → generic failure. Always run an Argon2 verify (dummy hash if user not found) to keep timing constant.
4. Verify password. On failure: `failed_attempts += 1`; at the `security.max_failed_attempts` setting (default 5) set `locked_until = now() + security.lockout_minutes`. Audit `auth.login_failed`.
5. If `!is_active` → fail (generic message), audit.
6. Success: reset counters, set `last_login_at`, create session: 32 random bytes → base64url token; store **SHA-256(token)** in `sessions.token_hash`; `expires_at = now() + security.session_hours`.
7. Set cookie `sid`: `HttpOnly; SameSite=Lax; Path=/; Secure` (when `COOKIE_SECURE`), Max-Age = session length. Audit `auth.login`.
8. Return the redirect target (`/dashboard`, or `/profile` if `must_change_pw`).

**Session middleware (every request)**
- Read cookie → hash → `SELECT` session joined with user + role (+ permissions, see §8). Reject if expired or user inactive/deleted.
- Sliding expiry: update `last_seen_at` / extend `expires_at` at most once per minute.
- Insert `CurrentUser { id, email, name, role, permissions: HashSet<String> }` into request extensions; server fns read it via `leptos_axum::extract`.

**Logout** — delete the session row, clear cookie, audit `auth.logout`.
**Change password** — verify old password, hash new, delete all *other* sessions of that user, audit.
**Deactivate user / reset password / role change** — delete that user's sessions (forces re-login, so permission changes apply instantly).
**CSRF** — server functions are POST + cookie `SameSite=Lax`; additionally verify `Origin`/`Host` header matches on all non-GET requests in middleware.
**Route protection** — SSR router guard: unauthenticated → redirect `/login`; plus a `PermissionGate` component for UI hiding. **UI hiding is cosmetic; the server function check is the real security.**

**First run** — `seed.rs`: if no users exist, create role `Super Admin` + user from `SEED_ADMIN_*` with `must_change_pw = true`.

---

## 8. Access Control Model (RBAC)

- **Permissions are defined in code** as a registry (single source of truth), synced into the `permissions` table on every startup (insert missing, mark/remove obsolete). Admins can't invent permissions; they only assign them.
- Key format: `module.action`.

```rust
// server/rbac.rs
pub const PERMISSIONS: &[(&str, &str)] = &[
  ("dashboard.view",      "View dashboard"),
  ("users.view",          "List and view users"),
  ("users.create",        "Create users"),
  ("users.update",        "Edit users, change role"),
  ("users.delete",        "Delete users"),
  ("users.reset_password","Reset another user's password"),
  ("roles.view",          "View roles"),
  ("roles.create",        "Create roles"),
  ("roles.update",        "Edit roles and their permissions"),
  ("roles.delete",        "Delete roles"),
  ("permissions.view",    "View permission catalog"),
  ("audit.view",          "View audit log"),
  ("audit.export",        "Export audit log"),
  ("settings.view",       "View system settings"),
  ("settings.update",     "Change system settings"),
  ("backup.view",         "View backups"),
  ("backup.create",       "Create backups"),
  ("backup.download",     "Download backups"),
  ("backup.restore",      "Restore a backup"),
  ("backup.delete",       "Delete backups"),
  // ---- added in revision 2 ----
  ("users.restore",       "Restore soft-deleted users"),
  ("users.purge",         "Permanently delete users"),
  ("users.export",        "Export users to CSV"),
  ("users.reset_2fa",     "Reset another user's two-factor authentication"),
  ("users.revoke_sessions","Revoke another user's sessions"),
  ("roles.export",        "Export roles to CSV"),
  ("backup.export",       "Export backup list to CSV"),
  ("flags.view",          "View feature flags"),
  ("flags.update",        "Change feature flags"),
  ("email.view_log",      "View the email delivery log"),
  ("email.send_test",     "Send a test email"),
  ("api_keys.create",     "Create own API tokens"),
  ("api_keys.view_all",   "View every user's API tokens"),
  ("api_keys.revoke_all", "Revoke any API token"),
  ("notifications.broadcast","Send an announcement to users"),
  ("organizations.view",  "View organizations (multi-tenant mode)"),
  ("organizations.manage","Create/edit/disable organizations (multi-tenant mode)"),
];
// Own-account actions (change own password, manage own 2FA, list/revoke own
// sessions, read own notifications) need a valid session only, no permission key.

pub fn require(user: &CurrentUser, key: &str) -> Result<(), ServerFnError> {
    if user.permissions.contains(key) { Ok(()) } else { Err(ServerFnError::new("forbidden")) }
}
```

**Rules**
- A user has exactly one role (simple, enough for a starter; many-to-many is a later extension).
- Role `Super Admin` is `is_system = true`: always holds *all* permissions (computed, not stored), can't be edited, renamed, or deleted.
- Cannot delete a role that still has users.
- Cannot deactivate/delete/demote the **last active Super Admin**, and a user cannot delete or deactivate themself.
- A user can't grant a role/permission they don't hold themselves (prevents privilege escalation).
- Permissions are loaded with the session query and held in `CurrentUser`; a role-permission change deletes the sessions of every user in that role (instant effect).

**Sidebar visibility** — the app shell receives `permissions` once from a `current_user` server fn and filters nav items:

| Nav item | Requires |
|---|---|
| Dashboard | `dashboard.view` |
| Access Control (group) | any of the children |
| ├ System Setting | `settings.view` |
| ├ User | `users.view` |
| ├ Role | `roles.view` |
| ├ Permission | `permissions.view` |
| ├ Audit | `audit.view` |
| └ Backup | `backup.view` |

---

## 9. Module Specifications

Each row: server function → permission → audit action.

### 9.1 Auth & Profile
| Server fn | Permission | Audit |
|---|---|---|
| `login(email, password)` | public | `auth.login` / `auth.login_failed` |
| `logout()` | session | `auth.logout` |
| `current_user()` | session | – |
| `update_profile(name, avatar)` | session (self) | `profile.update` |
| `change_password(old, new)` | session (self) | `profile.password_change` |

Avatar: upload (≤ 1 MB, png/jpg/webp, content-type sniffed) saved to `public/uploads/avatars/<uuid>.<ext>`; fallback to initials in the Rust/UI `Avatar` component.

### 9.2 Users
| Server fn | Permission | Audit |
|---|---|---|
| `list_users(page, q, role, status, sort)` | `users.view` | – |
| `create_user(email, name, role_id, temp_password)` | `users.create` | `user.create` |
| `update_user(id, name, role_id)` | `users.update` | `user.update` (before/after) |
| `set_user_active(id, bool)` | `users.update` | `user.activate` / `user.deactivate` |
| `reset_user_password(id)` | `users.reset_password` | `user.reset_password` (sets `must_change_pw`) |
| `delete_user(id)` (soft) | `users.delete` | `user.delete` |

UI: Rust/UI `Table`, search `Input`, role/status `Select` filters, pagination, `Dialog` for create/edit, `AlertDialog` for delete, `Toast` for results. Server-side pagination and sorting (whitelisted columns).

### 9.3 Roles
| Server fn | Permission | Audit |
|---|---|---|
| `list_roles()` | `roles.view` | – |
| `get_role(id)` → role + granted keys + user count | `roles.view` | – |
| `create_role(name, desc, keys[])` | `roles.create` | `role.create` |
| `update_role(id, name, desc, keys[])` | `roles.update` | `role.update` (diff of keys) |
| `delete_role(id)` | `roles.delete` | `role.delete` |

UI: role editor with permissions grouped by module — checkbox grid (rows = modules, columns = actions) + "select all in module".

### 9.4 Permissions
| Server fn | Permission |
|---|---|
| `list_permissions()` → grouped by module, with the roles that hold each | `permissions.view` |

Read-only catalog (permissions are code-defined). Shows module, key, description, and which roles have it.

### 9.5 Audit
| Server fn | Permission |
|---|---|
| `list_audit(filters: date range, actor, action, success, q; cursor)` | `audit.view` |
| `export_audit_csv(filters)` (streamed route) | `audit.export` (also audited as `audit.export`) |

UI: filter bar, table with expandable row showing before/after JSON diff, **live mode** (new rows pushed through SSE appear at the top). Use keyset pagination on `(at, id)`.

**Writing audit entries**: one helper `audit::record(&mut tx, ctx, action, entity, before, after)` called **inside the same transaction** as the change, so a change and its audit row commit or fail together. `ctx` = actor, IP, user agent from the request. Mask secrets (never log password fields).

### 9.6 System Settings
Seeded keys (extend per project):

| Key | Type | Default | Category |
|---|---|---|---|
| `app.name` | string | "Starter" | general |
| `app.timezone` | string | "UTC" | general |
| `security.session_hours` | int | 12 | security |
| `security.max_failed_attempts` | int | 5 | security |
| `security.lockout_minutes` | int | 15 | security |
| `security.password_min_length` | int | 10 | security |
| `backup.retention_count` | int | 14 | backup |
| `backup.schedule_enabled` | bool | true | backup |
| `backup.schedule_cron` | string | `0 0 2 * * *` | backup |

| Server fn | Permission | Audit |
|---|---|---|
| `list_settings()` grouped by category | `settings.view` | – |
| `update_settings(Vec<(key, value)>)` — validated against `value_type` and per-key rules | `settings.update` | `settings.update` (before/after) |

Settings are read through `settings_cache` (`Arc<RwLock<HashMap>>` in `AppState`) refreshed on update; changing `backup.schedule_*` re-registers the scheduler job.

### 9.6.1 Backup
| Server fn / route | Permission | Audit |
|---|---|---|
| `list_backups()` | `backup.view` | – |
| `create_backup()` | `backup.create` | `backup.create` |
| `GET /api/backups/:id/download` (streamed file) | `backup.download` | `backup.download` |
| `restore_backup(id, confirm_text)` | `backup.restore` | `backup.restore` |
| `delete_backup(id)` | `backup.delete` | `backup.delete` |

**Create** – insert row `status=running`, then spawn a task:
```rust
// server/services/backup.rs (sketch)
let file = format!("{}/{}.dump", backup_dir, filename);
let out = tokio::process::Command::new("pg_dump")
    .args(["--format=custom", "--no-owner", "--file", &file, &database_url])
    .output().await?;
// success → size via fs::metadata, sha256 of file, status=completed, finished_at=now()
// failure → status=failed, error=stderr
```
- Use the connection URL via `PGPASSWORD`/env rather than printing it into logs.
- After each completed backup, apply **retention**: keep the newest `backup.retention_count`, delete older files + rows.
- **Restore** – requires typing the exact filename as confirmation. Steps: (1) automatic `pre_restore` backup, (2) set a global *maintenance flag* (middleware returns 503 for everyone except the restoring request), (3) `pg_restore --clean --if-exists --no-owner -d <db> <file>`, (4) re-run migrations, clear sessions/caches, (5) clear flag, audit. Note that restoring replaces `audit_logs`/`sessions`/`backups` too — write the audit entry for the restore to a file/log as well, and re-insert it after the restore completes.
- **Download** – verify the path stays inside `BACKUP_DIR` (no path traversal: look up the file *by id*, never by user-supplied path).
- **Scheduled** – `tokio-cron-scheduler` job from the setting; kind `scheduled`, actor = system.
- Needs `pg_dump`/`pg_restore` of the same major version as the DB inside the app container (see Dockerfile note).

---

## 10. Dashboard & KPIs (all real queries)

**KPI cards** (each with delta vs previous period + sparkline):

| KPI | Query source |
|---|---|
| Total users | `count(*) FROM users WHERE deleted_at IS NULL` |
| Active users (30d) | `users WHERE last_login_at > now()-'30 days'` |
| Online now | `sessions WHERE last_seen_at > now()-'5 minutes'` (distinct user) |
| Logins today vs yesterday | `audit_logs WHERE action='auth.login'` |
| Failed logins (24h) | `audit_logs WHERE action='auth.login_failed'` |
| Last backup | `backups ORDER BY created_at DESC LIMIT 1` (status + age) |

**Charts**
1. **Logins & failed logins — last 14 days** (line): `date_trunc('day', at)` grouped, `generate_series` to fill empty days.
2. **Audit activity by module — last 7 days** (bar): group by `split_part(action,'.',1)`.
3. **Users by role** (donut): join `roles`.
4. **New users per week — 12 weeks** (bar).
5. **Recent activity** (list of last 10 audit rows, live).

Use one `get_dashboard(range)` server fn returning a single `DashboardData` struct (one round trip; queries run concurrently with `tokio::try_join!`). Permission: `dashboard.view`. Cache for ~5 s in memory to protect the DB when many clients refresh at once.

---

## 11. Realtime Design

```
DB trigger → pg_notify('app_events', {table, op})
      │
      ▼
PG listener task (sqlx::postgres::PgListener, auto-reconnect)
      │  debounce ~500ms, coalesce by table
      ▼
tokio::sync::broadcast::Sender<AppEvent>   (in AppState.hub)
      │
      ▼
GET /api/stream  (Axum Sse, requires valid session)
      │  filter per user permission:
      │    audit events  → only if audit.view
      │    user events   → only if users.view
      ▼
Browser EventSource → Leptos signal → refetch the affected resource
```

- Events are **signals, not payloads** ("audit changed") → client re-runs the relevant server fn via `Resource::refetch()`. This keeps authorization in one place and avoids leaking data through the stream.
- Dashboard: `dashboard` subscribes to `audit`, `users`, `sessions`, `backups`; audit page subscribes to `audit`; backup page to `backups` (shows progress `running → completed`).
- SSE keep-alive every 15 s; client `EventSource` auto-reconnects; on reconnect, refetch once.
- Session check on connect; the server closes streams of sessions that were revoked.

---

## 12. Frontend Structure

**Routes**

| Path | Page | Guard |
|---|---|---|
| `/login` | Login | public (redirect to `/` if logged in) |
| `/` → `/dashboard` | Dashboard | `dashboard.view` |
| `/access-control/settings` | System Setting | `settings.view` |
| `/access-control/users` | Users | `users.view` |
| `/access-control/roles` | Roles (+ editor) | `roles.view` |
| `/access-control/permissions` | Permissions | `permissions.view` |
| `/access-control/audit` | Audit | `audit.view` |
| `/access-control/backup` | Backup | `backup.view` |
| `/profile` | Profile | session |
| `/403`, `*` | Forbidden / Not found | – |

**App shell** – Rust/UI `Sidebar` (collapsible, "Access Control" as a collapsible group with 6 children, active-route highlight, remembers collapsed state), top bar with breadcrumb, theme toggle (light/dark), and the **avatar dropdown**:

```
[Avatar ▾]
 ├ name + email + role badge
 ├ ─────────
 ├ Profile    → /profile
 └ Logout     → logout() → /login
```

**Rust/UI components to install** (`cargo install ui-cli --force`, then `ui init`, then `ui add …`):
`button input label textarea select checkbox switch card badge avatar dropdown_menu dialog alert_dialog sheet table tabs toast tooltip skeleton separator pagination breadcrumb sidebar` (and `demo_*` variants as reference). Check the component list on rust-ui.com for exact names; if a component such as Sidebar/Chart isn't in the registry, build it locally with `clx!` from `leptos_ui` + Tailwind.

**Patterns**
- `Resource` + `Suspense` per page; `ServerAction` for mutations; success/error → `Toast`; on success refetch the list.
- A reusable `DataTable` (server-side pagination/sort/search) used by Users, Audit, Backups.
- `PermissionGate { perm, children }` for hiding buttons (Create, Delete…).
- Form validation: shared `validator` rules in `domain/`, shown inline.
- Loading skeletons, empty states, and error boundaries on every page.

---

## 13. Cross-Cutting Concerns

**Security checklist**
- Argon2id with tuned params (target ≈ 100–250 ms); never log passwords/tokens.
- Session token stored hashed; cookie `HttpOnly`, `SameSite=Lax`, `Secure` in prod.
- Security headers via `tower-http`: CSP, `X-Content-Type-Options`, `X-Frame-Options`/`frame-ancestors`, `Referrer-Policy`.
- All SQL via SQLx bind parameters; sort/filter columns from a whitelist.
- Generic login errors (no user enumeration); rate limiting + lockout.
- Body size limits (avatar upload), request timeouts.
- Backups contain all data — store with restrictive file permissions; consider encrypting at rest later.

**Errors** – one `AppError` enum (`Unauthorized`, `Forbidden`, `NotFound`, `Validation(Vec<FieldError>)`, `Conflict`, `Internal`) mapped to `ServerFnError` with safe messages; details only in `tracing` logs.

**Transactions** – every mutating service method opens a transaction, performs the change, calls `audit::record`, commits.

**Testing**
- Unit: password hashing, permission checks, settings validation.
- Integration: `#[sqlx::test]` (fresh DB per test) for services — login lockout, last-super-admin guard, role-permission changes invalidating sessions, audit written on every mutation, backup retention.
- E2E (optional): Playwright against `cargo leptos serve` for login → create user → change role → see audit entry live.

**Observability** – `tracing` with request id; `/healthz` (DB ping) for Docker/Kubernetes.

---

## 14. Build Order (milestones) — superseded by §30.6, kept for the core-only view

| # | Milestone | Done when |
|---|---|---|
| 0 | Toolchain & skeleton: nightly, `cargo-leptos`, Tailwind, `ui init`, Docker Postgres, migrations run on boot | "Hello" page renders SSR + hydrates, DB connected |
| 1 | Auth: schema, seed super admin, login/logout, session middleware, route guard | Can log in/out; protected routes redirect |
| 2 | RBAC core: permission registry sync, `require()`, `PermissionGate`, `current_user` | Sidebar filters by permission |
| 3 | App shell: sidebar, topbar, avatar menu, theme, profile page | Layout + Profile/Logout work |
| 4 | Audit service + page (filters, diff, export) | Every login logs an entry |
| 5 | Users module | Full CRUD + guards + audit |
| 6 | Roles + Permissions modules | Edit role perms → affected users re-auth |
| 7 | System Settings (+ settings cache) | Session length setting actually changes behavior |
| 8 | Realtime: NOTIFY triggers, listener, SSE, client hook | Audit page updates live in another browser |
| 9 | Dashboard: queries, KPI cards, 5 charts, live refresh | All numbers match SQL |
| 10 | Backup: create/list/download/delete/retention, then restore, then scheduler | Backup → restore round trip verified |
| 11 | Hardening: headers, rate limits, tests, production Dockerfile + compose, README | `docker compose -f … up` runs the whole app |

---

## 15. Known Risks / Things to Verify Early

1. **Nightly Rust** is required (Rust/UI's CLI warns about it) — pin the exact nightly date in `rust-toolchain.toml` so builds are reproducible.
2. **Leptos version drift** — Rust/UI components target a specific Leptos line (0.8 at the time of writing). Check `ui add`'s generated code compiles before adding anything else.
3. **SQLx offline mode** — commit `.sqlx/` (`cargo sqlx prepare`) so Docker builds work without a live DB.
4. **Feature flags** — server-only crates (`sqlx`, `argon2`, `tokio` process) must be `optional` and enabled only under `ssr`, or the WASM build fails.
5. **`pg_dump` version** must match the server's major version in the app image.
6. **Chart components** — confirm what rust-ui.com offers; otherwise use the custom SVG charts planned in §2.
7. **Restore** is the riskiest feature (replaces audit/sessions) — build it last, test it hardest.
8. **Single-role model** — fine for a starter; if a project needs multiple roles per user, replace `users.role_id` with a `user_roles` join table (the `CurrentUser.permissions` set already abstracts this).

---

## 16. Extending for a New Project

1. Clone the starter, rename the crate, change `app.name`.
2. Add domain tables as new migrations.
3. Add the new permissions to the registry in `rbac.rs` (they appear in the Permission page and role editor automatically).
4. Add a page + server fns following the same pattern: `require(...)` → service (transaction + `audit::record`) → return.
5. Add a sidebar item with its permission, and a KPI query to `get_dashboard`.
6. For realtime on a new table, add a `notify_event` trigger and subscribe the page to that table's events.

---

# PART B — Optional & Controllable Modules (Revision 2)

## 17. Module Control Panel (how everything is switched on/off)

Principle: **ship everything, enable per project.** Every module in Part B has a switch, so a simple internal tool turns most of them off and a SaaS turns them on. Three kinds of switch:

| Kind | Where it lives | Changed by | Used for |
|---|---|---|---|
| **Install-time** | `.env` | developer, before first run | `TENANCY_MODE`, secrets (`RESEND_API_KEY`, R2/Drive credentials, `BACKUP_PUBLIC_KEY`) |
| **Runtime setting** | `settings` table → System Setting page | admin with `settings.update` | `email.enabled`, `security.2fa_mode`, `api.public_enabled`, `i18n.enabled`, `backup.offsite_enabled` … |
| **Feature flag** | `feature_flags` table → System Setting → Feature Flags tab | admin with `flags.update` | gradual rollout of new features (§27) |

**Secrets never go into the database or the UI.** The settings page only shows "configured ✓ / missing ✗" for a secret.

### Master switch table (new `settings` seeds)

| Key | Type | Default | Effect when OFF |
|---|---|---|---|
| `email.enabled` | bool | `false` | No email is sent; forgot-password/invite UI hidden; admin sets temp passwords |
| `auth.password_reset_enabled` | bool | `true` | "Forgot password" link hidden (requires `email.enabled`) |
| `auth.invites_enabled` | bool | `true` | Admin must set a temporary password instead of sending an invite |
| `auth.email_verification` | bool | `false` | New users are trusted without verifying their address |
| `security.2fa_mode` | enum | `optional` | `off` · `optional` · `required_roles` · `required_all` |
| `security.2fa_grace_days` | int | `7` | Days a user may log in without enrolling once 2FA becomes required (0 = forced immediately) |
| `api.public_enabled` | bool | `false` | `/api/v1/*` returns 404; token UI hidden |
| `notifications.enabled` | bool | `true` | Bell icon hidden, nothing created |
| `i18n.enabled` | bool | `false` | Single locale; language switcher hidden |
| `storage.primary_driver` | enum | `local` | `local` · `r2` · `gdrive` (credentials from env) |
| `backup.offsite_enabled` | bool | `false` | Backups stay in primary storage only |
| `export.max_rows` | int | `50000` | Upper bound per CSV export |
| `bulk.max_items` | int | `1000` | Upper bound per bulk action |
| `users.deleted_retention_days` | int | `0` | `0` = keep soft-deleted users forever; N = purge after N days |

### Dependency rules (enforced by `update_settings` validation, not just the UI)

- `email.enabled = true` requires `RESEND_API_KEY` present **and** a successful *Send test email* in the last 24 h.
- `auth.password_reset_enabled`, `auth.invites_enabled`, `auth.email_verification` force-off while `email.enabled = false`.
- `security.2fa_mode = required_*` is refused unless the acting admin has 2FA enrolled themselves (prevents self-lockout).
- `backup.offsite_enabled = true` requires `BACKUP_PUBLIC_KEY` + off-site driver credentials present and a successful *Test off-site connection*.
- `api.public_enabled = true` requires `email.enabled` **or** an explicit acknowledgement (tokens are shown once; no email recovery).
- Turning a module off never deletes its data; it only hides/disables behavior.

### Code pattern

```rust
// server/modules.rs
pub struct Modules { /* reads settings_cache */ }
impl Modules {
    pub fn email_enabled(&self) -> bool { .. }
    pub fn twofa_mode(&self) -> TwoFaMode { .. }
    pub fn public_api(&self) -> bool { .. }
}
// every server fn / route of a module starts with:  state.modules.require_email()?;
// the UI gets the same booleans from `current_user()` → `ModuleFlags` and hides menus accordingly.
```

---

## 18. Multi-Tenancy: One Codebase, Single or Multi

**Answer to "some projects are single-tenant, how do I handle both?"** — build the tenant column **in from day one**, but run in *single mode* by default. The cost is one column; the benefit is you never retrofit.

### The switch
`TENANCY_MODE=single|multi` in `.env` (install-time; read-only in the UI).

| | `single` (default) | `multi` |
|---|---|---|
| `organizations` table | exactly one row, `is_default = true`, created by seed | many rows |
| Tenant resolution | always the default org | from subdomain (`acme.app.com`) or, fallback, from the user's `org_id` at login |
| UI | no org switcher, no Organization menu, no org field anywhere | Organization menu for platform admins, org name in top bar |
| Roles | all roles belong to the default org | org roles + platform roles |
| Super Admin | sees everything (only one org) | **Platform Admin** (role with `org_id IS NULL`) manages orgs; **Org Admin** manages only own org |
| Switching later | `single → multi` is supported (data is already in an org); `multi → single` is **not** supported | — |

### Data rules
- Tenant-owned tables carry `org_id NOT NULL`: `users`, `roles`, `audit_logs`, `notifications`, `feature_flags` (nullable = global), `api_tokens`, `email_outbox`, `files`, plus **every domain table a future project adds**.
- Global tables (no `org_id`): `permissions`, `settings` (platform-wide in v1; add `org_settings` later if needed), `backups` (platform-level — a backup covers all tenants).
- Email uniqueness is **per org** (`users_email_active`). In single mode this is identical to global uniqueness.

### Enforcement (defense in depth)
1. **Request context** — middleware builds `Ctx { org_id, user, permissions }`. Every service function takes `&Ctx`; there is no way to run a tenant query without it.
2. **Query rule** — every tenant query includes `WHERE org_id = $ctx_org`. Lint/grep check in CI: no `FROM users` without `org_id` in `server/services/`.
3. **PostgreSQL Row-Level Security (multi mode)** — the safety net if a developer forgets the `WHERE`:
```sql
ALTER TABLE users ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON users
  USING (org_id = current_setting('app.org_id', true)::uuid
         OR current_setting('app.platform_admin', true) = 'on');
```
   Each transaction starts with `SELECT set_config('app.org_id', $1, true)` (transaction-local). The app connects as a **non-owner role** (owners bypass RLS). In single mode the same `set_config` runs with the default org id, so code paths are identical.
4. **Tests** — an integration test per service: create two orgs, act as org A, assert zero rows/ids from org B are readable or writable (including via bulk actions, exports, SSE, notifications, backups listing).

### Tracking which mode a project is in
- `GET /healthz` and the System Setting page display `tenancy: single|multi`.
- `audit_logs.org_id` always filled → in multi mode, every audit/search/export is automatically scoped; Platform Admins may filter by org.
- SSE hub: events carry `org_id`; a subscriber only receives events of their org.

### Extra pieces only used in `multi`
- Page `/access-control/organizations` (`organizations.view/manage`): create org (+ first Org Admin invite), disable org (blocks login of its users), rename slug.
- Tenant resolver: `Host` header → `organizations.slug`; unknown host → 404; custom domains later.
- Per-org limits table (`org_limits`: max_users, storage_mb) if you sell plans.

---

## 19. Public API (optional module, `api.public_enabled`)

Needed only when a Flutter app or third party must talk to the system. Server functions stay **internal**; the public API is a separate, versioned layer that reuses the same services.

```
/api/v1/*   → Axum routes (JSON)  → same services/permissions/audit as the web UI
/api/docs   → OpenAPI (utoipa + Swagger/Scalar UI), only when enabled
```
Compile-time: behind Cargo feature `public-api` (lean binary for projects that don't need it). Runtime: `api.public_enabled`.

### Authentication for mobile/integrations
| Client | Mechanism |
|---|---|
| Mobile app (Flutter) | `POST /api/v1/auth/login` → **access token** (opaque, 15 min) + **refresh token** (opaque, 30 days, rotating). 2FA supported via `mfa_required` response + `POST /api/v1/auth/mfa`. |
| Server-to-server / scripts | **Personal API token** created in Profile → API Tokens (shown once, hashed in DB) |

Both are **opaque tokens stored hashed** in the DB (same revocation model as web sessions — deactivate a user = every token dies; no JWT denylist needed).

```sql
CREATE TABLE api_tokens (
  id           uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  org_id       uuid NOT NULL REFERENCES organizations(id),
  user_id      uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  name         text NOT NULL,                  -- "Warehouse scanner"
  prefix       text NOT NULL,                  -- first 8 chars, shown in UI: "sk_live_ab12cd34…"
  token_hash   bytea NOT NULL UNIQUE,
  scopes       text[] NOT NULL,                -- subset of the owner's permissions
  expires_at   timestamptz,
  last_used_at timestamptz,
  last_used_ip inet,
  revoked_at   timestamptz,
  created_at   timestamptz NOT NULL DEFAULT now()
);
-- mobile refresh tokens reuse `sessions` with kind='api_refresh' (rotation: used token invalidated)
CREATE TABLE idempotency_keys (
  key text, user_id uuid, request_hash bytea, response jsonb, created_at timestamptz DEFAULT now(),
  PRIMARY KEY (user_id, key)
);
```

### Rules
- **Effective permissions = token scopes ∩ owner's current permissions** — changing the user's role immediately narrows tokens.
- Versioned URL (`/api/v1`); breaking changes → `/api/v2`, old version kept N months.
- Errors in **RFC 9457 `application/problem+json`**: `{type, title, status, detail, code, errors[]}`; `code` is a stable machine key (also used for i18n).
- Pagination: cursor-based `?limit=50&cursor=…` → `{data:[…], next_cursor}`.
- Rate limit per token/IP (stored in Postgres, §28): default `api.rate_limit_per_min = 120`, `429` + `Retry-After`.
- `Idempotency-Key` header on POSTs that create things (mobile retries on bad networks).
- CORS: allow-list from setting `api.cors_origins` (mobile apps don't need CORS; browser integrations do).
- Every call audited with `channel='api'` and token id; `last_used_*` updated (throttled to once/min).
- Token UX: shown **once** at creation, prefix visible afterward, expiry default `api.token_default_ttl_days` (90), max `api.max_tokens_per_user` (10), revoke anytime.

### Initial endpoint set
`auth/login, auth/refresh, auth/logout, auth/mfa, me (GET/PATCH), me/sessions (GET/DELETE), users (CRUD + restore), roles (read), permissions (read), audit (read), notifications (list/read), backups (list/create/download)` + `GET /api/v1/health`. Each maps 1:1 to an existing service and permission — no new business logic.

Admin side: Profile → **API Tokens** (own), and `api_keys.view_all / revoke_all` for admins (list inside Access Control → User detail drawer).

---

## 20. Email via Resend (controllable)

**Provider:** [Resend](https://resend.com) HTTP API. Use the `resend-rs` crate or plain `reqwest` → `POST https://api.resend.com/emails` with `Authorization: Bearer $RESEND_API_KEY`. Secret in `.env` only.

**Prerequisites (one-time, outside the code):** verify your sending domain in Resend (SPF + DKIM DNS records), choose `from` like `no-reply@yourdomain.com`.

### Control
- Master switch `email.enabled` (see §17 dependency rules). While off, `email::send()` returns `Disabled` immediately and **callers have a non-email fallback** (e.g. admin sets temp password).
- Settings: `email.from_name`, `email.from_address`, `email.reply_to`.
- System Setting → **Email** tab: status of key (configured ✓/✗), **Send test email** button (`email.send_test`), and the **Delivery log** (`email.view_log`).

### What it sends (templates)
| Template | Trigger | Gate |
|---|---|---|
| `invite` | admin creates user with "send invite" | `auth.invites_enabled` |
| `password_reset` | "Forgot password" | `auth.password_reset_enabled` |
| `verify_email` | signup / email change | `auth.email_verification` |
| `new_login_alert` | login from new device/IP | user pref + `security.login_alerts` |
| `password_changed` | password change/reset | always (when email on) |
| `2fa_changed` | 2FA enabled/disabled/reset | always (when email on) |
| `backup_failed` | scheduled backup failed | admins with `backup.view` |

### Data model — outbox pattern (reliable, retryable, multi-instance safe)
```sql
CREATE TABLE email_outbox (
  id              uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  org_id          uuid REFERENCES organizations(id),
  to_email        citext NOT NULL,
  template        text NOT NULL,
  locale          text NOT NULL DEFAULT 'en',
  payload         jsonb NOT NULL,                 -- template variables (no secrets beyond one-time links)
  status          text NOT NULL DEFAULT 'queued', -- queued|sending|sent|delivered|bounced|complained|failed|suppressed
  attempts        int NOT NULL DEFAULT 0,
  next_attempt_at timestamptz NOT NULL DEFAULT now(),
  provider_id     text,                           -- Resend email id
  error           text,
  created_at      timestamptz NOT NULL DEFAULT now(),
  sent_at         timestamptz
);
CREATE INDEX email_due_idx ON email_outbox(next_attempt_at) WHERE status = 'queued';

CREATE TABLE auth_tokens (          -- one-time links: reset / invite / verify
  id         uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id    uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  purpose    text NOT NULL,         -- 'password_reset'|'invite'|'verify_email'
  token_hash bytea NOT NULL UNIQUE, -- SHA-256 of the emailed token
  expires_at timestamptz NOT NULL,  -- reset 1h, invite 7d, verify 24h
  used_at    timestamptz,
  created_at timestamptz NOT NULL DEFAULT now()
);
CREATE TABLE email_suppressions (email citext PRIMARY KEY, reason text, at timestamptz DEFAULT now());
```

### Flow
1. A service calls `email::enqueue(&mut tx, template, to, payload)` **in the same transaction** as the business change (no email for a rolled-back action).
2. **Worker** task: `SELECT … WHERE status='queued' AND next_attempt_at<=now() FOR UPDATE SKIP LOCKED LIMIT 20` → render template → call Resend → store `provider_id`. On failure: exponential back-off (1m, 5m, 30m, 2h), max 5 attempts → `failed` + notification to admins.
3. **Resend webhook** `POST /api/webhooks/resend` (verify the Svix signature header) → updates status to `delivered / bounced / complained`; bounces/complaints go to `email_suppressions` (never email them again).
4. Templates rendered with `minijinja` (or `askama`) per locale (`templates/email/<locale>/<name>.html` + `.txt`), fallback `en`.

### Security rules
- Reset/invite tokens: 32 random bytes, only the hash stored, single use, short expiry; using one deletes the user's other sessions.
- Forgot-password always answers "if the account exists, an email was sent" (no enumeration); rate limit per IP + per email.
- Never put passwords in email. Links use the configured public `APP_URL`.
- Auth routes: `/auth/forgot-password`, `/auth/reset-password?token=`, `/auth/accept-invite?token=`, `/auth/verify-email?token=`.

---

## 21. Two-Factor Authentication (TOTP)

**Library:** `totp-rs` (RFC 6238; works with Google Authenticator, Microsoft Authenticator, Authy, 1Password). Secrets encrypted at rest with `aes-gcm` using a key derived (HKDF) from `APP_SECRET`.

### Control (`security.2fa_mode`)
| Mode | Behavior |
|---|---|
| `off` | No 2FA UI, no enrollment possible |
| `optional` | Users may enroll in Profile → Security |
| `required_roles` | Users whose role has `roles.require_2fa = true` must enroll (checkbox in Role editor) |
| `required_all` | Everyone must enroll |

When required and the user hasn't enrolled: after password login they get a **restricted session** (`needs_2fa_setup`) that can only reach the enrollment page; after `security.2fa_grace_days` this becomes blocking, before that it's a dismissible banner.

### Schema
```sql
CREATE TABLE user_totp (
  user_id         uuid PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
  secret_enc      bytea NOT NULL,          -- AES-256-GCM(secret)
  confirmed_at    timestamptz,             -- NULL until the first valid code is entered
  last_used_step  bigint NOT NULL DEFAULT 0, -- blocks replay of the same 30 s code
  created_at      timestamptz NOT NULL DEFAULT now()
);
CREATE TABLE recovery_codes (
  id        uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id   uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  code_hash text NOT NULL,                 -- argon2 hash; 10 codes, shown once
  used_at   timestamptz
);
```

### Flows
**Enroll** (Profile → Security → "Enable two-factor"): `begin_2fa()` generates secret → shows QR (`otpauth://totp/Issuer:email?secret=…&issuer=…`) + manual key → user enters a code → `confirm_2fa(code)` sets `confirmed_at`, returns **10 recovery codes (shown once, downloadable)** → other sessions of the user are revoked → audit `2fa.enable`.

**Login with 2FA**
1. Password OK → if user has confirmed 2FA: create a session with `kind='pending_2fa'`, cookie `sid` (valid 5 min, grants **no** access except `/auth/2fa`).
2. `/auth/2fa`: enter TOTP or recovery code → `verify_2fa(code)` → promote the session: `kind='web'`, `mfa_verified_at=now()`, normal expiry; audit `auth.2fa_ok`.
3. Failures: counted (5 → pending session destroyed + account lock like password failures); audit `auth.2fa_failed`.
4. Accept codes in a ±1 step window; reject `step <= last_used_step`.

**Disable** — requires password + current code; audit; email `2fa_changed`.
**Regenerate recovery codes** — requires current code.
**Admin reset** (`users.reset_2fa`) — deletes `user_totp` + recovery codes + sessions; user must re-enroll if mode requires; audit `user.reset_2fa`; cannot be done to the last Super Admin by themselves. Reset requires re-auth (password) of the acting admin.
**API clients** — login returns `mfa_required: true` + a short-lived `mfa_token`; `POST /api/v1/auth/mfa` completes it.

---

## 22. Active Sessions (Profile → Security → Sessions)

Uses the existing `sessions` table (now with `device_label`, `mfa_verified_at`, `ip`, `last_seen_at`).

| Server fn | Permission | Audit |
|---|---|---|
| `list_my_sessions()` → device, IP, signed-in at, last active, **current** marker | session | – |
| `revoke_my_session(id)` (own only; can't revoke "current" here → use Logout) | session | `session.revoke` |
| `revoke_other_sessions()` ("Sign out everywhere else") | session | `session.revoke_all` |
| `admin_list_user_sessions(user_id)` / `admin_revoke_user_sessions(user_id)` | `users.view` / `users.revoke_sessions` | `user.revoke_sessions` |

- `device_label` built at login from the User-Agent (`woothee` or `uaparser`): "Chrome 131 · Windows".
- Optional approximate location from IP (MaxMind GeoLite2) — off by default; IP is shown regardless.
- Revocation = `DELETE FROM sessions`; the SSE hub closes that session's stream; the revoked browser is redirected to `/login` on its next request/event.
- New-device login creates a notification (§23) and, if email is on, an alert email.
- Sessions auto-purged by a daily job after expiry.

---

## 23. In-App Notifications (bell icon)

```sql
CREATE TABLE notifications (
  id         bigserial PRIMARY KEY,
  org_id     uuid REFERENCES organizations(id),
  user_id    uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  kind       text NOT NULL,            -- 'security.new_login', 'backup.failed', 'system.announcement'
  title      text NOT NULL,            -- or an i18n key + params (preferred)
  body       text,
  params     jsonb,                    -- i18n parameters
  link       text,                     -- in-app route to open
  severity   text NOT NULL DEFAULT 'info',   -- info|success|warning|critical
  read_at    timestamptz,
  created_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX notif_user_idx ON notifications(user_id, read_at, created_at DESC);

CREATE TABLE notification_prefs (
  user_id uuid REFERENCES users(id) ON DELETE CASCADE,
  kind    text,
  in_app  boolean NOT NULL DEFAULT true,
  email   boolean NOT NULL DEFAULT false,      -- only honoured if email.enabled
  PRIMARY KEY (user_id, kind)
);
```

**Creating:** `notify::send(&mut tx, Target, kind, params)` where `Target` = `User(id) | Role(id) | Permission("backup.view") | AllInOrg`. Inserted in the same transaction as the cause. A trigger/`pg_notify` emits `{table:'notifications', user_id}`.

**Delivering (reuses §11):** the SSE hub treats this event as *targeted* — only the stream of that `user_id` gets a `notification` signal; the client refetches `unread_count` + latest 10. Works across instances because each instance LISTENs.

**UI:** top-bar bell (Rust/UI `Popover`/`DropdownMenu`) with unread badge; list of latest 10, click → mark read + navigate to `link`; "Mark all as read"; full page `/notifications` with filters and pagination. Hidden when `notifications.enabled = false`.

| Server fn | Permission |
|---|---|
| `unread_count()`, `list_notifications(cursor)`, `mark_read(id)`, `mark_all_read()` | session (own only) |
| `send_announcement(title, body, target)` | `notifications.broadcast` (audited) |
| `get_prefs()`, `update_prefs()` | session |

**Built-in kinds:** new-device login, password changed, role changed, account locked, 2FA enabled/disabled/reset, backup completed/failed, restore finished, export ready (for large async exports later), email permanently failed, announcement.
**Retention:** job deletes read notifications older than `notifications.retention_days` (30).

---

## 24. Storage Abstraction & Encrypted Off-Site Backups

### 24.1 `Storage` trait
```rust
#[async_trait]
pub trait Storage: Send + Sync {
    async fn put(&self, key: &str, body: ByteStream, meta: PutMeta) -> Result<StoredObject>;
    async fn get(&self, key: &str) -> Result<ByteStream>;            // streaming, never load whole file in RAM
    async fn delete(&self, key: &str) -> Result<()>;
    async fn exists(&self, key: &str) -> Result<bool>;
    async fn list(&self, prefix: &str) -> Result<Vec<ObjectInfo>>;
    async fn presigned_get(&self, key: &str, ttl: Duration) -> Result<Option<Url>>; // None if unsupported
    fn name(&self) -> &'static str;
}
```
`AppState` holds a registry: `stores: { primary: Arc<dyn Storage>, offsite: Option<Arc<dyn Storage>> }` built at startup from env + settings.

| Driver | Implementation | Notes |
|---|---|---|
| `local` (default) | `tokio::fs` under `STORAGE_DIR` | Single instance only (see §28). Path traversal-safe: keys are validated (`[a-z0-9/_.-]`, no `..`). |
| `r2` / `s3` | `object_store` crate (S3-compatible: Cloudflare R2, AWS S3, MinIO, Backblaze) or `aws-sdk-s3` | R2: endpoint `https://<account>.r2.cloudflarestorage.com`, no egress fees. Multipart upload for large dumps. |
| `gdrive` | Custom driver over **Google Drive API v3** (`reqwest` + OAuth2) | ⚠ A *service account* has **no storage quota on a personal Drive**: use a **Shared Drive** (Workspace) or an **OAuth refresh token** of a real user. Keys map to file names inside one dedicated folder (Drive has no real paths). Slowest/most fragile option — fine for off-site backup copies, not for hot files like avatars. Behind Cargo feature `storage-gdrive`. |

Logical buckets (key prefixes): `avatars/`, `backups/`, `exports/`, `tmp/`.

Files table (for anything users upload):
```sql
CREATE TABLE files (
  id          uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  org_id      uuid REFERENCES organizations(id),
  store       text NOT NULL,             -- driver name where it lives
  key         text NOT NULL,
  content_type text NOT NULL,
  size_bytes  bigint NOT NULL,
  sha256      text NOT NULL,
  owner_id    uuid REFERENCES users(id),
  created_at  timestamptz NOT NULL DEFAULT now(),
  UNIQUE (store, key)
);
```
`users.avatar_key` points to a `files` key. Avatars are served through the app (`/files/avatars/:id`, with caching headers) so the bucket can stay private. Upload limits: ≤1 MB, MIME sniffed from magic bytes, re-encoded to WebP to strip metadata.

### 24.2 Backup pipeline with encryption + off-site copy
```
pg_dump (custom format)
   └─► encrypt (age, public key)          ← server holds only the PUBLIC key
        └─► sha256 of the encrypted file
             └─► put → primary store  (backups/<ts>.dump.age)
                  └─► [if backup.offsite_enabled] put → off-site store
                       └─► record in backup_copies, verify size/hash
```
**Encryption choice:** [`age`](https://age-encryption.org) (Rust crate `age`) with **asymmetric keys**:
- `BACKUP_PUBLIC_KEY` (age recipient) in `.env` — can encrypt, **cannot decrypt**.
- The **private key is kept offline** (password manager / safe) by the owner. A compromised server or a leaked cloud bucket can't read backups.
- Restore: admin pastes/uploads the private key in the restore dialog (used in memory only, never stored), or restores manually with `age -d`.
- Simpler fallback (`backup.encrypt_mode = symmetric`): AES-256-GCM with `BACKUP_ENCRYPTION_KEY` from env. Faster to set up, but the server can decrypt.
- Off-site copies are **always encrypted** (setting forced on when off-site is enabled).

```sql
ALTER TABLE backups ADD COLUMN encrypted boolean NOT NULL DEFAULT false;
ALTER TABLE backups ADD COLUMN enc_scheme text;            -- 'age-x25519' | 'aes256gcm'
CREATE TABLE backup_copies (
  id         uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  backup_id  uuid NOT NULL REFERENCES backups(id) ON DELETE CASCADE,
  store      text NOT NULL,              -- 'local' | 'r2' | 'gdrive'
  key        text NOT NULL,
  status     text NOT NULL,              -- pending|stored|failed|deleted
  size_bytes bigint,
  error      text,
  verified_at timestamptz,
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE (backup_id, store)
);
```
- Off-site upload runs as an async follow-up job with retries; backup status = `completed` once the primary copy is stored, `offsite` status tracked per copy (UI shows two badges: Local ✓ · Off-site ✓/⏳/✗; failed off-site raises a notification).
- Weekly **verification job**: for each off-site copy, check `exists` + size (and sample-download + hash on a schedule) → `verified_at`.
- Retention applies per target: `backup.retention_count` (primary), `backup.offsite_retention_count` (off-site).
- Restore source picker: prefer primary; if missing, fall back to off-site automatically.
- Settings: `backup.encrypt_mode`, `backup.offsite_enabled`, `backup.offsite_driver`, `backup.offsite_retention_count`; env: `BACKUP_PUBLIC_KEY`, `R2_*` / `GDRIVE_*`.
- **Disaster-recovery doc** (in README): how to decrypt and restore with only the private key + a bucket download, no running app needed.

---

## 25. Table Features: Bulk Actions, CSV Export, Restore Deleted

All three live in the shared `DataTable` component and a shared backend helper so every list behaves the same.

### 25.1 Bulk actions
UI: row checkboxes + "select all on this page"; banner "Select all 3,412 matching this filter"; floating action bar (Rust/UI `Sheet`/toolbar) with the actions allowed for the user's permissions; confirmation `AlertDialog` showing the count.

Backend contract:
```rust
pub enum Selection { Ids(Vec<Uuid>), Filter(UserFilter) }     // Filter = "everything matching these filters"
#[server] async fn bulk_users(action: BulkUserAction, sel: Selection) -> Result<BulkResult>
// BulkResult { succeeded: n, skipped: [{id, reason}], failed: [{id, reason}] }
```
Rules:
- Same permission as the single action (`users.update`, `users.delete`, …). Missing permission → the action isn't shown and is refused server-side.
- Capped at `bulk.max_items`; `Filter` selections are resolved server-side to ids inside the transaction (never trust client-supplied counts).
- **Every per-item guard is re-applied** (last Super Admin, self-delete, role escalation, org scope).
- Runs in one transaction; items that violate a guard are *skipped with reason* (rest proceed) — or `all_or_nothing=true` option.
- Audit: one entry per affected item (`user.deactivate`, with `bulk_id`) **plus** one summary entry (`bulk.user.deactivate`, count, filter JSON).

| Module | Bulk actions |
|---|---|
| Users | activate, deactivate, change role, soft-delete, restore, revoke sessions, reset 2FA, export selected |
| Roles | delete (only unused) |
| Backups | delete |
| Notifications | mark read, delete |
| Audit | export only (immutable) |

### 25.2 CSV export on every list
- Route per resource: `GET /api/export/:resource.csv?<same filter params as the list>` (e.g. `users`, `roles`, `audit`, `backups`). Permission `<module>.export` (`users.export`, `audit.export`, `roles.export`, `backup.export`).
- **Streaming** (`sqlx::query(..).fetch(..)` → CSV rows → `Body::from_stream`), constant memory; capped by `export.max_rows` (response header `X-Export-Truncated: true` if cut).
- Column whitelist per resource (never `password_hash`, token hashes, secrets). Dates ISO-8601 UTC; localized headers per user locale.
- UTF-8 **with BOM** so Excel opens non-Latin text correctly.
- **CSV-injection defense:** any cell starting with `= + - @ \t \r` is prefixed with `'`.
- Respects the active filters, sort, org scope, and — for "selected rows" exports — the chosen ids.
- Audit: `export.users` with filter JSON + row count (also a notification when async exports are added).
- Shared helper: `export::csv_response(columns, stream)`; adding export to a new list = declare columns + one route.

### 25.3 Restore deleted (Users → "Deleted" tab)
- Soft delete sets `deleted_at`, `deleted_by`, `is_active=false`, and kills sessions/tokens (`user.delete`).
- Users page tabs: **Active · Deleted** (Deleted tab visible with `users.restore`). Shows who/when deleted, search, bulk restore.
- `restore_user(id)` (`users.restore`):
  1. Re-check email uniqueness among non-deleted users in the org (`users_email_active`) → friendly error "this email is now used by another user".
  2. Clear `deleted_at/deleted_by`; set `is_active=false` and `must_change_pw=true` (admin explicitly re-activates) — avoids silently resurrecting access.
  3. Audit `user.restore`; notification to the restored user is **not** sent (inactive).
- `purge_user(id)` (`users.purge`, requires typing the email): hard delete; audit rows keep `actor_email` so history survives.
- Optional job: purge soft-deleted users older than `users.deleted_retention_days` (0 = never).
- Generalize: any new entity needing this adds `deleted_at/deleted_by`, a partial unique index `WHERE deleted_at IS NULL`, and reuses the same tab/bulk/restore pattern.

---

## 26. Internationalization (i18n)

**Control:** `i18n.enabled` (default off ⇒ single locale, switcher hidden) + `i18n.default_locale` + `i18n.enabled_locales`. **Even when off, all strings must go through the i18n layer** — the cost of extracting them later is what we are avoiding.

**Library:** `leptos_i18n` (compile-time-checked keys, works with SSR + hydrate). Translations in `locales/<code>.json` (or YAML); `en` is the source of truth.

**Locale resolution order:** `users.locale` → `locale` cookie → `Accept-Language` (only among enabled locales) → `i18n.default_locale`. SSR renders the correct locale on first paint (no flash).

**Rules for the codebase**
1. No hard-coded user-visible strings in `components/` or `pages/` — use `t!(i18n, key)`. A CI check greps for string literals in `view!` macros.
2. **Errors:** the server returns stable `code`s (`auth.invalid_credentials`, `user.last_super_admin`) + params; the client translates. Never ship English sentences from the server.
3. **Permissions & settings labels** are i18n keys (`perm.users.create`, `setting.security.session_hours.label`), not DB text; the DB `description` columns are only the English fallback.
4. **Audit log** stores machine keys (`user.create`); the UI renders them via `audit.action.user.create`.
5. **Emails** (§20) have per-locale templates with `en` fallback; locale = recipient's locale.
6. **Dates/numbers/currency:** stored UTC; formatted in the user's `timezone` + `locale` (via `icu`/`chrono` + formatting helper). One `fmt::date()` / `fmt::number()` helper — no ad-hoc formatting.
7. **RTL** (Arabic, etc.): set `dir="rtl"` on `<html>` by locale; use Tailwind logical utilities (`ms-*`, `me-*`, `ps-*`, `text-start`) instead of `ml/mr/pl/pr`, and `rtl:` variants for icons that must flip (chevrons).
8. Plurals via ICU-style plural forms in the locale files.
9. Missing key → falls back to `en`, logged once; CI fails if a non-`en` locale lacks keys that `en` has (configurable: warn vs fail).

**UI:** language switcher in the top bar and in Profile (saves `users.locale`). Avatar dropdown stays *Profile / Logout* only.

---

## 27. Feature Flags

Flags are for **rolling out features**, distinct from module settings (§17) which are operational configuration.

```sql
CREATE TABLE feature_flags (
  key             text NOT NULL,
  org_id          uuid REFERENCES organizations(id),   -- NULL = global; org row overrides global
  description     text NOT NULL DEFAULT '',
  enabled         boolean NOT NULL DEFAULT false,
  rollout_percent int NOT NULL DEFAULT 100 CHECK (rollout_percent BETWEEN 0 AND 100),
  allowed_role_ids uuid[] NOT NULL DEFAULT '{}',        -- empty = any role
  allowed_user_ids uuid[] NOT NULL DEFAULT '{}',        -- always-on for these users (beta testers)
  updated_by      uuid REFERENCES users(id),
  updated_at      timestamptz NOT NULL DEFAULT now()
);
-- expressions aren't allowed in a PRIMARY KEY, so uniqueness is an expression index:
CREATE UNIQUE INDEX feature_flags_key_org
  ON feature_flags (key, coalesce(org_id, '00000000-0000-0000-0000-000000000000'::uuid));
```
Flags are **declared in code** (like permissions): `flags::REGISTRY = [("new_reports", "New reports page", default_off)]`, synced into the table at startup, so unknown flags can't be invented and removed flags are flagged for cleanup.

**Evaluation**
```rust
fn is_enabled(flag, user) -> bool {
    let f = cache.get(flag, user.org_id).or_else(global);
    f.enabled
      && (f.allowed_role_ids.is_empty() || f.allowed_role_ids.contains(&user.role_id))
      && (f.allowed_user_ids.contains(&user.id) || bucket(user.id, flag) < f.rollout_percent)
}   // bucket = stable hash(user_id + flag) % 100  → same user always gets the same answer
```
**Use:** server side `require_flag(&ctx, "new_reports")?` ; UI `<FeatureGate flag="new_reports">…</FeatureGate>` (flag values delivered in `current_user()`).

**Admin UI:** System Setting → **Feature Flags** tab: list with toggle, % slider, role multi-select, beta-user list, last changed by. `flags.view` / `flags.update`, every change audited (`flag.update` before/after).
**Cache & multi-instance:** loaded into memory, invalidated by `pg_notify('settings_changed')` (§28) + 60 s TTL safety net.
**Hygiene:** each flag has an owner/description; a "stale flags" list shows flags at 100 % for 30+ days → remove from code.

---

## 28. Multi-Instance Safety (Postgres-only, no Redis required)

Goal: run N app containers behind a load balancer without correctness bugs. Everything stateful moves to Postgres; Redis stays optional.

| State (was in-memory) | Problem with N instances | Fix |
|---|---|---|
| Login / API rate limiter | each instance counts separately → limit × N | `rate_limits(key text, window_start timestamptz, count int, PRIMARY KEY(key, window_start))` with `INSERT … ON CONFLICT DO UPDATE SET count = count + 1 RETURNING count`; periodic cleanup. (Swap to Redis later behind a `RateLimiter` trait if throughput demands.) |
| Settings cache | instance A changes a setting, B keeps the old value | After commit: `pg_notify('settings_changed', key)`; every instance LISTENs and reloads that key. TTL 60 s fallback. Same channel for flags and permission-registry changes. |
| Restore maintenance flag | only the instance running the restore returns 503 | `system_state(key PK, value jsonb, since timestamptz, by uuid)` row `maintenance`; middleware checks a cached copy (≤2 s TTL, refreshed by NOTIFY). Health check reports `maintenance` so the load balancer can drain. |
| Scheduler (cron) | every instance fires the same scheduled backup | **Leader election with advisory lock:** each job does `SELECT pg_try_advisory_lock(hash('job:backup'))` on a dedicated connection; only the winner runs it. Same for cleanup jobs. |
| Email worker | duplicate sends | `FOR UPDATE SKIP LOCKED` (already in §20) |
| Backup / restore concurrency | two admins click at once on different instances | Advisory lock `hash('backup')` / `hash('restore')`; UI shows "already running" |
| SSE hub | client connected to instance A, event produced on B | Already safe: every instance has its own `PgListener` → broadcasts to its own clients (§11) |
| Sessions, tokens | – | Already in DB |
| Dashboard cache (5 s) | slight inconsistency | Acceptable; documented |
| **Local-disk storage** | file written on A isn't visible on B | **Requires `storage.primary_driver = r2/s3`** (or a shared volume) once >1 instance; startup warns/refuses when `local` + `INSTANCE_COUNT>1` (`APP_INSTANCES` env) |
| Static avatars | same | served from storage driver |

Other rules: migrations run by a single `app migrate` command in the deploy step (not on every instance boot); graceful shutdown (drain SSE, finish jobs); `/healthz` (liveness) and `/readyz` (DB + not in maintenance).

---

## 29. CI/CD (GitHub Actions) and VPS Deployment

### 29.1 Workflows

**`.github/workflows/ci.yml`** — on every PR and push to `main`
1. Checkout; install toolchain from `rust-toolchain.toml` (pinned nightly) + `wasm32-unknown-unknown`, `rustfmt`, `clippy`.
2. Cache: `Swatinem/rust-cache`.
3. Service container: `postgres:17` with healthcheck, `DATABASE_URL` env.
4. Install `cargo-leptos`, `sqlx-cli` (use `cargo-binstall` for speed); Node + pnpm for Tailwind.
5. `cargo fmt --all --check`
6. `cargo clippy --features ssr -- -D warnings` **and** `cargo clippy --features hydrate --target wasm32-unknown-unknown -- -D warnings`
7. `sqlx migrate run` → **`cargo sqlx prepare --check`** (fails if `.sqlx/` offline data is stale)
8. `cargo test --features ssr` (includes `#[sqlx::test]` isolation + tenant-leak tests)
9. `cargo leptos build --release` (proves the WASM + server build)
10. Supply chain: `cargo deny check` / `cargo audit`
11. Project checks: i18n keys complete, no raw strings in views, no tenant query without `org_id`.

**`.github/workflows/docker.yml`** — on tag `v*.*.*` (and optionally `main` → `:edge`)
- `docker/setup-buildx-action`, login to **GHCR** with `GITHUB_TOKEN`, `docker/metadata-action` for tags (`v1.2.3`, `1.2`, `sha-<short>`, `latest`), `docker/build-push-action` with GHA cache. Multi-arch (`linux/amd64`, optionally `arm64`).
- Image: `ghcr.io/<owner>/<repo>:<tag>`. Build uses `SQLX_OFFLINE=true`.

**`.github/workflows/deploy.yml`** — after docker.yml succeeds / manual `workflow_dispatch` with a tag input
- Uses **GitHub Environments** (`staging`, `production`); production requires manual approval.
- SSH to the VPS (`appleboy/ssh-action` or plain `ssh` with a deploy key):
```bash
cd /opt/app
export IMAGE_TAG=v1.2.3
docker compose pull app
docker compose run --rm app ./starter migrate        # migrations as a separate step
docker compose up -d app                              # recreate with the new image
./scripts/wait-healthy.sh http://127.0.0.1:3000/readyz 60 \
   || { IMAGE_TAG=$PREVIOUS_TAG docker compose up -d app; exit 1; }   # automatic rollback
```
- Writes `PREVIOUS_TAG` to a file before switching, so rollback is one command.
- Posts the result (Slack/Telegram/email, optional).

Also: **Dependabot** (cargo, npm, github-actions, docker), branch protection on `main` (CI required, 1 review), `CODEOWNERS`, release notes generated from tags.

### 29.2 VPS runbook (documented in `docs/DEPLOY.md`)
1. **Server:** Ubuntu LTS VPS; create non-root `deploy` user with SSH key; disable password SSH; `ufw allow 22,80,443`; `unattended-upgrades`; fail2ban.
2. **Docker:** install Docker Engine + compose plugin; add `deploy` to the `docker` group.
3. **Layout:**
```
/opt/app/
  docker-compose.yml      # app + db + caddy
  .env                    # secrets, NEVER in git
  Caddyfile
  data/postgres/  data/backups/  data/uploads/
  scripts/{wait-healthy.sh, rollback.sh}
```
4. **Compose (production):** `db` (postgres:17, volume, **no published port**), `app` (image from GHCR, env_file, volumes `backups`, `uploads`, `healthcheck`, `restart: unless-stopped`), `caddy` (automatic HTTPS, reverse proxy `app:3000`, ports 80/443). Postgres reachable only on the internal Docker network.
5. **DNS:** A record → VPS IP (and wildcard `*.yourdomain` if `TENANCY_MODE=multi`).
6. **GitHub secrets (per environment):** `SSH_HOST`, `SSH_USER`, `SSH_KEY`, `GHCR_PULL_TOKEN` (if image is private). App secrets stay on the server `.env`.
7. **First deploy:** copy compose/Caddy/.env → `docker compose up -d db` → `docker compose run --rm app ./starter migrate && ./starter seed` → `up -d`. Log in with the seeded admin (forced password change), enable 2FA, configure Email/Storage in System Setting.
8. **Backups:** enable scheduled + off-site (§24); also take a periodic VPS snapshot; **test a restore** before go-live and then quarterly.
9. **Updates:** push a tag → CI → image → approve production deploy → auto-rollback on failed health check.
10. **Monitoring:** UptimeRobot/BetterStack on `/healthz`, disk-space alert (backups + Postgres volume), `docker logs` shipped to a log service (optional).

---

# 30. Consolidated Changes (what Part B changes in the core)

### 30.1 New / changed tables
`organizations`, the `org_id` columns, and the new `users`/`roles`/`sessions`/`audit_logs` columns are already edited into migrations 0001–0003 in §6 (create `organizations` first in 0001). New migrations after 0006:
`0007_email` (`email_outbox`, `auth_tokens`, `email_suppressions`) · `0008_2fa` (`user_totp`, `recovery_codes`) · `0009_notifications` (`notifications`, `notification_prefs`) · `0010_files_storage` (`files`, `backups.encrypted/enc_scheme`, `backup_copies`) · `0011_feature_flags` · `0012_api` (`api_tokens`, `idempotency_keys`) · `0013_system_state` (`system_state`, `rate_limits`).

### 30.2 Updated settings seeds
All keys from §17 plus: `email.from_name`, `email.from_address`, `email.reply_to`, `security.login_alerts`, `security.2fa_issuer`, `api.token_default_ttl_days`, `api.max_tokens_per_user`, `api.rate_limit_per_min`, `api.cors_origins`, `i18n.default_locale`, `i18n.enabled_locales`, `notifications.retention_days`, `backup.encrypt_mode`, `backup.offsite_driver`, `backup.offsite_retention_count`.

### 30.3 System Setting page — tabs
`General · Security (sessions, lockout, 2FA mode) · Email · Storage & Backup · API · Localization · Feature Flags`. Each tab has its master switch at the top, dependency warnings, and "Test" buttons (test email, test off-site connection).

### 30.4 Added routes
| Path | Page |
|---|---|
| `/auth/forgot-password`, `/auth/reset-password`, `/auth/accept-invite`, `/auth/verify-email` | public email flows (only when enabled) |
| `/auth/2fa` | second step of login |
| `/profile` (tabs) | **Profile · Security (password, 2FA, Sessions) · API Tokens · Notifications · Language** |
| `/notifications` | full notification list |
| `/access-control/organizations` | multi-tenant mode only |
| `/api/v1/*`, `/api/docs` | public API (when enabled) |
| `/api/export/:resource.csv` | CSV export |
| `/api/webhooks/resend` | Resend delivery events |
| `/healthz`, `/readyz` | probes |

### 30.5 Added layout items
```
src/server/{modules.rs, tenancy.rs, email/, totp.rs, notify.rs, flags.rs, export.rs, ratelimit_pg.rs}
src/server/storage/{mod.rs, local.rs, s3.rs, gdrive.rs}
src/server/api_v1/            # only with feature "public-api"
locales/{en.json, ...}   templates/email/<locale>/
.github/workflows/{ci,docker,deploy}.yml   docs/DEPLOY.md   scripts/
```
CLI subcommands of the binary: `starter serve | migrate | seed | create-admin | reset-2fa <email> | backup-verify` (break-glass tools that work without the web UI).

### 30.6 Revised build order (supersedes §14)

| # | Milestone |
|---|---|
| 0 | Toolchain, skeleton, Docker Postgres, migrations, **organizations + default org + `Ctx`** |
| 1 | Auth core (login/logout/session middleware/route guard) |
| 2 | RBAC core + `ModuleFlags` + settings cache with NOTIFY invalidation |
| 3 | App shell: sidebar, top bar, avatar menu, theme, **i18n layer wired in from the start** |
| 4 | Audit service + page |
| 5 | Users (+ soft delete) with shared `DataTable`, **bulk actions, CSV export, Deleted tab** |
| 6 | Roles + Permissions |
| 7 | System Setting (tabs, dependency rules, feature flags) |
| 8 | Realtime (SSE, NOTIFY) + **notifications + bell** |
| 9 | Profile: password, **sessions page, 2FA (TOTP + recovery codes + modes)** |
| 10 | Dashboard KPIs/charts |
| 11 | **Storage trait + local driver**, avatars, Backup (create/download/delete/retention) |
| 12 | **Backup encryption (age), R2 driver, off-site copies + verification**, then restore (last) |
| 13 | **Email via Resend** (outbox, templates, reset/invite/verify, webhook) |
| 14 | **Multi-instance hardening** (PG rate limiter, system_state, advisory locks) |
| 15 | **Public API v1** (tokens, refresh, OpenAPI) — only if a project needs it |
| 16 | **Multi-tenant mode** (resolver, RLS, org page, leak tests) — only if a project needs it |
| 17 | **CI/CD + VPS deploy + DEPLOY.md**, Google Drive driver (optional) |

Milestones 15 and 16 are optional modules: the starter is complete and shippable without them, and the groundwork (org_id, `Ctx`, services behind traits) is already in place from milestone 0.

### 30.7 Additional risks
1. **RLS** requires the app to connect as a non-owner DB role and to `set_config` per transaction — easy to get wrong; keep the leak tests mandatory.
2. **Google Drive** quotas/auth model (§24) — prefer R2 for off-site; treat Drive as optional.
3. **2FA lockout** — keep the CLI `reset-2fa` and recovery codes; never allow requiring 2FA before the admin has enrolled.
4. **Resend deliverability** depends on DNS (SPF/DKIM/DMARC) — verify the domain before enabling `email.enabled`.
5. **`leptos_i18n` + `rust-ui` version compatibility** — verify both against the same Leptos version before committing to the layout.
6. **Backups contain all tenants** — in multi mode, restoring one org's data from a full dump needs a separate tool; document that restore is platform-wide.

---

# 31. Revision 3 — Leptos → Dioxus Translation Guide

**Why:** Rust/UI's repository is now "Rust/UI — Dioxus" (Dioxus 0.7 + Tailwind v4; 0 Leptos component files in its registry source). Its website registry no longer serves Leptos components, so `ui add button` fails for a Leptos project. The starter therefore uses Dioxus fullstack.

**What does not change:** database schema (§6), auth/session logic (§7), RBAC (§8), module specs (§9), KPI queries (§10), SSE design (§11, server side), Part B modules (§17–30 backend parts), Docker/CI/CD (§5, §29), build order milestones (§30.6) apart from milestone 0.

## 31.1 Concept mapping

| Was (Leptos) | Now (Dioxus 0.7) | Notes |
|---|---|---|
| `cargo-leptos watch/build` | `dx serve` / `dx bundle` | `dx` installed with `cargo install dioxus-cli`; `dx doctor` checks the toolchain |
| nightly toolchain | **stable** Rust + `wasm32-unknown-unknown` | no nightly needed |
| `#[server] async fn` | `#[server]` (and `#[get]`/`#[post]` for explicit routes) | server functions become Axum endpoints automatically |
| `Resource` + `Suspense` | `use_server_future` / `use_resource` / `use_action` + suspense | mutations via `use_action` |
| `view!{}` macro | `rsx!{}` macro | |
| Signals / `RwSignal` | Dioxus signals (Copy) / `use_signal`, context via `use_context_provider` | |
| `leptos_router` | Dioxus router with `#[derive(Routable)]` enum | route table = the one in §12 |
| `leptos_axum` extractors | server-function extractors + custom Axum router (`serve_dioxus_application`) | session middleware, SSE route, CSV export, backup download and Resend webhook are plain Axum routes on the same router |
| `leptos_i18n` (§26) | a Dioxus i18n crate (e.g. `dioxus-i18n`) **or** a small in-house `t!(key)` lookup over `locales/*.json` | verify crate/version at build time; the rules in §26 (no hard-coded strings, error codes, RTL, fallback to `en`) stay |
| `Cargo.toml` features `ssr`/`hydrate` | features `web` + `server` (`dioxus = { version = "0.7", features = ["fullstack"] }`) | server-only crates (sqlx, argon2, tokio process, aes-gcm) behind the `server` feature |
| Rust/UI via `ui add` (Leptos registry) | Rust/UI components **copied** from `rust-ui/ui` → `app_crates/registry/src/ui/*.rs` (or `ui add` if the Dioxus registry answers) | components depend on `dioxus`, `tw_merge`, `icons` |

## 31.2 Layout changes (replaces the tree in §4)

```
starter/
├─ Cargo.toml  Dioxus.toml  .env  docker-compose.yml  migrations/
├─ assets/  public/
└─ src/
   ├─ main.rs                 # dioxus::launch / custom Axum router (server) 
   ├─ app.rs  routes.rs       # Router enum, layout
   ├─ components/ui/          # Rust/UI components (copied)
   ├─ components/layout/      # sidenav, topbar, avatar menu, permission_gate, data_table wrapper
   ├─ pages/ …                # same pages as §12
   ├─ api/                    # #[server] functions (thin)
   ├─ domain/                 # shared DTOs + validation
   └─ server/                 # cfg(feature="server"): state, auth, rbac, services, realtime, jobs, storage, email
```

## 31.3 Milestone 0 (replaces §30.6 row 0)
Install `dx` + stable Rust → `dx new` fullstack project → Tailwind v4 working → Rust/UI `button`/`card` rendered → Docker Postgres + SQLx migrations on boot → organizations + default org + `Ctx`.

## 31.4 Risks specific to this switch
1. Rust/UI states it is **early stage / experimental** — expect breaking changes; pin the commit you copy components from.
2. Dioxus fullstack (0.7) is younger than Leptos; check each needed feature early (server-function extractors for the session cookie, custom Axum routes next to the Dioxus app, SSE streaming).
3. Dioxus 0.7 has built-in zero-setup Tailwind; Rust/UI expects its own `style/tailwind.css` theme — reconcile on day one.
4. Hot-reload/port behaviour of `dx serve` (bind address) must be confirmed from `dx serve --help`.