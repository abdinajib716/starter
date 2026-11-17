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

**Try it:** remove the existing `home-title` line from `so.ftl` and reload with Somali selected. Expected: the title shows the English "Rust/UI is working", because the key is missing in `so` and `en` is the fallback.

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

---

# Milestone 1 — Authentication (commit `76ac93d`, branch `m1/auth`)

## 17. Password verification in constant time
**Feature:** checks a password with Argon2id and spends the same time whether the account exists or not, so response times do not reveal which emails are registered (M1.1).

**Code:**

`src/server/auth/password.rs` lines 36–53

```rust
pub async fn verify(password: String, stored: Option<String>) -> bool {
    let known = stored.is_some();
    let hash = stored.unwrap_or_else(|| DUMMY_HASH.clone());
    let ok = tokio::task::spawn_blocking(move || match PasswordHash::new(&hash) {
        Ok(parsed) => hasher()
            .verify_password(password.as_bytes(), &parsed)
            .is_ok(),
        Err(_) => {
            // A corrupt stored hash must not become a fast path: burn the same time.
            let _ = PasswordHash::new(&DUMMY_HASH)
                .map(|d| hasher().verify_password(password.as_bytes(), &d));
            false
        }
    })
    .await
    .unwrap_or(false);
    known && ok
}
```

**Line notes:**

1. L36 — the parameters are owned (`String`, `Option<String>`) because they are moved into another thread below.
2. L38 — `unwrap_or_else(|| DUMMY_HASH.clone())` substitutes a real hash of a random secret when the user was not found, so the expensive computation still runs.
3. L39 — `tokio::task::spawn_blocking` runs CPU-heavy work on a separate thread pool, keeping the async worker threads free to serve other requests.
4. L39 — `move ||` transfers ownership of `hash` and `password` into the closure, which is required because the closure runs on another thread.
5. L45 — `let _ =` discards a value on purpose; the call is kept only for its timing effect.
6. L50 — awaiting a `JoinHandle` yields `Result<T, JoinError>`; `unwrap_or(false)` treats a panicked task as a failed verification.
7. L52 — `known && ok` makes the dummy path always fail even if a guess matched the random secret.

**Concepts:** [threads and spawn_blocking](CONCEPTS.md#blocking), [closure](CONCEPTS.md#closure), [Option combinators](CONCEPTS.md#option), [password hashing](CONCEPTS.md#hashing)

**Try it:** in the test `unknown_user_never_verifies_but_runs_argon2`, add `let t = std::time::Instant::now();` before the `verify` call and `println!("{:?}", t.elapsed());` after it, then run `just test -- --nocapture unknown_user`. Expected: a duration of the same order as `hash_then_verify` (tens to hundreds of milliseconds in a debug build), not microseconds.

## 18. Session tokens: random, and stored only as a hash
**Feature:** the browser receives a random token; the database keeps only its SHA-256, so a leaked database cannot be used to sign in (M1.3).

**Code:**

`src/server/auth/token.rs` lines 1–14

```rust
//! Opaque random tokens; only their SHA-256 is stored (§7 step 6).
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use sha2::{Digest, Sha256};

/// 32 random bytes, base64url without padding (43 chars).
pub fn generate() -> String {
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes).expect("OS RNG available");
    URL_SAFE_NO_PAD.encode(bytes)
}

pub fn hash(token: &str) -> Vec<u8> {
    Sha256::digest(token.as_bytes()).to_vec()
}
```

**Line notes:**

1. L2 — `Engine` is a trait; importing it brings its `encode` method into scope for `URL_SAFE_NO_PAD`.
2. L7 — `[0u8; 32]` is an array of 32 bytes, each initialised to zero; arrays have a fixed length known at compile time.
3. L8 — `getrandom::fill(&mut bytes)` fills the array from the operating system's cryptographic random source; `&mut` lends it for writing.
4. L8 — `.expect("...")` panics with that message on error; it is used where failure means the system cannot run safely at all.
5. L9 — base64url without padding produces 43 characters that are safe in cookies and URLs.
6. L13 — `Sha256::digest(...)` returns a fixed-size array, and `.to_vec()` copies it into a `Vec<u8>` to match the `bytea` column.

**Concepts:** [arrays and slices](CONCEPTS.md#array), [traits](CONCEPTS.md#trait), [password hashing](CONCEPTS.md#hashing), [Result and ?](CONCEPTS.md#result)

**Try it:** sign in, then run `psql "$DATABASE_URL" -c "select encode(token_hash,'hex') from sessions"` and compare with the `sid` cookie in the browser's developer tools. Expected: they differ completely; running `printf '%s' '<cookie value>' | sha256sum` reproduces the stored hex.

## 19. Counting failures and locking the account
**Feature:** after `security.max_failed_attempts` wrong passwords the account is locked for `security.lockout_minutes`, and every failure is audited in the same transaction (M1.2, M1.14).

**Code:**

`src/server/services/auth.rs` lines 108–131

```rust
    if !password_ok {
        // 4. Count the failure; lock at the threshold and start a fresh count after it.
        let attempts = u.failed_attempts + 1;
        let lock = attempts >= s.max_failed_attempts;
        sqlx::query!(
            r#"UPDATE users SET
                 failed_attempts = CASE WHEN $2 THEN 0 ELSE $3 END,
                 locked_until = CASE WHEN $2 THEN now() + make_interval(mins => $4) ELSE locked_until END
               WHERE id = $1"#,
            u.id,
            lock,
            attempts,
            i32::try_from(s.lockout_minutes).unwrap_or(15),
        )
        .execute(&mut *tx)
        .await?;
        audit::record(&mut tx, fail("bad_password")).await?;
        if lock {
            // M1.19: the in-app notification is added with the notifications table (M8.7).
            audit::record(
                &mut tx,
                Entry {
                    action: "auth.account_locked",
                    entity_type: Some("user"),
```

**Line notes:**

1. L110 — `u.failed_attempts + 1` reads a field of the row returned by `query!`; its type (`i32`) was inferred from the `int` column at compile time.
2. L113 — `CASE WHEN $2 THEN ... ELSE ... END` lets one `UPDATE` both count and lock, so the decision is applied atomically.
3. L120 — `i32::try_from(...)` converts a wider integer and fails instead of silently truncating; `unwrap_or(15)` supplies the default.
4. L122 — `&mut *tx` reborrows the transaction as a connection, so the statement runs inside the open transaction.
5. L124 — `fail("bad_password")` calls a closure defined earlier that builds an audit `Entry` with the shared fields filled in.
6. L129 — `Entry { action: ..., ..fail("threshold") }` is struct update syntax: listed fields are set, the rest are copied from the given value.

**Concepts:** [transactions](CONCEPTS.md#transaction), [struct](CONCEPTS.md#struct), [closure](CONCEPTS.md#closure), [sqlx query macros](CONCEPTS.md#sqlx-macros), [conversions](CONCEPTS.md#conversion)

**Try it:** run `just test lockout` and confirm it passes. Then change `attempts >= s.max_failed_attempts` to `attempts > s.max_failed_attempts` on line 111 and run it again. Expected: the test fails because five failures no longer lock the account. Revert.

## 20. Loading the user on every request
**Feature:** the middleware turns the session cookie into a `CurrentUser` placed in the request context, so every server function knows who is calling (M1.5).

**Code:**

`src/server/ctx.rs` lines 89–101

```rust
    let mut user = None;
    if !is_asset(req.uri().path())
        && let Some(raw) = cookie::read(headers, state.config.session_cookie_name())
    {
        let hours = SecuritySettings::load(&state.pool)
            .await
            .unwrap_or_default()
            .session_hours;
        match session::load(&state.pool, raw, hours).await {
            Ok(u) => user = u,
            Err(e) => tracing::error!(error = %e, "session lookup failed"),
        }
    }
```

**Line notes:**

1. L89 — `let mut user = None;` starts with "anonymous"; the type `Option<CurrentUser>` is inferred from the later assignment.
2. L90 — `if cond && let Some(raw) = ...` is a let chain: the pattern is tried only when the condition is true, and both must succeed (Rust 2024 edition).
3. L95 — `unwrap_or_default()` uses `SecuritySettings::default()` if the settings query fails, so a settings problem does not sign everyone out.
4. L97 — `match` on the `Result` separates "no valid session" (`Ok(None)`) from a database failure (`Err`).
5. L99 — `%e` logs the error with its `Display` text while the request continues as anonymous.

**Concepts:** [let chains and if let](CONCEPTS.md#iflet), [middleware and layers](CONCEPTS.md#middleware), [match and patterns](CONCEPTS.md#match), [Default](CONCEPTS.md#derive)

**Try it:** sign in, then in `psql` run `UPDATE users SET is_active = false;` and reload the page. Expected: you are redirected to `/login`, because the session query only accepts active users. Run `UPDATE users SET is_active = true;` afterwards.

## 21. Server-side page guard
**Feature:** before any page renders, anonymous visitors are redirected to `/login?next=…`, users who must change their password to `/profile`, and signed-in users away from `/login` (M1.10, M1.13).

**Code:**

`src/server/auth/guard.rs` lines 87–107

```rust
pub async fn pages(req: Request, next: Next) -> Response {
    let path = req.uri().path().to_owned();
    if req.method() != Method::GET || is_passthrough(&path) {
        return next.run(req).await;
    }
    // Decide first (borrowing the request), then act (moving it).
    let redirect = match req.extensions().get::<Ctx>().and_then(|c| c.user.as_ref()) {
        None if !is_public_page(&path) => Some(if path == "/" {
            "/login".to_string()
        } else {
            format!("/login?next={path}")
        }),
        Some(u) if u.must_change_pw && path != "/profile" => Some("/profile".to_string()),
        Some(_) if path == "/login" => Some("/".to_string()),
        _ => None,
    };
    match redirect {
        Some(to) => Redirect::to(&to).into_response(),
        None => next.run(req).await,
    }
}
```

**Line notes:**

1. L88 — `.to_owned()` copies the path into a `String`, so it no longer borrows `req`.
2. L93 — the `match` borrows `req` only while it computes `redirect`; the borrow ends at the semicolon on line 102.
3. L94 — `None if !is_public_page(&path)` combines a pattern with a guard; unmatched cases fall through to later arms.
4. L94 — `if path == "/" { ... } else { ... }` is an expression whose value becomes the content of `Some(...)`.
5. L105 — `next.run(req)` moves `req`, which compiles only because no borrow of it is still alive (see note 2).

**Concepts:** [references and borrowing](CONCEPTS.md#borrow), [match and patterns](CONCEPTS.md#match), [expressions vs statements](CONCEPTS.md#expr), [middleware and layers](CONCEPTS.md#middleware)

**Try it:** run `curl -si http://127.0.0.1:8080/profile | head -3` without a cookie. Expected: `HTTP/1.1 303 See Other` and `location: /login?next=/profile`.

## 22. Login server function and coded errors
**Feature:** the browser calls `login`; on success the server sets the session cookie, on failure it returns a stable error code that the client translates (M1.2, §26 rule 2).

**Code:**

`src/api/auth.rs` lines 7–15

```rust
#[post("/api/auth/login")]
pub async fn login(email: String, password: String) -> Result<AfterLogin, ServerFnError> {
    use crate::server::{auth::cookie, ctx, services::auth, state};
    let ctx = ctx::Ctx::current().await?;
    let st = state::get()?;
    let ok = auth::login(&st.pool, &ctx, &email, password).await?;
    ctx::set_cookie(&cookie::set(&st.config, &ok.token, ok.max_age_secs))?;
    Ok(ok.next)
}
```

`src/server/services/auth.rs` lines 253–266

```rust
impl From<AuthError> for dioxus::prelude::ServerFnError {
    fn from(e: AuthError) -> Self {
        use crate::domain::errors::coded;
        match e {
            AuthError::InvalidCredentials => coded(401, "auth.invalid_credentials"),
            AuthError::RateLimited => coded(429, "auth.rate_limited"),
            AuthError::WrongPassword => coded(400, "auth.wrong_password"),
            AuthError::Policy(p) => coded(400, p.code()),
            AuthError::Internal(err) => {
                tracing::error!(error = ?err, "auth internal error");
                coded(500, "internal")
            }
        }
    }
```

**Line notes:**

1. L9 (api/auth.rs) — `use` inside a function body limits the imports to that function, which matters here because these modules exist only in the server build.
2. L12 — `?` converts the service's `AuthError` into `ServerFnError` through the `From` implementation shown below.
3. L13 — the cookie is attached to this response with a `Set-Cookie` header; the client code never sees the token.
4. L253 (services/auth.rs) — `impl From<AuthError> for ServerFnError` is allowed because `AuthError` is defined in this crate (the orphan rule).
5. L256 — the `match` maps each internal error to an HTTP status and a code; adding a variant without a mapping is a compile error.
6. L262 — `?err` logs the internal error with `Debug` formatting on the server only; the client receives just `internal`.

**Concepts:** [server function](CONCEPTS.md#serverfn), [conversions](CONCEPTS.md#conversion), [cookies](CONCEPTS.md#cookie), [enum](CONCEPTS.md#enum), [traits](CONCEPTS.md#trait)

**Try it:** `curl -s -X POST http://127.0.0.1:8080/api/auth/login -H 'content-type: application/json' -d '{"email":"x@y.z","password":"a"}'`. Expected: status 401 and a body containing `"message":"auth.invalid_credentials"`; the login page shows the same code as "Incorrect email or password.".

## 23. Login form: signals, submit handler, navigation
**Feature:** the form keeps its inputs in signals, submits without reloading the page, and navigates according to the server's answer (M1.11).

**Code:**

`src/pages/login.rs` lines 19–43

```rust
    let mut email = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut error = use_signal(|| None::<String>);
    let mut busy = use_signal(|| false);

    let submit = move |evt: FormEvent| {
        evt.prevent_default();
        let next = next.clone();
        async move {
            if busy() {
                return;
            }
            busy.set(true);
            error.set(None);
            match login(email(), password()).await {
                Ok(AfterLogin::ChangePassword) => {
                    navigator().replace(Route::Profile {});
                }
                Ok(AfterLogin::Home) => {
                    // Only follow internal paths; anything unparsable goes home.
                    let target = next.parse::<Route>().unwrap_or(Route::Home {});
                    navigator().replace(target);
                }
                Err(e) => {
                    password.set(String::new());
```

**Line notes:**

1. L21 — `None::<String>` names the type of an empty `Option` explicitly, because nothing else tells the compiler what the signal holds.
2. L24 — the handler is a `move` closure that copies the signals (signals are `Copy`) and returns a future for Dioxus to run.
3. L25 — `prevent_default()` runs synchronously, before the browser would submit the form and reload the page.
4. L26 — `next.clone()` gives the async block its own `String`, since the closure may be called more than once.
5. L33 — `email()` calls the signal to clone its current value.
6. L34 — matching on `Ok(AfterLogin::ChangePassword)` uses the enum returned by the server function, shared through `domain`.
7. L39 — `next.parse::<Route>()` reuses the router's own URL parser; any external or malformed value fails and falls back to `Route::Home {}`.

**Concepts:** [signals and hooks](CONCEPTS.md#signals), [closure](CONCEPTS.md#closure), [async/await and Future](CONCEPTS.md#async), [conversions](CONCEPTS.md#conversion), [enum](CONCEPTS.md#enum)

**Try it:** open `http://127.0.0.1:8080/login?next=https://evil.example` and sign in. Expected: you land on `/`, not on the external site, because `"https://evil.example".parse::<Route>()` fails (this prevents an open redirect).

## 24. Sharing the signed-in user through context
**Feature:** the layout loads the current user once (during SSR) and shares it with every page below it; pages can refresh it (M1.8, M1.13).

**Code:**

`src/components/layout/app_layout.rs` lines 13–34

```rust
/// The signed-in user, shared with every page under the layout. Pages call `.restart()`
/// after changing something `current_user()` reports (e.g. `must_change_pw`).
pub type UserResource = Resource<Result<Option<UserInfo>, ServerFnError>>;

/// Layout for every page that needs a session (§12).
#[component]
pub fn AppLayout() -> Element {
    // Fetched during SSR and reused on hydration; the server guard already redirected
    // anonymous page loads, so `None` here means the session ended in this tab.
    let user: UserResource = use_server_future(current_user)?;
    use_context_provider(|| user);

    let current = user
        .read()
        .as_ref()
        .and_then(|r| r.as_ref().ok())
        .cloned()
        .flatten();
    let Some(me) = current else {
        return rsx! { GoToLogin {} };
    };
    let initials = me.initials();
```

**Line notes:**

1. L15 — `pub type` declares a type alias, a shorter name for a long generic type; it creates no new type.
2. L22 — `use_server_future(current_user)?` runs the server function during SSR and suspends the component until its result is ready.
3. L23 — `use_context_provider(|| user)` makes the resource available to descendants; `Resource` is `Copy`, so the closure can return it by value.
4. L25 — `.read()` borrows the resource state, `.as_ref()` converts `&Option<Result<..>>` into `Option<&Result<..>>`, and the chain ends with an owned `Option<UserInfo>`.
5. L30 — `.flatten()` turns `Option<Option<UserInfo>>` into `Option<UserInfo>`.
6. L31 — `let Some(me) = current else { return ...; };` is let-else: it binds `me` or runs the `else` block, which must leave the function.

**Concepts:** [context](CONCEPTS.md#context), [type aliases](CONCEPTS.md#alias), [Option combinators](CONCEPTS.md#option), [let-else](CONCEPTS.md#iflet), [SSR and hydration](CONCEPTS.md#hydration)

**Try it:** sign in as the seeded admin (password `ChangeMe!234`), change the password on `/profile`, and watch the yellow "must set a new password" banner. Expected: it disappears without a page reload, because `Profile` calls `user.restart()` and the layout re-reads `current_user()`.

---

# Milestone 1 — fix (commit `acb0566`)

## 25. CSRF decision with Fetch Metadata
**Feature:** state-changing requests are accepted only when the browser itself reports them as same-origin, which also works behind tunnels and forwarded ports (M1.9).

**Code:**

`src/server/auth/guard.rs` lines 42–60

```rust
fn allowed(
    fetch_site: Option<&str>,
    origin: Option<&str>,
    app_url: Option<&str>,
    host: Option<&str>,
) -> bool {
    if let Some(site) = fetch_site {
        return matches!(site, "same-origin" | "none");
    }
    match (origin, app_url) {
        (None, _) => true,
        (Some(o), Some(app_url)) => same_origin(o, app_url),
        (Some(o), None) => {
            host.is_some_and(|h| o.split_once("://").is_some_and(|(_, rest)| rest == h))
        }
    }
}

/// Scheme + host + port equality; trailing slash on `APP_URL` ignored.
```

**Line notes:**

1. L42 — the decision is a plain function of four optional strings, with no request or state, so it can be unit-tested directly.
2. L43 — `Option<&str>` borrows header text from the request instead of copying it; `None` means the header is absent.
3. L48 — `if let Some(site) = fetch_site { return ...; }` gives the browser-provided header priority and exits early.
4. L49 — `matches!(site, "same-origin" | "none")` tests a value against patterns, here two string literals joined by `|`.
5. L51 — `match (origin, app_url)` matches on a tuple, so the arms can combine the two optional values.
6. L55 — `is_some_and` returns `false` for `None` and otherwise applies the predicate; nesting two of them reads "the host exists and the origin's host part equals it".

**Concepts:** [match and patterns](CONCEPTS.md#match), [Option combinators](CONCEPTS.md#option), [cookies](CONCEPTS.md#cookie), [unit tests and cfg(test)](CONCEPTS.md#tests), [CSRF](CONCEPTS.md#csrf)

**Try it:** run `curl -s -o /dev/null -w '%{http_code}\n' -X POST http://127.0.0.1:8080/api/auth/login -H 'content-type: application/json' -H 'Sec-Fetch-Site: cross-site' -d '{"email":"a@b.c","password":"x"}'`, then repeat with `same-origin`. Expected: `403`, then `401` (the request reached the login logic and the credentials were rejected).

---

# UI port — P1 (commit `2ced4fb`, branch `ui/shadcn-port`)

## 26. One list of static paths, shared by two middlewares
**Feature:** files under `/fonts/`, `/images/`, `/assets/` and the wasm bundle are served without a session lookup and are never redirected to `/login` (SPEC-02 P1).

**Code:**

`src/server/ctx.rs` lines 69–75

```rust
pub const STATIC_PREFIXES: &[&str] = &[
    "/assets/", "/wasm/", "/_dioxus", "/favicon", "/fonts/", "/images/",
];

pub fn is_asset(path: &str) -> bool {
    STATIC_PREFIXES.iter().any(|p| path.starts_with(p))
}
```

`src/server/auth/guard.rs` lines 120–122

```rust
fn is_passthrough(path: &str) -> bool {
    path.starts_with("/api/") || path == "/healthz" || crate::server::ctx::is_asset(path)
}
```

**Line notes:**

1. L69 (ctx.rs) — `pub const STATIC_PREFIXES: &[&str]` is a constant slice of string slices; the `&[...]` literal is stored in the binary, so the constant has the `'static` lifetime.
2. L74 — `.iter()` yields `&&str` items, and `.any(...)` stops at the first element for which the closure returns `true`.
3. L74 — `path.starts_with(p)` accepts the `&&str` because `starts_with` takes any pattern type, and `&&str` dereferences to `&str`.
4. L121 (guard.rs) — `||` short-circuits: `is_asset` runs only when the cheaper comparisons are false.
5. L121 — `crate::server::ctx::is_asset(path)` calls the function through its full path, so both middlewares use one definition and cannot drift apart.

**Concepts:** [static and lifetimes](CONCEPTS.md#static), [arrays and slices](CONCEPTS.md#array), [iterators](CONCEPTS.md#iterator), [module](CONCEPTS.md#module), [middleware and layers](CONCEPTS.md#middleware)

**Try it:** run `curl -s -o /dev/null -w '%{http_code}\n' http://127.0.0.1:8080/fonts/inter-latin-wght-normal.woff2`; it prints `200`. Remove `"/fonts/"` from the list, wait for the rebuild and run it again. Expected: `303`, because the page guard now treats the font as a protected page and redirects to `/login`. Restore the entry.

---

# UI port — app shell + dashboard UI (commit `2a762ef`, branch `ui/shadcn-port`)

## 27. Shared sidebar state as a `Copy` context
**Feature:** every part of the shell (trigger, rail, nav groups, menus) reads and toggles one sidebar state; on desktop the choice is saved in a cookie, on mobile it opens a sheet (SPEC-02 P2).

**Code:**

`src/components/ui_extra/sidebar.rs` lines 24–45

```rust
impl SidebarCtx {
    pub fn get() -> Self {
        use_context::<SidebarCtx>()
    }
    pub fn open(&self) -> bool {
        (self.ui.sidebar_open)()
    }
    pub fn collapsed(&self) -> bool {
        !self.open()
    }
    pub fn toggle(mut self) {
        if (self.is_mobile)() {
            let v = (self.open_mobile)();
            self.open_mobile.set(!v);
        } else {
            self.ui.set_sidebar_open(!self.open());
        }
    }
    pub fn close_mobile(mut self) {
        self.open_mobile.set(false);
    }
}
```

**Line notes:**

1. L24 — `impl SidebarCtx` adds methods to a struct whose fields are signals; because signals are `Copy`, the whole context is `Copy` and can be passed to closures freely.
2. L26 — `use_context::<SidebarCtx>()` fetches the value provided by the nearest `SidebarProvider` ancestor; it panics if there is none.
3. L29 — `(self.ui.sidebar_open)()` calls a signal stored in a field; the parentheses around the field access are needed so Rust calls the field's value, not a method named `sidebar_open`.
4. L34 — `mut self` takes the context by value and allows mutation; since it is `Copy`, the caller keeps its own copy.
5. L36 — the value is read into `v` first, then written: reading and writing the same signal in one expression would hold a read borrow during the write.
6. L39 — on desktop the change goes through `Ui::set_sidebar_open`, which also writes the `sidebar_state` cookie read during the next server render.

**Concepts:** [context](CONCEPTS.md#context), [signals and hooks](CONCEPTS.md#signals), [Arc and Clone](CONCEPTS.md#arc), [impl block and methods](CONCEPTS.md#impl), [mutability](CONCEPTS.md#mut)

**Try it:** sign in, press Ctrl+B, then reload the page. Expected: the sidebar stays collapsed after the reload, because the server rendered it from the `sidebar_state=false` cookie.

## 28. Browser events into Rust through `eval`
**Feature:** a media query (mobile width) and a keyboard shortcut (Ctrl/Cmd+B) are observed in the browser and delivered to Rust as messages (SPEC-02 P2).

**Code:**

`src/components/ui_extra/sidebar.rs` lines 61–78

```rust
    use_effect(move || {
        let mut is_mobile = is_mobile;
        spawn(async move {
            let mut ev = document::eval(
                "const mq = matchMedia('(max-width: 767px)'); dioxus.send(['m', mq.matches]);
                 mq.addEventListener('change', e => dioxus.send(['m', e.matches]));
                 window.addEventListener('keydown', e => {
                   if (e.key === 'b' && (e.metaKey || e.ctrlKey)) { e.preventDefault(); dioxus.send(['b', true]); }
                 });",
            );
            while let Ok((kind, value)) = ev.recv::<(String, bool)>().await {
                match kind.as_str() {
                    "m" => is_mobile.set(value),
                    _ => ctx.toggle(),
                }
            }
        });
    });
```

**Line notes:**

1. L61 — `use_effect` runs after the component has rendered in the browser (never during SSR), which is where browser APIs are available.
2. L63 — `spawn` starts an async task owned by the component; it is cancelled when the component unmounts.
3. L64 — `document::eval` runs JavaScript in the page and returns a handle for two-way messages.
4. L65 — inside that JavaScript, `dioxus.send(value)` sends a JSON-serialisable value back to Rust.
5. L71 — `ev.recv::<(String, bool)>()` awaits the next message and deserialises it into a Rust tuple; `while let Ok(...)` loops until the channel closes.
6. L72 — `match kind.as_str()` compares an owned `String` with string literals by borrowing it as `&str`.

**Concepts:** [JS interop with eval](CONCEPTS.md#eval), [async/await and Future](CONCEPTS.md#async), [signals and hooks](CONCEPTS.md#signals), [match and patterns](CONCEPTS.md#match)

**Try it:** with the dashboard open, narrow the browser window below 768 px and press the sidebar button in the header. Expected: the sidebar opens as a sheet from the left (the `m` message set `is_mobile` to `true`, so `toggle` opened the mobile sheet instead of collapsing the desktop sidebar).

## 29. Positioning a menu from the trigger's measured rectangle
**Feature:** dropdowns open next to their button and are never clipped by scrolling areas, because they are placed with `position: fixed` from the button's on-screen position (SPEC-02 P2).

**Code:**

`src/components/ui_extra/menu.rs` lines 45–67

```rust
    let toggle = move |_| async move {
        if open() {
            open.set(false);
            return;
        }
        let Some(el) = anchor() else { return };
        let Ok(r) = el.get_client_rect().await else {
            return;
        };
        let (top, left, bottom, right, width) =
            (r.min_y(), r.min_x(), r.max_y(), r.max_x(), r.width());
        let vh = viewport_height();
        let mut css = match side {
            // Open upwards when the trigger sits in the lower part of the screen.
            Side::Bottom if bottom > vh * 0.6 => {
                format!("top:{}px;transform:translateY(-100%);", top - side_offset)
            }
            Side::Bottom => format!("top:{}px;", bottom + side_offset),
            Side::Right => match align {
                Align::Start => format!("top:{top}px;"),
                Align::End => format!("top:{bottom}px;transform:translateY(-100%);"),
            },
        };
```

**Line notes:**

1. L45 — `move |_| async move { ... }` is an event handler that returns a future, so it may await the measurement.
2. L50 — `let Some(el) = anchor() else { return };` takes the trigger's `MountedData` (saved by `onmounted`) or stops if it is not mounted yet.
3. L51 — `get_client_rect().await` asks the browser for the element's position and size, like `getBoundingClientRect()` in JavaScript.
4. L54 — a tuple pattern binds five values at once from one expression.
5. L59 — `Side::Bottom if bottom > vh * 0.6` is a match arm with a guard: near the bottom of the screen the menu opens upwards.
6. L63 — a `match` nested inside a `match` arm handles the second dimension (alignment) only where it matters.

**Concepts:** [match and patterns](CONCEPTS.md#match), [if let and let-else](CONCEPTS.md#iflet), [async/await and Future](CONCEPTS.md#async), [Vec and tuples](CONCEPTS.md#vec)

**Try it:** open the user menu at the bottom of the sidebar, then shrink the window height. Expected: the menu still appears fully on screen above the button (the `translateY(-100%)` branch), not cut off below it.

## 30. Filtering the menu tree by permission
**Feature:** links the user may not open are removed, and a section or group with nothing left disappears (§8, SPEC-02 §7.6).

**Code:**

`src/components/layout/nav_config.rs` lines 138–160

```rust
pub fn visible_groups(user: &UserInfo) -> Vec<NavGroup> {
    let allowed = |l: &NavLink| l.permission.is_none_or(|p| user.can(p));
    nav_groups()
        .into_iter()
        .filter_map(|g| {
            let items: Vec<NavItem> = g
                .items
                .into_iter()
                .filter_map(|item| match item {
                    NavItem::Link(l) => allowed(&l).then_some(NavItem::Link(l)),
                    NavItem::Collapsible {
                        title,
                        icon,
                        badge,
                        items,
                    } => {
                        let items: Vec<NavLink> =
                            items.into_iter().filter(|l| allowed(l)).collect();
                        (!items.is_empty()).then_some(NavItem::Collapsible {
                            title,
                            icon,
                            badge,
                            items,
```

**Line notes:**

1. L139 — the closure `allowed` captures `user` by reference and is reused for every link.
2. L139 — `is_none_or(|p| user.can(p))` reads "no permission required, or the user holds it".
3. L141 — `into_iter()` consumes the vector, so items are moved into the result instead of cloned.
4. L142 — `filter_map` both transforms each group and drops the ones that become empty (`None`).
5. L147 — `bool::then_some(value)` keeps the link only when `allowed` returned `true`.
6. L148 — the `NavItem::Collapsible { title, icon, badge, items }` pattern moves all four fields out of the enum variant so they can be reused in the rebuilt value.

**Concepts:** [iterators](CONCEPTS.md#iterator), [closure](CONCEPTS.md#closure), [Option combinators](CONCEPTS.md#option), [enum](CONCEPTS.md#enum), [match and patterns](CONCEPTS.md#match)

**Try it:** in the test `users_link_needs_permission`, change `"users.view"` to `"users.create"` and run `just test users_link`. Expected: the test fails, because the Users link requires exactly `users.view`.

## 31. Reproducing a library's algorithm: chart ticks
**Feature:** the Y axis shows the same values as the reference's Recharts chart (0, 1500, 3000, 4500, 6000 for a maximum of 5,800), computed in Rust (SPEC-02 P8).

**Code:**

`src/components/ui_extra/chart.rs` lines 24–41

```rust
pub fn nice_ticks(max: f64, count: usize) -> Vec<f64> {
    if max <= 0.0 || count < 2 {
        return vec![0.0];
    }
    let rough = max / (count - 1) as f64;
    let digits = rough.log10().floor() + 1.0;
    let unit = 10f64.powf(digits);
    // Scale in hundredths of `unit`, so steps like 1,500 stay exact in floating point.
    let scale = if digits == 1.0 { 10.0 } else { 5.0 };
    let mut correction = 0.0;
    loop {
        let step = ((rough / unit * 100.0 / scale).ceil() + correction) * scale * unit / 100.0;
        if step * (count - 1) as f64 >= max {
            return (0..count).map(|i| i as f64 * step).collect();
        }
        correction += 1.0;
    }
}
```

**Line notes:**

1. L25 — an early `return` handles the degenerate inputs before any logarithm is taken.
2. L28 — `as f64` converts the integer `count` to a float; Rust never converts numeric types implicitly.
3. L29 — `log10().floor() + 1.0` counts the digits of the rough step (1450 → 4).
4. L32 — the scale is kept in hundredths, so `5.0 * 10000.0 / 100.0` is exact, whereas `0.05 * 10000.0` is not in binary floating point.
5. L34 — `loop` repeats until a `return` inside it; Rust's `loop` is an expression and could also `break` with a value.
6. L37 — `(0..count).map(...).collect()` builds the tick list from a range.

**Concepts:** [expressions vs statements](CONCEPTS.md#expr), [iterators](CONCEPTS.md#iterator), [floating point](CONCEPTS.md#float), [unit tests and cfg(test)](CONCEPTS.md#tests)

**Try it:** change `5.0` to `10.0` on line 32 and run `just test nice_ticks`. Expected: the test fails with ticks `[0, 2000, 4000, 6000]`, the result of rounding steps to 0.1 × 10ⁿ (what d3 would do), not Recharts' 0.05 × 10ⁿ.

## 32. Preferences that are correct on the first paint
**Feature:** theme, sidebar state, layout and direction come from cookies on the server, so the first HTML already matches the user's choice, and hydration does not change it (SPEC-02 P2, P5).

**Code:**

`src/state/prefs.rs` lines 124–133

```rust
    pub fn from_request() -> Self {
        let c = |n| cookies::request(n).unwrap_or_default();
        Self {
            theme: Theme::parse(&c(THEME_COOKIE)),
            variant: Variant::parse(&c(VARIANT_COOKIE)),
            collapsible: Collapsible::parse(&c(COLLAPSIBLE_COOKIE)),
            sidebar_open: c(SIDEBAR_COOKIE) != "false",
            dir: Dir::parse(&c(DIR_COOKIE)),
        }
    }
```

`src/app.rs` lines 41–46

```rust
    let initial = use_server_cached(i18n::request_locale);
    use_init_i18n(|| i18n::config(&initial));
    // Theme, sidebar, layout and direction from cookies (SSR), shared app-wide.
    let prefs = use_server_cached(UiPrefs::from_request);
    use_hook(|| Ui::provide(prefs));

```

**Line notes:**

1. L125 (prefs.rs) — the closure `c` takes a cookie name and returns its value or an empty string, keeping the struct literal short.
2. L130 — `c(SIDEBAR_COOKIE) != "false"` makes "open" the default when the cookie is absent.
3. L131 — each enum has a `parse` function that maps unknown text to its default variant, so a tampered cookie cannot cause an error.
4. L44 (app.rs) — `use_server_cached(UiPrefs::from_request)` runs on the server during SSR and ships the result to the browser, where the same value is used during hydration.
5. L45 — `use_hook(|| Ui::provide(prefs))` runs the provider once per component instance, creating the signals only on the first render.

**Concepts:** [SSR and hydration](CONCEPTS.md#hydration), [cookies](CONCEPTS.md#cookie), [closure](CONCEPTS.md#closure), [signals and hooks](CONCEPTS.md#signals), [context](CONCEPTS.md#context)

**Try it:** choose "Dark" in the theme menu and reload with JavaScript disabled in the browser's developer tools. Expected: the page is still dark, because the `theme` cookie was applied before the app's JavaScript runs (the inline script in `index.html`).

## 33. Table state that lives in the URL
**Feature:** the Tasks and Users tables keep page, page size, search text and column filters in the address bar, so a copied link or a reload shows the same rows (SPEC-02 P8, P10; reference `use-table-url-state.ts`).

**Code:**

`src/components/data_table/mod.rs` lines 104–122

```rust
    pub fn build(&self, filter_key: &str, filter_columns: &[&'static str]) -> Query {
        let page = self.page + 1;
        let mut q = Query::default()
            .with("page", (page > 1).then(|| page.to_string()))
            .with(
                "pageSize",
                (self.page_size != DEFAULT_PAGE_SIZE).then(|| self.page_size.to_string()),
            )
            .with(filter_key, Some(self.filter.trim().to_string()));
        for c in filter_columns {
            let v = self
                .columns
                .iter()
                .find(|(k, _)| k == c)
                .map(|(_, v)| v.join(","));
            q = q.with(c, v);
        }
        q
    }
```

**Line notes:**

1. L104 — `build` turns the plain `UrlState` data into a `Query`; it takes `&self` because it only reads.
2. L105 — the URL is 1-based (`page=2`) while the state is 0-based, so 1 is added once, here.
3. L107 — `(page > 1).then(|| ...)` gives `None` for the first page; `Query::with(key, None)` removes the key, so default values never appear in the URL.
4. L112 — the search text is trimmed; an empty string is also removed by `with`, so "   " clears the `filter` key.
5. L113–119 — for each filter column, `find` looks up its selected values and `join(",")` writes them as `status=todo,done`; a column without a filter maps to `None` and disappears.
6. L121 — the finished `Query` is returned; the page's `use_effect` compares its text with the last one and calls `navigator().replace(...)` only when it changed.

**Concepts:** [Option combinators](CONCEPTS.md#option), [iterators](CONCEPTS.md#iterator), [query strings in routes](CONCEPTS.md#query-route), [use_effect and peek](CONCEPTS.md#effect), [unit tests](CONCEPTS.md#tests)

**Try it:** open `/tasks`, choose Status → Todo and go to page 2. Expected URL: `/tasks?page=2&status=todo`. Reload: the same page and filter come back. Then run `cargo test --features server url_` to see the four ported tests pass.

## 34. Faceted counts: counting with every other filter applied
**Feature:** each option in the Status / Priority filter shows how many rows it would match, given the search text and the *other* filters — the same numbers TanStack Table's faceted model shows in the reference.

**Code:**

`src/pages/tasks/table.rs` lines 30–54

```rust
fn filtered(state: &TableState, skip: Option<&str>) -> Vec<Task> {
    let term = (state.filter)().trim().to_lowercase();
    TASKS
        .iter()
        .filter(|t| {
            term.is_empty()
                || t.id.to_lowercase().contains(&term)
                || t.title.to_lowercase().contains(&term)
        })
        .filter(|t| {
            FILTER_COLUMNS
                .iter()
                .filter(|c| Some(**c) != skip)
                .all(|c| state.matches(c, field(t, c)))
        })
        .cloned()
        .collect()
}

fn counts(rows: &[Task], column: &str, values: &[&'static str]) -> Vec<(&'static str, usize)> {
    values
        .iter()
        .map(|v| (*v, rows.iter().filter(|t| field(t, column) == *v).count()))
        .collect()
}
```

**Line notes:**

1. L30 — `skip` names the column whose own filter must be ignored; `None` means "apply everything" (the rows shown in the table).
2. L31 — the search term is lower-cased once, outside the loop.
3. L35–38 — the global search matches the task ID or the title, case-insensitively, as `globalFilterFn` does in the reference.
4. L41–43 — `.filter(|c| Some(**c) != skip)` drops the skipped column; `.all(...)` keeps a row only if every remaining column filter accepts it.
5. L46 — `.cloned().collect()` turns the borrowed rows into an owned `Vec<Task>` the caller can keep.
6. L49–53 — `counts` walks the option values and counts matching rows; the Status filter receives `counts(&filtered(&state, Some("status")), ...)`, so selecting "Todo" does not make the other statuses drop to zero.

**Concepts:** [iterators](CONCEPTS.md#iterator), [closures](CONCEPTS.md#closure), [Option](CONCEPTS.md#option), [borrowing](CONCEPTS.md#borrow)

**Try it:** on `/tasks`, open Status and note the count next to "Done". Now set Priority → High. Expected: the Status counts get smaller (they respect Priority), but choosing "Todo" in Status does not change the count shown next to "Done".

## 35. One "which dialog is open" signal, many dialogs
**Feature:** each feature page keeps a single `Signal<Option<Dialog>>` (the reference's `useDialogState`); every dialog gets its own `bool` signal that follows it, and closing the dialog resets the shared state.

**Code:**

`src/components/ui_extra/dialog.rs` lines 124–137

```rust
pub fn use_dialog_flag<D: PartialEq + Copy + 'static>(
    open: Signal<Option<D>>,
    which: D,
) -> Signal<bool> {
    let mut open = open;
    let mut flag = use_signal(|| false);
    use_effect(move || flag.set(open() == Some(which)));
    use_effect(move || {
        if !flag() && *open.peek() == Some(which) {
            open.set(None);
        }
    });
    flag
}
```

**Line notes:**

1. L124 — the hook is generic over the dialog enum `D`; `Copy + PartialEq` lets it compare and move the value into closures, `'static` is required for values stored in signals.
2. L128 — `let mut open = open;` rebinds the parameter as mutable so the second effect can call `open.set`.
3. L129 — the `bool` signal the `Modal` / `SideSheet` / `ConfirmDialog` components expect.
4. L130 — first effect: whenever `open` changes, the flag becomes `true` only for the matching dialog.
5. L131–135 — second effect: when the dialog closes itself (Escape, ×, Cancel), its flag turns `false`; if the shared state still names this dialog it is cleared. `peek()` reads `open` without subscribing, so this effect re-runs only when the flag changes.

**Concepts:** [generics](CONCEPTS.md#generics), [use_effect and peek](CONCEPTS.md#effect), [signals](CONCEPTS.md#signals), [context](CONCEPTS.md#context)

**Try it:** on `/tasks`, click Create, press Escape, then click Create again. Expected: the sheet opens both times. Comment out the second `use_effect` and repeat: the second click does nothing, because the shared state was never cleared and setting it to the same value is not a change.

## 36. Logical start / end for popovers in right-to-left mode
**Feature:** dropdown menus (row actions, profile, team switcher) open on the correct side when the direction is Right to Left, like Radix does with `dir="rtl"` (SPEC-02 P5).

**Code:**

`src/components/ui_extra/menu.rs` lines 77–88

```rust
        // Start / end and "right" are logical: in RTL they mirror (Radix uses `dir`).
        let x = match (side, align, rtl) {
            (Side::Right, _, false) => format!("left:{}px;", right + side_offset),
            (Side::Right, _, true) => format!("left:{}px;translate:-100% 0;", left - side_offset),
            (Side::Bottom, Align::Start, false) | (Side::Bottom, Align::End, true) => {
                format!("left:{left}px;")
            }
            (Side::Bottom, Align::Start, true) | (Side::Bottom, Align::End, false) => {
                format!("left:{right}px;translate:-100% 0;")
            }
        };
        css.push_str(&x);
```

**Line notes:**

1. L77 — "start" and "end" mean left and right in LTR, but right and left in RTL; the code works out the physical side once, here.
2. L78 — matching on a tuple `(side, align, rtl)` lists every combination in one place; the compiler checks that none is missing.
3. L80 — a `Right`-side submenu in RTL opens to the left: it is placed at the trigger's left edge and moved back by its own width with `translate:-100% 0`.
4. L81 — an `|` pattern joins two cases with the same result: Start in LTR and End in RTL both line up with the trigger's left edge.
5. L84–85 — the other two cases line up with the trigger's right edge, again using `translate` because the menu's width is not known yet.
6. L88 — the computed `left`/`translate` is appended to the inline style of the fixed-position menu.

**Concepts:** [match and patterns](CONCEPTS.md#match), [context](CONCEPTS.md#context), [signals](CONCEPTS.md#signals)

**Try it:** open Settings → Appearance drawer → Direction → Right to Left, go to `/tasks` and open a row's "…" menu. Expected: the menu stays on screen and opens towards the table; hover Labels and the submenu opens to the right. Change `Align::End, true` to `Align::End, false` in the first pattern and the menu opens off the left edge.

## 37. Skeletons that render the same on the server and the browser
**Feature:** while the app or a page is waiting for data, grey pulsing blocks shaped like the real page are shown instead of a blank area (shadcn `Skeleton` and `SidebarMenuSkeleton`; `SuspenseBoundary` fallbacks).

**Code:**

`src/components/ui_extra/sidebar.rs` lines 403–423

```rust
pub fn SidebarMenuSkeleton(
    #[props(default)] show_icon: bool,
    #[props(default)] seed: usize,
    #[props(into, optional)] class: Option<String>,
) -> Element {
    let width = 50 + (seed * 37 + 13) % 40;
    rsx! {
        div {
            "data-slot": "sidebar-menu-skeleton",
            "data-sidebar": "menu-skeleton",
            class: tw_merge!("flex h-8 items-center gap-2 rounded-md px-2", class.as_deref().unwrap_or("")),
            if show_icon {
                crate::ui::skeleton::Skeleton { class: "size-4 rounded-md" }
            }
            crate::ui::skeleton::Skeleton {
                class: "h-4 max-w-(--skeleton-width) flex-1",
                style: "--skeleton-width: {width}%",
            }
        }
    }
}
```

**Line notes:**

1. L404–406 — props with `#[props(default)]` can be left out by the caller; `seed` defaults to 0.
2. L408 — the reference picks a random width between 50% and 90% on each render. Random values would differ between the server's HTML and the browser's first render (a hydration mismatch), so the width is calculated from `seed`: `(seed * 37 + 13) % 40` is always 0–39, so the result is 50–89%.
3. L413 — `tw_merge!` joins the base classes with the caller's `class`, letting the caller override, e.g., the height.
4. L414–416 — the small square stands in for the menu icon only when `show_icon` is true.
5. L418–419 — the text bar is `flex-1` (it takes the free space) but capped by `max-w-(--skeleton-width)`; the CSS variable is set in the inline `style`, so every row can have a different cap without generating new Tailwind classes.

**Concepts:** [component props](CONCEPTS.md#component), [rsx](CONCEPTS.md#rsx), [SSR and hydration](CONCEPTS.md#hydration)

**Try it:** in the browser's developer tools, set Network → throttling to "Slow 3G", sign out and sign in again. Expected: the shell skeleton (sidebar rows, header bar, cards) appears until `/api/auth/me` answers, then the real dashboard replaces it with no jump in layout.

## 38. Never lock out the last Super Admin
**Feature:** a change that would leave the system with no active Super Admin (deactivate, delete or demote the last one) is refused, even when two admins try it at the same moment (§8, M2.4).

**Code:**

`src/server/rbac.rs` lines 181–198

```rust
pub async fn keep_last_super_admin(
    tx: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
) -> Result<(), AppError> {
    let ids = sqlx::query_scalar!(
        r#"SELECT u.id FROM users u JOIN roles r ON r.id = u.role_id
           WHERE r.is_system AND r.name = $1 AND u.is_active AND u.deleted_at IS NULL
           FOR UPDATE OF u"#,
        super::seed::SUPER_ADMIN
    )
    .fetch_all(&mut **tx)
    .await?;
    if ids.len() == 1 && ids[0] == user_id {
        Err(AppError::Conflict("users.last_super_admin"))
    } else {
        Ok(())
    }
}
```

**Line notes:**

1. L182 — the guard takes the caller's open transaction, so the check and the change it protects commit or roll back together.
2. L185–188 — the query lists every active, non-deleted user whose role is the system "Super Admin" role.
3. L188 — `FOR UPDATE OF u` locks those user rows until the transaction ends. If two admins demote each other at once, the second transaction waits here, then sees the first one's result — without the lock both could pass the check and leave zero Super Admins.
4. L189 — the role name is a bind parameter (`$1`), never pasted into the SQL text.
5. L193 — the change is refused only if exactly one Super Admin is left and it is the user being changed.
6. L194 — `AppError::Conflict("users.last_super_admin")` becomes HTTP 409 with that code; the browser shows the translated message `err-users-last-super-admin`.

**Concepts:** [transactions](CONCEPTS.md#transaction), [Result and ?](CONCEPTS.md#result), [SQLx macros](CONCEPTS.md#sqlx-macros), [enum](CONCEPTS.md#enum)

**Try it:** run `cargo test --features server last_super_admin`. Expected: `last_super_admin_is_protected` and `one_of_two_super_admins_may_go` pass. Remove `&& ids[0] == user_id` and run again: the second test now fails, because demoting one of two admins is wrongly refused.

## 39. Bulk actions that skip, not fail
**Feature:** selecting many users and pressing Deactivate / Delete / Restore applies the action to each one separately, re-checking every rule per user; users that may not be changed are reported with a reason instead of cancelling the whole batch (§25.1, M5.6).

**Code:**

`src/server/services/users.rs` lines 569–590

```rust
pub async fn bulk(
    pool: &PgPool,
    ctx: &Ctx,
    action: BulkUserAction,
    ids: Vec<Uuid>,
) -> Result<BulkResult, AppError> {
    let actor = ctx.user.clone().ok_or(AppError::Unauthenticated)?;
    let max = settings::value(pool, "bulk.max_items")
        .await
        .as_u64()
        .unwrap_or(1000) as usize;
    if ids.len() > max {
        return Err(AppError::Validation("bulk.too_many"));
    }
    let mut res = BulkResult::default();
    for id in ids.iter().copied() {
        match apply(pool, ctx, action, id).await {
            Ok(()) => res.done += 1,
            Err(AppError::Internal(e)) => return Err(AppError::Internal(e)),
            Err(e) => res.skipped.push((id, e.code().to_string())),
        }
    }
```

**Line notes:**

1. L575 — the signed-in user is needed for the summary audit row; without a session the call ends with `Unauthenticated` (401).
2. L576–579 — `bulk.max_items` comes from the settings cache (M2.9), so an admin can change the limit without a deploy.
3. L580–582 — a too-large selection is refused as a whole, before any change.
4. L584–585 — each id goes through `apply`, the same function the single row menu uses. It opens its own transaction and re-checks permission, "not yourself" and "not the last Super Admin" for that user.
5. L587 — a database or programming error (`Internal`) stops the batch: something is wrong and continuing could hide it.
6. L588 — any other error (a broken rule) is recorded as `(id, code)` and the loop goes on; the page shows "Skipped 1: You can't do that to your own account."

**Concepts:** [match and patterns](CONCEPTS.md#match), [Result and ?](CONCEPTS.md#result), [transactions](CONCEPTS.md#transaction), [iterators](CONCEPTS.md#iterator)

**Try it:** on `/users`, tick "Select all" (your own row is on the page) and press Deactivate. Expected: a success toast for the others and a warning toast "Skipped 1: You can't do that to your own account." Your own account stays active.

## 40. A chart with no missing days: `generate_series` + `LEFT JOIN`
**Feature:** the dashboard's sign-in chart always has 14 points, one per day, even on days with no sign-ins (§10, M10.3).

**Code:**

`src/server/services/dashboard.rs` lines 72–85

```rust
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
```

**Line notes:**

1. L76–77 — `generate_series(start, end, interval '1 day')` produces one row per day, today and the 13 days before, whether or not anything happened.
2. L78 — `LEFT JOIN` keeps every day even when no audit row matches it (an inner `JOIN` would drop empty days, and the chart would jump over them).
3. L79 — each audit row is attached to the day whose 24 hours contain it.
4. L74–75 — `count(a.id) FILTER (WHERE …)` counts successful and failed sign-ins separately in one pass; `count(a.id)` is 0 for an empty day because `a.id` is NULL there.
5. L73 — `AS "day!"`: the `!` tells SQLx the column is never NULL, so Rust gets a `String`, not `Option<String>`.
6. L85 — `.fetch_all(pool)` only builds the future here; it runs later inside `tokio::try_join!` together with the other eight dashboard queries.

**Concepts:** [SQLx macros](CONCEPTS.md#sqlx-macros), [async and Future](CONCEPTS.md#async), [Option](CONCEPTS.md#option)

**Try it:** run `cargo test --features server numbers_match_the_data`. Then change `LEFT JOIN` to `JOIN` and run it again: the test fails on `logins_14d.len() == 14`, because days without sign-ins disappear.
