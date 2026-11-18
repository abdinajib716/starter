//! `config-drawer.tsx`: theme, sidebar variant, layout, direction, each as a radio group of
//! illustrated cards (`assets/custom/*`) with a per-section reset.
use crate::t;
use dioxus::prelude::*;

use crate::components::icons::custom::{Custom, CustomIcon};
use crate::components::ui_extra::icon::{Icon, IconType};
use crate::components::ui_extra::sheet::SideSheet;
use crate::state::{Collapsible, Dir, Theme, Ui, Variant};
use crate::ui::button::{Button, ButtonSize, ButtonVariant};

#[component]
pub fn ConfigDrawer() -> Element {
    let mut open = use_signal(|| false);
    let ui = Ui::get();
    let layout = if (ui.sidebar_open)() {
        "default"
    } else {
        (ui.collapsible)().as_str()
    };
    let theme = (ui.theme)();
    let variant = (ui.variant)();
    let dir = (ui.dir)();

    rsx! {
        Button {
            size: ButtonSize::Icon,
            variant: ButtonVariant::Ghost,
            aria_label: t!("config-open"),
            class: "rounded-full",
            onclick: move |_| open.set(true),
            Icon { icon: IconType::Settings }
        }
        SideSheet { open, class: "flex flex-col",
            div { class: "flex flex-col gap-1.5 p-4 pb-0 text-start",
                h2 { class: "font-semibold text-foreground", {t!("config-title")} }
                p { class: "text-sm text-muted-foreground", {t!("config-desc")} }
            }
            div { class: "space-y-6 overflow-y-auto px-4",
                div {
                    SectionTitle { title: t!("theme-label"), show_reset: theme != Theme::default(), reset_label: t!("config-reset-theme"), on_reset: move |_| ui.set_theme(Theme::default()) }
                    div { role: "radiogroup", "aria-label": t!("config-select-theme"), class: "grid w-full max-w-md grid-cols-3 gap-4",
                        Card { label: t!("theme-system"), icon: Custom::ThemeSystem, is_theme: true, checked: theme == Theme::System, onselect: move |_| ui.set_theme(Theme::System) }
                        Card { label: t!("theme-light"), icon: Custom::ThemeLight, is_theme: true, checked: theme == Theme::Light, onselect: move |_| ui.set_theme(Theme::Light) }
                        Card { label: t!("theme-dark"), icon: Custom::ThemeDark, is_theme: true, checked: theme == Theme::Dark, onselect: move |_| ui.set_theme(Theme::Dark) }
                    }
                }
                div { class: "max-md:hidden",
                    SectionTitle { title: t!("config-sidebar"), show_reset: variant != Variant::default(), reset_label: t!("config-reset-sidebar"), on_reset: move |_| ui.set_variant(Variant::default()) }
                    div { role: "radiogroup", "aria-label": t!("config-select-sidebar"), class: "grid w-full max-w-md grid-cols-3 gap-4",
                        Card { label: t!("config-inset"), icon: Custom::SidebarInset, checked: variant == Variant::Inset, onselect: move |_| ui.set_variant(Variant::Inset) }
                        Card { label: t!("config-floating"), icon: Custom::SidebarFloating, checked: variant == Variant::Floating, onselect: move |_| ui.set_variant(Variant::Floating) }
                        Card { label: t!("config-sidebar-plain"), icon: Custom::SidebarSidebar, checked: variant == Variant::Sidebar, onselect: move |_| ui.set_variant(Variant::Sidebar) }
                    }
                }
                div { class: "max-md:hidden",
                    SectionTitle {
                        title: t!("config-layout"),
                        show_reset: layout != "default",
                        reset_label: t!("config-reset-layout"),
                        on_reset: move |_| { ui.set_sidebar_open(true); ui.set_collapsible(Collapsible::default()) },
                    }
                    div { role: "radiogroup", "aria-label": t!("config-select-layout"), class: "grid w-full max-w-md grid-cols-3 gap-4",
                        Card { label: t!("config-layout-default"), icon: Custom::LayoutDefault, checked: layout == "default", onselect: move |_| ui.set_sidebar_open(true) }
                        Card { label: t!("config-layout-compact"), icon: Custom::LayoutCompact, checked: layout == "icon", onselect: move |_| { ui.set_sidebar_open(false); ui.set_collapsible(Collapsible::Icon) } }
                        Card { label: t!("config-layout-full"), icon: Custom::LayoutFull, checked: layout == "offcanvas", onselect: move |_| { ui.set_sidebar_open(false); ui.set_collapsible(Collapsible::Offcanvas) } }
                    }
                }
                div {
                    SectionTitle { title: t!("config-direction"), show_reset: dir != Dir::default(), reset_label: t!("config-reset-dir"), on_reset: move |_| ui.set_dir(Dir::default()) }
                    div { role: "radiogroup", "aria-label": t!("config-select-dir"), class: "grid w-full max-w-md grid-cols-3 gap-4",
                        Card { label: t!("config-ltr"), icon: Custom::Dir, checked: dir == Dir::Ltr, onselect: move |_| ui.set_dir(Dir::Ltr) }
                        Card { label: t!("config-rtl"), icon: Custom::Dir, rtl: true, checked: dir == Dir::Rtl, onselect: move |_| ui.set_dir(Dir::Rtl) }
                    }
                }
            }
            div { class: "mt-auto flex flex-col gap-2 p-4",
                Button { variant: ButtonVariant::Destructive, class: "w-full", aria_label: t!("config-reset-all"), onclick: move |_| ui.reset(), {t!("config-reset")} }
            }
        }
    }
}

#[component]
fn SectionTitle(
    title: String,
    show_reset: bool,
    reset_label: String,
    on_reset: EventHandler<()>,
) -> Element {
    rsx! {
        div { class: "mb-2 flex items-center gap-2 text-sm font-semibold text-muted-foreground",
            "{title}"
            if show_reset {
                button {
                    r#type: "button",
                    class: "inline-flex size-4 items-center justify-center rounded-full bg-secondary text-secondary-foreground shadow-xs hover:bg-secondary/80",
                    "aria-label": reset_label,
                    onclick: move |_| on_reset.call(()),
                    Icon { icon: IconType::RotateCcw, class: "size-3" }
                }
            }
        }
    }
}

/// `RadioGroupItem`: the card, a check badge when selected, and the label under it.
#[component]
fn Card(
    label: String,
    icon: Custom,
    checked: bool,
    onselect: EventHandler<()>,
    #[props(default)] is_theme: bool,
    #[props(default)] rtl: bool,
) -> Element {
    let state = if checked { "checked" } else { "unchecked" };
    rsx! {
        button {
            r#type: "button",
            role: "radio",
            "aria-checked": "{checked}",
            "data-state": state,
            "aria-label": crate::ta!("config-select-option", option: label.to_lowercase()),
            class: "group outline-none transition duration-200 ease-in",
            onclick: move |_| onselect.call(()),
            div { class: "relative rounded-[6px] ring-[1px] ring-border group-focus-visible:ring-2 group-data-[state=checked]:shadow-2xl group-data-[state=checked]:ring-primary",
                Icon { icon: IconType::CircleCheck, class: "absolute top-0 right-0 size-6 translate-x-1/2 -translate-y-1/2 fill-primary stroke-white group-data-[state=unchecked]:hidden" }
                CustomIcon {
                    icon,
                    rtl,
                    class: if is_theme { "" } else { "fill-primary stroke-primary group-data-[state=unchecked]:fill-muted-foreground group-data-[state=unchecked]:stroke-muted-foreground" },
                }
            }
            div { class: "mt-1 text-xs", "{label}" }
        }
    }
}
