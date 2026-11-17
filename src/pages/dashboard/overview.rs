//! Overview tab: users and sign-in KPIs, sign-ins over 14 days, recent activity.
use dioxus::prelude::*;
use dioxus_i18n::t;

use super::{Kpi, KpiIcon, Panel, action_label, delta, num, short_date};
use crate::components::ui_extra::chart::{AreaChart, Series};
use crate::domain::dashboard::DashboardData;
use crate::ui::avatar::{Avatar, AvatarFallback};

#[component]
pub fn Overview(data: DashboardData) -> Element {
    let d = &data;
    // Label every other day, counted back from today so the last date is always shown.
    let n = d.logins_14d.len();
    let labels: Vec<String> = d
        .logins_14d
        .iter()
        .enumerate()
        .map(|(i, (day, _, _))| {
            if !(n - 1 - i).is_multiple_of(2) {
                String::new()
            } else {
                short_date(day)
            }
        })
        .collect();
    let ok: Vec<f64> = d.logins_14d.iter().map(|x| x.1 as f64).collect();
    let bad: Vec<f64> = d.logins_14d.iter().map(|x| x.2 as f64).collect();
    rsx! {
        div { class: "grid gap-4 sm:grid-cols-2 lg:grid-cols-4",
            Kpi { title: t!("dash-total-users"), value: num(d.total_users), note: crate::ta!("dash-total-users-note", n: num(d.new_users_30d)), icon: KpiIcon::Users }
            Kpi { title: t!("dash-active-30d"), value: num(d.active_30d), note: crate::ta!("dash-vs-prev-30d", delta: delta(d.active_30d, d.active_prev_30d)), icon: KpiIcon::Pulse }
            Kpi { title: t!("dash-online"), value: num(d.online_now), note: t!("dash-online-note"), icon: KpiIcon::Online }
            Kpi { title: t!("dash-logins-today"), value: num(d.logins_today), note: crate::ta!("dash-vs-yesterday", n: num(d.logins_yesterday)), icon: KpiIcon::LogIn }
        }
        div { class: "grid grid-cols-1 gap-4 lg:grid-cols-7",
            Panel { title: t!("dash-signins-14d"), description: t!("dash-signins-14d-desc"), class: "col-span-1 lg:col-span-4",
                AreaChart {
                    labels,
                    series: vec![
                        Series { values: ok, class: "text-primary", fill_opacity: 0.15 },
                        Series { values: bad, class: "text-destructive", fill_opacity: 0.1 },
                    ],
                    height: 320.0,
                    integer: true,
                }
                div { class: "mt-3 flex items-center gap-4 ps-2 text-xs text-muted-foreground",
                    span { class: "flex items-center gap-1.5", span { class: "size-2 rounded-full bg-primary" } {t!("dash-legend-ok")} }
                    span { class: "flex items-center gap-1.5", span { class: "size-2 rounded-full bg-destructive" } {t!("dash-legend-failed")} }
                }
            }
            Panel { title: t!("dash-recent"), description: t!("dash-recent-desc"), class: "col-span-1 lg:col-span-3",
                if d.recent.is_empty() {
                    p { class: "text-sm text-muted-foreground", {t!("dash-recent-empty")} }
                }
                div { class: "space-y-6",
                    for item in d.recent.iter() {
                        div { key: "{item.id}", class: "flex items-center gap-4",
                            Avatar { class: "h-9 w-9",
                                AvatarFallback { class: "text-foreground text-xs", "{initials(item.actor_email.as_deref())}" }
                            }
                            div { class: "flex min-w-0 flex-1 flex-wrap items-center justify-between gap-x-2",
                                div { class: "min-w-0 space-y-1",
                                    p { class: if item.success { "truncate text-sm leading-none font-medium" } else { "truncate text-sm leading-none font-medium text-destructive" },
                                        "{action_label(&item.action)}"
                                    }
                                    p { class: "truncate text-sm text-muted-foreground", {item.actor_email.clone().unwrap_or_else(|| t!("dash-system"))} }
                                }
                                div { class: "text-xs whitespace-nowrap text-muted-foreground tabular-nums", "{when(item.at)}" }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn initials(email: Option<&str>) -> String {
    email
        .and_then(|e| e.split('@').next())
        .map(|n| {
            let parts: Vec<&str> = n.split(['.', '_', '-']).filter(|p| !p.is_empty()).collect();
            let s: String = match parts.as_slice() {
                [a, b, ..] => [a.chars().next(), b.chars().next()]
                    .into_iter()
                    .flatten()
                    .collect(),
                [a] => a.chars().take(2).collect(),
                [] => "?".into(),
            };
            s.to_uppercase()
        })
        .unwrap_or_else(|| "SY".into())
}

/// "Nov 16, 14:05" (UTC until the per-user time zone helpers of M3.11).
pub(crate) fn when(t: time::OffsetDateTime) -> String {
    let m = &format!("{:?}", t.month())[..3];
    format!("{m} {}, {:02}:{:02}", t.day(), t.hour(), t.minute())
}
