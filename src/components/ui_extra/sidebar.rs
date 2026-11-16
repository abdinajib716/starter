//! Port of shadcn `components/ui/sidebar.tsx` (as used by shadcn-admin): provider with
//! cookie-persisted state and Ctrl/Cmd+B, desktop sidebar (offcanvas / icon / none;
//! sidebar / floating / inset), mobile sheet, rail, inset, groups, menus, sub-menus,
//! tooltips in icon mode. Classes follow the reference; `inset-s/e` → `start/end`
//! (Tailwind 4.1), `data-[active=true]` → `aria-[current=page]` (Dioxus `Link`).
use std::rc::Rc;

use dioxus::prelude::*;
use tw_merge::tw_merge;

use super::icon::{Icon, IconType};
use super::sheet::{SheetSide, SideSheet};
use crate::state::{Collapsible, Ui, Variant};
use crate::ui::button::{Button, ButtonSize, ButtonVariant};

#[derive(Clone, Copy)]
pub struct SidebarCtx {
    /// Desktop expanded/collapsed (cookie `sidebar_state`, via `Ui`).
    pub ui: Ui,
    pub open_mobile: Signal<bool>,
    pub is_mobile: Signal<bool>,
}

impl SidebarCtx {
    pub fn get() -> Self {
        use_context::<SidebarCtx>()
    }
    pub fn open(&self) -> bool {
        (self.ui.sidebar_open)()
    }
    pub fn collapsed(&self) -> bool {
        !self.open()
    }
    pub fn toggle(mut self) {
        if (self.is_mobile)() {
            let v = (self.open_mobile)();
            self.open_mobile.set(!v);
        } else {
            self.ui.set_sidebar_open(!self.open());
        }
    }
    pub fn close_mobile(mut self) {
        self.open_mobile.set(false);
    }
}

#[component]
pub fn SidebarProvider(
    #[props(into, optional)] class: Option<String>,
    children: Element,
) -> Element {
    let ui = Ui::get();
    let is_mobile = use_signal(|| false);
    let ctx = use_context_provider(|| SidebarCtx {
        ui,
        open_mobile: Signal::new(false),
        is_mobile,
    });

    // useIsMobile (<768px) + Ctrl/Cmd+B. Browser listeners report back through `dioxus.send`.
    use_effect(move || {
        let mut is_mobile = is_mobile;
        spawn(async move {
            let mut ev = document::eval(
                "const mq = matchMedia('(max-width: 767px)'); dioxus.send(['m', mq.matches]);
                 mq.addEventListener('change', e => dioxus.send(['m', e.matches]));
                 window.addEventListener('keydown', e => {
                   if (e.key === 'b' && (e.metaKey || e.ctrlKey)) { e.preventDefault(); dioxus.send(['b', true]); }
                 });",
            );
            while let Ok((kind, value)) = ev.recv::<(String, bool)>().await {
                match kind.as_str() {
                    "m" => is_mobile.set(value),
                    _ => ctx.toggle(),
                }
            }
        });
    });

    let class = tw_merge!(
        "group/sidebar-wrapper flex min-h-svh w-full has-data-[variant=inset]:bg-sidebar",
        class.as_deref().unwrap_or("")
    );
    rsx! {
        div {
            "data-slot": "sidebar-wrapper",
            style: "--sidebar-width: 16rem; --sidebar-width-icon: 3rem;",
            class: "{class}",
            {children}
        }
    }
}

#[component]
pub fn Sidebar(
    #[props(default)] variant: Variant,
    #[props(default)] collapsible: Collapsible,
    #[props(into, optional)] class: Option<String>,
    /// Same content again for the mobile sheet (a rendered node can only be mounted once).
    mobile: Element,
    children: Element,
) -> Element {
    let ctx = SidebarCtx::get();
    let class = class.unwrap_or_default();

    if collapsible == Collapsible::None {
        let c = tw_merge!(
            "flex h-full w-(--sidebar-width) flex-col bg-sidebar text-sidebar-foreground",
            class
        );
        return rsx! { div { "data-slot": "sidebar", class: "{c}", {children} } };
    }

    let state = if ctx.open() { "expanded" } else { "collapsed" };
    let data_collapsible = if ctx.collapsed() {
        collapsible.as_str()
    } else {
        ""
    };
    let floating_or_inset = matches!(variant, Variant::Floating | Variant::Inset);
    let gap = tw_merge!(
        "relative w-(--sidebar-width) bg-transparent transition-[width] duration-200 ease-linear group-data-[collapsible=offcanvas]:w-0 group-data-[side=right]:rotate-180",
        if floating_or_inset {
            "group-data-[collapsible=icon]:w-[calc(var(--sidebar-width-icon)+(--spacing(4)))]"
        } else {
            "group-data-[collapsible=icon]:w-(--sidebar-width-icon)"
        }
    );
    let container = tw_merge!(
        // `fixed z-10` is a stacking context: while a menu or tooltip is open inside, lift the
        // whole container above the page header (z-50), since overlays here are not portaled.
        "fixed inset-y-0 z-10 hidden h-svh w-(--sidebar-width) transition-[inset-inline,width] duration-200 ease-linear md:flex start-0 group-data-[collapsible=offcanvas]:-start-(--sidebar-width) has-[[role=menu]]:z-[72] has-[[role=tooltip]]:z-[72]",
        if floating_or_inset {
            "p-2 group-data-[collapsible=icon]:w-[calc(var(--sidebar-width-icon)+(--spacing(4))+2px)]"
        } else {
            "group-data-[collapsible=icon]:w-(--sidebar-width-icon) group-data-[side=left]:border-e"
        },
        class.clone()
    );
    let open_mobile = ctx.open_mobile;

    rsx! {
        // Desktop (≥ md). Hidden on small screens by CSS, so SSR needs no screen size.
        div {
            class: "group peer hidden text-sidebar-foreground md:block",
            "data-state": state,
            "data-collapsible": data_collapsible,
            "data-variant": variant.as_str(),
            "data-side": "left",
            "data-slot": "sidebar",
            div { "data-slot": "sidebar-gap", class: "{gap}" }
            div { "data-slot": "sidebar-container", class: "{container}",
                div {
                    "data-sidebar": "sidebar",
                    "data-slot": "sidebar-inner",
                    class: "flex h-full w-full flex-col bg-sidebar group-data-[variant=floating]:rounded-lg group-data-[variant=floating]:border group-data-[variant=floating]:border-sidebar-border group-data-[variant=floating]:shadow-sm",
                    {children}
                }
            }
        }
        // Mobile (< md): the same content in a sheet.
        if open_mobile() {
            SideSheet {
                open: open_mobile,
                side: SheetSide::Start,
                class: "w-(--sidebar-width) gap-0 bg-sidebar p-0 text-sidebar-foreground",
                style: "--sidebar-width: 18rem;",
                div { class: "sr-only", h2 { "Sidebar" } }
                div {
                    "data-sidebar": "sidebar",
                    "data-mobile": "true",
                    class: "flex h-full w-full flex-col",
                    {mobile}
                }
            }
        }
    }
}

#[component]
pub fn SidebarTrigger(
    #[props(default = ButtonVariant::Ghost)] variant: ButtonVariant,
    #[props(into, optional)] class: Option<String>,
) -> Element {
    let ctx = SidebarCtx::get();
    let class = tw_merge!("size-7", class.as_deref().unwrap_or(""));
    rsx! {
        Button {
            variant,
            size: ButtonSize::Icon,
            class,
            aria_label: "Toggle Sidebar",
            onclick: move |_| ctx.toggle(),
            Icon { icon: IconType::PanelLeft }
            span { class: "sr-only", "Toggle Sidebar" }
        }
    }
}

#[component]
pub fn SidebarRail() -> Element {
    let ctx = SidebarCtx::get();
    rsx! {
        button {
            "data-sidebar": "rail",
            "aria-label": "Toggle Sidebar",
            tabindex: "-1",
            title: "Toggle Sidebar",
            onclick: move |_| ctx.toggle(),
            class: "absolute inset-y-0 z-20 hidden w-4 -translate-x-1/2 transition-all ease-linear group-data-[side=left]:-end-4 after:absolute after:inset-y-0 after:start-1/2 after:w-0.5 hover:after:bg-sidebar-border sm:flex in-data-[side=left]:cursor-w-resize [[data-side=left][data-state=collapsed]_&]:cursor-e-resize group-data-[collapsible=offcanvas]:translate-x-0 group-data-[collapsible=offcanvas]:after:start-full hover:group-data-[collapsible=offcanvas]:bg-sidebar [[data-side=left][data-collapsible=offcanvas]_&]:-end-2 rtl:translate-x-1/2 rtl:in-data-[side=left]:cursor-e-resize rtl:[[data-side=left][data-state=collapsed]_&]:cursor-w-resize",
        }
    }
}

#[component]
pub fn SidebarInset(#[props(into, optional)] class: Option<String>, children: Element) -> Element {
    let class = tw_merge!(
        "relative flex w-full flex-1 flex-col bg-background md:peer-data-[variant=inset]:m-2 md:peer-data-[variant=inset]:ms-0 md:peer-data-[variant=inset]:rounded-xl md:peer-data-[variant=inset]:shadow-sm md:peer-data-[variant=inset]:peer-data-[state=collapsed]:ms-2",
        class.as_deref().unwrap_or("")
    );
    rsx! { div { "data-slot": "sidebar-inset", class: "{class}", {children} } }
}

macro_rules! simple {
    ($name:ident, $tag:ident, $slot:literal, $classes:literal) => {
        #[component]
        pub fn $name(#[props(into, optional)] class: Option<String>, children: Element) -> Element {
            let class = tw_merge!($classes, class.as_deref().unwrap_or(""));
            rsx! { $tag { "data-slot": $slot, "data-sidebar": $slot, class: "{class}", {children} } }
        }
    };
}

simple!(SidebarHeader, div, "header", "flex flex-col gap-2 p-2");
simple!(SidebarFooter, div, "footer", "flex flex-col gap-2 p-2");
simple!(
    SidebarContent,
    div,
    "content",
    "flex min-h-0 flex-1 flex-col gap-2 overflow-auto group-data-[collapsible=icon]:overflow-hidden"
);
simple!(
    SidebarGroup,
    div,
    "group",
    "relative flex w-full min-w-0 flex-col p-2"
);
simple!(
    SidebarGroupLabel,
    div,
    "group-label",
    "flex h-8 shrink-0 items-center rounded-md px-2 text-xs font-medium text-sidebar-foreground/70 ring-sidebar-ring outline-hidden transition-[margin,opacity] duration-200 ease-linear focus-visible:ring-2 [&>svg]:size-4 [&>svg]:shrink-0 group-data-[collapsible=icon]:-mt-8 group-data-[collapsible=icon]:opacity-0"
);
simple!(
    SidebarMenu,
    ul,
    "menu",
    "flex w-full min-w-0 flex-col gap-1"
);
simple!(SidebarMenuItem, li, "menu-item", "group/menu-item relative");
simple!(
    SidebarMenuSub,
    ul,
    "menu-sub",
    "mx-3.5 flex min-w-0 translate-x-px flex-col gap-1 border-s border-sidebar-border px-2.5 py-0.5 group-data-[collapsible=icon]:hidden"
);
simple!(
    SidebarMenuSubItem,
    li,
    "menu-sub-item",
    "group/menu-sub-item relative"
);

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum MenuButtonSize {
    #[default]
    Default,
    Lg,
}

const MENU_BUTTON: &str = "peer/menu-button flex w-full items-center gap-2 overflow-hidden rounded-md p-2 text-start text-sm outline-hidden ring-sidebar-ring transition-[width,height,padding] hover:bg-sidebar-accent hover:text-sidebar-accent-foreground focus-visible:ring-2 active:bg-sidebar-accent active:text-sidebar-accent-foreground disabled:pointer-events-none disabled:opacity-50 aria-disabled:pointer-events-none aria-disabled:opacity-50 aria-[current=page]:bg-sidebar-accent aria-[current=page]:font-medium aria-[current=page]:text-sidebar-accent-foreground in-data-[state=open]:hover:bg-sidebar-accent in-data-[state=open]:hover:text-sidebar-accent-foreground group-data-[collapsible=icon]:size-8! [&>span:last-child]:truncate [&>svg]:size-4 [&>svg]:shrink-0";

pub fn menu_button_class(size: MenuButtonSize, extra: &str) -> String {
    tw_merge!(
        MENU_BUTTON,
        match size {
            // The icon-mode padding is chosen here, not overridden: tw_merge does not treat
            // `p-2!`/`p-0!` (Tailwind 4 important suffix) as conflicting, so both would be emitted
            // and `p-2!` would win, pushing the 32px logo/avatar out of the 32px button.
            MenuButtonSize::Default => "h-8 text-sm group-data-[collapsible=icon]:p-2!",
            MenuButtonSize::Lg => "h-12 text-sm group-data-[collapsible=icon]:p-0!",
        },
        extra
    )
}

/// A menu button that is a link (`asChild` + `<Link>` in the reference).
#[component]
pub fn SidebarMenuLink(
    #[props(into)] to: NavigationTarget,
    #[props(default)] is_active: bool,
    #[props(into, optional)] tooltip: Option<String>,
    #[props(into, optional)] class: Option<String>,
    children: Element,
) -> Element {
    let ctx = SidebarCtx::get();
    let class = menu_button_class(MenuButtonSize::Default, class.as_deref().unwrap_or(""));
    rsx! {
        WithTooltip { text: tooltip,
            Link {
                to,
                class,
                aria_current: if is_active { "page" } else { "false" },
                onclick: move |_| ctx.close_mobile(),
                {children}
            }
        }
    }
}

/// A menu button that is a `<button>` (triggers, collapsible headers).
#[component]
pub fn SidebarMenuButton(
    #[props(default)] size: MenuButtonSize,
    #[props(default)] is_active: bool,
    #[props(into, optional)] tooltip: Option<String>,
    #[props(into, optional)] class: Option<String>,
    #[props(optional)] onclick: Option<EventHandler<MouseEvent>>,
    children: Element,
) -> Element {
    let class = menu_button_class(size, class.as_deref().unwrap_or(""));
    rsx! {
        WithTooltip { text: tooltip,
            button {
                r#type: "button",
                class,
                aria_current: if is_active { "page" } else { "false" },
                onclick: move |e| if let Some(h) = &onclick { h.call(e) },
                {children}
            }
        }
    }
}

#[component]
pub fn SidebarMenuSubLink(
    #[props(into)] to: NavigationTarget,
    #[props(default)] is_active: bool,
    children: Element,
) -> Element {
    let ctx = SidebarCtx::get();
    rsx! {
        Link {
            to,
            class: "flex h-7 min-w-0 -translate-x-px items-center gap-2 overflow-hidden rounded-md px-2 text-sm text-sidebar-foreground ring-sidebar-ring outline-hidden hover:bg-sidebar-accent hover:text-sidebar-accent-foreground focus-visible:ring-2 active:bg-sidebar-accent active:text-sidebar-accent-foreground [&>span:last-child]:truncate [&>svg]:size-4 [&>svg]:shrink-0 [&>svg]:text-inherit aria-[current=page]:bg-sidebar-accent aria-[current=page]:text-sidebar-accent-foreground group-data-[collapsible=icon]:hidden",
            aria_current: if is_active { "page" } else { "false" },
            onclick: move |_| ctx.close_mobile(),
            {children}
        }
    }
}

/// Tooltip shown to the right of a menu button, only while the desktop sidebar is collapsed
/// (reference: `hidden={state !== 'collapsed' || isMobile}`). Fixed-positioned so the
/// icon-mode content area (`overflow-hidden`) cannot clip it.
#[component]
fn WithTooltip(text: Option<String>, children: Element) -> Element {
    let ctx = SidebarCtx::get();
    let mut anchor = use_signal(|| None::<Rc<MountedData>>);
    let mut pos = use_signal(|| None::<(f64, f64)>);
    let Some(text) = text else { return children };
    let enabled = ctx.collapsed() && !(ctx.is_mobile)();

    rsx! {
        div {
            class: "w-full",
            onmounted: move |e| anchor.set(Some(e.data())),
            onmouseenter: move |_| async move {
                if !enabled { return }
                let Some(el) = anchor() else { return };
                if let Ok(r) = el.get_client_rect().await {
                    pos.set(Some((r.max_x() + 8.0, r.min_y() + r.height() / 2.0)));
                }
            },
            onmouseleave: move |_| pos.set(None),
            {children}
        }
        if let (true, Some((x, y))) = (enabled, pos()) {
            div {
                role: "tooltip",
                class: "pointer-events-none fixed z-[90] w-fit -translate-y-1/2 animate-in rounded-md bg-primary px-3 py-1.5 text-xs text-balance text-primary-foreground fade-in-0 zoom-in-95 slide-in-from-left-2",
                style: "left:{x}px;top:{y}px;",
                "{text}"
            }
        }
    }
}
