//! Dropdown menu (shadcn `dropdown-menu.tsx` look) positioned with `position: fixed`
//! from the trigger's measured rectangle, so it is never clipped by scrolling containers
//! (sidebar). Pure Dioxus state: no inline scripts.
//!
//! Overlays render in place (Radix portals them to `<body>`), so they use z-indexes above
//! the page header (`z-50`): sheet 60, menu 70, dialog 80, tooltip 90.
use std::rc::Rc;

use dioxus::prelude::*;
use tw_merge::tw_merge;

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum Side {
    #[default]
    Bottom,
    Right,
}

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum Align {
    #[default]
    Start,
    End,
}

/// `trigger` is rendered inside a wrapper that carries `data-state="open|closed"`, so
/// trigger styles can use `in-data-[state=open]:…`.
#[component]
pub fn Menu(
    trigger: Element,
    children: Element,
    #[props(default)] side: Side,
    #[props(default)] align: Align,
    #[props(default = 4.0)] side_offset: f64,
    /// Make the content at least as wide as the trigger (`w-(--radix-…-trigger-width)`).
    #[props(default)]
    match_trigger_width: bool,
    #[props(into, optional)] class: Option<String>,
    #[props(into, optional)] trigger_class: Option<String>,
    /// `false` for content with inputs (combobox search, calendar): close via selection only.
    #[props(default = true)]
    close_on_click: bool,
    /// Lets the parent close the menu (e.g. after a combobox selection).
    #[props(optional)]
    open_signal: Option<Signal<bool>>,
) -> Element {
    let own_open = use_signal(|| false);
    let mut open = open_signal.unwrap_or(own_open);
    let mut anchor = use_signal(|| None::<Rc<MountedData>>);
    let mut style = use_signal(String::new);

    let rtl = try_use_context::<crate::state::Ui>()
        .is_some_and(|ui| (ui.dir)() == crate::state::Dir::Rtl);
    let toggle = move |_| async move {
        if open() {
            open.set(false);
            return;
        }
        let Some(el) = anchor() else { return };
        let Ok(r) = el.get_client_rect().await else {
            return;
        };
        let (top, left, bottom, right, width) =
            (r.min_y(), r.min_x(), r.max_y(), r.max_x(), r.width());
        let vh = viewport_height();
        let mut css = match side {
            // Open upwards when the trigger sits in the lower part of the screen.
            Side::Bottom if bottom > vh * 0.6 => {
                format!("top:{}px;transform:translateY(-100%);", top - side_offset)
            }
            Side::Bottom => format!("top:{}px;", bottom + side_offset),
            Side::Right => match align {
                Align::Start => format!("top:{top}px;"),
                Align::End => format!("top:{bottom}px;transform:translateY(-100%);"),
            },
        };
        // Start / end and "right" are logical: in RTL they mirror (Radix uses `dir`).
        let x = match (side, align, rtl) {
            (Side::Right, _, false) => format!("left:{}px;", right + side_offset),
            (Side::Right, _, true) => format!("left:{}px;translate:-100% 0;", left - side_offset),
            (Side::Bottom, Align::Start, false) | (Side::Bottom, Align::End, true) => {
                format!("left:{left}px;")
            }
            (Side::Bottom, Align::Start, true) | (Side::Bottom, Align::End, false) => {
                format!("left:{right}px;translate:-100% 0;")
            }
        };
        css.push_str(&x);
        if match_trigger_width {
            css.push_str(&format!("min-width:{width}px;"));
        }
        style.set(css);
        open.set(true);
    };

    let content = tw_merge!(
        "fixed z-[71] min-w-32 overflow-x-hidden overflow-y-auto rounded-md border bg-popover p-1 text-popover-foreground shadow-md animate-in fade-in-0 zoom-in-95",
        class.as_deref().unwrap_or("")
    );
    let state = if open() { "open" } else { "closed" };

    rsx! {
        div {
            class: trigger_class.unwrap_or_default(),
            "data-state": state,
            onmounted: move |e| anchor.set(Some(e.data())),
            onclick: toggle,
            {trigger}
        }
        if open() {
            // Click outside closes; the layer sits under the content.
            div { class: "fixed inset-0 z-[70]", onclick: move |_| open.set(false) }
            div {
                role: "menu",
                tabindex: "-1",
                class: "{content}",
                style: "{style}",
                onmounted: move |e| async move { let _ = e.data().set_focus(true).await; },
                onkeydown: move |e| if e.key() == Key::Escape { open.set(false) },
                // Any item click closes the menu (links navigate, buttons run their handler).
                onclick: move |_| if close_on_click { open.set(false) },
                {children}
            }
        }
    }
}

fn viewport_height() -> f64 {
    #[cfg(target_arch = "wasm32")]
    {
        web_sys::window()
            .and_then(|w| w.inner_height().ok())
            .and_then(|v| v.as_f64())
            .unwrap_or(800.0)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        800.0
    }
}

pub const ITEM: &str = "relative flex w-full cursor-default items-center gap-2 rounded-sm px-2 py-1.5 text-sm outline-hidden select-none hover:bg-accent hover:text-accent-foreground focus:bg-accent focus:text-accent-foreground data-disabled:pointer-events-none data-disabled:opacity-50 [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4 [&_svg:not([class*='text-'])]:text-muted-foreground";
pub const ITEM_DESTRUCTIVE: &str = "text-destructive hover:bg-destructive/10 hover:text-destructive focus:bg-destructive/10 focus:text-destructive dark:hover:bg-destructive/20 *:[svg]:text-destructive!";

/// A clickable row. `destructive` = shadcn `variant="destructive"`.
#[component]
pub fn MenuItem(
    #[props(optional)] onclick: Option<EventHandler<MouseEvent>>,
    #[props(default)] destructive: bool,
    #[props(into, optional)] class: Option<String>,
    children: Element,
) -> Element {
    let class = tw_merge!(
        ITEM,
        if destructive { ITEM_DESTRUCTIVE } else { "" },
        class.as_deref().unwrap_or("")
    );
    rsx! {
        button {
            r#type: "button",
            role: "menuitem",
            class: "{class}",
            onclick: move |e| if let Some(h) = &onclick { h.call(e) },
            {children}
        }
    }
}

#[component]
pub fn MenuLabel(#[props(into, optional)] class: Option<String>, children: Element) -> Element {
    let class = tw_merge!(
        "px-2 py-1.5 text-sm font-medium",
        class.as_deref().unwrap_or("")
    );
    rsx! { div { class: "{class}", {children} } }
}

#[component]
pub fn MenuSeparator() -> Element {
    rsx! { div { role: "separator", class: "-mx-1 my-1 h-px bg-border" } }
}

#[component]
pub fn MenuGroup(children: Element) -> Element {
    rsx! { div { role: "group", {children} } }
}

#[component]
pub fn MenuShortcut(#[props(into, optional)] class: Option<String>, children: Element) -> Element {
    let class = tw_merge!(
        "ms-auto text-xs tracking-widest text-muted-foreground",
        class.as_deref().unwrap_or("")
    );
    rsx! { span { class: "{class}", {children} } }
}
