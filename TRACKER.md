# Starter — Build Tracker

Source of truth: [RUST FULLSTACK STARTER BLUEPRINT.md](RUST%20FULLSTACK%20STARTER%20BLUEPRINT.md) (Revision 3, Dioxus 0.7).
This file tracks **what is built**; the blueprint says **how**. `§` numbers point into the blueprint.
Order follows the revised build order (§30.6), with Milestone 0 from §31.3.

**Status legend:** `[x]` done & verified · `[~]` in progress / partial · `[ ]` not started · `[-]` skipped (reason noted)

Update rule: tick an item only when its **Done when** check has actually been run. Log anything that deviates from the blueprint in [Decisions](#decisions--deviations).

---

## Overview

| # | Milestone | Status | Notes |
|---|---|---|---|
| 0 | Toolchain, skeleton, Postgres, migrations, org + `Ctx` | 🟡 in progress | 23/25 done; open: M0.24 (after M1), M0.25 i18n spike |
| 1 | Auth core | ⚪ | |
| 2 | RBAC core + `ModuleFlags` + settings cache | ⚪ | |
| 3 | App shell + i18n layer | ⚪ | |
| 4 | Audit | ⚪ | M4.8 done early |
| 5 | Users + DataTable (bulk, CSV, Deleted tab) | ⚪ | |
| 6 | Roles + Permissions | ⚪ | |
| 7 | System Setting (tabs, deps, feature flags) | ⚪ | |
| 8 | Realtime (SSE) + notifications | ⚪ | |
| 9 | Profile: password, sessions, 2FA | ⚪ | |
| 10 | Dashboard KPIs + charts | ⚪ | |
| 11 | Storage (local) + avatars + Backup | ⚪ | |
| 12 | Backup encryption, R2, off-site, **restore** | ⚪ | |
| 13 | Email via Resend | ⚪ | |
| 14 | Multi-instance hardening | ⚪ | |
| 15 | Public API v1 *(optional)* | ⚪ | only if a project needs it |
| 16 | Multi-tenant mode *(optional)* | ⚪ | only if a project needs it |
| 17 | CI/CD + VPS deploy (+ GDrive driver) | ⚪ | |

Legend: ⚪ not started · 🟡 in progress · 🟢 done

---

## M0 — Toolchain & skeleton (§31.3, §5, §6, §18)

**Done when:** page renders SSR + hydrates with Rust/UI styled, app connects to Docker Postgres, migrations run on boot, default org exists, `Ctx` is available to server fns.

- [x] M0.1 Stable Rust + `wasm32-unknown-unknown`, `dx` 0.7.10 installed
- [x] M0.2 `dx new` fullstack skeleton (Dioxus 0.7, edition 2024)
- [x] M0.3 Tailwind v4 via `dx`; Rust/UI theme (tokens, `.dark`, `@theme inline`, base layer) in `tailwind.css`
- [x] M0.4 Rust/UI components copied into `src/ui/` (35) + hooks into `src/hooks/` (4), pinned to rust-ui `4f5a516`
- [x] M0.5 Toast runtime (`toast.js/css`) + ApexCharts + `chart_init.js` loaded in `App`
- [x] M0.6 Smoke-test page: Card / Avatar / Button variants / ThemeToggle / Toast render correctly
- [x] M0.7 Cargo features split: `web` / `server`; server-only crates (`sqlx`, `argon2`, `tokio` process…) `optional` under `server` (§15.4) — `server` feature enables sqlx, figment, dotenvy, tower-http, tokio, tracing, anyhow (all `optional`); wasm tree verified clean
- [x] M0.8 Restructure to §31.2 layout: `app.rs`, `routes.rs`, `components/{ui,layout}`, `pages/`, `api/`, `domain/`, `server/` — `ui/` + `hooks/` kept at crate root (see Decisions); `components/layout` is an empty placeholder for M3
- [x] M0.9 `docker-compose.yml` with `postgres:17` + healthcheck (§5) — compose already existed; host port **5440** (5432 is taken on this machine)
- [x] M0.10 `.env.example` (`DATABASE_URL`, `APP_SECRET`, `SITE_ADDR`, `COOKIE_SECURE`, `BACKUP_DIR`, `SEED_ADMIN_*`, `TENANCY_MODE`) — `SITE_ADDR` replaced by `IP`/`PORT` (read by Dioxus); adds `APP_ENV`, `TEST_DATABASE_URL`, `DB_MAX_CONNECTIONS`
- [x] M0.11 Config loader (`figment`/`config` + `.env`) and `tracing` subscriber with request id — figment + dotenvy; prod refuses short `APP_SECRET` / `COOKIE_SECURE=false`; tracing via the Dioxus logger + `x-request-id` (set, propagated, in span)
- [x] M0.12 SQLx pool + `sqlx migrate` on boot; commit `.sqlx/` offline data (§15.3) — pool + `sqlx::migrate!` on boot; `.sqlx/` committed
- [x] M0.13 Core migrations from §6, timestamp-named via `sqlx migrate add`: `init_auth` (organizations first; roles, users, sessions), `rbac` (permissions, role_permissions), `audit` (audit_logs + immutability trigger), `settings`, `backups`, `notify_triggers` — applied to dev DB
- [x] M0.14 Seed: default organization (`is_default = true`) — `seed::ensure_default_org`, race-safe via `one_default_org`; test `default_org_is_created_once`
- [x] M0.15 `Ctx { org_id, user, permissions }` built per request (§18 Enforcement 1) — `ctx_layer` middleware + `Ctx::current()` in server fns; proven by `/api/system/info`
- [x] M0.16 Custom Axum router next to the Dioxus app (`serve_dioxus_application`) — proves custom routes work (§31.4.2) — `dioxus::serve` + `dioxus::server::router(App)` + own routes and layers
- [x] M0.17 `/healthz` (DB ping) — shows `tenancy: single|multi` — `GET /healthz` → 200/503 with `db`, `tenancy`, `version`
- [x] M0.18 Remove template leftovers (`Hero`, `Echo`, `Blog`, `Navbar`, `assets/main.css`, `header.svg`)
- [x] M0.19 **Risk spike (throwaway branch `spike/dioxus-server`)**: (a) a server function that reads a request cookie and returns it; (b) a custom Axum route `/spike/ping` next to the Dioxus app; (c) a 3-line SSE stream route that emits one event per second; (d) confirm `sqlx` and other server-only crates are absent from the WASM build. Record results in Decisions. Ticks R3, R4, R5, R6. — all four passed — see Decisions; code kept on branch `spike/dioxus-server` (`efda3c5`)
- [x] M0.20 Test DB setup: `#[sqlx::test]` with a separate test database (`starter_test`), documented in README — `just test`; documented in README
- [x] M0.21 `justfile` with: `dev`, `db-up`, `db-down`, `migrate`, `prepare` (`cargo sqlx prepare`), `lint` (fmt + clippy server + wasm), `test` — `just --list`; also `migration <name>` and `hooks`
- [x] M0.22 Pre-commit hook: `cargo fmt --check` and clippy; `.sqlx/` freshness check — `.githooks/pre-commit`, install with `just hooks`; `#[rustfmt::skip]` keeps vendored files untouched
- [x] M0.23 `NOTICE` / `src/ui/README.md`: record Rust/UI source commit `4f5a516`, its license and attribution — `NOTICE`, `src/ui/README.md`, `src/ui/LICENSE-rust-ui` (MIT, Max Wells)
- [ ] M0.24 Early production smoke build: `dx bundle --release` and a minimal Docker build run once (after M1), to catch WASM/server split and `pg_dump` version problems (R6, R9) early
- [ ] M0.25 i18n spike: chosen crate works with SSR + hydration; record decision. Must be done before M3 (ticks R7)

---

## M1 — Auth core (§7, §9.1)

**Done when:** can log in/out; protected routes redirect to `/login`; lockout after N failures works.

- [ ] M1.1 Argon2id hashing, params tuned to ~100–250 ms; dummy verify when user not found (constant time)
- [ ] M1.2 `login(email, password)` — rate limit (IP + email), lockout (`security.max_failed_attempts`, `security.lockout_minutes`), inactive check, generic errors
- [ ] M1.3 Session create: 32 random bytes → base64url token; store SHA-256 in `sessions.token_hash`; `expires_at` from `security.session_hours`
- [ ] M1.4 Cookie `sid`: `HttpOnly; SameSite=Lax; Path=/; Secure` (when `COOKIE_SECURE`)
- [ ] M1.5 Session middleware: cookie → hash → session + user + role + permissions; sliding expiry (≤ once/min); `CurrentUser` in request extensions
- [ ] M1.6 Server-function extractor reads the session cookie (verify Dioxus 0.7 extractor support early — §31.4.2)
- [ ] M1.7 `logout()` deletes session, clears cookie
- [ ] M1.8 `current_user()` server fn (permissions + `ModuleFlags` + flags later)
- [ ] M1.9 CSRF: `Origin`/`Host` check on all non-GET requests
- [ ] M1.10 Route guard: unauthenticated → `/login`; logged-in on `/login` → `/`
- [ ] M1.11 Login page (Rust/UI `Card`, `Input`, `Label`, `Button`, inline errors)
- [ ] M1.12 First-run seed: `Super Admin` role + admin from `SEED_ADMIN_*` with `must_change_pw = true`
- [ ] M1.13 `must_change_pw` → redirect to `/profile` after login
- [ ] M1.14 Audit hooks for `auth.login`, `auth.login_failed`, `auth.logout` (wired fully in M4)
- [ ] M1.15 Tests: lockout, inactive user, expired session, constant-time path
- [ ] M1.16 Security headers via `tower-http` (CSP, nosniff, frame-ancestors, Referrer-Policy), body limits, timeouts (§13; moved from M17)
- [ ] M1.17 Cookie name `__Host-sid` when `COOKIE_SECURE=true` (requires `Secure`, `Path=/`, no `Domain`); plain `sid` in local dev
- [ ] M1.18 Password policy: minimum length (setting `security.password_min_length`, §9.6), reject common passwords list; applied in create, reset and change flows
- [ ] M1.19 Lockout notification to the user (in-app now, email after M13)

---

## M2 — RBAC core + module flags + settings cache (§8, §17, §28)

**Done when:** sidebar filters by permission; server fns refuse without permission; changing a setting on one instance reaches the others.

- [ ] M2.1 `PERMISSIONS` registry in code (all keys from §8, incl. revision 2 additions)
- [ ] M2.2 Startup sync into `permissions` table (insert missing, remove obsolete)
- [ ] M2.3 `require(&ctx, key)` helper; Super Admin computed as all permissions
- [ ] M2.4 Guards: last active Super Admin, no self-delete/deactivate, no granting what you don't hold
- [ ] M2.5 Role-permission change deletes sessions of every user in that role
- [ ] M2.6 `PermissionGate { perm, children }` component
- [ ] M2.7 `AppError` enum → `ServerFnError` with stable codes (§13)
- [ ] M2.8 Settings seeds (§9.6 + §17 master switches + §30.2)
- [ ] M2.9 `settings_cache` (`Arc<RwLock<HashMap>>`) + `pg_notify('settings_changed')` invalidation + 60 s TTL
- [ ] M2.10 `Modules` struct (`email_enabled`, `twofa_mode`, `public_api`, …) + `ModuleFlags` sent to UI via `current_user()`
- [ ] M2.11 Tests: permission checks, last-super-admin guard, escalation guard
- [ ] M2.12 Convention test / CI check: every `#[server]` / `#[post]` / `#[get]` function under `api/` either calls `require()` / `require_flag()` or is on an explicit allow-list of public endpoints (login, health, etc.). A forgotten check fails CI.

---

## M3 — App shell + i18n layer (§12, §26)

**Done when:** layout + Profile/Logout work; sidebar shows only permitted items; every UI string goes through `t!`.

- [ ] M3.1 Router enum with all routes from §12 + §30.4 (stubs OK)
- [ ] M3.2 Layout: Rust/UI `sidenav` — collapsible, "Access Control" group with 6 children, active-route highlight, remembers collapsed state
- [ ] M3.3 Sidebar filtering per §8 visibility table
- [ ] M3.4 Top bar: breadcrumb, `ThemeToggle`, (bell slot for M8), (language switcher slot)
- [ ] M3.5 Avatar dropdown: name + email + role badge → Profile / Logout
- [ ] M3.6 `/403` and `*` Not Found pages
- [ ] M3.7 Loading skeletons, empty states (`Empty`), error boundaries pattern
- [ ] M3.8 i18n layer wired in, using the crate chosen in M0.25 (no UI strings before M0.25 is decided); `en` source of truth
- [ ] M3.9 Locale resolution: `users.locale` → cookie → `Accept-Language` → default; SSR first paint correct
- [ ] M3.10 RTL: `dir="rtl"` by locale; logical Tailwind utilities (`ms-*`, `ps-*`, `text-start`)
- [ ] M3.11 `fmt::date()` / `fmt::number()` helpers (UTC stored, user timezone + locale displayed)

---

## M4 — Audit (§9.5)

**Done when:** every login writes an entry; filters, JSON diff and CSV export work.

- [ ] M4.1 `audit::record(&mut tx, ctx, action, entity, before, after)` inside the same transaction
- [ ] M4.2 Secret masking (never log password/token fields)
- [ ] M4.3 `list_audit(filters; cursor)` with keyset pagination on `(at, id)`
- [ ] M4.4 Audit page: filter bar (date range, actor, action, success, q), expandable before/after diff
- [ ] M4.5 `export_audit_csv` (streamed, `audit.export`, itself audited)
- [ ] M4.6 Actions rendered via i18n keys (`audit.action.user.create`)
- [ ] M4.7 Live mode hook (wired in M8)
- [x] M4.8 Test: append-only trigger rejects UPDATE/DELETE — `audit_log_rejects_update_and_delete` (done early, with the M0 test setup)
- [ ] M4.9 Growth plan: decide and document one of (a) monthly partitioning of `audit_logs`, (b) archive-to-storage after `audit.retention_days`. The append-only trigger must still hold for the active partition. Record decision.

---

## M5 — Users + shared DataTable (§9.2, §25)

**Done when:** full CRUD with guards and audit; bulk actions, CSV export and Deleted tab work.

- [ ] M5.1 Shared `DataTable` wrapper (Rust/UI `data_table`): server-side pagination, sort (whitelisted columns), search
- [ ] M5.2 `list_users(page, q, role, status, sort)` (org-scoped)
- [ ] M5.3 `create_user` / `update_user` (before/after audit) / `set_user_active` / `reset_user_password` / `delete_user` (soft)
- [ ] M5.4 Sessions killed on deactivate / reset password / role change / delete
- [ ] M5.5 UI: search `Input`, role/status `Select`, `Pagination`, `Dialog` create/edit, `AlertDialog` delete, `Toast` results
- [ ] M5.6 Bulk actions: `Selection::{Ids, Filter}`, cap `bulk.max_items`, per-item guards re-applied, skipped-with-reason, per-item + summary audit
- [ ] M5.7 Bulk UI: row checkboxes, select-all-matching banner, floating action bar, count confirmation
- [ ] M5.8 CSV export helper `export::csv_response` — streamed, BOM, column whitelist, CSV-injection prefix, `export.max_rows` + `X-Export-Truncated`
- [ ] M5.9 `GET /api/export/users.csv` (`users.export`)
- [ ] M5.10 Deleted tab (`users.restore`): `restore_user` (email re-check, inactive + `must_change_pw`)
- [ ] M5.11 `purge_user` (`users.purge`, type email to confirm)
- [ ] M5.12 Optional purge job for `users.deleted_retention_days`
- [ ] M5.13 Tests: guards, restore email conflict, bulk skip reasons, CSV injection

---

## M6 — Roles + Permissions (§9.3, §9.4)

**Done when:** editing a role's permissions forces affected users to re-authenticate.

- [ ] M6.1 `list_roles`, `get_role` (+ granted keys + user count)
- [ ] M6.2 `create_role` / `update_role` (audit diff of keys) / `delete_role` (refuse if users assigned)
- [ ] M6.3 System roles protected (no edit/rename/delete)
- [ ] M6.4 Role editor: checkbox grid (rows = modules, cols = actions) + "select all in module"
- [ ] M6.5 `require_2fa` checkbox on role (used by M9)
- [ ] M6.6 Permissions page: read-only catalog grouped by module, with roles holding each
- [ ] M6.7 Roles CSV export (`roles.export`); bulk delete unused roles
- [ ] M6.8 Tests: role change invalidates sessions; cannot delete role in use

---

## M7 — System Setting (§9.6, §17, §27, §30.3)

**Done when:** changing `security.session_hours` actually changes session length; dependency rules refuse invalid combos server-side.

- [ ] M7.1 `list_settings()` grouped by category; `update_settings()` validated by `value_type` + per-key rules (audited)
- [ ] M7.2 Tabs: General · Security · Email · Storage & Backup · API · Localization · Feature Flags
- [ ] M7.3 Master switch at top of each tab + dependency warnings
- [ ] M7.4 Dependency rules enforced in `update_settings` (§17 list)
- [ ] M7.5 Secrets shown only as "configured ✓ / missing ✗"
- [ ] M7.6 Changing `backup.schedule_*` re-registers the scheduler job
- [ ] M7.11 Migration `feature_flags` (§27; was blueprint `0011_feature_flags`) — do before M7.7
- [ ] M7.7 Feature flags: code registry + startup sync, `is_enabled` with stable bucket hash, org override
- [ ] M7.8 `require_flag(&ctx, …)` + `FeatureGate` component; flags in `current_user()`
- [ ] M7.9 Feature Flags tab: toggle, % slider, role multi-select, beta users, stale-flag list (`flags.view` / `flags.update`)
- [ ] M7.10 Display `tenancy: single|multi` (read-only)

---

## M8 — Realtime + notifications (§11, §23)

**Done when:** audit page updates live in another browser; bell shows unread count live.

- [ ] M8.1 `PgListener` task (auto-reconnect), debounce ~500 ms, coalesce by table → `broadcast` hub
- [ ] M8.2 `GET /api/stream` SSE (custom Axum route), session check, permission filtering, keep-alive 15 s
- [ ] M8.3 Server closes streams of revoked sessions; events carry `org_id`
- [ ] M8.4 Client `EventSource` hook → refetch resources (refetch once on reconnect)
- [ ] M8.5 Wire live mode: audit, dashboard, backups
- [ ] M8.6 Migration `notifications`: `notifications`, `notification_prefs`
- [ ] M8.7 `notify::send(&mut tx, Target, kind, params)`; targeted SSE event per `user_id`
- [ ] M8.8 Server fns: `unread_count`, `list_notifications`, `mark_read`, `mark_all_read`, prefs, `send_announcement` (`notifications.broadcast`)
- [ ] M8.9 Bell (Rust/UI `Popover`/`DropdownMenu`) with unread badge + latest 10; `/notifications` page
- [ ] M8.10 Retention job (`notifications.retention_days`); hidden when `notifications.enabled = false`

---

## M9 — Profile, sessions, 2FA (§9.1, §21, §22)

**Done when:** a user can enroll TOTP, log in with code or recovery code, and revoke other sessions.

- [ ] M9.1 Profile tabs: Profile · Security · API Tokens · Notifications · Language
- [ ] M9.2 `update_profile(name, avatar)`; `change_password(old, new)` (kills other sessions)
- [ ] M9.3 Sessions: `device_label` from User-Agent; `list_my_sessions`, `revoke_my_session`, `revoke_other_sessions`
- [ ] M9.4 Admin: `admin_list_user_sessions`, `admin_revoke_user_sessions` (`users.revoke_sessions`)
- [ ] M9.5 Daily expired-session purge job
- [ ] M9.6 Migration `user_totp_recovery`: `user_totp`, `recovery_codes`
- [ ] M9.7 TOTP (`totp-rs`), secret AES-256-GCM with HKDF from `APP_SECRET`
- [ ] M9.8 Enroll: QR + manual key → confirm → 10 recovery codes (shown once, downloadable) → revoke other sessions
- [ ] M9.9 Login step 2: `pending_2fa` session (5 min) → `/auth/2fa` → `verify_2fa` (±1 step, replay block, 5 failures → lock)
- [ ] M9.10 Disable / regenerate codes / admin reset (`users.reset_2fa`, re-auth of admin)
- [ ] M9.11 Modes `off|optional|required_roles|required_all` + grace days banner → blocking
- [ ] M9.12 Refuse `required_*` unless acting admin is enrolled
- [ ] M9.13 New-device login → notification (+ email when on)

---

## M10 — Dashboard (§10)

**Done when:** every number matches a hand-run SQL query.

- [ ] M10.1 `get_dashboard(range)` → one `DashboardData`, queries via `tokio::try_join!`, ~5 s cache
- [ ] M10.2 KPI cards with delta + sparkline: total users, active 30d, online now, logins today vs yesterday, failed 24h, last backup
- [ ] M10.3 Chart: logins & failed logins, 14 days (line, `generate_series` gap fill)
- [ ] M10.4 Chart: audit activity by module, 7 days (bar)
- [ ] M10.5 Chart: users by role (donut)
- [ ] M10.6 Chart: new users per week, 12 weeks (bar)
- [ ] M10.7 Recent activity list (last 10 audit rows, live)
- [ ] M10.8 Charts via Rust/UI `charts` (ApexCharts runtime) — confirm SSR/hydration behaviour, else custom SVG (§15.6)
- [ ] M10.9 Dev-only `seed --demo` command (users, logins, failed logins, audit rows spread over 90 days) so charts can be checked against hand-run SQL. Refuses to run when `APP_ENV=production`.

---

## M11 — Storage (local) + avatars + Backup (§9.6.1, §24.1)

**Done when:** backup create → list → download → delete works; retention trims old files.

- [ ] M11.1 `Storage` trait + `local` driver (key validation, no traversal, streaming)
- [ ] M11.2 Migration `files`: `files` table
- [ ] M11.3 Avatar upload: ≤ 1 MB, magic-byte MIME sniff, re-encode to WebP; served via `/files/avatars/:id`
- [ ] M11.4 `list_backups`, `create_backup` (`pg_dump --format=custom`, password via env, sha256, status)
- [ ] M11.5 `GET /api/backups/:id/download` (lookup by id only, streamed, audited)
- [ ] M11.6 `delete_backup`; retention by `backup.retention_count`
- [ ] M11.7 Scheduler (`tokio-cron-scheduler`) from `backup.schedule_*`; actor = system
- [ ] M11.8 Backup page with live status (running → completed) and CSV export (`backup.export`)
- [ ] M11.9 Restrictive file permissions on dumps

---

## M12 — Encryption, off-site, restore (§24.2, §9.6.1)

**Done when:** backup → restore round trip verified, including from off-site copy with only the private key.

- [ ] M12.1 `age` encryption with `BACKUP_PUBLIC_KEY`; symmetric fallback (`backup.encrypt_mode`)
- [ ] M12.2 Migration `backups_encryption_copies`: `backups.encrypted/enc_scheme`, `backup_copies`
- [ ] M12.3 R2/S3 driver (`object_store` or `aws-sdk-s3`), multipart
- [ ] M12.4 Off-site copy job with retries; per-copy badges (Local ✓ · Off-site ✓/⏳/✗); failure notification
- [ ] M12.5 Weekly verification job (`exists` + size, periodic hash)
- [ ] M12.6 Off-site retention (`backup.offsite_retention_count`); "Test off-site connection" button
- [ ] M12.7 **Restore** (last): typed filename confirm → `pre_restore` backup → maintenance flag → `pg_restore --clean --if-exists --no-owner` → migrations → clear sessions/caches → clear flag → re-insert audit entry
- [ ] M12.8 Restore source picker (primary → off-site fallback); private key used in memory only
- [ ] M12.9 Disaster-recovery doc in README (decrypt + restore with no running app)

---

## M13 — Email via Resend (§20)

**Done when:** test email delivered; reset and invite flows work end to end; bounce lands in suppressions.

- [ ] M13.1 Migration `email_outbox`: `email_outbox`, `auth_tokens`, `email_suppressions`
- [ ] M13.2 `email::enqueue(&mut tx, …)` in the business transaction
- [ ] M13.3 Worker: `FOR UPDATE SKIP LOCKED`, back-off 1m/5m/30m/2h, max 5 → failed + admin notification
- [ ] M13.4 Templates (`minijinja`/`askama`) per locale, `.html` + `.txt`, `en` fallback: invite, password_reset, verify_email, new_login_alert, password_changed, 2fa_changed, backup_failed
- [ ] M13.5 Routes: `/auth/forgot-password`, `/auth/reset-password`, `/auth/accept-invite`, `/auth/verify-email` (no enumeration, rate limited)
- [ ] M13.6 `POST /api/webhooks/resend` with Svix signature check → delivered/bounced/complained
- [ ] M13.7 Email tab: key status, Send test email (`email.send_test`), Delivery log (`email.view_log`)
- [ ] M13.8 Non-email fallbacks when `email.enabled = false` (admin sets temp password)

---

## M14 — Multi-instance hardening (§28)

**Done when:** two app instances behind a proxy pass rate-limit, scheduler and maintenance-flag tests.

- [ ] M14.1 Migration `system_state_rate_limits`: `system_state`, `rate_limits`
- [ ] M14.2 Postgres rate limiter (behind `RateLimiter` trait) for login + API
- [ ] M14.3 Maintenance flag in `system_state` (cached ≤ 2 s, NOTIFY refresh); `/readyz` reports it
- [ ] M14.4 Advisory-lock leader election for every scheduled job; backup/restore concurrency locks
- [ ] M14.5 Startup refuses `local` storage when `APP_INSTANCES > 1`
- [ ] M14.6 `/readyz` (DB + not maintenance); graceful shutdown (drain SSE, finish jobs)
- [ ] M14.7 Binary CLI: `serve | migrate | seed | create-admin | reset-2fa <email> | backup-verify`

---

## M15 — Public API v1 *(optional)* (§19)

**Done when:** a script can log in, refresh, and list users with a scoped token; every call audited with `channel='api'`.

- [ ] M15.1 Cargo feature `public-api`; runtime `api.public_enabled` (404 when off)
- [ ] M15.2 Migration `api_tokens_idempotency`: `api_tokens`, `idempotency_keys`
- [ ] M15.3 Mobile auth: access (15 min) + rotating refresh (30 d, `sessions.kind='api_refresh'`), MFA step
- [ ] M15.4 Personal API tokens (shown once, prefix, scopes ∩ owner perms, TTL, max per user)
- [ ] M15.5 RFC 9457 problem+json errors, cursor pagination, `Idempotency-Key`, CORS allow-list, rate limit + `Retry-After`
- [ ] M15.6 Endpoint set from §19 (1:1 with existing services)
- [ ] M15.7 OpenAPI (`utoipa`) at `/api/docs`
- [ ] M15.8 Profile → API Tokens tab; admin view/revoke all

---

## M16 — Multi-tenant mode *(optional)* (§18)

**Done when:** leak tests pass for every service (two orgs, zero cross-reads/writes).

- [ ] M16.1 `TENANCY_MODE=multi` switch; tenant resolver from `Host` → `organizations.slug`
- [ ] M16.2 RLS policies on tenant tables; app connects as non-owner role; `set_config('app.org_id')` per transaction
- [ ] M16.3 Platform Admin vs Org Admin; `/access-control/organizations` page (`organizations.view/manage`)
- [ ] M16.4 CI grep: no tenant query without `org_id`
- [ ] M16.5 Leak tests incl. bulk, export, SSE, notifications, backups listing
- [ ] M16.6 Optional migration `org_limits` (max_users, storage_mb) if selling plans (§18; never numbered in the blueprint)

---

## M17 — CI/CD + deploy (§29, §5)

**Done when:** pushing a tag deploys to the VPS with automatic rollback on failed health check.

- [ ] M17.1 Production multi-stage `Dockerfile` (dx build → `debian:bookworm-slim` + `postgresql-client-17`)
- [ ] M17.2 `docker-compose.prod.yml`: db (no public port), app, caddy (TLS)
- [ ] M17.3 `ci.yml`: fmt, clippy (server + wasm), `sqlx prepare --check`, tests with postgres service, `dx bundle`, `cargo deny`/`audit`, i18n/raw-string/org_id checks
- [ ] M17.4 `docker.yml`: GHCR push on `v*.*.*`, `SQLX_OFFLINE=true`
- [ ] M17.5 `deploy.yml`: environments, SSH, migrate step, health wait, rollback to `PREVIOUS_TAG`
- [ ] M17.6 Dependabot, branch protection, CODEOWNERS
- [ ] M17.7 `docs/DEPLOY.md` VPS runbook (§29.2)
- [ ] M17.8 Verify security headers (from M1.16) are present in the production config
- [ ] M17.9 Google Drive storage driver behind `storage-gdrive` *(optional)*

---

## Risks to verify early (§15, §30.7, §31.4)

Blockers first; each names the item that proves it.

- [x] R3 Session cookie readable in server functions (blocks M1) → proven by **M0.19**
- [x] R4 Custom Axum routes next to Dioxus (SSE, downloads, webhooks) (blocks M8, M11, M13) → proven by **M0.19**
- [x] R5 SSE through Dioxus fullstack (blocks M8) → proven by **M0.19**
- [~] R6 Server-only crates excluded from WASM (`optional` under `server`) → proven by **M0.19** and **M0.24** (M0.19 ✓: no sqlx/figment/tower-http/dotenvy in the wasm tree; M0.24 pending)
- [ ] R7 i18n crate with SSR + hydration (blocks M3) → proven by **M0.25**
- [x] R1 Rust/UI on Dioxus 0.7 compiles on stable — verified (pinned `4f5a516`)
- [x] R2 Rust/UI theme reconciled with dx Tailwind — verified
- [ ] R8 Rust/UI charts hydrate cleanly (ApexCharts is client-side JS)
- [ ] R9 `pg_dump` major version matches DB in app image
- [ ] R10 RLS + non-owner DB role (only if M16)

---

## Decisions & deviations

| Date | Decision | Why |
|---|---|---|
| 2026-11-15 | Dioxus 0.7 fullstack instead of Leptos | Blueprint Revision 3 (§31) |
| 2026-11-15 | Rust/UI components **copied** from local clone `/home/karshe/rust-ui-src` @ `4f5a516`, not `ui add` | ui-cli 0.3.13 + rust-ui.com docs still describe Leptos; §31.1 allows copying |
| 2026-11-15 | Theme copied verbatim from rust-ui `tailwind.css`; iOS bottom-nav pins, `app_crates` source and `highlight_code.css` dropped | upstream-app-only |
| 2026-11-15 | `#main { height: 100%; overflow-y: auto }` injected in `App` | Rust/UI base layer sets `html { overflow: hidden }`; Dioxus mount div breaks the h-full chain (same workaround as upstream) |
| 2026-11-15 | Only blueprint-required Rust/UI components installed (35), not all 87 | keep the starter lean; add more on demand |
| 2026-11-15 | Stable Rust, no nightly | §31.1 (nightly was a Leptos requirement) |
| 2026-11-15 | Risk spikes (M0.19, M0.25) run before M1 and M3 | R3–R7 block later milestones; cheaper to find out first |
| 2026-11-15 | SQLx timestamp migrations instead of fixed 00NN numbers | Migrations are created in milestone order, not numeric order |
| 2026-11-15 | Security headers and `__Host-` cookie move into M1 | Cheap now, easy to forget at M17 |
| 2026-11-15 | Blueprint `0011_feature_flags` was missing from the tracker; added as M7.11 (`org_limits` was never numbered, kept optional in M16.6) | Patch 1, section C check |
| 2026-11-15 | **Spike results (M0.19)** on Dioxus 0.7.10: (a) server fn sets a cookie with `FullstackContext::add_response_header(SET_COOKIE, …)` and reads it with `#[post("/…", headers: HeaderMap)]` ✓ (b) `dioxus::serve` + `dioxus::server::router(App).route(…)` serves plain Axum routes ✓ (c) plain `axum::response::Sse` route streams through the dx dev proxy ✓ (d) `sqlx` only in the server tree ✓ | Unblocks M1 (sessions), M8 (SSE), M11/M13 (download, webhook routes). Server fns can also read middleware-inserted extensions via `FullstackContext::extract::<Extension<T>>()` (used by `Ctx`) |
| 2026-11-15 | `src/ui` + `src/hooks` stay at the crate root, not `components/ui` (§31.2) | Upstream files import `crate::ui::…`/`crate::hooks::…`; moving them means editing vendored code. App components go in `src/components/` |
| 2026-11-15 | Vendored modules get `#[rustfmt::skip]` and `#[allow(dead_code, unused_imports, clippy::all)]` | Keeps them byte-identical to upstream while `just lint` / the hook run with `-D warnings` |
| 2026-11-15 | Migrations run on app boot during development; the separate `starter migrate` step comes in M14.7 | Simplest for M0; §28 wants single-runner migrations for multi-instance |
| 2026-11-15 | `SITE_ADDR` replaced by Dioxus' `IP` / `PORT`; dev DB on host port 5440 | Dioxus reads IP/PORT; 5432 is already used on this machine |
| 2026-11-15 | Removed unused deps `heck`, `gloo-timers`, `time` (came with the Rust/UI copy) | No current file uses them; re-add when a copied component needs one |
| 2026-11-15 | Password minimum length uses the blueprint's existing key `security.password_min_length`, not a new `security.min_password_length` | Avoid two keys for one setting |

## Rust/UI components installed

`alert alert_dialog avatar badge breadcrumb button card charts checkbox collapsible data_table dialog dropdown_menu empty field form input input_group label pagination popover progress select separator sheet sidenav skeleton spinner switch table tabs textarea theme_toggle toast tooltip`
Hooks: `use_can_scroll_vertical use_form use_pagination use_random`
