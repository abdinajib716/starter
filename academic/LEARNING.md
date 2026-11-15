# Learning notes

One entry per finished feature or tracker item, in build order. Each entry quotes the code
**as it was when the item was finished** (commit noted per milestone); later milestones may change
the file, so line numbers refer to that commit. Concepts are defined once in
[CONCEPTS.md](CONCEPTS.md) and linked from every entry that uses them.

---

# Milestone 0 — Foundation (commit `99ff9b4`, branch `m0/foundation`)

## 1. One crate, two binaries: conditional compilation
**Feature:** the same source builds a browser (wasm) client and a native server, each with only the code it needs (M0.7).

**Code:**

`src/main.rs` lines 14–28

```rust
#[cfg(feature = "server")]
mod server;
#[allow(dead_code, unused_imports, clippy::all)]
#[rustfmt::skip]
mod ui;

fn main() {
    #[cfg(not(feature = "server"))]
    dioxus::launch(app::App);

    // `dioxus::serve` binds to IP/PORT (set by `dx` in dev, by the container in prod)
    // and re-invokes the closure on hot-patch; `server::router` initialises state once.
    #[cfg(feature = "server")]
    dioxus::serve(|| async move { server::router().await })
}
```

**Line notes:**

1. L14 — `#[cfg(feature = "server")]` removes the following item from compilation unless the `server` Cargo feature is enabled, so the `server` module never exists in the wasm build.
2. L15 — `mod server;` declares a module whose body lives in `src/server/mod.rs`; the module tree is declared explicitly, not discovered from the file system.
3. L20 — `fn main()` is the program entry point for both binaries; what it does depends on which `cfg` branches survive compilation.
4. L21 — `#[cfg(not(...))]` is the logical complement, so exactly one of the two statements below remains in any given build.
5. L22 — `app::App` is a path: item `App` inside module `app`; a function is passed as a value without calling it.
6. L27 — `|| async move { ... }` is a closure with no parameters that returns an `async` block, i.e. a value that produces a `Future` each time it is called.
7. L27 — `.await` suspends the async block until `server::router()` completes and yields its `Result`.

**Concepts:** [module](CONCEPTS.md#module), [attribute](CONCEPTS.md#attribute), [cfg and Cargo features](CONCEPTS.md#cfg), [closure](CONCEPTS.md#closure), [async/await and Future](CONCEPTS.md#async)

**Try it:** run `cargo check --target wasm32-unknown-unknown`, then temporarily delete line 14 and run it again. Expected: the wasm build fails with unresolved imports such as `sqlx`/`figment`, because `server` now compiles for the browser where those crates are not enabled.

## 2. Root component and route table
**Feature:** the root of the UI, which loads stylesheets/scripts into `<head>` and renders the page for the current URL (M0.8).

**Code:**

`src/app.rs` lines 7–15

```rust
#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[route("/")]
    Home {},
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");
```

`src/app.rs` lines 22–35

```rust
#[component]
pub fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        // Dioxus mounts the app inside <div id="main">; give it full height so
        // Rust/UI's html(100dvh) -> body(h-full) chain keeps working.
        document::Style { "#main {{ height: 100%; overflow-y: auto; }}" }
        document::Stylesheet { href: TAILWIND_CSS }
        document::Stylesheet { href: TOAST_RUNTIME_CSS }
        document::Script { src: APEXCHARTS_JS }
        document::Script { src: CHART_INIT_JS }
        ToastToaster {}
        script { src: TOAST_RUNTIME_JS }
        Router::<Route> {}
```

**Line notes:**

1. L7 — `#[derive(...)]` asks the compiler to generate trait implementations; `Routable` (from Dioxus) generates URL parsing and formatting for the enum.
2. L9 — `pub enum Route` is a sum type: a value is exactly one of the listed variants; here each variant is one page.
3. L10 — `#[route("/")]` binds the URL path `/` to the variant below it; the variant name `Home` must match a component function `Home`.
4. L14 — `const` declares a compile-time constant; `asset!` is a macro that registers the file with the bundler and yields its final (hashed) URL.
5. L22 — `#[component]` turns a function into a Dioxus component, whose parameters become props.
6. L23 — `Element` is the return type of every component: a description of UI to render, or an error.
7. L24 — `rsx!` is a macro with HTML-like syntax that builds that `Element` at compile time.
8. L28 — `{{` and `}}` are escaped braces, because inside `rsx!` string literals `{name}` interpolates a variable.
9. L35 — `Router::<Route> {}` renders the component matching the current URL; `::<Route>` (turbofish) supplies the generic type explicitly.

**Concepts:** [derive](CONCEPTS.md#derive), [enum](CONCEPTS.md#enum), [macro](CONCEPTS.md#macro), [component and Element](CONCEPTS.md#component), [rsx!](CONCEPTS.md#rsx), [generics and turbofish](CONCEPTS.md#generics)

**Try it:** add a variant `#[route("/about")] About {},` and a component `#[component] fn About() -> Element { rsx! { h1 { "About" } } }` (import it in `app.rs`). Expected: `http://127.0.0.1:8080/about` shows "About"; removing the component instead gives a compile error naming `About`.

## 3. Typed configuration with serde
**Feature:** environment variables are parsed into a typed struct, so a missing or malformed setting fails at startup rather than later (M0.11).

**Code:**

`src/server/config.rs` lines 10–26

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AppEnv {
    #[default]
    Development,
    Production,
}

/// No `Debug` derive on purpose: this holds secrets.
#[derive(Clone, Deserialize)]
pub struct Config {
    pub database_url: String,
    #[serde(default)]
    pub app_env: AppEnv,
    pub app_secret: Option<String>,
    #[serde(default)]
    pub cookie_secure: bool,
```

**Line notes:**

1. L10 — deriving `Deserialize` lets serde build an `AppEnv` from text such as `"production"`; `Default` makes `AppEnv::default()` available.
2. L11 — `#[serde(rename_all = "lowercase")]` maps the variant `Production` to the input string `production`.
3. L13 — `#[default]` marks which variant `Default::default()` returns.
4. L18 — `///` is a documentation comment attached to the next item; the comment records a deliberate omission of `Debug`, which prevents secrets from being printed with `{:?}`.
5. L20 — `pub struct Config` is a product type: every value has all the listed fields.
6. L21 — `String` is an owned, growable UTF-8 string; the struct owns its data instead of borrowing it.
7. L22 — `#[serde(default)]` uses `Default::default()` when the key is absent, instead of failing.
8. L24 — `Option<String>` encodes "may be absent" in the type: the value is either `Some(text)` or `None`.

**Concepts:** [struct](CONCEPTS.md#struct), [Option](CONCEPTS.md#option), [serde](CONCEPTS.md#serde), [String vs &str](CONCEPTS.md#string), [derive](CONCEPTS.md#derive)

**Try it:** in `.env` set `APP_ENV=staging` and restart `just dev`. Expected: startup fails with "invalid configuration (see .env.example)" because `staging` is not a variant of `AppEnv`. Restore the value afterwards.

## 4. Loading and validating configuration
**Feature:** reads `.env` and the process environment, then refuses to start production with unsafe settings (M0.11).

**Code:**

`src/server/config.rs` lines 59–69

```rust
impl Config {
    pub fn load() -> anyhow::Result<Self> {
        // `.env` is optional (absent in containers); real env vars win.
        let _ = dotenvy::dotenv();
        let cfg: Config = Figment::new()
            .merge(Env::raw().only(KEYS))
            .extract()
            .context("invalid configuration (see .env.example)")?;
        cfg.validate()?;
        Ok(cfg)
    }
```

`src/server/config.rs` lines 75–87

```rust
    fn validate(&self) -> anyhow::Result<()> {
        if self.is_production() {
            match &self.app_secret {
                Some(s) if s.len() >= 32 => {}
                _ => bail!("APP_SECRET must be set and at least 32 bytes in production"),
            }
            if !self.cookie_secure {
                bail!("COOKIE_SECURE must be true in production");
            }
        } else if self.app_secret.as_deref().is_none_or(|s| s.len() < 32) {
            tracing::warn!("APP_SECRET missing or < 32 bytes; acceptable in development only");
        }
        Ok(())
```

**Line notes:**

1. L59 — `impl Config { ... }` attaches functions to the type; functions without `self` (like `load`) are associated functions called as `Config::load()`.
2. L60 — `anyhow::Result<Self>` is `Result<Config, anyhow::Error>`; `Self` names the type being implemented.
3. L62 — `let _ = ...` evaluates the call and deliberately discards its `Result`, because a missing `.env` file is acceptable.
4. L63 — the chained calls form a builder: each method takes the value and returns a configured one.
5. L66 — `.context(...)` adds a human-readable message to an error, and `?` returns early from `load` with that error if the `Result` is `Err`.
6. L68 — `Ok(cfg)` constructs the success variant of `Result`; the last expression without a semicolon is the function's return value.
7. L75 — `&self` borrows the value immutably: `validate` can read the config but not modify or consume it.
8. L77 — `match &self.app_secret` inspects the `Option` by reference, so the `String` inside is not moved out of `self`.
9. L78 — `Some(s) if s.len() >= 32` is a pattern with a guard: it matches only when the condition also holds.
10. L79 — `bail!(...)` is shorthand for `return Err(anyhow!(...))`.
11. L84 — `as_deref()` turns `Option<String>` into `Option<&str>`, and `is_none_or` tests "absent, or present and failing the predicate".

**Concepts:** [impl block and methods](CONCEPTS.md#impl), [Result and ?](CONCEPTS.md#result), [anyhow and context](CONCEPTS.md#anyhow), [references and borrowing](CONCEPTS.md#borrow), [match and patterns](CONCEPTS.md#match)

**Try it:** in `.env` set `APP_ENV=production` (leave `COOKIE_SECURE` unset) and restart. Expected: startup fails with "APP_SECRET must be set and at least 32 bytes in production". Then add a 32-character `APP_SECRET`; the error changes to the `COOKIE_SECURE` one. Restore `.env`.

## 5. Connection pool and embedded migrations
**Feature:** opens a bounded pool of Postgres connections and applies pending migrations before serving (M0.12).

**Code:**

`src/server/db.rs` lines 9–23

```rust
pub async fn connect(cfg: &Config) -> anyhow::Result<PgPool> {
    PgPoolOptions::new()
        .max_connections(cfg.db_max_connections)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&cfg.database_url)
        .await
        .context("cannot connect to Postgres (is `docker compose up -d db` running?)")
}

pub async fn migrate(pool: &PgPool) -> anyhow::Result<()> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .context("database migration failed")
}
```

**Line notes:**

1. L9 — `pub async fn` declares a function whose body runs as a `Future`; callers must `.await` it to get the `Result`.
2. L9 — `cfg: &Config` borrows the configuration, so the caller keeps ownership.
3. L10 — `PgPoolOptions::new()` starts a builder for a pool that reuses connections across requests instead of opening one per query.
4. L13 — `&cfg.database_url` passes a `&String`, which the function accepts as `&str` through deref coercion.
5. L19 — `sqlx::migrate!("./migrations")` reads the SQL files at compile time and embeds them in the binary, so the server needs no migration files at runtime.

**Concepts:** [async/await and Future](CONCEPTS.md#async), [references and borrowing](CONCEPTS.md#borrow), [connection pool](CONCEPTS.md#pool), [migrations](CONCEPTS.md#migrations), [anyhow and context](CONCEPTS.md#anyhow)

**Try it:** run `just db-down`, then `just dev`. Expected: the server stops at startup with "cannot connect to Postgres (is `docker compose up -d db` running?)". Run `just db-up` and start again.

## 6. Idempotent seed with compile-time-checked SQL
**Feature:** guarantees exactly one default organization, even if two servers start at the same time (M0.14).

**Code:**

`src/server/seed.rs` lines 6–25

```rust
pub async fn ensure_default_org(pool: &PgPool) -> sqlx::Result<Uuid> {
    if let Some(id) = sqlx::query_scalar!("SELECT id FROM organizations WHERE is_default")
        .fetch_optional(pool)
        .await?
    {
        return Ok(id);
    }
    // Racing instances: the partial unique index `one_default_org` lets only one insert win.
    let inserted = sqlx::query_scalar!(
        r#"INSERT INTO organizations (name, slug, is_default)
           VALUES ('Default', 'default', true)
           ON CONFLICT DO NOTHING
           RETURNING id"#
    )
    .fetch_optional(pool)
    .await?;
    match inserted {
        Some(id) => {
            tracing::info!(%id, "created default organization");
            Ok(id)
```

**Line notes:**

1. L6 — `sqlx::Result<Uuid>` is `Result<Uuid, sqlx::Error>`; `Uuid` is the Rust type for a Postgres `uuid` column.
2. L7 — `if let Some(id) = ...` runs the block only when the expression matches the pattern, binding `id` from inside the `Option`.
3. L7 — `query_scalar!` checks the SQL against the database schema at compile time (or against `.sqlx/` offline data) and infers the column's Rust type.
4. L8 — `fetch_optional` returns `Option<T>`: `None` when the query yields no row, which is not treated as an error.
5. L11 — `return Ok(id);` exits the function early from inside the `if let` block.
6. L15 — `r#"..."#` is a raw string literal: backslashes and double quotes inside are taken literally, which suits multi-line SQL.
7. L17 — `ON CONFLICT DO NOTHING` together with the partial unique index makes a concurrent second insert yield no row instead of an error.
8. L22 — `match inserted` must handle both `Some` and `None`; the compiler rejects a `match` that omits a variant (exhaustiveness).
9. L24 — `%id` in `tracing::info!` records the field using its `Display` formatting.

**Concepts:** [if let](CONCEPTS.md#iflet), [match and patterns](CONCEPTS.md#match), [sqlx query macros](CONCEPTS.md#sqlx-macros), [raw strings](CONCEPTS.md#rawstr), [tracing](CONCEPTS.md#tracing)

**Try it:** change `organizations` to `organisation` in the query on line 7 and run `cargo check --no-default-features --features server`. Expected: a compile-time error from `query_scalar!` saying the relation does not exist. This is the database being checked by the compiler.

## 7. Process-wide state initialised once
**Feature:** config, pool and default org id are created once and shared by every request (M0.12, M0.14).

**Code:**

`src/server/state.rs` lines 20–44

```rust
static STATE: OnceCell<AppState> = OnceCell::const_new();

pub async fn init() -> anyhow::Result<&'static AppState> {
    STATE
        .get_or_try_init(|| async {
            let config = Config::load()?;
            let pool = db::connect(&config).await?;
            db::migrate(&pool).await?;
            let default_org_id = seed::ensure_default_org(&pool).await?;
            tracing::info!(tenancy = %config.tenancy_mode, "app state ready");
            Ok(AppState {
                pool,
                config: Arc::new(config),
                default_org_id,
            })
        })
        .await
}

/// For server fns. Always initialised before the router serves requests.
pub fn get() -> Result<&'static AppState, ServerFnError> {
    STATE
        .get()
        .ok_or_else(|| ServerFnError::new("server not initialised"))
}
```

**Line notes:**

1. L20 — `static` declares one value for the whole program; `OnceCell::const_new()` creates it empty at compile time.
2. L22 — `&'static AppState` is a reference valid for the entire program run, which is possible because the value lives in a `static`.
3. L24 — `get_or_try_init` runs the async closure only on first use; later calls (including hot-reload re-runs of the router) return the stored value.
4. L25 — `?` inside the async block propagates the error out of the block, so a failed init leaves the cell empty and is reported to the caller.
5. L32 — `Arc::new(config)` puts the config behind an atomically reference-counted pointer, so cloning `AppState` copies a pointer, not the data.
6. L40 — `Result<&'static AppState, ServerFnError>` uses the error type that server functions return to the client.
7. L43 — `ok_or_else` converts `Option<T>` into `Result<T, E>`, building the error lazily only in the `None` case.

**Concepts:** [static and lifetimes](CONCEPTS.md#static), [OnceCell](CONCEPTS.md#oncecell), [Arc and Clone](CONCEPTS.md#arc), [Result and ?](CONCEPTS.md#result), [Option](CONCEPTS.md#option)

**Try it:** add `tracing::info!("init called");` as the first line inside the `async { ... }` block, run `just dev`, then save any `.rs` file to trigger a hot reload. Expected: "init called" appears once at startup and not again after the reload.

## 8. Request context middleware
**Feature:** every request carries a `Ctx` (organization, user, request id) that server functions can read (M0.15).

**Code:**

`src/server/ctx.rs` lines 30–37

```rust
impl Ctx {
    /// Read the `Ctx` the middleware attached to the current request (inside a server fn).
    pub async fn current() -> Result<Ctx, ServerFnError> {
        let Extension(ctx) = FullstackContext::extract::<Extension<Ctx>, _>()
            .await
            .map_err(|_| ServerFnError::new("request context missing"))?;
        Ok(ctx)
    }
```

`src/server/ctx.rs` lines 42–58

```rust
pub async fn ctx_layer(
    Extension(state): Extension<AppState>,
    mut req: Request,
    next: Next,
) -> Response {
    let request_id = req
        .headers()
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .map(str::to_owned);
    req.extensions_mut().insert(Ctx {
        org_id: state.default_org_id,
        user: None,
        request_id,
    });
    next.run(req).await
}
```

**Line notes:**

1. L33 — `let Extension(ctx) = ...` destructures the tuple struct `Extension` in a `let` pattern, binding its inner value to `ctx`.
2. L33 — `extract::<Extension<Ctx>, _>()` uses an Axum extractor; `_` lets the compiler infer the second generic parameter.
3. L35 — `map_err` replaces the extractor's error with a `ServerFnError` before `?` propagates it.
4. L43 — `Extension(state): Extension<AppState>` destructures directly in the parameter list; Axum fills it from the request's extensions.
5. L44 — `mut req` makes the binding mutable, which is required to insert into its extensions.
6. L50 — `and_then(|v| v.to_str().ok())` chains a fallible conversion on an `Option`, keeping `None` if any step fails.
7. L51 — `.map(str::to_owned)` passes a function path instead of a closure, converting `&str` into an owned `String`.
8. L52 — `extensions_mut().insert(...)` stores a value keyed by its type on the request, so later handlers retrieve it by type.
9. L57 — `next.run(req).await` hands the modified request to the next layer or handler and returns its response.

**Concepts:** [pattern destructuring](CONCEPTS.md#match), [extractor](CONCEPTS.md#extractor), [middleware and layers](CONCEPTS.md#middleware), [Option combinators](CONCEPTS.md#option), [mutability](CONCEPTS.md#mut)

**Try it:** run `curl -s http://127.0.0.1:8080/api/system/info -H 'x-request-id: my-test-1'` and look at the `just dev` output. Expected: the log line for `/api/system/info` shows `request_id=my-test-1`, because the incoming id is kept instead of a new UUID being generated.

## 9. Assembling the router with layers
**Feature:** combines the Dioxus app, plain routes and cross-cutting layers (request id, tracing, state, context) into one server (M0.16).

**Code:**

`src/server/mod.rs` lines 14–25

```rust
pub async fn router() -> anyhow::Result<Router> {
    let state = state::init().await?.clone();

    let router = dioxus::server::router(crate::app::App)
        .route("/healthz", get(routes::healthz))
        .layer(axum::middleware::from_fn(ctx::ctx_layer))
        .layer(Extension(state))
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(TraceLayer::new_for_http().make_span_with(routes::make_span))
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid));
    Ok(router)
}
```

**Line notes:**

1. L15 — `state::init().await?.clone()` awaits the initialiser, propagates any error, then clones the `&'static AppState` into an owned value.
2. L17 — `dioxus::server::router(crate::app::App)` builds an Axum `Router` that already serves SSR pages, server functions and assets.
3. L18 — `.route("/healthz", get(...))` adds a plain handler for `GET /healthz` beside the Dioxus routes.
4. L19 — `.layer(...)` wraps every route registered so far; the layer added last runs first on an incoming request.
5. L20 — `Extension(state)` is a layer that inserts `state` into every request, which is why the `ctx_layer` added before it can read it.

**Concepts:** [middleware and layers](CONCEPTS.md#middleware), [Result and ?](CONCEPTS.md#result), [Arc and Clone](CONCEPTS.md#arc)

**Try it:** swap lines 19 and 20 and request `/healthz`. Expected: every request now fails with HTTP 500 "Missing request extension", because `ctx_layer` runs before the `Extension` layer has inserted `AppState`. Swap them back.

## 10. Server function consumed by a component
**Feature:** the home page shows live server status, fetched during SSR and reused by the browser (M0.15).

**Code:**

`src/api/system.rs` lines 6–11

```rust
#[get("/api/system/info")]
pub async fn server_info() -> Result<ServerInfo, ServerFnError> {
    let ctx = crate::server::ctx::Ctx::current().await?;
    let state = crate::server::state::get()?;
    Ok(state.server_info(&ctx).await)
}
```

`src/pages/home.rs` lines 13–15

```rust
pub fn Home() -> Element {
    let mut count = use_signal(|| 0);
    let info = use_server_future(server_info)?;
```

`src/pages/home.rs` lines 34–44

```rust
                    match &*info.read() {
                        Some(Ok(i)) => rsx! {
                            div { class: "flex flex-wrap gap-2 text-sm",
                                Badge { variant: BadgeVariant::Secondary, "v{i.version}" }
                                Badge { variant: BadgeVariant::Secondary, "tenancy: {i.tenancy}" }
                                Badge { variant: BadgeVariant::Secondary, "db: {i.db}" }
                            }
                        },
                        Some(Err(e)) => rsx! { p { class: "text-destructive text-sm", "server: {e}" } },
                        None => rsx! { p { class: "text-muted-foreground text-sm", "loading…" } },
                    }
```

**Line notes:**

1. L6 (system.rs) — `#[get("/api/system/info")]` makes the function a server function: its body is compiled only into the server, while the client gets a generated function that performs `GET /api/system/info`.
2. L7 — the return type must be serialisable, because the value crosses the network as JSON.
3. L14 (home.rs) — `use_signal(|| 0)` creates reactive local state; writing to it re-renders the component.
4. L15 — `use_server_future(server_info)?` runs the server function during server rendering and sends the result with the HTML, so hydration starts with the same data; `?` suspends the component while the value is pending.
5. L34 — `&*info.read()` borrows the current value of the resource: `read()` returns a guard, `*` dereferences it, `&` re-borrows it.
6. L35 — `Some(Ok(i))` is a nested pattern over `Option<Result<ServerInfo, ServerFnError>>`.
7. L37 — `"v{i.version}"` interpolates a field inside an `rsx!` string.

**Concepts:** [server function](CONCEPTS.md#serverfn), [signals and hooks](CONCEPTS.md#signals), [SSR and hydration](CONCEPTS.md#hydration), [match and patterns](CONCEPTS.md#match), [serde](CONCEPTS.md#serde)

**Try it:** stop the database (`just db-down`) while `just dev` is running and reload the page. Expected: the badge changes to `db: down` (the server function still answers, but `db_ok` fails). `just db-up` restores `db: ok`.

## 11. Health endpoint
**Feature:** `GET /healthz` returns 200 with status JSON, or 503 when the database is unreachable (M0.17).

**Code:**

`src/server/routes.rs` lines 11–27

```rust
pub async fn healthz(Extension(state): Extension<AppState>) -> impl IntoResponse {
    let db = state.db_ok().await;
    let code = if db {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    (
        code,
        Json(json!({
            "status": if db { "ok" } else { "degraded" },
            "db": if db { "ok" } else { "down" },
            "tenancy": state.config.tenancy_mode,
            "version": env!("CARGO_PKG_VERSION"),
        })),
    )
}
```

**Line notes:**

1. L11 — `-> impl IntoResponse` returns some type implementing `IntoResponse` without naming it; Axum converts it into an HTTP response.
2. L13 — `if` is an expression in Rust, so its value (a status code) can be bound to `code`.
3. L18 — returning a tuple `(StatusCode, Json<...>)` sets the status from the first element and the body from the second, because Axum implements `IntoResponse` for such tuples.
4. L20 — `json!` builds a `serde_json::Value` with JSON syntax checked at compile time.
5. L24 — `env!("CARGO_PKG_VERSION")` reads an environment variable at compile time; Cargo sets this one from `Cargo.toml`.

**Concepts:** [impl Trait](CONCEPTS.md#impltrait), [expressions vs statements](CONCEPTS.md#expr), [macro](CONCEPTS.md#macro), [traits](CONCEPTS.md#trait)

**Try it:** run `curl -i http://127.0.0.1:8080/healthz`, then `just db-down` and run it again. Expected: `HTTP/1.1 200 OK` with `"db":"ok"`, then `503 Service Unavailable` with `"status":"degraded"`.

## 12. Database tests with isolated databases
**Feature:** each test runs against a fresh, migrated database, so tests are independent of each other and of dev data (M0.20, M4.8).

**Code:**

`src/server/seed.rs` lines 35–49

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[sqlx::test(migrations = "./migrations")]
    async fn default_org_is_created_once(pool: PgPool) -> sqlx::Result<()> {
        let first = ensure_default_org(&pool).await?;
        let second = ensure_default_org(&pool).await?;
        assert_eq!(first, second);
        let n: i64 = sqlx::query_scalar("SELECT count(*) FROM organizations WHERE is_default")
            .fetch_one(&pool)
            .await?;
        assert_eq!(n, 1);
        Ok(())
    }
```

**Line notes:**

1. L35 — `#[cfg(test)]` compiles the module only for `cargo test`, so test code is absent from the real binary.
2. L37 — `use super::*;` imports everything from the parent module, including private items, which a child module may access.
3. L39 — `#[sqlx::test(migrations = "./migrations")]` creates a new database, applies the migrations and passes a pool connected to it as the `pool` argument.
4. L40 — a test may return `Result`; returning `Err` (for example through `?`) fails the test.
5. L43 — `assert_eq!(a, b)` panics with both values printed when they differ, which fails the test.
6. L44 — `let n: i64` annotates the type that the non-macro `query_scalar` must decode into, since it cannot inspect the SQL at compile time.

**Concepts:** [unit tests and cfg(test)](CONCEPTS.md#tests), [sqlx query macros](CONCEPTS.md#sqlx-macros), [Result and ?](CONCEPTS.md#result), [module](CONCEPTS.md#module)

**Try it:** change line 47 to `assert_eq!(n, 2);` and run `just test`. Expected: `default_org_is_created_once` fails with `left: 1, right: 2`. Restore it.

---

# Milestone 0 — i18n spike, M0.25 (commit `d838bcd`)

## 13. Translation catalogue and bundle configuration
**Feature:** UI text lives in per-language Fluent files; the app embeds them and falls back to English for any missing key (M0.25).

**Code:**

`locales/en.ftl` lines 2–7

```
home-title = Rust/UI is working
home-subtitle = Dioxus 0.7 + Tailwind v4 theme
home-clicks = { $count ->
    [one] Clicked once
   *[other] Clicked { $count } times
}
```

`src/i18n.rs` lines 16–22

```rust
pub fn config(locale: &str) -> I18nConfig {
    let selected: LanguageIdentifier = locale.parse().unwrap_or(langid!("en"));
    I18nConfig::new(selected)
        .with_locale((langid!("en"), include_str!("../locales/en.ftl")))
        .with_locale((langid!("so"), include_str!("../locales/so.ftl")))
        .with_fallback(langid!("en"))
}
```

**Line notes:**

1. L4 (en.ftl) — `{ $count -> ... }` is a Fluent selector: the branch is chosen by the plural category of `$count` for the active language (`one`, `other`; other languages have more).
2. L6 (en.ftl) — `*[other]` marks the default branch, which Fluent requires in every selector.
3. L17 (i18n.rs) — `locale.parse()` converts text into a `LanguageIdentifier` through the `FromStr` trait; the annotation on `selected` tells `parse` which type to produce.
4. L17 — `unwrap_or(langid!("en"))` substitutes English if the text is not a valid language tag; `langid!` validates its literal at compile time.
5. L19 — `include_str!` reads the file at compile time and embeds its contents as a `&'static str`, so the `.ftl` files ship inside both binaries.
6. L19 — `(langid!("en"), include_str!(...))` is a tuple; `with_locale` accepts it because the crate implements `From<(LanguageIdentifier, T)>` for `Locale`.
7. L21 — `with_fallback` loads the English resources first, so the selected language overrides only the keys it defines.

**Concepts:** [Fluent](CONCEPTS.md#fluent), [traits](CONCEPTS.md#trait), [From/Into and parse](CONCEPTS.md#conversion), [macro](CONCEPTS.md#macro)

**Try it:** remove the existing `home-title` line from `so.ftl` instead and reload with Somali selected. Expected: the title shows the English "Rust/UI is working", because the key is missing in `so` and `en` is the fallback.

## 14. Resolving the request locale
**Feature:** chooses the language from the user setting, cookie or browser preference, in that order, limited to enabled locales (M0.25, §26).

**Code:**

`src/i18n.rs` lines 47–70

```rust
    let pick = |tag: &str| {
        let p = primary(tag);
        enabled.contains(&p.as_str()).then_some(p)
    };
    if let Some(found) = user.and_then(pick).or_else(|| cookie.and_then(pick)) {
        return found;
    }
    if let Some(header) = accept_language {
        // "so-SO,so;q=0.9,en;q=0.8" → highest q first; q=0 means "not acceptable".
        let mut langs: Vec<(f32, &str)> = header
            .split(',')
            .filter_map(|part| {
                let mut it = part.trim().split(';');
                let tag = it.next()?.trim();
                let q = it
                    .find_map(|p| p.trim().strip_prefix("q="))
                    .and_then(|q| q.parse().ok())
                    .unwrap_or(1.0);
                (!tag.is_empty() && q > 0.0).then_some((q, tag))
            })
            .collect();
        langs.sort_by(|a, b| b.0.total_cmp(&a.0));
        if let Some(found) = langs.into_iter().find_map(|(_, tag)| pick(tag)) {
            return found;
```

**Line notes:**

1. L47 — `let pick = |tag: &str| { ... }` stores a closure in a variable; it captures `enabled` by reference from the surrounding function.
2. L49 — `bool::then_some(p)` yields `Some(p)` when the condition is true and `None` otherwise.
3. L51 — `user.and_then(pick).or_else(|| cookie.and_then(pick))` tries the user setting first and evaluates the cookie only if that produced `None`.
4. L56 — `Vec<(f32, &str)>` is a growable array of tuples; the `&str` items borrow from `header`, so no strings are copied.
5. L57 — `.split(',')` returns an iterator, a lazy sequence that produces items only when consumed.
6. L58 — `filter_map` applies a closure returning `Option` and keeps only the `Some` values.
7. L60 — `it.next()?` uses `?` on an `Option` inside the closure: a missing item makes the closure return `None`.
8. L67 — `.collect()` consumes the iterator and builds the collection named in the type annotation (`Vec<...>`).
9. L68 — `sort_by(|a, b| b.0.total_cmp(&a.0))` sorts in place by the first tuple field, descending; `total_cmp` gives floats a total order.
10. L69 — `into_iter()` consumes the vector, and `find_map` returns the first `Some` produced by `pick`.

**Concepts:** [closure](CONCEPTS.md#closure), [iterators](CONCEPTS.md#iterator), [Option combinators](CONCEPTS.md#option), [Vec and tuples](CONCEPTS.md#vec), [references and borrowing](CONCEPTS.md#borrow)

**Try it:** add `assert_eq!(resolve(None, None, Some("so;q=0.2, en;q=0.9"), ENABLED, "en"), "en");` to the test `accept_language_respects_q_and_enabled_list` and run `just test`. It passes. Then change the expected value to `"so"`. Expected: the test fails, because the higher `q` value (`en`, 0.9) wins.

## 15. Same language on server and client
**Feature:** the locale chosen during server rendering is reused by the browser, so hydration starts in the same language with no flash (M0.25).

**Code:**

`src/app.rs` lines 25–31

```rust
#[component]
pub fn App() -> Element {
    // Resolved on the server during SSR; the client reuses that value when hydrating.
    let initial = use_server_cached(i18n::request_locale);
    use_init_i18n(|| i18n::config(&initial));

    rsx! {
```

`src/app.rs` lines 46–56

```rust
/// Sets `lang`/`dir` for the current locale. A separate component so that a language
/// switch re-renders only this subtree, not `App`'s `<head>` elements.
#[component]
fn LocaleRoot(children: Element) -> Element {
    let lang = dioxus_i18n::prelude::i18n().language().to_string();
    let dir = if i18n::is_rtl(&lang) { "rtl" } else { "ltr" };
    // Tailwind `rtl:` variants and logical utilities key off the nearest `dir`.
    rsx! {
        div { class: "h-full", lang: "{lang}", dir: "{dir}", {children} }
    }
}
```

**Line notes:**

1. L28 — `use_server_cached(i18n::request_locale)` runs the function on the server during SSR, serialises the result into the page, and on the client returns that stored value instead of calling the function.
2. L29 — `use_init_i18n` provides the i18n state as context to every descendant component; children obtain it with `i18n()` and `t!`.
3. L42 — `LocaleRoot { Router::<Route> {} }` passes the router as the `children` prop of `LocaleRoot`.
4. L49 — `children: Element` is the conventional prop name for nested content written inside a component's braces.
5. L50 — `i18n().language()` reads a signal, so `LocaleRoot` (and only it) re-renders when the language changes.
6. L54 — `{children}` places the received elements inside the `div`.

**Concepts:** [SSR and hydration](CONCEPTS.md#hydration), [context](CONCEPTS.md#context), [signals and hooks](CONCEPTS.md#signals), [component and Element](CONCEPTS.md#component)

**Try it:** move the two lines `let lang = ...` and `let dir = ...` from `LocaleRoot` into `App`, use them on a `div` there, and switch language in the browser while `just dev` runs. Expected: the dev server log prints the warning "Changing the props of `Style {}` is not supported", because `App` now re-renders its `<head>` elements on every switch. Revert afterwards.

## 16. Switching language and persisting it in a cookie
**Feature:** the language buttons change the UI immediately and store the choice in a cookie so the next server render uses it (M0.25).

**Code:**

`src/pages/home.rs` lines 52–63

```rust
                        for code in crate::i18n::SUPPORTED.iter().copied() {
                            Button {
                                id: "lang-{code}",
                                size: ButtonSize::Sm,
                                variant: if code == current { ButtonVariant::Default } else { ButtonVariant::Outline },
                                onclick: move |_| async move {
                                    i18n.set_language(code.parse().expect("valid locale"));
                                    let _ = set_locale(code.to_string()).await;
                                },
                                "{code}"
                            }
                        }
```

`src/api/i18n.rs` lines 11–19

```rust
    let ctx = FullstackContext::current().ok_or_else(|| ServerFnError::new("no request"))?;
    let cookie = format!(
        "{}={locale}; Path=/; Max-Age=31536000; SameSite=Lax",
        crate::i18n::LOCALE_COOKIE
    );
    ctx.add_response_header(
        SET_COOKIE,
        HeaderValue::from_str(&cookie).map_err(|e| ServerFnError::new(e.to_string()))?,
    );
```

**Line notes:**

1. L52 (home.rs) — `for` inside `rsx!` renders one element per item; `.copied()` turns the iterator's `&&str` items into `&str`.
2. L57 — `move |_| async move { ... }` is an event handler returning a future; Dioxus spawns it, so the handler may `.await`.
3. L58 — `set_language` writes the i18n signal, which re-renders every component that translated text.
4. L59 — `let _ = set_locale(...).await;` calls the server function over HTTP and deliberately ignores a failure, since the switch already happened locally.
5. L11 (i18n.rs) — `FullstackContext::current()` gives the server function access to the current request and response.
6. L12 — `format!` builds a `String`; `{locale}` captures the variable of that name directly (inline format arguments).
7. L16 — `add_response_header(SET_COOKIE, ...)` adds a `Set-Cookie` header, so the browser stores `locale=...` and sends it with later requests.

**Concepts:** [rsx!](CONCEPTS.md#rsx), [async/await and Future](CONCEPTS.md#async), [server function](CONCEPTS.md#serverfn), [cookies](CONCEPTS.md#cookie), [signals and hooks](CONCEPTS.md#signals)

**Try it:** run `curl -si -X POST http://127.0.0.1:8080/api/i18n/locale -H 'content-type: application/json' -d '{"locale":"so"}' | grep -i set-cookie`, then the same with `"fr"`. Expected: the first prints `set-cookie: locale=so; Path=/; Max-Age=31536000; SameSite=Lax`; the second returns an error body containing "unsupported locale" and no cookie.
