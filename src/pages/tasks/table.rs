//! `tasks-table.tsx` + `tasks-columns.tsx`.
use dioxus::prelude::*;
use dioxus_i18n::t;
use tw_merge::tw_merge;

use super::actions::{RowActions, TasksBulkActions};
use super::data::{self, LABELS, PRIORITIES, STATUSES, TASKS, Task};
use crate::app::Route;
use crate::components::data_table::{
    ColumnHeader, FilterDef, Pagination, TD, TH, TR, TableState, Toolbar, paginate,
};
use crate::components::ui_extra::form::CheckBox;
use crate::components::ui_extra::icon::Icon;
use crate::util::query::Query;

const FILTER_COLUMNS: &[&str] = &["status", "priority"];
pub const BADGE_OUTLINE: &str = "inline-flex w-fit shrink-0 items-center justify-center gap-1 rounded-md border px-2 py-0.5 text-xs font-medium whitespace-nowrap text-foreground";

fn field<'a>(t: &'a Task, column: &str) -> &'a str {
    match column {
        "id" => &t.id,
        "title" => &t.title,
        "status" => &t.status,
        "priority" => &t.priority,
        _ => &t.label,
    }
}

/// Rows passing the global filter and every column filter except `skip` (for facet counts).
fn filtered(state: &TableState, skip: Option<&str>) -> Vec<Task> {
    let term = (state.filter)().trim().to_lowercase();
    TASKS
        .iter()
        .filter(|t| {
            term.is_empty()
                || t.id.to_lowercase().contains(&term)
                || t.title.to_lowercase().contains(&term)
        })
        .filter(|t| {
            FILTER_COLUMNS
                .iter()
                .filter(|c| Some(**c) != skip)
                .all(|c| state.matches(c, field(t, c)))
        })
        .cloned()
        .collect()
}

fn counts(rows: &[Task], column: &str, values: &[&'static str]) -> Vec<(String, usize)> {
    values
        .iter()
        .map(|v| {
            (
                v.to_string(),
                rows.iter().filter(|t| field(t, column) == *v).count(),
            )
        })
        .collect()
}

#[component]
pub fn TasksTable(query: Query) -> Element {
    let state = TableState::use_from_query(&query, "filter", FILTER_COLUMNS);

    // URL sync (`use-table-url-state.ts`): replace, so typing does not fill the history.
    let mut last = use_signal(|| query.to_string());
    use_effect(move || {
        let q = state.to_query(FILTER_COLUMNS);
        if q.to_string() != *last.peek() {
            last.set(q.to_string());
            navigator().replace(Route::Tasks { query: q });
        }
    });

    let rows = use_memo(move || {
        let mut rows = filtered(&state, None);
        if let Some((col, desc)) = (state.sort)() {
            rows.sort_by(|a, b| {
                let o = field(a, col)
                    .to_lowercase()
                    .cmp(&field(b, col).to_lowercase());
                if desc { o.reverse() } else { o }
            });
        }
        rows
    });
    let page_count = use_memo(move || paginate(&rows(), &state).1);
    use_effect(move || state.clamp_page(page_count()));

    let (page_rows, pages) = paginate(&rows(), &state);
    let page_ids: Vec<String> = page_rows.iter().map(|t| t.id.clone()).collect();
    let mut selected = state.selected;
    let sel = selected();
    let all = !page_ids.is_empty() && page_ids.iter().all(|id| sel.contains(id));
    let some = page_ids.iter().any(|id| sel.contains(id));
    let selected_rows: Vec<Task> = rows().into_iter().filter(|t| sel.contains(&t.id)).collect();

    let status_vals: Vec<_> = STATUSES.iter().map(|s| s.0).collect();
    let priority_vals: Vec<_> = PRIORITIES.iter().map(|s| s.0).collect();
    let filters = vec![
        FilterDef {
            column: "status",
            title: t!("tasks-col-status"),
            options: data::options(&STATUSES),
            counts: counts(&filtered(&state, Some("status")), "status", &status_vals),
        },
        FilterDef {
            column: "priority",
            title: t!("tasks-col-priority"),
            options: data::options(&PRIORITIES),
            counts: counts(
                &filtered(&state, Some("priority")),
                "priority",
                &priority_vals,
            ),
        },
    ];
    let hideable = vec![
        ("title", "title".to_string()),
        ("status", "status".to_string()),
        ("priority", "priority".to_string()),
    ];
    let visible_count = 3 + ["title", "status", "priority"]
        .iter()
        .filter(|c| state.visible(c))
        .count();

    rsx! {
        div { class: "flex flex-1 flex-col gap-4 max-sm:has-[div[role=toolbar]]:mb-16",
            Toolbar { state, placeholder: t!("tasks-search"), filters, columns: hideable }
            div { class: "overflow-hidden rounded-md border",
                div { class: "relative w-full overflow-x-auto",
                    table { class: "w-full min-w-xl caption-bottom text-sm",
                        thead { class: "[&_tr]:border-b",
                            tr { class: TR,
                                th { class: TH,
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
                                th { class: TH, ColumnHeader { state, column: "id", title: t!("tasks-col-task"), sortable: false } }
                                if state.visible("title") {
                                    th { class: tw_merge!(TH, "ps-1 max-w-0 w-2/3"), ColumnHeader { state, column: "title", title: t!("tasks-col-title") } }
                                }
                                if state.visible("status") {
                                    th { class: tw_merge!(TH, "ps-1"), ColumnHeader { state, column: "status", title: t!("tasks-col-status") } }
                                }
                                if state.visible("priority") {
                                    th { class: tw_merge!(TH, "ps-1"), ColumnHeader { state, column: "priority", title: t!("tasks-col-priority") } }
                                }
                                th { class: TH }
                            }
                        }
                        tbody { class: "[&_tr:last-child]:border-0",
                            if page_rows.is_empty() {
                                tr { class: TR,
                                    td { class: tw_merge!(TD, "h-24 text-center"), colspan: "{visible_count}", {t!("dt-no-results")} }
                                }
                            }
                            for task in page_rows {
                                TaskRow { key: "{task.id}", state, task: task.clone(), selected: sel.contains(&task.id) }
                            }
                        }
                    }
                }
            }
            Pagination { state, page_count: pages, class: "mt-auto" }
            TasksBulkActions { state, rows: selected_rows }
        }
    }
}

#[component]
fn TaskRow(state: TableState, task: Task, selected: bool) -> Element {
    let mut sel = state.selected;
    let id = task.id.clone();
    let label = LABELS.iter().find(|(v, _)| *v == task.label);
    let status = data::find(&STATUSES, &task.status);
    let priority = data::find(&PRIORITIES, &task.priority);
    rsx! {
        tr { class: TR, "data-state": if selected { "selected" } else { "" },
            td { class: TD,
                CheckBox {
                    checked: selected,
                    aria_label: t!("dt-select-row"),
                    class: "translate-y-0.5",
                    onchange: move |on: bool| { if on { sel.write().insert(id.clone()); } else { sel.write().remove(&id); } },
                }
            }
            td { class: TD, div { class: "w-20", "{task.id}" } }
            if state.visible("title") {
                td { class: tw_merge!(TD, "max-w-0 w-2/3 ps-4"),
                    div { class: "flex space-x-2",
                        if let Some((_, key)) = label { span { class: BADGE_OUTLINE, {t!(*key)} } }
                        span { class: "truncate font-medium", "{task.title}" }
                    }
                }
            }
            if state.visible("status") {
                td { class: tw_merge!(TD, "ps-4"),
                    if let Some((_, key, icon)) = status {
                        div { class: "flex w-25 items-center gap-2",
                            Icon { icon: *icon, class: "size-4 text-muted-foreground" }
                            span { {t!(*key)} }
                        }
                    }
                }
            }
            if state.visible("priority") {
                td { class: tw_merge!(TD, "ps-3"),
                    if let Some((_, key, icon)) = priority {
                        div { class: "flex items-center gap-2",
                            Icon { icon: *icon, class: "size-4 text-muted-foreground" }
                            span { {t!(*key)} }
                        }
                    }
                }
            }
            td { class: TD, RowActions { task } }
        }
    }
}
