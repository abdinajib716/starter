//! `analytics.tsx` + `analytics-chart.tsx`.
use dioxus::prelude::*;
use dioxus_i18n::t;

use super::{Kpi, KpiIcon};
use crate::components::ui_extra::chart::{AreaChart, Series};
use crate::ui::card::{Card, CardContent, CardDescription, CardHeader, CardTitle};

#[component]
pub fn Analytics() -> Element {
    rsx! {
        div { class: "space-y-4",
            Card { class: super::CARD,
                CardHeader {
                    CardTitle { {t!("dash-traffic")} }
                    CardDescription { {t!("dash-traffic-desc")} }
                }
                CardContent { class: "px-6",
                    AreaChart {
                        labels: vec!["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"],
                        series: vec![
                            Series { values: vec![640.0, 820.0, 510.0, 930.0, 700.0, 460.0, 780.0], class: "text-primary", fill_opacity: 0.15 },
                            Series { values: vec![410.0, 560.0, 300.0, 620.0, 480.0, 250.0, 520.0], class: "text-muted-foreground", fill_opacity: 0.1 },
                        ],
                    }
                }
            }
            div { class: "grid gap-4 sm:grid-cols-2 lg:grid-cols-4",
                Kpi { title: t!("dash-clicks"), value: "1,248", note: t!("dash-clicks-note"), icon: KpiIcon::Chart }
                Kpi { title: t!("dash-uniques"), value: "832", note: t!("dash-uniques-note"), icon: KpiIcon::User }
                Kpi { title: t!("dash-bounce"), value: "42%", note: t!("dash-bounce-note"), icon: KpiIcon::Bounce }
                Kpi { title: t!("dash-session"), value: "3m 24s", note: t!("dash-session-note"), icon: KpiIcon::Clock }
            }
            div { class: "grid grid-cols-1 gap-4 lg:grid-cols-7",
                Card { class: "col-span-1 gap-6 lg:col-span-4",
                    CardHeader {
                        CardTitle { {t!("dash-referrers")} }
                        CardDescription { {t!("dash-referrers-desc")} }
                    }
                    CardContent {
                        BarList {
                            items: vec![("Direct".into(), 512.0), ("Product Hunt".into(), 238.0), ("Twitter".into(), 174.0), ("Blog".into(), 104.0)],
                            bar_class: "bg-primary",
                            suffix: "",
                        }
                    }
                }
                Card { class: "col-span-1 gap-6 lg:col-span-3",
                    CardHeader {
                        CardTitle { {t!("dash-devices")} }
                        CardDescription { {t!("dash-devices-desc")} }
                    }
                    CardContent {
                        BarList {
                            items: vec![(t!("dash-desktop"), 74.0), (t!("dash-mobile"), 22.0), (t!("dash-tablet"), 4.0)],
                            bar_class: "bg-muted-foreground",
                            suffix: "%",
                        }
                    }
                }
            }
        }
    }
}

/// `SimpleBarList`: label, proportional bar, value.
#[component]
fn BarList(items: Vec<(String, f64)>, bar_class: &'static str, suffix: &'static str) -> Element {
    let max = items.iter().map(|i| i.1).fold(1.0, f64::max);
    rsx! {
        ul { class: "space-y-3",
            for (name, value) in items {
                li { class: "flex items-center justify-between gap-3",
                    div { class: "min-w-0 flex-1",
                        div { class: "mb-1 truncate text-xs text-muted-foreground", "{name}" }
                        div { class: "h-2.5 w-full rounded-full bg-muted",
                            div { class: "h-2.5 rounded-full {bar_class}", style: "width: {(value / max * 100.0).round()}%" }
                        }
                    }
                    div { class: "ps-2 text-xs font-medium tabular-nums", "{value}{suffix}" }
                }
            }
        }
    }
}
