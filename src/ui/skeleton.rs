use dioxus::prelude::*;
use tw_merge::tw_merge;

/// shadcn `skeleton.tsx`: a pulsing placeholder block. (Rust/UI used `bg-muted`; the reference
/// uses `bg-accent`, and `style` is needed for `--skeleton-width`.)
#[component]
pub fn Skeleton(
    #[props(into, optional)] class: Option<String>,
    #[props(into, optional)] style: Option<String>,
) -> Element {
    let merged_class = tw_merge!("animate-pulse rounded-md bg-accent", class.as_deref().unwrap_or(""));

    rsx! {
        div { "data-slot": "skeleton", class: "{merged_class}", style: style.unwrap_or_default() }
    }
}
