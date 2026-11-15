//! Starter entry point. Layout follows blueprint §31.2.
//! - `web` build: hydrates the Dioxus app in the browser.
//! - `server` build: custom Axum router (SSR + server fns + plain routes), see `server::router`.

mod api;
mod app;
mod components;
mod domain;
// Rust/UI library code (copied verbatim); not every component/hook is used yet.
#[allow(dead_code, unused_imports, clippy::all)]
#[rustfmt::skip]
mod hooks;
mod pages;
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
