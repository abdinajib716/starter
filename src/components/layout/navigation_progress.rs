//! `navigation-progress.tsx`: a 2px `--muted-foreground` bar at the top of the window on every
//! page change. Dioxus routes render immediately (data loads behind Suspense), so the bar runs a
//! fixed start → complete animation per change instead of following a pending state.
use dioxus::prelude::*;

use crate::app::Route;

#[component]
pub fn NavigationProgress() -> Element {
    let route = use_route::<Route>();
    // Path only: query changes (typing in a table filter) are not page changes.
    let path = route
        .to_string()
        .split('?')
        .next()
        .unwrap_or_default()
        .to_string();
    let mut runs = use_signal(|| 0u32);
    let mut last = use_signal(|| path.clone());
    use_effect(use_reactive!(|path| {
        if *last.peek() != path {
            last.set(path);
            *runs.write() += 1;
        }
    }));
    let n = runs();
    if n == 0 {
        return rsx! {};
    }
    rsx! {
        div { key: "{n}", class: "pointer-events-none fixed inset-x-0 top-0 z-[100] h-0.5",
            div { class: "h-full bg-muted-foreground shadow-[0_0_10px_var(--muted-foreground),0_0_5px_var(--muted-foreground)] animate-[nav-progress_600ms_ease-out_forwards]" }
        }
    }
}
