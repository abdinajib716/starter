//! Activity tab: security and backup KPIs, audit activity by module, users by role, new
//! users per week.
use dioxus::prelude::*;
use dioxus_i18n::t;

use super::overview::when;
use super::{BarList, Kpi, KpiIcon, Panel, delta, num, short_date};
use crate::components::ui_extra::chart::BarChart;
use crate::domain::dashboard::DashboardData;

#[component]
pub fn Activity(data: DashboardData) -> Element {
    let d = &data;
    let events_7d: i64 = d.activity_by_module.iter().map(|x| x.1).sum();
    let (backup_value, backup_note) = match &d.last_backup {
        Some(b) => (
            dioxus_i18n::te!(&format!("dash-backup-{}", b.status))
                .unwrap_or_else(|_| b.status.clone()),
            when(b.created_at),
        ),
        None => (t!("dash-backup-none"), t!("dash-backup-none-note")),
    };
    let modules: Vec<String> = d.activity_by_module.iter().map(|m| m.0.clone()).collect();
    let module_counts: Vec<f64> = d.activity_by_module.iter().map(|m| m.1 as f64).collect();
    let weeks: Vec<String> = d.new_users_12w.iter().map(|w| short_date(&w.0)).collect();
    let week_counts: Vec<f64> = d.new_users_12w.iter().map(|w| w.1 as f64).collect();
    rsx! {
        div { class: "grid gap-4 sm:grid-cols-2 lg:grid-cols-4",
            Kpi { title: t!("dash-failed-24h"), value: num(d.failed_24h), note: crate::ta!("dash-vs-prev-24h", delta: delta(d.failed_24h, d.failed_prev_24h)), icon: KpiIcon::Alert }
            Kpi { title: t!("dash-last-backup"), value: backup_value, note: backup_note, icon: KpiIcon::Archive }
            Kpi { title: t!("dash-events-7d"), value: num(events_7d), note: crate::ta!("dash-modules-n", n: d.activity_by_module.len()), icon: KpiIcon::List }
            Kpi { title: t!("dash-roles"), value: num(d.users_by_role.len() as i64), note: t!("dash-roles-note"), icon: KpiIcon::Shield }
        }
        div { class: "grid grid-cols-1 gap-4 lg:grid-cols-7",
            Panel { title: t!("dash-by-module"), description: t!("dash-by-module-desc"), class: "col-span-1 lg:col-span-4",
                if modules.is_empty() {
                    p { class: "text-sm text-muted-foreground", {t!("dash-recent-empty")} }
                } else {
                    BarChart { labels: modules, values: module_counts, height: 300.0, integer: true }
                }
            }
            Panel { title: t!("dash-by-role"), description: t!("dash-by-role-desc"), class: "col-span-1 lg:col-span-3",
                BarList { items: d.users_by_role.iter().map(|r| (r.0.clone(), r.1 as f64)).collect(), bar_class: "bg-primary" }
            }
        }
        Panel { title: t!("dash-new-users-12w"), description: t!("dash-new-users-12w-desc"), class: "",
            BarChart { labels: weeks, values: week_counts, height: 260.0, integer: true }
        }
    }
}
