# Starter — integration guide

**Read this before building a new system on this starter.** It explains what already exists,
where it lives, and the exact steps to add your system's own features, so nothing is rebuilt
from scratch. Give this file (plus `TRACKER.md`) to any developer or AI agent as their context.

---

## 1. What the starter is

A production-ready admin application in **one Rust codebase**: the browser UI and the server are
both Rust, rendered on the server first (SSR) and then made interactive in the browser.

| Layer | Technology |
|---|---|
| UI + server | Dioxus 0.7 fullstack (Rust → WebAssembly in the browser, Axum on the server) |
| Components | Rust/UI (vendored, `src/ui`) + shadcn-admin look ported to Dioxus (`src/components`) |
| Styling | Tailwind CSS v4 (`tailwind.css` → `assets/tailwind.css`) |
| Database | PostgreSQL 17 (Docker) via SQLx (compile-time checked SQL, `migrations/`) |
| PDF | Headless Chrome in Docker (`chrome` service), driven over the DevTools protocol |
| Languages | Fluent files `locales/en.ftl`, `locales/so.ftl` (`dioxus-i18n`) |

Every new system = **this starter + your own modules**. You add tables, pages and permissions;
you do not rewrite login, users, roles, audit, settings, exports, backups or the layout.

---

## 2. Already built — do NOT rebuild

| Area | What works | Where |
|---|---|---|
| Sign in / out | Argon2id passwords, DB sessions (HttpOnly cookie), lockout after failed attempts, rate limit, forced password change, CSRF check | `src/server/auth/`, `src/server/services/auth.rs`, `src/pages/auth/` |
| Session ended | Any `401` from the server → reload to `/sign-in?reason=session` | `src/util/mod.rs` (`toast_error`) |
| Permissions (RBAC) | 37 permission keys in code, synced to DB; Super Admin = all; guards: last Super Admin, not-self, no escalation | `src/server/rbac.rs` |
| Sidebar | Dashboard + Access Control (System Setting, User, Role, Permission, Audit, Backup), filtered by permission | `src/components/layout/nav_config.rs` |
| Dashboard | Real KPIs + charts (sign-ins, activity by module, users by role, new users/week), recent activity | `src/server/services/dashboard.rs`, `src/pages/dashboard/` |
| Users | Server-side list (search, filters, sort, paging in URL), create/edit, activate/deactivate, soft delete, Deleted tab + restore + purge, bulk actions, details page `/users/:id` | `src/server/services/users.rs`, `src/pages/users/` |
| Roles | Role editor (permission grid), require-2FA flag, delete only unused, bulk delete; changing permissions signs those users out | `src/server/services/roles.rs`, `src/pages/roles/` |
| Permission catalog | Read-only list by module with holding roles | `src/pages/permissions.rs` |
| Audit log | Every change written in the same transaction; secrets masked; filters, date range, keyset paging, before/after diff | `src/server/audit.rs`, `src/server/services/audit_log.rs`, `src/pages/audit.rs` |
| System Setting | 38 typed settings in tabs, validation + dependency rules, cache refreshed on every instance (`pg_notify`) | `src/server/settings.rs`, `src/server/services/system_settings.rs`, `src/pages/system_settings.rs` |
| Feature flags | Code registry, % rollout (stable per user), role/beta targeting, `FeatureGate` / `require_flag` | `src/server/flags.rs`, `src/components/feature_gate.rs` |
| Module switches | `Modules` (email, 2FA mode, API, notifications, i18n) delivered to the UI as `ModuleFlags` | `src/server/modules.rs` |
| Exports | CSV (streamed, injection-safe, row cap) and PDF (headless Chrome, ~200 ms) for users, audit, roles, backups | `src/server/export.rs`, `src/server/pdf.rs`, `src/server/services/reports.rs`, `src/components/export_menu.rs` |
| Backups | `pg_dump` create, download, delete, retention, cron schedule (one leader across instances) | `src/server/services/backups.rs`, `src/pages/backups.rs` |
| Storage | `Storage` trait + local driver (traversal-safe, 0600 files) | `src/server/storage.rs` |
| Time zones | Stored UTC, shown in the user's zone (own or organization default) | `src/util/time.rs` |
| Profile | Own name, time zone, password, theme, font | `src/pages/profile.rs` |
| UI kit | Data table kit, dialogs, sheets, menus, forms, date picker, charts, skeletons, toasts, permission/feature gates | `src/components/` |
| Errors | One `AppError` → stable codes → translated messages | `src/server/error.rs`, `locales/en.ftl` (`err-*`) |
| Multi-tenant groundwork | `organizations` table, `org_id` on users/roles/audit, request `Ctx { org_id, user }` | `src/server/ctx.rs` |

**Also count as built** (finished in the starter, not in your system — see §9): 2FA, own
sessions page, notifications + live updates, email, backup restore/encryption/off-site, avatars,
language switching per user, multi-instance hardening, public API, multi-tenant mode, CI/CD +
deploy. Plan your system as if these exist; when they land, your modules plug into them (§9).

---

## 3. Run it

```bash
just db-up                          # Postgres in Docker (127.0.0.1:5440)
docker compose up -d chrome         # PDF service (127.0.0.1:9223)
cp .env.example .env                # then set SEED_ADMIN_EMAIL / SEED_ADMIN_PASSWORD / APP_SECRET
just dev                            # http://127.0.0.1:8080  (migrations + seed run on boot)
```

First start creates the default organization, the Super Admin role and the first admin from
`SEED_ADMIN_*` (must change password at first sign-in).

| Command | Does |
|---|---|
| `just test` | All tests; each `#[sqlx::test]` gets a fresh database |
| `just lint` | fmt + clippy for server and wasm (must be clean) |
| `just prepare` | Refresh `.sqlx/` after changing any SQL (commit it) |
| `just migration name` | New migration file |
| `just hooks` | Pre-commit hook: fmt, clippy ×2, `sqlx prepare --check` |

Gotchas:
- Run **one** `dx serve` only. Two copies fight over `target/` and `assets/tailwind.css`.
- New Tailwind classes in a brand-new file sometimes miss the watcher: run
  `~/.local/share/.dx/tools/tailwindcss-v4.1.5/tailwindcss --input tailwind.css --output assets/tailwind.css`.
- Remote VS Code: forward port 8080 in the Ports panel.
- A broken `.ftl` file (duplicate key, stray `}`) breaks every page — `just test` catches it.

---

## 4. How a request flows

```
Browser (WASM)                       Server (Axum)
page component ──server fn──►  ctx_layer: session cookie → Ctx { org_id, user + permissions }
                               api/<module>.rs   (thin: ctx.require("perm") → service)
                               server/services/<module>.rs
                                  BEGIN → change → audit::record(...) → COMMIT
                               ◄── Result<T, AppError>  (error = stable code, e.g. users.email_taken)
toast / reload list
```

Rules that keep it safe:
1. **Every** server function checks access: `ctx.require("module.action")?`, or `ctx.require_user()?`
   for own-account actions, or is on the public allow-list. A test (`api::convention`) fails if you forget.
2. Every change happens in a transaction **with its audit row** (`audit::record`).
3. Every query is scoped by organization: `WHERE org_id = $ctx.org_id`.
4. The server never sends sentences, only codes (`AppError::Conflict("orders.locked")`);
   the browser shows `err-orders-locked` from `locales/en.ftl`.
5. The UI hides what you may not do (`PermissionGate`), but the server always re-checks.

---

## 5. Folder map

```
src/
├─ main.rs, app.rs            # entry; Route enum (all pages); App (i18n, prefs, toaster)
├─ api/<module>.rs            # #[get]/#[post] server functions (thin)
├─ domain/<module>.rs         # DTOs + validation shared by server and browser
├─ server/                    # server-only (feature "server")
│  ├─ ctx.rs  rbac.rs  error.rs  audit.rs  settings.rs  modules.rs  flags.rs
│  ├─ export.rs  pdf.rs  storage.rs  state.rs  seed.rs  config.rs
│  ├─ auth/                   # password, session, cookie, guard (page redirects + CSRF)
│  └─ services/<module>.rs    # business logic + tests (#[sqlx::test])
├─ pages/<module>/            # one folder per page (mod.rs = page, table.rs, dialogs.rs, actions.rs)
├─ components/
│  ├─ layout/                 # app shell, sidebar, header, nav_config.rs (menu tree)
│  ├─ data_table/             # TableState (URL sync), toolbar, filters, pagination, bulk bar
│  ├─ ui_extra/               # form inputs, dialog, sheet, menu, chart, icon, sidebar primitives
│  └─ permission_gate.rs feature_gate.rs export_menu.rs skeletons.rs long_text.rs
├─ ui/                        # vendored Rust/UI (button, card, avatar, tabs, toast, …)
├─ util/                      # toast, download, time zones, cookies, query string
└─ i18n.rs                    # locales, ta! macro (t! with arguments)
migrations/                   # SQL, applied on boot
locales/en.ftl so.ftl         # every visible string
academic/                     # learning notes (LEARNING.md, CONCEPTS.md)
```

---

## 6. Adding a feature module — the recipe

Example: an **Orders** module for the new system. Follow the Users module as the reference
implementation for every step.

1. **Migration** — `just migration orders`:
   ```sql
   CREATE TABLE orders (
     id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
     org_id uuid NOT NULL REFERENCES organizations(id),
     number text NOT NULL, customer text NOT NULL, total_cents bigint NOT NULL,
     status text NOT NULL DEFAULT 'draft',
     created_by uuid REFERENCES users(id) ON DELETE SET NULL,
     created_at timestamptz NOT NULL DEFAULT now(), updated_at timestamptz NOT NULL DEFAULT now(),
     deleted_at timestamptz                     -- only if you want soft delete + Deleted tab
   );
   CREATE UNIQUE INDEX orders_number ON orders(org_id, number) WHERE deleted_at IS NULL;
   ```
2. **Permissions** — add to `PERMISSIONS` in `src/server/rbac.rs`:
   `("orders.view", …), ("orders.create", …), ("orders.update", …), ("orders.delete", …), ("orders.export", …)`.
   They appear automatically in the Role editor and the Permission page; Super Admin has them.
3. **DTOs** — `src/domain/orders.rs`: `OrderRow`, `OrderQuery` (q, filters, sort, page, page_size),
   `OrderPage { rows, total, … }`, `OrderInput` + `validate_input()` (returns error codes).
4. **Service** — `src/server/services/orders.rs`: `list` (QueryBuilder + bound values, copy
   `push_base`/`push_filters` from users), `create`/`update`/`delete` each:
   `ctx.require(..)` → `pool.begin()` → change → `audit::record(&mut tx, Entry { action: "order.create", before, after, .. })` → `commit`.
   Add `#[sqlx::test]` tests next to it.
5. **Server functions** — `src/api/orders.rs`: one thin function per action, starting with
   `ctx.require("orders.…")?`. Register the module in `src/api/mod.rs`.
6. **Page** — `src/pages/orders/` copying `src/pages/users/`:
   `TableState::use_from_query(&query, "filter", &["status"])` → build `OrderQuery` → `use_resource`
   calling `list_orders` → `Toolbar` + table + `Pagination` + `BulkActions` + dialogs.
   Route in `src/app.rs`: `#[route("/orders?:..query")] Orders { query: Query }`.
7. **Sidebar** — add to `nav_groups()` in `src/components/layout/nav_config.rs`
   (new group or inside Access Control) with `permission: "orders.view"`.
8. **Texts** — all labels in `locales/en.ftl` (`orders-*`), error codes as `err-orders-*`,
   audit labels as `audit-order-create`. Use `t!("key")`, or `crate::ta!("key", n: 3)` with arguments.
9. **Exports** — CSV: copy `export_route` from users (route in `src/server/mod.rs`).
   PDF: add a function in `src/server/services/reports.rs` (columns + rows) and a route; put
   `ExportMenu` on the page.
10. **Dashboard** — add a query to `load()` in `src/server/services/dashboard.rs` and a `Kpi`
    card in `src/pages/dashboard/overview.rs`.
11. **Settings / switches** (optional) — new setting: one line in `settings::REGISTRY`
    (type + default + range); it appears in System Setting automatically. New gradual feature:
    one line in `flags::REGISTRY`, then `FeatureGate { flag: "orders_v2" }` / `require_flag`.
12. **Finish** — `just prepare`, `just lint`, `just test`, commit.

---

## 7. Building blocks to reuse (with the call to use)

| Need | Use |
|---|---|
| Who is signed in (server) | `let ctx = Ctx::current().await?; let user = ctx.require("x.y")?;` |
| Who is signed in (page) | `use_current_user()` → `UserInfo { permissions, modules, flags, timezone, … }` |
| Hide a button | `PermissionGate { perm: "orders.create", Button {…} }` |
| Error to the user | return `AppError::Validation("orders.number_required")`; page: `toast_error(&e)` |
| Success message | `toast(ToastType::Success, t!("orders-created"))` |
| Confirm a dangerous action | `ConfirmDialog` (`title_destructive`, `body`, `disabled`) |
| Form fields | `TextInput`, `TextArea`, `SelectBox`, `CheckBox`, `Switch`, `Radio`, `DatePicker` (`components/ui_extra/form.rs`) |
| Table with URL-synced filters | `TableState`, `Toolbar`, `FacetedFilter`, `ColumnHeader`, `Pagination`, `BulkActions` |
| Loading state | `TableSkeleton`, `CardsSkeleton`, `FormSkeleton` |
| Dates in the user's zone | `crate::util::time::datetime(t, &use_tz())` |
| Read a setting (server) | `settings::value(pool, "orders.max_lines").await` |
| Store a file | `storage::Local` (`put_file`, `path`, `delete`) |
| PDF of any HTML | `server::pdf::render(chrome_url, html, &PrintOptions{…})` |
| Bulk action with per-item rules | pattern in `services::users::bulk` (skips with a reason, one summary audit row) |

---

## 8. What to skip when planning the new system

Do not plan or estimate these: they exist and are tested.

- Login, logout, sessions, lockout, forced password change, sign-out on password/role change.
- User, Role, Permission, Audit, System Setting, Backup pages and their server logic.
- Permission checks, audit trail, error codes and translations, CSV and PDF export, bulk actions,
  soft delete + restore, data tables, filters in URL, dialogs, toasts, loading skeletons.
- Dashboard frame (add your KPIs only), sidebar, header, theme/dark mode, RTL, fonts, time zones.
- Settings framework, feature flags, module switches, backups + schedule.
- Everything in §9: 2FA, sessions page, notifications + live updates, email (reset, invites,
  verification), backup restore/encryption/off-site, avatars and file uploads, per-user
  language, rate limit + maintenance mode, public API, multi-tenant mode, CI/CD + deploy.

Plan only: **your domain tables, their permissions, pages, KPIs, reports, and any settings or
flags they need** (§6).

---

## 9. Starter features finishing in the starter — count them as built

These belong to the starter, not to your system. Never build them inside your system's modules;
they are finished in the starter (tracked in `TRACKER.md`) and your code only calls them.
Until a milestone lands, its file paths below do not exist yet, so leave a `// TODO(M8)` at the
call site instead of writing your own version.

| Milestone | Provides | How your modules will use it |
|---|---|---|
| M9 | Own sessions page, 2FA (TOTP + recovery codes), 2FA required per role | Nothing to do; set "require 2FA" on sensitive roles |
| M8 | Notifications bell, live updates (SSE) | `notify(user_ids, kind, payload)` after a change; pages refresh live |
| M13 | Email (Resend) with outbox: forgot password, invites, verification | `email::queue(tx, to, template, data)` inside your transaction |
| M12 | Backup restore, encryption, off-site copies (R2 / Drive) | Nothing to do; your tables are in every backup |
| M11 | Files table + uploads, avatars | `files::upload(...)` returns a file id to store on your row |
| M3 | Per-user language (Somali / English) | Keep every text in `locales/*.ftl` — already required |
| M5 | "Select all matching" across pages, auto-purge of deleted rows | Same `TableState` / bulk pattern picks it up |
| M14 | Rate limit + maintenance mode in Postgres, `/readyz` | Nothing to do; works across several app servers |
| M15 | Public API v1 (tokens, OpenAPI) | Expose a module by adding its routes under `/api/v1` |
| M16 | Multi-tenant mode (subdomains, row-level security) | Works if every table has `org_id` and every query filters on it (§4 rule 3) |
| M17 | Production Docker, CI, VPS deploy with rollback | Your migrations and code ship with the same pipeline |

---

## 10. Brief for the new system (copy into its first prompt)

> We build **<system name>** on the existing Rust starter at `/var/www/startup` (Dioxus 0.7
> fullstack, SQLx/Postgres, Rust/UI, Tailwind). Read `docs/STARTER_GUIDE.md` and `TRACKER.md`
> first. Auth, users, roles, permissions, audit, settings, feature flags, exports (CSV/PDF),
> backups, dashboard frame and the UI kit already exist — reuse them, do not rebuild.
> 2FA, sessions, notifications, email, uploads, API, multi-tenant and deploy belong to the
> starter (guide §9) — never build them inside our modules; only call them.
> For each new feature follow §6 of the guide: migration with `org_id` → permissions in
> `rbac.rs` → DTOs in `domain/` → service with transaction + `audit::record` + tests →
> thin server functions with `ctx.require` → page copied from `pages/users` → sidebar entry →
> texts in `locales/en.ftl` → `just prepare && just lint && just test`.
> Our modules are: <list your modules>.
