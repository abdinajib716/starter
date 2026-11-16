use dioxus::prelude::*;

/// A tab that cannot be selected (`<TabsTrigger disabled>`); Rust/UI's trigger has no
/// `disabled`. Same classes as the reference trigger.
#[component]
pub fn TabsTriggerDisabled(children: Element) -> Element {
    rsx! {
        button {
            r#type: "button",
            disabled: true,
            class: "inline-flex h-[calc(100%-1px)] flex-1 items-center justify-center gap-1.5 rounded-md border border-transparent px-2 py-1 text-sm font-medium whitespace-nowrap text-foreground disabled:pointer-events-none disabled:opacity-50 dark:text-muted-foreground",
            {children}
        }
    }
}
