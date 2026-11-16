//! `users-table.tsx` + `users-columns.tsx`.
use dioxus::prelude::*;
use dioxus_i18n::t;
use tw_merge::tw_merge;

use super::actions::{RowActions, UsersBulkActions};
use super::data::{ROLES, STATUSES, USERS, User, status_class};
use crate::app::Route;
use crate::components::data_table::{
    ColumnHeader, FilterDef, FilterOption, Pagination, TD, TH, TR, TableState, Toolbar, paginate,
};
use crate::components::long_text::LongText;
use crate::components::ui_extra::form::CheckBox;
use crate::components::ui_extra::icon::Icon;
use crate::util::query::Query;

const FILTER_COLUMNS: &[&str] = &["status", "role"];
/// Cells take the row background so the sticky (mobile) columns cover what scrolls under them.
const CELL_BG: &str =
    "bg-background group-hover/row:bg-muted group-data-[state=selected]/row:bg-muted";
const SELECT_META: &str = "start-0 z-10 rounded-tl-[inherit] max-md:sticky";
const USERNAME_META: &str = "drop-shadow-[0_1px_2px_rgb(0_0_0_/_0.1)] dark:drop-shadow-[0_1px_2px_rgb(255_255_255_/_0.1)] start-6 ps-0.5 max-md:sticky @4xl/content:table-cell @4xl/content:drop-shadow-none";
const BADGE_OUTLINE: &str = "inline-flex w-fit shrink-0 items-center justify-center gap-1 rounded-md border px-2 py-0.5 text-xs font-medium whitespace-nowrap text-foreground";

fn full_name(u: &User) -> String {
    format!("{} {}", u.first_name, u.last_name)
}

fn sort_key(u: &User, column: &str) -> String {
    match column {
        "username" => u.username.to_lowercase(),
        "fullName" => full_name(u).to_lowercase(),
        _ => u.email.to_lowercase(),
    }
}

fn field<'a>(u: &'a User, column: &str) -> &'a str {
    if column == "role" { &u.role } else { &u.status }
}

/// Username text filter + every column filter except `skip`.
fn filtered(state: &TableState, skip: Option<&str>) -> Vec<User> {
    let term = (state.filter)().trim().to_lowercase();
    USERS
        .iter()
        .filter(|u| term.is_empty() || u.username.to_lowercase().contains(&term))
        .filter(|u| {
            FILTER_COLUMNS
                .iter()
                .filter(|c| Some(**c) != skip)
                .all(|c| state.matches(c, field(u, c)))
        })
        .cloned()
        .collect()
}

fn counts(rows: &[User], column: &str, values: &[&'static str]) -> Vec<(&'static str, usize)> {
    values
        .iter()
        .map(|v| (*v, rows.iter().filter(|u| field(u, column) == *v).count()))
        .collect()
}

#[component]
pub fn UsersTable(query: Query) -> Element {
    let state = TableState::use_from_query(&query, "username", FILTER_COLUMNS);

    let mut last = use_signal(|| query.to_string());
    use_effect(move || {
        let q = state.to_query(FILTER_COLUMNS);
        if q.to_string() != *last.peek() {
            last.set(q.to_string());
            navigator().replace(Route::Users { query: q });
        }
    });

    let rows = use_memo(move || {
        let mut rows = filtered(&state, None);
        if let Some((col, desc)) = (state.sort)() {
            rows.sort_by(|a, b| {
                let o = sort_key(a, col).cmp(&sort_key(b, col));
                if desc { o.reverse() } else { o }
            });
        }
        rows
    });
    let page_count = use_memo(move || paginate(&rows(), &state).1);
    use_effect(move || state.clamp_page(page_count()));

    let (page_rows, pages) = paginate(&rows(), &state);
    let page_ids: Vec<String> = page_rows.iter().map(|u| u.id.clone()).collect();
    let mut selected = state.selected;
    let sel = selected();
    let all = !page_ids.is_empty() && page_ids.iter().all(|id| sel.contains(id));
    let some = page_ids.iter().any(|id| sel.contains(id));
    let selected_rows: Vec<User> = rows().into_iter().filter(|u| sel.contains(&u.id)).collect();

    let status_vals: Vec<_> = STATUSES.iter().map(|s| s.0).collect();
    let role_vals: Vec<_> = ROLES.iter().map(|s| s.0).collect();
    let filters = vec![
        FilterDef {
            column: "status",
            title: t!("users-col-status"),
            options: STATUSES
                .iter()
                .map(|(v, k)| FilterOption {
                    value: v,
                    label: t!(*k),
                    icon: None,
                })
                .collect(),
            counts: counts(&filtered(&state, Some("status")), "status", &status_vals),
        },
        FilterDef {
            column: "role",
            title: t!("users-col-role"),
            options: ROLES
                .iter()
                .map(|(v, k, i)| FilterOption {
                    value: v,
                    label: t!(*k),
                    icon: Some(*i),
                })
                .collect(),
            counts: counts(&filtered(&state, Some("role")), "role", &role_vals),
        },
    ];
    let hideable = vec![
        ("fullName", "fullName".to_string()),
        ("email", "email".to_string()),
        ("phoneNumber", "phoneNumber".to_string()),
    ];
    let visible_count = 5 + ["fullName", "email", "phoneNumber"]
        .iter()
        .filter(|c| state.visible(c))
        .count();
    let th = |meta: &str| tw_merge!(TH, CELL_BG, meta);

    rsx! {
        div { class: "flex flex-1 flex-col gap-4 max-sm:has-[div[role=toolbar]]:mb-16",
            Toolbar { state, placeholder: t!("users-search"), filters, columns: hideable }
            div { class: "overflow-hidden rounded-md border",
                div { class: "relative w-full overflow-x-auto",
                    table { class: "w-full caption-bottom text-sm",
                        thead { class: "[&_tr]:border-b",
                            tr { class: "{TR} group/row",
                                th { class: th(SELECT_META),
                                    CheckBox {
                                        checked: all,
                                        indeterminate: some && !all,
                                        aria_label: t!("dt-select-all"),
                                        class: "translate-y-0.5",
                                        onchange: move |on: bool| {
                                            let mut s = selected.write();
                                            for id in &page_ids { if on { s.insert(id.clone()); } else { s.remove(id); } }
                                        },
                                    }
                                }
                                th { class: th(USERNAME_META), ColumnHeader { state, column: "username", title: t!("users-col-username"), hideable: false } }
                                if state.visible("fullName") {
                                    th { class: th("w-36"), ColumnHeader { state, column: "fullName", title: t!("users-col-name"), sortable: false } }
                                }
                                if state.visible("email") {
                                    th { class: th(""), ColumnHeader { state, column: "email", title: t!("users-col-email") } }
                                }
                                if state.visible("phoneNumber") {
                                    th { class: th(""), ColumnHeader { state, column: "phoneNumber", title: t!("users-col-phone"), sortable: false } }
                                }
                                th { class: th(""), ColumnHeader { state, column: "status", title: t!("users-col-status"), sortable: false } }
                                th { class: th(""), ColumnHeader { state, column: "role", title: t!("users-col-role"), sortable: false } }
                                th { class: th("") }
                            }
                        }
                        tbody { class: "[&_tr:last-child]:border-0",
                            if page_rows.is_empty() {
                                tr { class: TR,
                                    td { class: tw_merge!(TD, "h-24 text-center"), colspan: "{visible_count}", {t!("dt-no-results")} }
                                }
                            }
                            for user in page_rows {
                                UserRow { key: "{user.id}", state, user: user.clone(), selected: sel.contains(&user.id) }
                            }
                        }
                    }
                }
            }
            Pagination { state, page_count: pages, class: "mt-auto" }
            UsersBulkActions { state, rows: selected_rows }
        }
    }
}

#[component]
fn UserRow(state: TableState, user: User, selected: bool) -> Element {
    let mut sel = state.selected;
    let id = user.id.clone();
    let role = ROLES.iter().find(|r| r.0 == user.role);
    let td = |meta: &str| tw_merge!(TD, CELL_BG, meta);
    rsx! {
        tr { class: "{TR} group/row", "data-state": if selected { "selected" } else { "" },
            td { class: td(SELECT_META),
                CheckBox {
                    checked: selected,
                    aria_label: t!("dt-select-row"),
                    class: "translate-y-0.5",
                    onchange: move |on: bool| { if on { sel.write().insert(id.clone()); } else { sel.write().remove(&id); } },
                }
            }
            td { class: td(USERNAME_META), LongText { text: user.username.clone(), class: "max-w-36 ps-3" } }
            if state.visible("fullName") {
                td { class: td("w-36"), LongText { text: full_name(&user), class: "max-w-36" } }
            }
            if state.visible("email") {
                td { class: td(""), div { class: "w-fit ps-2 text-nowrap", "{user.email}" } }
            }
            if state.visible("phoneNumber") {
                td { class: td(""), div { "{user.phone_number}" } }
            }
            td { class: td(""),
                div { class: "flex space-x-2",
                    span { class: tw_merge!(BADGE_OUTLINE, "capitalize", status_class(&user.status)), "{user.status}" }
                }
            }
            td { class: td(""),
                if let Some((_, _, icon)) = role {
                    div { class: "flex items-center gap-x-2",
                        Icon { icon: *icon, class: "size-4 text-muted-foreground" }
                        span { class: "text-sm capitalize", "{user.role}" }
                    }
                }
            }
            td { class: td(""), RowActions { user: user.clone() } }
        }
    }
}
