//! Body of the "type DELETE to confirm" multi-delete dialogs (`*-multi-delete-dialog.tsx`)
//! and the bulk-action icon button style.
use dioxus::prelude::*;
use dioxus_i18n::t;

/// Outline `size-8` icon button used in the bulk-actions toolbar.
pub const ICON_BTN: &str = "inline-flex size-8 items-center justify-center rounded-md border bg-background shadow-xs transition-all hover:bg-accent hover:text-accent-foreground dark:border-input dark:bg-input/30 dark:hover:bg-input/50 [&_svg]:size-4";
/// Destructive `size-8` icon button.
pub const ICON_BTN_DESTRUCTIVE: &str = "inline-flex size-8 items-center justify-center rounded-md bg-destructive text-white shadow-xs hover:bg-destructive/90 dark:bg-destructive/60 [&_svg]:size-4";

/// shadcn `Alert variant="destructive"` with the "Warning!" text of the delete dialogs.
#[component]
pub fn DeleteWarning() -> Element {
    rsx! {
        div { role: "alert", class: "relative grid w-full grid-cols-[0_1fr] items-start gap-y-0.5 rounded-lg border bg-card px-4 py-3 text-sm text-destructive",
            div { class: "col-start-2 line-clamp-1 min-h-4 font-medium tracking-tight", {t!("dt-warning")} }
            div { class: "col-start-2 grid justify-items-start gap-1 text-sm text-destructive/90", p { {t!("dt-warning-desc")} } }
        }
    }
}

/// Body shared by the tasks / users multi-delete dialogs.
#[component]
pub fn MultiDeleteBody(word: Signal<String>, entity_plural: String) -> Element {
    let mut word = word;
    rsx! {
        div { class: "space-y-4",
            p { class: "mb-2",
                {crate::ta!("dt-multi-delete-sure", entity: entity_plural)}
                br {}
                {t!("dt-cannot-undo")}
            }
            label { class: "my-4 flex flex-col items-start gap-1.5 text-sm font-medium text-foreground",
                span { {crate::ta!("dt-confirm-typing", word: "DELETE")} }
                input {
                    class: "flex h-9 w-full min-w-0 rounded-md border border-input bg-transparent px-3 py-1 text-base font-normal shadow-xs outline-none placeholder:text-muted-foreground focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 md:text-sm dark:bg-input/30",
                    value: word(),
                    placeholder: crate::ta!("dt-type-to-confirm", word: "DELETE"),
                    autofocus: true,
                    oninput: move |e| word.set(e.value()),
                }
            }
            DeleteWarning {}
        }
    }
}
