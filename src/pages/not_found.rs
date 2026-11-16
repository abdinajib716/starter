use dioxus::prelude::*;

use crate::pages::errors::{ErrorKind, ErrorView};

/// Catch-all route = the root `notFoundComponent` of the reference (`NotFoundError`).
/// Reached by signed-in users only: the server guard sends anonymous page loads to `/login`.
#[component]
pub fn NotFound(segments: Vec<String>) -> Element {
    let _ = segments;
    rsx! { ErrorView { kind: ErrorKind::NotFound } }
}
