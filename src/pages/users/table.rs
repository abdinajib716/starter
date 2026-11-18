//! `users-table.tsx` + `users-columns.tsx` on server data: the table state (URL-synced) is
//! turned into a `UserQuery`; the server filters, sorts, pages and counts facets.
use dioxus::prelude::*;
use dioxus_i18n::t;
use tw_merge::tw_merge;
use uuid::Uuid;

use super::UsersCtx;
use super::actions::{RowActions, UsersBulkActions};
use crate::api::users::list_users;
use crate::app::Route;
use crate::components::data_table::{
    ColumnHeader, FilterDef, FilterOption, Pagination, TD, TH, TR, TableState, Toolbar,
};
use crate::components::long_text::LongText;
use crate::components::skeletons::TableSkeleton;
use crate::components::ui_extra::form::CheckBox;
use crate::components::ui_extra::icon::{Icon, IconType};
use crate::domain::users::{UserPage, UserQuery, UserRow, UserSort, UserStatus};
use crate::util::query::Query;

const FILTER_COLUMNS: &[&str] = &["status", "role"];
const CELL_BG: &str =
    "bg-background group-hover/row:bg-muted group-data-[state=selected]/row:bg-muted";
const SELECT_META: &str = "start-0 z-10 rounded-tl-[inherit] max-md:sticky";
const NAME_META: &str = "drop-shadow-[0_1px_2px_rgb(0_0_0_/_0.1)] dark:drop-shadow-[0_1px_2px_rgb(255_255_255_/_0.1)] start-6 ps-0.5 max-md:sticky @4xl/content:table-cell @4xl/content:drop-shadow-none";
const BADGE_OUTLINE: &str = "inline-flex w-fit shrink-0 items-center justify-center gap-1 rounded-md border px-2 py-0.5 text-xs font-medium whitespace-nowrap text-foreground";

/// `callTypes`: badge colours per status.
pub fn status_class(s: UserStatus) -> &'static str {
    match s {
        UserStatus::Active => "bg-teal-100/30 text-teal-900 dark:text-teal-200 border-teal-200",
        UserStatus::Inactive | UserStatus::Deleted => "bg-neutral-300/40 border-neutral-300",
        UserStatus::Invited => "bg-sky-200/40 text-sky-900 dark:text-sky-100 border-sky-300",
        UserStatus::Suspended => {
            "bg-destructive/10 dark:bg-destructive/50 text-destructive dark:text-primary border-destructive/10"
        }
    }
}

pub fn status_key(s: UserStatus) -> &'static str {
    match s {
        UserStatus::Active => "user-status-active",
        UserStatus::Inactive => "user-status-inactive",
        UserStatus::Invited => "user-status-invited",
        UserStatus::Suspended => "user-status-suspended",
        UserStatus::Deleted => "user-status-deleted",
    }
}

fn role_icon(is_system: bool) -> IconType {
    if is_system {
        IconType::Shield
    } else {
        IconType::UserCheck
    }
}

/// The server query for the current table state.
fn to_user_query(state: &TableState, deleted: bool) -> UserQuery {
    UserQuery {
        q: (state.filter)().trim().to_string(),
        statuses: state
            .column_filter("status")
            .iter()
            .filter_map(|s| UserStatus::parse(s))
            .collect(),
        role_ids: state
            .column_filter("role")
            .iter()
            .filter_map(|s| s.parse().ok())
            .collect(),
        sort: (state.sort)().and_then(|(c, d)| UserSort::parse(c).map(|s| (s, d))),
        page: (state.page)() as u32,
        page_size: (state.page_size)() as u32,
        deleted,
    }
}

/// Download `/api/export/users.{csv,pdf}` with the list's filters (and optionally only `ids`).
pub fn download_export(
    format: crate::components::export_menu::Format,
    query: &Query,
    ids: Option<Vec<Uuid>>,
) {
    let ext = format.ext();
    let p = |k: &str| query.get(k).unwrap_or("").to_string();
    let mut q = Query::default()
        .with("q", Some(p("filter")))
        .with("status", Some(p("status")))
        .with("role", Some(p("role")))
        .with(
            "deleted",
            (query.get("tab") == Some("deleted")).then(|| "1".to_string()),
        );
    if let Some(ids) = ids {
        q = q.with(
            "ids",
            Some(
                ids.iter()
                    .map(Uuid::to_string)
                    .collect::<Vec<_>>()
                    .join(","),
            ),
        );
    }
    crate::util::download(
        &format!("/api/export/users.{ext}?{q}"),
        &format!("users.{ext}"),
    );
}

#[component]
pub fn UsersTable(query: Query, deleted: bool) -> Element {
    let ctx = use_context::<UsersCtx>();
    let state = TableState::use_from_query(&query, "filter", FILTER_COLUMNS);
    let tab = query.get("tab").map(str::to_owned);

    // URL sync (replace, so typing does not fill the history).
    let mut last = use_signal(|| query.to_string());
    use_effect(move || {
        let q = state.to_query(FILTER_COLUMNS).with("tab", tab.clone());
        if q.to_string() != *last.peek() {
            last.set(q.to_string());
            navigator().replace(Route::Users { query: q });
        }
    });

    // Server data. Typing in the search box waits 250 ms; the previous page stays on screen
    // while the next one loads.
    let mut data = use_signal(|| None::<UserPage>);
    let mut loaded_q = use_signal(|| None::<String>);
    let refresh = ctx.refresh;
    let live = crate::components::layout::use_live();
    let _list = use_resource(move || async move {
        let _ = refresh();
        let _ = (live.users)();
        let q = to_user_query(&state, deleted);
        if loaded_q.peek().as_ref().is_some_and(|l| *l != q.q) {
            crate::util::sleep_ms(250).await;
        }
        match list_users(q.clone()).await {
            Ok(page) => {
                loaded_q.set(Some(q.q));
                data.set(Some(page));
            }
            Err(e) => crate::util::toast_error(&e),
        }
    });

    let size = (state.page_size)().max(1);
    let pages = data
        .read()
        .as_ref()
        .map(|p| (p.total as usize).div_ceil(size).max(1))
        .unwrap_or(1);
    use_effect(use_reactive!(|pages| state.clamp_page(pages)));

    let Some(page) = data() else {
        return rsx! { TableSkeleton { cols: 5 } };
    };

    let page_ids: Vec<String> = page.rows.iter().map(|u| u.id.to_string()).collect();
    let mut selected = state.selected;
    let sel = selected();
    let all = !page_ids.is_empty() && page_ids.iter().all(|id| sel.contains(id));
    let some = page_ids.iter().any(|id| sel.contains(id));
    let selected_ids: Vec<Uuid> = sel.iter().filter_map(|s| s.parse().ok()).collect();

    let roles = (ctx.roles)();
    let count = |v: Option<u64>| v.unwrap_or(0) as usize;
    let mut filters = vec![];
    if !deleted {
        filters.push(FilterDef {
            column: "status",
            title: t!("users-col-status"),
            options: UserStatus::FILTERABLE
                .iter()
                .map(|s| FilterOption {
                    value: s.as_str().to_string(),
                    label: t!(status_key(*s)),
                    icon: None,
                })
                .collect(),
            counts: UserStatus::FILTERABLE
                .iter()
                .map(|s| {
                    (
                        s.as_str().to_string(),
                        count(page.status_counts.iter().find(|(x, _)| x == s).map(|x| x.1)),
                    )
                })
                .collect(),
        });
    }
    filters.push(FilterDef {
        column: "role",
        title: t!("users-col-role"),
        options: roles
            .iter()
            .map(|r| FilterOption {
                value: r.id.to_string(),
                label: r.name.clone(),
                icon: Some(role_icon(r.is_system)),
            })
            .collect(),
        counts: roles
            .iter()
            .map(|r| {
                (
                    r.id.to_string(),
                    count(
                        page.role_counts
                            .iter()
                            .find(|(x, _)| *x == r.id)
                            .map(|x| x.1),
                    ),
                )
            })
            .collect(),
    });
    let hideable = if deleted {
        vec![("email", t!("users-col-email"))]
    } else {
        vec![
            ("email", t!("users-col-email")),
            ("lastLogin", t!("users-col-last-login")),
        ]
    };
    let visible_count = 5 + hideable.iter().filter(|(c, _)| state.visible(c)).count();
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
                                th { class: th(NAME_META), ColumnHeader { state, column: "name", title: t!("users-col-name"), hideable: false } }
                                if state.visible("email") {
                                    th { class: th(""), ColumnHeader { state, column: "email", title: t!("users-col-email") } }
                                }
                                if !deleted && state.visible("lastLogin") {
                                    th { class: th(""), ColumnHeader { state, column: "lastLogin", title: t!("users-col-last-login") } }
                                }
                                if deleted {
                                    th { class: th(""), ColumnHeader { state, column: "deleted", title: t!("users-col-deleted"), sortable: false } }
                                } else {
                                    th { class: th(""), ColumnHeader { state, column: "status", title: t!("users-col-status"), sortable: false } }
                                }
                                th { class: th(""), ColumnHeader { state, column: "role", title: t!("users-col-role"), sortable: false } }
                                th { class: th("") }
                            }
                        }
                        tbody { class: "[&_tr:last-child]:border-0",
                            if page.rows.is_empty() {
                                tr { class: TR,
                                    td { class: tw_merge!(TD, "h-24 text-center"), colspan: "{visible_count}", {t!("dt-no-results")} }
                                }
                            }
                            for user in page.rows.iter().cloned() {
                                UserRowView {
                                    key: "{user.id}",
                                    state,
                                    selected: sel.contains(&user.id.to_string()),
                                    is_system: roles.iter().any(|r| r.id == user.role_id && r.is_system),
                                    user,
                                    deleted,
                                }
                            }
                        }
                    }
                }
            }
            Pagination { state, page_count: pages, class: "mt-auto" }
            UsersBulkActions { state, ids: selected_ids, deleted }
        }
    }
}

#[component]
fn UserRowView(
    state: TableState,
    user: UserRow,
    selected: bool,
    is_system: bool,
    deleted: bool,
) -> Element {
    let tz = crate::components::layout::use_tz();
    let mut sel = state.selected;
    let id = user.id.to_string();
    let td = |meta: &str| tw_merge!(TD, CELL_BG, meta);
    let deleted_info = match (&user.deleted_by_email, user.deleted_at) {
        (Some(by), Some(at)) => {
            crate::ta!("users-deleted-by", date: crate::util::time::date(at, &tz), by: by.clone())
        }
        (None, Some(at)) => crate::util::time::date(at, &tz),
        _ => String::new(),
    };
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
            td { class: td(NAME_META),
                Link { to: Route::UserDetailPage { id: user.id }, class: "block hover:underline underline-offset-4",
                    LongText { text: user.full_name.clone(), class: "max-w-36 ps-3" }
                }
            }
            if state.visible("email") {
                td { class: td(""), div { class: "w-fit ps-2 text-nowrap", "{user.email}" } }
            }
            if !deleted && state.visible("lastLogin") {
                td { class: td(""),
                    match user.last_login_at {
                        Some(at) => rsx! { div { title: "{crate::util::time::datetime(at, &tz)}", "{crate::util::time::date(at, &tz)}" } },
                        None => rsx! { div { class: "text-muted-foreground", {t!("users-never")} } },
                    }
                }
            }
            if deleted {
                td { class: td(""), div { class: "text-nowrap text-muted-foreground", "{deleted_info}" } }
            } else {
                td { class: td(""),
                    div { class: "flex space-x-2",
                        span { class: tw_merge!(BADGE_OUTLINE, status_class(user.status)), {t!(status_key(user.status))} }
                    }
                }
            }
            td { class: td(""),
                div { class: "flex items-center gap-x-2",
                    Icon { icon: role_icon(is_system), class: "size-4 text-muted-foreground" }
                    span { class: "text-sm", "{user.role_name}" }
                }
            }
            td { class: td(""), RowActions { user: user.clone(), deleted } }
        }
    }
}
