//! `features/chats/*` (demo page, flag `demo.chats` in M7): inbox, conversation view,
//! "new message" dialog. Data = the reference's `convo.json`. Profile photos are remote in
//! the reference (randomuser.me); we show the initials fallback instead (CSP: same-origin only).
use dioxus::prelude::*;
use dioxus_i18n::t;
use serde::Deserialize;
use serde_json::json;
use tw_merge::tw_merge;

use crate::components::layout::{ConfigDrawer, Header, Main, ProfileDropdown, Search, ThemeSwitch};
use crate::components::ui_extra::dialog::Modal;
use crate::components::ui_extra::icon::{Icon, IconType};
use crate::ui::avatar::{Avatar, AvatarFallback};
use crate::util::show_submitted_data;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatUser {
    pub id: String,
    pub username: String,
    pub full_name: String,
    pub title: String,
    pub messages: Vec<Convo>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Convo {
    pub sender: String,
    pub message: String,
    pub timestamp: String,
}

#[derive(Deserialize)]
struct Data {
    conversations: Vec<ChatUser>,
}

fn conversations() -> Vec<ChatUser> {
    serde_json::from_str::<Data>(include_str!("convo.json"))
        .map(|d| d.conversations)
        .unwrap_or_default()
}

/// `lib/utils.ts` `getDisplayNameInitials`.
pub fn initials(name: &str) -> String {
    let parts: Vec<&str> = name.split_whitespace().collect();
    match parts.as_slice() {
        [] => "?".into(),
        [one] => one.chars().take(2).collect::<String>().to_uppercase(),
        [first, .., last] => format!(
            "{}{}",
            first.chars().next().unwrap_or(' '),
            last.chars().next().unwrap_or(' ')
        )
        .to_uppercase(),
    }
}

/// "2024-08-24T11:15:15" → ("24 Aug, 2024", "11:15 AM") — date-fns `d MMM, yyyy` / `h:mm a`.
fn split_timestamp(ts: &str) -> (String, String) {
    const MONTHS: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let (date, time) = ts.split_once('T').unwrap_or((ts, "00:00"));
    let d: Vec<u32> = date.split('-').filter_map(|p| p.parse().ok()).collect();
    let t: Vec<u32> = time.split(':').filter_map(|p| p.parse().ok()).collect();
    let (y, m, day) = (
        d.first().copied().unwrap_or(0),
        d.get(1).copied().unwrap_or(1),
        d.get(2).copied().unwrap_or(1),
    );
    let (h, min) = (
        t.first().copied().unwrap_or(0),
        t.get(1).copied().unwrap_or(0),
    );
    let h12 = if h % 12 == 0 { 12 } else { h % 12 };
    (
        format!("{day} {}, {y}", MONTHS[(m.clamp(1, 12) - 1) as usize]),
        format!("{h12}:{min:02} {}", if h < 12 { "AM" } else { "PM" }),
    )
}

#[component]
pub fn Chats() -> Element {
    let all = use_hook(conversations);
    let mut search = use_signal(String::new);
    let mut selected = use_signal(|| None::<String>);
    let mut mobile_selected = use_signal(|| false);
    let mut new_chat = use_signal(|| false);

    let q = search().trim().to_lowercase();
    let list: Vec<ChatUser> = all
        .iter()
        .filter(|c| c.full_name.to_lowercase().contains(&q))
        .cloned()
        .collect();
    let current = selected().and_then(|id| all.iter().find(|c| c.id == id).cloned());

    rsx! {
        Header {
            Search { class: "me-auto" }
            ThemeSwitch {}
            ConfigDrawer {}
            ProfileDropdown {}
        }
        Main { fixed: true,
            section { class: "flex h-full gap-6",
                // Left side: inbox
                div { class: "flex w-full flex-col gap-2 sm:w-56 lg:w-72 2xl:w-80",
                    div { class: "sticky top-0 z-10 -mx-4 bg-background px-4 pb-3 shadow-md sm:static sm:z-auto sm:mx-0 sm:p-0 sm:shadow-none",
                        div { class: "flex items-center justify-between py-2",
                            div { class: "flex gap-2",
                                h1 { class: "text-2xl font-bold", {t!("chats-inbox")} }
                                Icon { icon: IconType::MessagesSquare, class: "size-5" }
                            }
                            button { r#type: "button", class: "inline-flex size-9 items-center justify-center rounded-lg hover:bg-accent", onclick: move |_| new_chat.set(true),
                                Icon { icon: IconType::SquarePen, class: "size-6 stroke-muted-foreground" }
                            }
                        }
                        label { class: "flex h-10 w-full items-center space-x-0 rounded-md border border-border ps-2 focus-within:ring-1 focus-within:ring-ring focus-within:outline-hidden",
                            Icon { icon: IconType::Search, class: "me-2 size-[15px] stroke-slate-500" }
                            span { class: "sr-only", {t!("search-placeholder")} }
                            input {
                                r#type: "text",
                                class: "w-full flex-1 bg-inherit text-sm focus-visible:outline-hidden",
                                placeholder: t!("chats-search"),
                                value: search(),
                                oninput: move |e| search.set(e.value()),
                            }
                        }
                    }
                    div { class: "-mx-3 h-full overflow-auto p-3",
                        for c in list {
                            ChatListItem {
                                user: c.clone(),
                                active: selected() == Some(c.id.clone()),
                                onselect: move |id: String| { selected.set(Some(id)); mobile_selected.set(true); },
                            }
                            div { role: "separator", class: "my-1 h-px w-full bg-border" }
                        }
                    }
                }
                // Right side: conversation or empty state
                if let Some(user) = current {
                    Conversation { user, mobile_open: mobile_selected }
                } else {
                    div { class: "absolute inset-0 start-full z-50 hidden w-full flex-1 flex-col justify-center rounded-md border bg-card shadow-xs sm:static sm:z-auto sm:flex",
                        div { class: "flex flex-col items-center space-y-6",
                            div { class: "flex size-16 items-center justify-center rounded-full border-2 border-border",
                                Icon { icon: IconType::MessagesSquare, class: "size-8" }
                            }
                            div { class: "space-y-2 text-center",
                                h1 { class: "text-xl font-semibold", {t!("chats-your-messages")} }
                                p { class: "text-sm text-muted-foreground", {t!("chats-start")} }
                            }
                            button {
                                r#type: "button",
                                class: "inline-flex h-9 items-center justify-center rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground shadow-xs hover:bg-primary/90",
                                onclick: move |_| new_chat.set(true),
                                {t!("chats-send-message")}
                            }
                        }
                    }
                }
            }
            NewChat { open: new_chat, users: all.clone() }
        }
    }
}

#[component]
fn ChatListItem(user: ChatUser, active: bool, onselect: EventHandler<String>) -> Element {
    let last = user.messages.first().cloned();
    let last_msg = last
        .map(|m| {
            if m.sender == "You" {
                format!("You: {}", m.message)
            } else {
                m.message
            }
        })
        .unwrap_or_default();
    let id = user.id.clone();
    rsx! {
        button {
            r#type: "button",
            class: tw_merge!("group flex w-full rounded-md px-2 py-2 text-start text-sm hover:bg-accent hover:text-accent-foreground", if active { "sm:bg-muted" } else { "" }),
            onclick: move |_| onselect.call(id.clone()),
            div { class: "flex gap-2",
                Avatar { AvatarFallback { class: "text-foreground", "{initials(&user.full_name)}" } }
                div {
                    span { class: "col-start-2 row-span-2 font-medium", "{user.full_name}" }
                    span { class: "col-start-2 row-span-2 row-start-2 line-clamp-2 text-ellipsis text-muted-foreground group-hover:text-accent-foreground/90", "{last_msg}" }
                }
            }
        }
    }
}

#[component]
fn Conversation(user: ChatUser, mobile_open: Signal<bool>) -> Element {
    let mut mobile_open = mobile_open;
    // Messages are newest-first; group by day, keeping that order (rendered column-reverse).
    let mut groups: Vec<(String, Vec<(Convo, String)>)> = Vec::new();
    for m in &user.messages {
        let (day, time) = split_timestamp(&m.timestamp);
        match groups.last_mut() {
            Some((d, list)) if *d == day => list.push((m.clone(), time)),
            _ => groups.push((day, vec![(m.clone(), time)])),
        }
    }
    let ghost = "inline-flex items-center justify-center rounded-md hover:bg-accent hover:text-accent-foreground";
    rsx! {
        div { class: tw_merge!("absolute inset-0 start-full z-50 hidden w-full flex-1 flex-col border bg-background shadow-xs sm:static sm:z-auto sm:flex sm:rounded-md", if mobile_open() { "start-0 flex" } else { "" }),
            div { class: "mb-1 flex flex-none justify-between bg-card p-4 shadow-lg sm:rounded-t-md",
                div { class: "flex gap-3",
                    button { r#type: "button", class: "{ghost} -ms-2 h-full w-9 sm:hidden", onclick: move |_| mobile_open.set(false),
                        Icon { icon: IconType::ArrowLeft, class: "size-4 rtl:rotate-180" }
                    }
                    div { class: "flex items-center gap-2 lg:gap-4",
                        Avatar { class: "size-9 lg:size-11", AvatarFallback { class: "text-foreground", "{initials(&user.full_name)}" } }
                        div {
                            span { class: "col-start-2 row-span-2 text-sm font-medium lg:text-base", "{user.full_name}" }
                            span { class: "col-start-2 row-span-2 row-start-2 line-clamp-1 block max-w-32 text-xs text-nowrap text-ellipsis text-muted-foreground lg:max-w-none lg:text-sm", "{user.title}" }
                        }
                    }
                }
                div { class: "-me-1 flex items-center gap-1 lg:gap-2",
                    button { r#type: "button", class: "{ghost} hidden size-8 rounded-full sm:inline-flex lg:size-10", Icon { icon: IconType::Video, class: "size-[22px] stroke-muted-foreground" } }
                    button { r#type: "button", class: "{ghost} hidden size-8 rounded-full sm:inline-flex lg:size-10", Icon { icon: IconType::Phone, class: "size-[22px] stroke-muted-foreground" } }
                    button { r#type: "button", class: "{ghost} h-10 w-9 sm:h-8 sm:w-4 lg:h-10 lg:w-6", Icon { icon: IconType::EllipsisVertical, class: "size-4 stroke-muted-foreground sm:size-5" } }
                }
            }
            div { class: "flex flex-1 flex-col gap-2 rounded-md px-4 pt-0 pb-4",
                div { class: "flex size-full flex-1",
                    div { class: "relative -me-4 flex flex-1 flex-col overflow-y-hidden",
                        div { class: "flex h-40 w-full grow flex-col-reverse justify-start gap-4 overflow-y-auto py-2 pe-4 pb-4",
                            for (day, msgs) in groups {
                                for (m, time) in msgs {
                                    div { class: tw_merge!(
                                            "max-w-72 px-3 py-2 wrap-break-word shadow-lg",
                                            if m.sender == "You" { "self-end rounded-[16px_16px_0_16px] bg-primary/90 text-primary-foreground/75" } else { "self-start rounded-[16px_16px_16px_0] bg-muted" }
                                        ),
                                        "{m.message} "
                                        span { class: tw_merge!("mt-1 block text-xs font-light text-foreground/75 italic", if m.sender == "You" { "text-end text-primary-foreground/85" } else { "" }), "{time}" }
                                    }
                                }
                                div { class: "text-center text-xs", "{day}" }
                            }
                        }
                    }
                }
                form { class: "flex w-full flex-none gap-2", onsubmit: move |e| e.prevent_default(),
                    div { class: "flex flex-1 items-center gap-2 rounded-md border border-input bg-card px-2 py-1 focus-within:ring-1 focus-within:ring-ring focus-within:outline-hidden lg:gap-4",
                        div { class: "space-x-1",
                            button { r#type: "button", class: "{ghost} h-8 w-9", Icon { icon: IconType::Plus, class: "size-5 stroke-muted-foreground" } }
                            button { r#type: "button", class: "{ghost} hidden h-8 w-9 lg:inline-flex", Icon { icon: IconType::ImagePlus, class: "size-5 stroke-muted-foreground" } }
                            button { r#type: "button", class: "{ghost} hidden h-8 w-9 lg:inline-flex", Icon { icon: IconType::Paperclip, class: "size-5 stroke-muted-foreground" } }
                        }
                        label { class: "flex-1",
                            span { class: "sr-only", {t!("chats-text-box")} }
                            input { r#type: "text", placeholder: t!("chats-type"), class: "h-8 w-full bg-inherit focus-visible:outline-hidden" }
                        }
                        button { r#type: "submit", class: "{ghost} hidden size-9 sm:inline-flex", Icon { icon: IconType::Send, class: "size-5" } }
                    }
                    button { r#type: "submit", class: "inline-flex h-full items-center justify-center gap-2 rounded-md bg-primary px-4 text-sm font-medium text-primary-foreground sm:hidden",
                        Icon { icon: IconType::Send, class: "size-[18px]" }
                        {t!("chats-send")}
                    }
                }
            }
        }
    }
}

/// `new-chat.tsx`: pick people, then "Chat" shows the selection (demo behaviour).
#[component]
fn NewChat(open: Signal<bool>, users: Vec<ChatUser>) -> Element {
    let mut picked = use_signal(Vec::<String>::new);
    let mut query = use_signal(String::new);
    let q = query().to_lowercase();
    let shown: Vec<ChatUser> = users
        .iter()
        .filter(|u| {
            u.full_name.to_lowercase().contains(&q) || u.username.to_lowercase().contains(&q)
        })
        .cloned()
        .collect();
    let chosen: Vec<ChatUser> = picked()
        .iter()
        .filter_map(|id| users.iter().find(|u| &u.id == id).cloned())
        .collect();
    let mut toggle = move |id: String| {
        let mut p = picked.write();
        if let Some(i) = p.iter().position(|x| *x == id) {
            p.remove(i);
        } else {
            p.push(id);
        }
    };
    rsx! {
        Modal { open, title: t!("chats-new-message"), class: "sm:max-w-150", on_close: move |_| { picked.set(Vec::new()); query.set(String::new()); },
            div { class: "flex flex-col gap-4",
                div { class: "flex flex-wrap items-baseline-last gap-2",
                    span { class: "min-h-6 text-sm text-muted-foreground", {t!("chats-to")} }
                    for u in chosen.clone() {
                        span { class: "inline-flex items-center justify-center gap-1 rounded-md border border-transparent bg-primary px-2 py-0.5 text-xs font-medium text-primary-foreground",
                            "{u.full_name}"
                            button { r#type: "button", class: "ms-1 rounded-full outline-hidden focus:ring-2 focus:ring-ring focus:ring-offset-2", onclick: { let id = u.id.clone(); move |_| toggle(id.clone()) },
                                Icon { icon: IconType::X, class: "h-3 w-3 text-muted-foreground hover:text-foreground" }
                            }
                        }
                    }
                }
                div { class: "rounded-lg border",
                    div { class: "flex h-9 items-center gap-2 border-b px-3",
                        Icon { icon: IconType::Search, class: "size-4 shrink-0 opacity-50" }
                        input { class: "flex h-10 w-full bg-transparent py-3 text-sm text-foreground outline-hidden placeholder:text-muted-foreground", placeholder: t!("chats-search-people"), value: query(), oninput: move |e| query.set(e.value()) }
                    }
                    div { class: "max-h-[300px] overflow-y-auto p-1",
                        if shown.is_empty() { div { class: "py-6 text-center text-sm", {t!("chats-no-people")} } }
                        for u in shown {
                            button {
                                r#type: "button",
                                class: "flex w-full items-center justify-between gap-2 rounded-sm px-2 py-1.5 text-sm hover:bg-accent hover:text-accent-foreground",
                                onclick: { let id = u.id.clone(); move |_| toggle(id.clone()) },
                                div { class: "flex items-center gap-2",
                                    Avatar { class: "h-8 w-8", AvatarFallback { class: "text-foreground text-xs", "{initials(&u.full_name)}" } }
                                    div { class: "flex flex-col text-start",
                                        span { class: "text-sm font-medium", "{u.full_name}" }
                                        span { class: "text-xs text-accent-foreground/70", "{u.username}" }
                                    }
                                }
                                if picked().contains(&u.id) { Icon { icon: IconType::Check, class: "h-4 w-4" } }
                            }
                        }
                    }
                }
                button {
                    r#type: "button",
                    disabled: chosen.is_empty(),
                    class: "inline-flex h-9 items-center justify-center rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground shadow-xs hover:bg-primary/90 disabled:pointer-events-none disabled:opacity-50",
                    onclick: move |_| {
                        let data: Vec<_> = chosen.iter().map(|u| json!({ "id": u.id, "username": u.username, "fullName": u.full_name, "title": u.title })).collect();
                        show_submitted_data(&serde_json::Value::Array(data), &t!("submitted-title"));
                    },
                    {t!("chats-chat")}
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_and_helpers() {
        assert_eq!(conversations().len(), 10);
        assert_eq!(initials("Alex John"), "AJ");
        assert_eq!(initials("cher"), "CH");
        assert_eq!(
            split_timestamp("2024-08-24T11:15:15"),
            ("24 Aug, 2024".into(), "11:15 AM".into())
        );
        assert_eq!(split_timestamp("2024-08-24T00:05:00").1, "12:05 AM");
    }
}
