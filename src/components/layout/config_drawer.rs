//! `config-drawer.tsx`: theme, sidebar variant, layout, direction. P5 adds the reference's
//! illustrated option cards; this version has the same options and behaviour.
use dioxus::prelude::*;
use dioxus_i18n::t;
use tw_merge::tw_merge;

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
                Section { title: t!("theme-label"),
                    Opt { label: t!("theme-system"), active: (ui.theme)() == Theme::System, onclick: move |_| ui.set_theme(Theme::System) }
                    Opt { label: t!("theme-light"), active: (ui.theme)() == Theme::Light, onclick: move |_| ui.set_theme(Theme::Light) }
                    Opt { label: t!("theme-dark"), active: (ui.theme)() == Theme::Dark, onclick: move |_| ui.set_theme(Theme::Dark) }
                }
                Section { title: t!("config-sidebar"),
                    Opt { label: t!("config-inset"), active: (ui.variant)() == Variant::Inset, onclick: move |_| ui.set_variant(Variant::Inset) }
                    Opt { label: t!("config-floating"), active: (ui.variant)() == Variant::Floating, onclick: move |_| ui.set_variant(Variant::Floating) }
                    Opt { label: t!("config-sidebar-plain"), active: (ui.variant)() == Variant::Sidebar, onclick: move |_| ui.set_variant(Variant::Sidebar) }
                }
                Section { title: t!("config-layout"),
                    Opt { label: t!("config-layout-default"), active: layout == "default", onclick: move |_| ui.set_sidebar_open(true) }
                    Opt { label: t!("config-layout-compact"), active: layout == "icon", onclick: move |_| { ui.set_sidebar_open(false); ui.set_collapsible(Collapsible::Icon) } }
                    Opt { label: t!("config-layout-full"), active: layout == "offcanvas", onclick: move |_| { ui.set_sidebar_open(false); ui.set_collapsible(Collapsible::Offcanvas) } }
                }
                Section { title: t!("config-direction"),
                    Opt { label: t!("config-ltr"), active: (ui.dir)() == Dir::Ltr, onclick: move |_| ui.set_dir(Dir::Ltr) }
                    Opt { label: t!("config-rtl"), active: (ui.dir)() == Dir::Rtl, onclick: move |_| ui.set_dir(Dir::Rtl) }
                }
            }
            div { class: "mt-auto flex flex-col gap-2 p-4",
                Button { variant: ButtonVariant::Destructive, onclick: move |_| ui.reset(), {t!("config-reset")} }
            }
        }
    }
}

#[component]
fn Section(title: String, children: Element) -> Element {
    rsx! {
        div {
            div { class: "mb-2 text-sm font-semibold text-muted-foreground", "{title}" }
            div { class: "grid w-full max-w-md grid-cols-3 gap-4", {children} }
        }
    }
}

#[component]
fn Opt(label: String, active: bool, onclick: EventHandler<MouseEvent>) -> Element {
    rsx! {
        button {
            r#type: "button",
            "aria-pressed": "{active}",
            class: tw_merge!(
                "relative flex h-16 items-end rounded-md border p-2 text-start text-xs transition hover:bg-accent",
                if active { "border-primary ring-1 ring-primary" } else { "" }
            ),
            onclick: move |e| onclick.call(e),
            if active { Icon { icon: IconType::CircleCheck, class: "absolute top-1.5 end-1.5 size-4 fill-primary text-background" } }
            "{label}"
        }
    }
}
