# Concepts

Each concept is explained once, here, and linked from [LEARNING.md](LEARNING.md). "First seen" names
the entry where it first appears in this codebase. Official references: *The Rust Programming
Language* ("the Book", doc.rust-lang.org/book), the Dioxus 0.7 guide (dioxuslabs.com/learn/0.7),
and the docs.rs page of each crate.

---

## Language

<a id="module"></a>
### Module (`mod`, `use`, paths)
A module is a named scope for items (functions, types, constants). `mod name;` declares a module
whose contents live in `name.rs` or `name/mod.rs`; the compiler only sees files reachable through
such declarations from `main.rs`. Items are private to their module unless marked `pub`. A path
(`crate::server::ctx::Ctx`) names an item starting from the crate root (`crate`), the parent
(`super`) or the current module (`self`); `use` brings a path into scope under a shorter name.
First seen: entry 1. Book ch. 7.

<a id="attribute"></a>
### Attribute
Metadata attached to the item that follows it: `#[name]` or `#[name(args)]`. `#![...]` (with `!`)
applies to the enclosing item instead. Attributes drive compiler behaviour (`#[cfg]`, `#[derive]`,
`#[allow(...)]` to silence a lint), tools (`#[rustfmt::skip]`) and procedural macros
(`#[component]`, `#[sqlx::test]`). First seen: entry 1.

<a id="cfg"></a>
### Conditional compilation (`cfg`) and Cargo features
`#[cfg(predicate)]` keeps the item only when the predicate is true at compile time; otherwise the
item does not exist (it is not merely skipped at runtime). Predicates include `feature = "x"`,
`test`, `target_arch = "wasm32"`, combined with `not`, `all`, `any`. Cargo features are named
switches declared in `[features]` in `Cargo.toml`; a feature can enable optional dependencies
(`"dep:sqlx"`). This project builds twice: `web` (wasm, browser) and `server` (native). Server-only
crates are `optional = true` and enabled only by `server`, so they cannot be linked into the wasm
bundle. First seen: entry 1. Cargo Book, "Features".

<a id="closure"></a>
### Closure
An anonymous function that can capture variables from its surrounding scope: `|args| body`.
`move` forces captured variables to be moved (owned) into the closure rather than borrowed, which
is required when the closure outlives the current scope (for example when it runs later on another
task). Closures implement one or more of the traits `Fn`, `FnMut`, `FnOnce`, depending on how they
use captured values. First seen: entry 1. Book ch. 13.1.

<a id="async"></a>
### `async`, `.await` and `Future`
An `async fn` or `async { }` block does not run when called; it produces a `Future`, a value
describing a computation that can be polled to completion. `.await` (only valid inside async code)
polls the future and suspends the current task while it is not ready, letting the runtime (Tokio
here) run other tasks on the same thread. Typical uses are I/O: database queries, HTTP calls.
First seen: entry 1. Book ch. 17; *Asynchronous Programming in Rust*.

<a id="derive"></a>
### `#[derive]`
Generates a standard implementation of a trait from the type's definition. `Debug` (format with
`{:?}`), `Clone` (explicit `.clone()`), `Copy` (implicit bitwise copy, small types only),
`PartialEq`/`Eq` (`==`), `Default` (`T::default()`), and library derives such as serde's
`Deserialize`/`Serialize` or Dioxus' `Routable`. First seen: entry 2. Book appendix C.

<a id="enum"></a>
### Enum
A type whose value is exactly one of several variants; variants may carry data
(`Some(T)`, `Home {}`, `Err(E)`). Combined with `match`, the compiler guarantees every variant is
handled. `Option` and `Result` are enums from the standard library. First seen: entry 2. Book ch. 6.

<a id="struct"></a>
### Struct
A type grouping named fields (`struct Config { database_url: String, ... }`). Tuple structs have
unnamed fields (`struct Extension<T>(T)`) and are accessed by pattern or `.0`. A struct owns its
fields; fields are private to the module unless `pub`. First seen: entry 3. Book ch. 5.

<a id="macro"></a>
### Macro
Code that generates code at compile time; invoked with `!` (`println!`, `json!`, `rsx!`,
`asset!`, `env!`) or as an attribute/derive. Declarative macros (`macro_rules!`) match token
patterns; procedural macros are Rust functions over token streams. Macros can check things a
normal function cannot, e.g. `query_scalar!` validates SQL against the database at compile time.
First seen: entry 2. Book ch. 20.5.

<a id="generics"></a>
### Generics and the turbofish
A generic item is parameterised by types: `Option<T>`, `Router<Route>`. The compiler usually infers
the parameters; when it cannot, they are given explicitly with the "turbofish" `::<>`:
`Router::<Route> {}`, `extract::<Extension<Ctx>, _>()`. `_` asks the compiler to infer that one
parameter. First seen: entry 2. Book ch. 10.1.

<a id="option"></a>
### `Option<T>` and its combinators
`Option<T>` is `Some(T)` or `None`; Rust has no null, so possible absence is explicit in the type.
Combinators transform it without `match`: `map(f)` applies `f` to the inner value,
`and_then(f)` chains another `Option`-returning step, `ok_or_else(f)` converts to `Result`,
`as_deref()` turns `Option<String>` into `Option<&str>`, `unwrap_or(x)` supplies a fallback.
`Result::ok()` converts `Result<T, E>` to `Option<T>`, discarding the error.
First seen: entry 3. Book ch. 6.1; std docs `Option`.

<a id="string"></a>
### `String` vs `&str`
`String` is an owned, heap-allocated, growable UTF-8 buffer. `&str` is a borrowed view ("string
slice") into UTF-8 data owned elsewhere (a `String`, or the binary for literals like `"ok"`).
Functions that only read text take `&str`; a `&String` converts to `&str` automatically (deref
coercion). `.to_owned()` / `.to_string()` / `.into()` create a `String` from a `&str`.
First seen: entry 3. Book ch. 8.2.

<a id="serde"></a>
### serde (serialisation)
The standard framework for converting Rust values to and from formats such as JSON or environment
variables. `#[derive(Serialize, Deserialize)]` generates the conversions; field attributes adjust
them (`#[serde(default)]`, `#[serde(rename_all = "lowercase")]`). Server functions use serde to
send arguments and results over HTTP. First seen: entry 3. serde.rs.

<a id="impl"></a>
### `impl` blocks, methods and associated functions
`impl Type { ... }` defines functions belonging to a type. A function whose first parameter is
`self`, `&self` or `&mut self` is a method, called as `value.method()`; without it, it is an
associated function, called as `Type::function()` (constructors like `new`, `load`). `Self` is
an alias for the implementing type. First seen: entry 4. Book ch. 5.3.

<a id="result"></a>
### `Result<T, E>` and the `?` operator
`Result` is `Ok(T)` or `Err(E)`: errors are values, not exceptions. `expr?` in a function that
returns `Result` means "if `Err`, return it now (converting the error type with `From`), otherwise
unwrap the `Ok` value". `Result::map_err` changes the error type; `.is_ok()` tests success.
Type aliases like `anyhow::Result<T>` or `sqlx::Result<T>` fix the error type.
First seen: entry 4. Book ch. 9.

<a id="anyhow"></a>
### `anyhow`: application errors with context
`anyhow::Error` can hold any error type plus a chain of context messages; it suits application
code where errors are reported rather than matched on. `.context("msg")` (trait `Context`) wraps
an error with a message; `bail!("msg")` returns early with a new error. Library-style code
(server functions, services) uses typed errors instead (planned: `AppError`, M2.7).
First seen: entry 4. docs.rs/anyhow.

<a id="borrow"></a>
### Ownership, references and borrowing
Every value has one owner; when the owner goes out of scope the value is dropped. A reference
borrows a value without taking ownership: `&T` (shared, read-only, many allowed) or `&mut T`
(exclusive, read-write, one at a time). The borrow checker enforces at compile time that
references never outlive the value and that shared and exclusive borrows never overlap.
Passing `&cfg` lets a function read `cfg` while the caller keeps it.
First seen: entry 4. Book ch. 4.

<a id="match"></a>
### `match`, patterns and destructuring
`match value { pattern => expr, ... }` selects the first arm whose pattern fits; arms must cover
every possibility (exhaustiveness). Patterns can nest (`Some(Ok(i))`), bind variables, use guards
(`Some(s) if s.len() >= 32`) and the wildcard `_`. The same patterns destructure in `let`
(`let Extension(ctx) = ...`) and in function parameters (`Extension(state): Extension<AppState>`).
First seen: entry 4. Book ch. 6.2, ch. 19.

<a id="iflet"></a>
### `if let`
`if let PATTERN = expr { ... }` runs the block only when the pattern matches, binding its
variables; it is a `match` with one interesting arm. Used for `Option`/`Result` when only one case
needs handling. First seen: entry 6. Book ch. 6.3.

<a id="rawstr"></a>
### Raw string literals
`r"..."` or `r#"..."#` disables escape processing; with `#` delimiters the string may contain `"`.
Suited to SQL and regexes. First seen: entry 6. Rust Reference, "Raw string literals".

<a id="static"></a>
### `static` items and the `'static` lifetime
A `static` is a single value at a fixed memory location for the whole program run. A lifetime
(`'a`) names how long a reference is valid; `'static` means "for the entire program", which a
reference into a `static` satisfies. Mutable global state must be synchronised, hence types like
`OnceCell`. First seen: entry 7. Book ch. 10.3, ch. 20.1.

<a id="oncecell"></a>
### `OnceCell`
A cell written at most once and then only read. `tokio::sync::OnceCell::get_or_try_init` runs an
async initialiser the first time, stores the result, and makes concurrent callers wait for that
same initialisation; on error the cell stays empty. Used here so hot reload does not re-run
migrations or open a second pool. First seen: entry 7. docs.rs/tokio `sync::OnceCell`.

<a id="arc"></a>
### `Arc` and `Clone`
`Arc<T>` (atomically reference-counted) is a pointer that allows several owners of one heap value
across threads; `.clone()` increments a counter instead of copying `T`, and the value is dropped
when the last `Arc` is. `Clone` is the trait for explicit duplication; for `AppState` it clones a
pool handle (internally an `Arc`), an `Arc<Config>` and a `Uuid`, all cheap.
First seen: entry 7. Book ch. 15.4, ch. 16.3.

<a id="mut"></a>
### Mutability
Bindings are immutable by default. `let mut x` (or a `mut` parameter) allows mutation of that
binding and taking `&mut` references to it. Mutability belongs to the binding, not the type.
First seen: entry 8. Book ch. 3.1.

<a id="trait"></a>
### Traits
A trait declares behaviour types can implement (`impl Display for TenancyMode`). Generic code is
written against traits (`T: IntoResponse`). Standard examples: `Display`, `Debug`, `Clone`,
`From`/`Into`, `Default`. First seen: entry 11. Book ch. 10.2.

<a id="impltrait"></a>
### `impl Trait` in return position
`fn f() -> impl Trait` returns one concrete type that implements `Trait` without naming it; callers
can only use it through that trait. Common for complex types such as Axum responses or iterators.
First seen: entry 11. Book ch. 10.2.

<a id="expr"></a>
### Expressions vs statements
Most constructs in Rust are expressions that produce a value: blocks, `if`, `match`. A block's
value is its final expression without a trailing semicolon; adding `;` turns it into a statement
with value `()`. This is why functions often end with `Ok(x)` and no `return`.
First seen: entry 11. Book ch. 3.3.

<a id="tests"></a>
### Unit tests and `#[cfg(test)]`
Functions marked `#[test]` (or a test attribute such as `#[sqlx::test]`) are run by `cargo test`;
a panic (`assert!`, `assert_eq!`, `unwrap` on `Err`) or a returned `Err` fails the test. Placing
tests in a `#[cfg(test)] mod tests` inside the file under test gives access to private items
through `use super::*`. First seen: entry 12. Book ch. 11.

<a id="conversion"></a>
### Conversions: `From`/`Into`, `FromStr` and `parse`
`From<T> for U` defines an infallible conversion; it automatically provides `Into<U> for T`, so APIs
can accept "anything convertible" (`impl Into<Locale>`). `FromStr` defines parsing from text, used by
`"en".parse::<LanguageIdentifier>()`; the target type comes from the turbofish or a type annotation,
and the result is a `Result` because parsing can fail. First seen: entry 13. Book ch. 10.2; std docs `From`.

<a id="iterator"></a>
### Iterators and adapters
An iterator yields items one at a time through `next()`, returning `Option<Item>`. Adapters such as
`map`, `filter_map`, `flat_map` build new lazy iterators; nothing runs until a consumer pulls items:
`collect()` (build a collection), `find_map()` (first `Some`), `for` loops. `iter()` borrows a
collection's items, `into_iter()` consumes the collection and yields owned items, `copied()` turns
`&T` items into `T` for `Copy` types. First seen: entry 14. Book ch. 13.2.

<a id="vec"></a>
### `Vec<T>` and tuples
`Vec<T>` is a growable, heap-allocated array that owns its elements. A tuple `(A, B)` groups a fixed
number of values of possibly different types, accessed by position (`t.0`, `t.1`) or destructured
in patterns (`|(_, tag)|`). First seen: entry 14. Book ch. 8.1, ch. 3.2.

---

## Libraries and architecture

<a id="component"></a>
### Dioxus component and `Element`
A component is a function annotated with `#[component]` that returns `Element`. Its parameters are
props (owned values implementing `PartialEq + Clone`). A component re-renders when its props change
or when a signal it read is written. Component names start with a capital letter.
First seen: entry 2. Dioxus guide, "Components".

<a id="rsx"></a>
### `rsx!`
Dioxus' macro for declaring UI: elements (`div { class: "..", "text" }`), components
(`Card { .. }`), attributes, string interpolation (`"Count: {count}"`), and Rust control flow
(`if`, `for`, `match`) directly inside. It compiles to efficient templates; literal braces in text
are written `{{` and `}}`. First seen: entry 2. Dioxus guide, "RSX".

<a id="signals"></a>
### Signals and hooks
A hook (`use_*`) is a function called at the top level of a component that keeps state across
renders, in call order. `use_signal(|| init)` creates a signal: a `Copy` handle to reactive state;
reading it (`count()`, `.read()`) subscribes the component, writing it (`count += 1`, `.set()`,
`.write()`) re-renders subscribers. First seen: entry 10. Dioxus guide, "State".

<a id="serverfn"></a>
### Server function
`#[get("/path")]` / `#[post("/path")]` on an `async fn` returning `Result<T, ServerFnError>`
generates an HTTP endpoint on the server and, on the client, a function with the same signature that
calls it. The body only exists in the server binary, so it may use server-only crates. Arguments and
results are serialised with serde. Server-only extractors can be listed in the attribute
(`#[post("/x", headers: HeaderMap)]`). Because any client can call the endpoint, it must do its own
authentication and permission checks (blueprint §3). First seen: entry 10. Dioxus guide, "Fullstack".

<a id="hydration"></a>
### SSR and hydration
Server-side rendering (SSR) produces the initial HTML on the server, so the page is visible before
any wasm loads. Hydration is the client taking over that HTML: the wasm app renders the same tree,
attaches event handlers and continues interactively. The first client render must match the server
HTML; `use_server_future` achieves this by sending the server's data with the page. Browser-only
APIs (e.g. `localStorage`) belong in `use_effect`, which runs after hydration.
First seen: entry 10. Dioxus guide, "Hydration".

<a id="extractor"></a>
### Axum extractor
A type implementing `FromRequest` / `FromRequestParts` that Axum builds from the incoming request
and passes to a handler parameter: `Extension<T>` (a value stored on the request), `HeaderMap`,
`Json<T>`, `State<S>`. If extraction fails, the handler is not called and an error response is
returned. First seen: entry 8. docs.rs/axum `extract`.

<a id="middleware"></a>
### Middleware and Tower layers
A layer wraps a service (handler) with extra behaviour before and/or after it runs: logging,
request ids, inserting state, authentication. In Axum, `router.layer(L)` wraps all routes added so
far; with several `.layer` calls, the last one added is the outermost and sees the request first.
`axum::middleware::from_fn` turns an async function `(extractors…, Request, Next) -> Response` into
a layer; it calls `next.run(req)` to continue. First seen: entry 8. docs.rs/axum `middleware`,
docs.rs/tower-http.

<a id="pool"></a>
### Connection pool
Opening a database connection is expensive, so a pool keeps a bounded set of open connections and
lends one to each query. `PgPool` is cheap to clone (shared handle); `max_connections` caps
concurrent connections, and `acquire_timeout` bounds how long a query waits for a free one.
First seen: entry 5. docs.rs/sqlx `Pool`.

<a id="migrations"></a>
### Database migrations
Versioned SQL files that evolve the schema in order. SQLx records applied versions in
`_sqlx_migrations`, applies only new ones, and refuses to run if an applied file was edited
(checksum). This project names files by timestamp (`just migration <name>`), so order equals
creation order. Never edit an applied migration; add a new one. First seen: entry 5.

<a id="sqlx-macros"></a>
### SQLx query macros and offline mode
`query!`, `query_as!` and `query_scalar!` send the SQL to Postgres during compilation to check
syntax, table and column names, and to infer Rust types for parameters and results. Without a
database (CI, Docker, wasm-free builds), `SQLX_OFFLINE=true` makes them read the saved results in
`.sqlx/`, created by `cargo sqlx prepare` (`just prepare`). Column overrides such as `AS "one!"`
force non-null. The non-macro forms (`sqlx::query("...")`) skip compile-time checks.
First seen: entry 6. docs.rs/sqlx `macro.query`.

<a id="tracing"></a>
### `tracing`
Structured logging: events (`tracing::info!(field = value, "message")`) and spans (a context such as
one HTTP request) carry key-value fields. `%x` records a field with `Display`, `?x` with `Debug`.
All events inside a span inherit its fields, which is how `request_id` appears on every log line of
a request. First seen: entry 6. docs.rs/tracing.

<a id="fluent"></a>
### Fluent (`.ftl`) translations
Project Fluent's message format: each line `key = text` defines a message; `{ $name }` inserts an
argument; selectors `{ $count -> [one] ... *[other] ... }` choose a variant by CLDR plural category
(languages differ: English has `one`/`other`, others have up to six). The default variant `*[...]`
is mandatory. Arguments are wrapped in invisible Unicode isolation marks (U+2068/U+2069) so mixed
left-to-right and right-to-left text renders correctly. In this project `dioxus-i18n` loads the
files and `t!("key", name: value)` translates. First seen: entry 13. projectfluent.org.

<a id="context"></a>
### Context (Dioxus)
A value provided by a component (`use_context_provider`, or a library hook such as `use_init_i18n`)
that any descendant can obtain (`use_context`, `i18n()`) without passing it through props. When the
provided value is a signal, readers re-render on change. First seen: entry 15. Dioxus guide, "Context".

<a id="cookie"></a>
### HTTP cookies
A server sets a cookie with a `Set-Cookie` response header; the browser stores it and sends it back
in the `Cookie` request header to matching paths. Attributes: `Path`, `Max-Age` (lifetime in
seconds), `SameSite=Lax` (not sent on cross-site sub-requests, a CSRF defence), `HttpOnly`
(invisible to JavaScript; used for the session cookie in M1), `Secure` (HTTPS only). The `locale`
cookie is not secret, so it is not `HttpOnly`. First seen: entry 16. MDN, "Using HTTP cookies".

