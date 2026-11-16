//! `search.tsx` (header button) + `command-menu.tsx` (Ctrl/Cmd+K palette).
use dioxus::prelude::*;
use dioxus_i18n::t;
use tw_merge::tw_merge;

use super::nav_config::{NavItem, visible_groups};
use super::use_current_user;
use crate::components::ui_extra::icon::{Icon, IconType};
use crate::state::{SearchOpen, Theme, Ui};

#[component]
pub fn Search(#[props(into, optional)] class: Option<String>) -> Element {
    let mut open = use_context::<SearchOpen>().0;
    let class = tw_merge!(
        "group relative inline-flex h-8 w-full flex-1 items-center justify-start rounded-md border bg-muted/25 px-4 text-sm font-normal text-muted-foreground shadow-none transition-colors hover:bg-accent sm:w-40 sm:pe-12 md:flex-none lg:w-52 xl:w-64",
        class.as_deref().unwrap_or("")
    );
    rsx! {
        button {
            r#type: "button",
            class: "{class}",
            "aria-keyshortcuts": "Meta+K Control+K",
            onclick: move |_| open.set(true),
            Icon { icon: IconType::Search, class: "absolute start-1.5 top-1/2 size-4 -translate-y-1/2" }
            span { class: "ms-4", {t!("search-placeholder")} }
            kbd { class: "pointer-events-none absolute end-[0.3rem] top-[0.3rem] hidden h-5 items-center gap-1 rounded border bg-muted px-1.5 font-mono text-[10px] font-medium opacity-100 select-none group-hover:bg-accent sm:flex",
                span { class: "text-xs", "⌘" }
                "K"
            }
        }
    }
}

#[derive(Clone, PartialEq)]
enum Action {
    Go(&'static str),
    Theme(Theme),
}

#[derive(Clone, PartialEq)]
struct Entry {
    group: String,
    label: String,
    action: Action,
}

/// Command palette: every permitted page (sub-pages as "Parent › Child") and the theme.
#[component]
pub fn CommandMenu() -> Element {
    let mut open = use_context::<SearchOpen>().0;
    let ui = Ui::get();
    let mut query = use_signal(String::new);
    let mut selected = use_signal(|| 0usize);
    let Some(user) = use_current_user() else {
        return rsx! {};
    };
    if !open() {
        return rsx! {};
    }

    let mut entries = Vec::new();
    for g in visible_groups(&user) {
        let group = t!(g.title);
        for item in g.items {
            match item {
                NavItem::Link(l) => entries.push(Entry {
                    group: group.clone(),
                    label: t!(l.title),
                    action: Action::Go(l.url),
                }),
                NavItem::Collapsible { title, items, .. } => {
                    let parent = t!(title);
                    for l in items {
                        entries.push(Entry {
                            group: group.clone(),
                            label: format!("{parent} › {}", t!(l.title)),
                            action: Action::Go(l.url),
                        });
                    }
                }
            }
        }
    }
    let theme_group = t!("theme-label");
    for (th, key) in [
        (Theme::Light, "theme-light"),
        (Theme::Dark, "theme-dark"),
        (Theme::System, "theme-system"),
    ] {
        entries.push(Entry {
            group: theme_group.clone(),
            label: t!(key),
            action: Action::Theme(th),
        });
    }
    let q = query().to_lowercase();
    let shown: Vec<Entry> = entries
        .into_iter()
        .filter(|e| q.is_empty() || e.label.to_lowercase().contains(&q))
        .collect();
    let count = shown.len();
    let sel = selected().min(count.saturating_sub(1));

    let mut run = move |a: Action| {
        open.set(false);
        query.set(String::new());
        selected.set(0);
        match a {
            Action::Go(url) => {
                navigator().push(url);
            }
            Action::Theme(t) => ui.set_theme(t),
        }
    };
    let shown_for_keys = shown.clone();

    rsx! {
        div { class: "fixed inset-0 z-[80] bg-black/50 animate-in fade-in-0", onclick: move |_| open.set(false) }
        div {
            role: "dialog",
            "aria-modal": "true",
            class: "fixed top-[50%] left-[50%] z-[81] grid w-full max-w-[calc(100%-2rem)] translate-x-[-50%] translate-y-[-50%] overflow-hidden rounded-lg border bg-background p-0 shadow-lg animate-in fade-in-0 zoom-in-95 sm:max-w-lg",
            onkeydown: move |e| match e.key() {
                Key::Escape => open.set(false),
                Key::ArrowDown => { e.prevent_default(); selected.set((sel + 1).min(count.saturating_sub(1))) }
                Key::ArrowUp => { e.prevent_default(); selected.set(sel.saturating_sub(1)) }
                Key::Enter => if let Some(entry) = shown_for_keys.get(sel) { run(entry.action.clone()) },
                _ => {}
            },
            div { class: "flex h-12 items-center gap-2 border-b px-3",
                Icon { icon: IconType::Search, class: "size-4 shrink-0 opacity-50" }
                input {
                    class: "flex h-10 w-full rounded-md bg-transparent py-3 text-sm outline-hidden placeholder:text-muted-foreground",
                    placeholder: t!("search-command-placeholder"),
                    value: query(),
                    onmounted: move |e| async move { let _ = e.data().set_focus(true).await; },
                    oninput: move |e| { query.set(e.value()); selected.set(0); },
                }
            }
            div { class: "max-h-[300px] scroll-py-1 overflow-x-hidden overflow-y-auto p-1",
                if shown.is_empty() {
                    div { class: "py-6 text-center text-sm", {t!("search-empty")} }
                }
                for (i, e) in shown.into_iter().enumerate() {
                    button {
                        r#type: "button",
                        class: tw_merge!(
                            "relative flex w-full cursor-default items-center gap-2 rounded-sm px-2 py-1.5 text-sm outline-hidden select-none [&_svg]:size-4 [&_svg]:text-muted-foreground",
                            if i == sel { "bg-accent text-accent-foreground" } else { "" }
                        ),
                        onmouseenter: move |_| selected.set(i),
                        onclick: { let a = e.action.clone(); move |_| run(a.clone()) },
                        Icon { icon: match e.action { Action::Go(_) => IconType::ArrowRight, Action::Theme(Theme::Dark) => IconType::Moon, Action::Theme(Theme::Light) => IconType::Sun, Action::Theme(Theme::System) => IconType::Laptop } }
                        span { class: "text-xs text-muted-foreground", "{e.group}" }
                        span { "{e.label}" }
                    }
                }
            }
        }
    }
}
