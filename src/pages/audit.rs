//! Audit log page (§9.5, M4.4): search, module / result filters, date range (all in the URL),
//! keyset paging (Newer / Older), expandable rows with the before/after diff, CSV export.
//! Same table look as the reference data tables.
use dioxus::prelude::*;
use dioxus_i18n::t;
use tw_merge::tw_merge;

use crate::api::audit::list_audit;
use crate::app::Route;
use crate::components::data_table::{FilterDef, FilterOption, TD, TH, TR, TableState, Toolbar};
use crate::components::layout::{
    ConfigDrawer, Header, Main, ProfileDropdown, Search, ThemeSwitch, use_current_user,
};
use crate::components::permission_gate::PermissionGate;
use crate::components::skeletons::TableSkeleton;
use crate::components::ui_extra::form::DatePicker;
use crate::components::ui_extra::icon::{Icon, IconType};
use crate::domain::audit::{AuditCursor, AuditPage, AuditQuery, AuditRow, diff};
use crate::pages::dashboard::action_label;
use crate::pages::errors::{ErrorKind, ErrorView};
use crate::ui::button::{Button, ButtonVariant};
use crate::util::query::Query;

const FILTER_COLUMNS: &[&str] = &["module", "result"];
const PAGE: u32 = 25;
const BADGE: &str = "inline-flex w-fit shrink-0 items-center justify-center rounded-md border px-2 py-0.5 text-xs font-medium whitespace-nowrap";

fn parse_date(s: Option<&str>) -> Option<time::Date> {
    s.and_then(|v| {
        time::Date::parse(
            v,
            &time::macros::format_description!("[year]-[month]-[day]"),
        )
        .ok()
    })
}

fn iso(d: time::Date) -> String {
    format!("{:04}-{:02}-{:02}", d.year(), d.month() as u8, d.day())
}

#[component]
pub fn Audit(query: Query) -> Element {
    let allowed = use_current_user().is_some_and(|u| u.can("audit.view"));
    if !allowed {
        return rsx! { ErrorView { kind: ErrorKind::Forbidden, minimal: true, class: "h-[calc(100svh-4rem)]" } };
    }
    rsx! {
        Header { fixed: true,
            Search { class: "me-auto" }
            ThemeSwitch {}
            ConfigDrawer {}
            ProfileDropdown {}
        }
        Main { class: "flex flex-1 flex-col gap-4 sm:gap-6",
            div { class: "flex flex-wrap items-end justify-between gap-2",
                div {
                    h2 { class: "text-2xl font-bold tracking-tight", {t!("audit-title")} }
                    p { class: "text-muted-foreground", {t!("audit-desc")} }
                }
                PermissionGate { perm: "audit.export",
                    crate::components::export_menu::ExportMenu { on_export: move |f: crate::components::export_menu::Format| download(f.ext()) }
                }
            }
            AuditTable { query }
        }
    }
}

/// `/api/export/audit.{csv,pdf}` with the filters currently in the URL.
fn download(ext: &str) {
    let _ = document::eval(&format!(
        "(() => {{ const p = new URLSearchParams(location.search);
           const q = new URLSearchParams();
           for (const [a, b] of [['filter','q'],['module','module'],['result','result'],['from','from'],['to','to']]) {{ if (p.get(a)) q.set(b, p.get(a)); }}
           const el = document.createElement('a'); el.href = '/api/export/audit.{ext}?' + q; el.download = 'audit.{ext}';
           document.body.appendChild(el); el.click(); el.remove(); }})()"
    ));
}

#[component]
fn AuditTable(query: Query) -> Element {
    let tz = crate::components::layout::use_tz();
    let state = TableState::use_from_query(&query, "filter", FILTER_COLUMNS);
    let from = use_signal(|| parse_date(query.get("from")));
    let to = use_signal(|| parse_date(query.get("to")));
    // Keyset paging: the cursors of the pages above the current one.
    let mut cursors = use_signal(Vec::<Option<AuditCursor>>::new);
    let mut cursor = use_signal(|| None::<AuditCursor>);
    let mut open_row = use_signal(|| None::<i64>);

    // URL sync (no `page`: keyset pages are not addressable).
    let mut last = use_signal(|| query.to_string());
    use_effect(move || {
        let q = state
            .to_query(FILTER_COLUMNS)
            .with("page", None::<String>)
            .with("pageSize", None::<String>)
            .with("from", from().map(iso))
            .with("to", to().map(iso));
        if q.to_string() != *last.peek() {
            last.set(q.to_string());
            navigator().replace(Route::Audit { query: q });
        }
    });
    // A filter change starts again from the newest row.
    use_effect(move || {
        let _ = ((state.filter)(), (state.column_filters)(), from(), to());
        cursors.write().clear();
        cursor.set(None);
    });

    let mut data = use_signal(|| None::<AuditPage>);
    let mut loaded_q = use_signal(|| None::<String>);
    let live = crate::components::layout::use_live();
    let _list = use_resource(move || async move {
        let _ = (live.audit)();
        let result = state.column_filter("result");
        let q = AuditQuery {
            q: (state.filter)().trim().to_string(),
            modules: state.column_filter("module"),
            success: match result.as_slice() {
                [r] if r == "success" => Some(true),
                [r] if r == "failed" => Some(false),
                _ => None,
            },
            from: from(),
            to: to(),
            before: cursor(),
            limit: PAGE,
        };
        if loaded_q.peek().as_ref().is_some_and(|l| *l != q.q) {
            crate::util::sleep_ms(250).await;
        }
        match list_audit(q.clone()).await {
            Ok(p) => {
                loaded_q.set(Some(q.q));
                data.set(Some(p));
            }
            Err(e) => crate::util::toast_error(&e),
        }
    });

    let Some(page) = data() else {
        return rsx! { TableSkeleton { cols: 5 } };
    };
    let filters = vec![
        FilterDef {
            column: "module",
            title: t!("audit-col-module"),
            options: page
                .modules
                .iter()
                .map(|m| FilterOption {
                    value: m.clone(),
                    label: m.clone(),
                    icon: None,
                })
                .collect(),
            counts: vec![],
        },
        FilterDef {
            column: "result",
            title: t!("audit-col-result"),
            options: vec![
                FilterOption {
                    value: "success".into(),
                    label: t!("audit-success"),
                    icon: Some(IconType::CircleCheck),
                },
                FilterOption {
                    value: "failed".into(),
                    label: t!("audit-failed"),
                    icon: Some(IconType::CircleX),
                },
            ],
            counts: vec![],
        },
    ];
    let columns = vec![
        ("entity", t!("audit-col-entity")),
        ("ip", t!("audit-col-ip")),
    ];
    let cols = 4 + columns.iter().filter(|(c, _)| state.visible(c)).count();
    let has_newer = !cursors().is_empty();
    let next = page.next;
    let dates_set = from().is_some() || to().is_some();

    rsx! {
        div { class: "flex flex-1 flex-col gap-4",
            Toolbar { state, placeholder: t!("audit-search"), filters, columns }
            div { class: "flex flex-wrap items-center gap-2",
                DatePicker { selected: from, placeholder: t!("audit-from"), class: "h-8 w-44 px-3" }
                DatePicker { selected: to, placeholder: t!("audit-to"), class: "h-8 w-44 px-3" }
                if dates_set {
                    Button { variant: ButtonVariant::Ghost, class: "h-8 px-2 lg:px-3",
                        onclick: move |_| { let (mut f, mut t2) = (from, to); f.set(None); t2.set(None); },
                        {t!("dt-reset")}
                        Icon { icon: IconType::X, class: "ms-2 h-4 w-4" }
                    }
                }
            }
            div { class: "overflow-hidden rounded-md border",
                div { class: "relative w-full overflow-x-auto",
                    table { class: "w-full caption-bottom text-sm",
                        thead { class: "[&_tr]:border-b",
                            tr { class: TR,
                                th { class: TH, {crate::ta!("audit-col-time", zone: crate::util::time::zone_label(&tz))} }
                                th { class: TH, {t!("audit-col-actor")} }
                                th { class: TH, {t!("audit-col-action")} }
                                if state.visible("entity") { th { class: TH, {t!("audit-col-entity")} } }
                                th { class: TH, {t!("audit-col-result")} }
                                if state.visible("ip") { th { class: TH, {t!("audit-col-ip")} } }
                            }
                        }
                        tbody { class: "[&_tr:last-child]:border-0",
                            if page.rows.is_empty() {
                                tr { class: TR, td { class: tw_merge!(TD, "h-24 text-center"), colspan: "{cols}", {t!("dt-no-results")} } }
                            }
                            for row in page.rows.iter().cloned() {
                                AuditLine { key: "{row.id}", open: open_row() == Some(row.id), row, state, cols, on_toggle: move |id: i64| open_row.set(if open_row() == Some(id) { None } else { Some(id) }) }
                            }
                        }
                    }
                }
            }
            div { class: "mt-auto flex items-center justify-end gap-2 px-2",
                Button { variant: ButtonVariant::Outline, class: "h-8", disabled: !has_newer,
                    onclick: move |_| { let prev = cursors.write().pop().flatten(); cursor.set(prev); },
                    Icon { icon: IconType::ChevronLeft, class: "size-4" }
                    {t!("audit-newer")}
                }
                Button { variant: ButtonVariant::Outline, class: "h-8", disabled: next.is_none(),
                    onclick: move |_| {
                        if let Some(n) = next {
                            let cur = cursor();
                            cursors.write().push(cur);
                            cursor.set(Some(n));
                        }
                    },
                    {t!("audit-older")}
                    Icon { icon: IconType::ChevronRight, class: "size-4" }
                }
            }
        }
    }
}

#[component]
fn AuditLine(
    row: AuditRow,
    open: bool,
    state: TableState,
    cols: usize,
    on_toggle: EventHandler<i64>,
) -> Element {
    let tz = crate::components::layout::use_tz();
    let id = row.id;
    let changes = diff(row.before.as_deref(), row.after.as_deref());
    let entity = match (&row.entity_type, &row.entity_id) {
        (Some(t), Some(i)) => format!("{t} · {}", i.chars().take(8).collect::<String>()),
        (Some(t), None) => t.clone(),
        _ => String::new(),
    };
    rsx! {
        tr {
            class: "{TR} cursor-pointer",
            "data-state": if open { "selected" } else { "" },
            "aria-expanded": "{open}",
            onclick: move |_| on_toggle.call(id),
            td { class: tw_merge!(TD, "text-muted-foreground tabular-nums"), "{crate::util::time::datetime_secs(row.at, &tz)}" }
            td { class: TD, {row.actor_email.clone().unwrap_or_else(|| t!("dash-system"))} }
            td { class: TD,
                div { class: "font-medium", "{action_label(&row.action)}" }
                div { class: "font-mono text-xs text-muted-foreground", "{row.action}" }
            }
            if state.visible("entity") { td { class: tw_merge!(TD, "text-muted-foreground"), "{entity}" } }
            td { class: TD,
                if row.success {
                    span { class: "{BADGE} bg-teal-100/30 text-teal-900 dark:text-teal-200 border-teal-200", {t!("audit-success")} }
                } else {
                    span { class: "{BADGE} bg-destructive/10 text-destructive border-destructive/10 dark:bg-destructive/50 dark:text-primary", {t!("audit-failed")} }
                }
            }
            if state.visible("ip") { td { class: tw_merge!(TD, "font-mono text-xs text-muted-foreground"), {row.ip.clone().unwrap_or_default()} } }
        }
        if open {
            tr { class: "border-b bg-muted/30",
                td { class: "p-4", colspan: "{cols}",
                    div { class: "grid gap-4 lg:grid-cols-3",
                        div { class: "space-y-1 text-xs text-muted-foreground",
                            p { span { class: "font-medium text-foreground", {t!("audit-channel")} } " {row.channel}" }
                            if let Some(e) = &row.entity_id { p { span { class: "font-medium text-foreground", {t!("audit-col-entity")} } " {e}" } }
                            if let Some(ip) = &row.ip { p { span { class: "font-medium text-foreground", "IP" } " {ip}" } }
                            if let Some(ua) = &row.user_agent { p { class: "break-all", span { class: "font-medium text-foreground", {t!("audit-agent")} } " {ua}" } }
                        }
                        div { class: "lg:col-span-2",
                            if changes.is_empty() {
                                p { class: "text-sm text-muted-foreground", {t!("audit-no-changes")} }
                            } else {
                                table { class: "w-full text-xs",
                                    thead {
                                        tr { class: "text-start text-muted-foreground",
                                            th { class: "pb-1 text-start font-medium", {t!("audit-field")} }
                                            th { class: "pb-1 text-start font-medium", {t!("audit-before")} }
                                            th { class: "pb-1 text-start font-medium", {t!("audit-after")} }
                                        }
                                    }
                                    tbody {
                                        for (k, b, a) in changes {
                                            tr { class: "border-t align-top",
                                                td { class: "py-1 pe-3 font-mono", "{k}" }
                                                td { class: "py-1 pe-3 break-all text-destructive/80 line-through decoration-destructive/30", "{b}" }
                                                td { class: "py-1 break-all text-teal-700 dark:text-teal-300", "{a}" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
