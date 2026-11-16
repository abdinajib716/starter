use dioxus::prelude::*;
use dioxus_i18n::t;

use crate::app::Route;

/// Catch-all route (§12 `*`). Reached by signed-in users only: the server guard sends
/// anonymous page loads to `/login` first.
#[component]
pub fn NotFound(segments: Vec<String>) -> Element {
    let path = format!("/{}", segments.join("/"));
    rsx! {
        div { class: "flex min-h-full flex-col items-center justify-center gap-3 bg-background p-8 text-foreground",
            h1 { class: "text-2xl font-semibold", {t!("notfound-title")} }
            p { class: "text-muted-foreground font-mono text-sm", "{path}" }
            Link { to: Route::Dashboard {}, class: "underline", {t!("notfound-back")} }
        }
    }
}
