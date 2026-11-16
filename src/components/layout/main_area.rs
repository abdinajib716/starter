use dioxus::prelude::*;
use tw_merge::tw_merge;

/// `main.tsx`: page content area; `fixed` fills the viewport, `fluid` drops the max width.
#[component]
pub fn Main(
    #[props(default)] fixed: bool,
    #[props(default)] fluid: bool,
    #[props(into, optional)] class: Option<String>,
    children: Element,
) -> Element {
    let class = tw_merge!(
        "px-4 py-6",
        if fixed {
            "flex grow flex-col overflow-hidden"
        } else {
            ""
        },
        if fluid {
            ""
        } else {
            "@7xl/content:mx-auto @7xl/content:w-full @7xl/content:max-w-7xl"
        },
        class.as_deref().unwrap_or("")
    );
    rsx! {
        main { id: "content", "data-layout": if fixed { "fixed" } else { "auto" }, class: "{class}", {children} }
    }
}
