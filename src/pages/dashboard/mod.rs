//! Dashboard on real data (§10): KPI cards, sign-in chart and recent activity (Overview);
//! failed sign-ins, backups, audit activity by module, users by role and new users per week
//! (Activity). Layout and card styles from the reference's `features/dashboard`.
mod activity;
pub mod overview;

use dioxus::prelude::*;
use dioxus_i18n::t;

use crate::api::dashboard::get_dashboard;
use crate::components::layout::{ConfigDrawer, Header, Main, ProfileDropdown, Search, ThemeSwitch};
use crate::components::skeletons::CardsSkeleton;
use crate::domain::dashboard::DashboardData;
use crate::ui::card::{Card, CardContent, CardHeader, CardTitle};
use crate::ui::tabs::{Tabs, TabsContent, TabsList, TabsTrigger};

/// Rust/UI → shadcn differences, fixed by class overrides (src/ui stays vendored).
const TAB_CONTENT: &str = "space-y-4 text-base group-data-[orientation=Horizontal]/tabs:mt-0";
const TAB_TRIGGER: &str = "px-2 py-1 text-foreground";
/// shadcn cards use `gap-6` (Rust/UI default `gap-4`).
pub(crate) const CARD: &str = "gap-6";
/// Reload interval while the page is open (live updates arrive with SSE in M8).
const REFRESH_MS: u32 = 30_000;

#[component]
pub fn Dashboard() -> Element {
    let mut data = use_signal(|| None::<DashboardData>);
    let mut failed = use_signal(|| false);
    // Reload on every live signal (SSE, debounced on the server), plus a quiet refresh
    // every 30 s as a fallback; the server caches for 5 s.
    let live = crate::components::layout::use_live();
    let _live = use_resource(move || async move {
        if (live.dashboard)() == 0 {
            return;
        }
        if let Ok(d) = get_dashboard().await {
            data.set(Some(d));
            failed.set(false);
        }
    });
    use_future(move || async move {
        loop {
            match get_dashboard().await {
                Ok(d) => {
                    data.set(Some(d));
                    failed.set(false);
                }
                Err(e) => {
                    if !failed() {
                        crate::util::toast_error(&e);
                    }
                    failed.set(true);
                }
            }
            crate::util::sleep_ms(REFRESH_MS).await;
        }
    });

    rsx! {
        Header {
            Search { class: "me-auto" }
            ThemeSwitch {}
            ConfigDrawer {}
            ProfileDropdown {}
        }
        Main {
            div { class: "mb-2 flex items-center justify-between space-y-2",
                h1 { class: "text-2xl font-bold tracking-tight", {t!("nav-dashboard")} }
            }
            match data() {
                None => rsx! { CardsSkeleton {} },
                Some(d) => rsx! {
                    Tabs { default_value: "overview", class: "space-y-4",
                        div { class: "w-full overflow-x-auto pb-2",
                            TabsList { class: "h-9 p-0.75",
                                TabsTrigger { value: "overview", class: TAB_TRIGGER, {t!("dash-tab-overview")} }
                                crate::components::feature_gate::FeatureGate { flag: "dashboard_activity",
                                    TabsTrigger { value: "activity", class: TAB_TRIGGER, {t!("dash-tab-activity")} }
                                }
                            }
                        }
                        TabsContent { value: "overview", class: TAB_CONTENT, overview::Overview { data: d.clone() } }
                        TabsContent { value: "activity", class: TAB_CONTENT, activity::Activity { data: d } }
                    }
                },
            }
        }
    }
}

/// The inline SVG icons of the reference's KPI cards.
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum KpiIcon {
    Users,
    Pulse,
    Online,
    LogIn,
    Alert,
    Archive,
    List,
    Shield,
}

#[component]
pub(crate) fn KpiSvg(icon: KpiIcon) -> Element {
    use crate::components::ui_extra::icon::{Icon, IconType};
    let i = match icon {
        KpiIcon::Users => IconType::Users,
        KpiIcon::Pulse => IconType::Activity,
        KpiIcon::Online => IconType::Radio,
        KpiIcon::LogIn => IconType::LogIn,
        KpiIcon::Alert => IconType::ShieldAlert,
        KpiIcon::Archive => IconType::DatabaseBackup,
        KpiIcon::List => IconType::ScrollText,
        KpiIcon::Shield => IconType::ShieldCheck,
    };
    rsx! { Icon { icon: i, class: "h-4 w-4 text-muted-foreground" } }
}

/// One KPI card (title + icon, value, note).
#[component]
pub(crate) fn Kpi(title: String, value: String, note: String, icon: KpiIcon) -> Element {
    rsx! {
        Card { class: CARD,
            CardHeader { class: "flex flex-row items-center justify-between space-y-0 pb-2 sm:flex",
                CardTitle { class: "text-sm font-medium", "{title}" }
                KpiSvg { icon }
            }
            CardContent {
                div { class: "text-2xl font-bold tabular-nums", "{value}" }
                p { class: "text-xs text-muted-foreground", "{note}" }
            }
        }
    }
}

/// Signed change between two periods, as the reference's "+20.1% from last month".
pub(crate) fn delta(now: i64, before: i64) -> String {
    if before == 0 {
        return if now == 0 {
            "0".into()
        } else {
            format!("+{now}")
        };
    }
    let pct = (now - before) as f64 / before as f64 * 100.0;
    format!("{}{:.1}%", if pct >= 0.0 { "+" } else { "" }, pct)
}

/// "1,234" — thousands separators.
pub(crate) fn num(n: i64) -> String {
    let s = n.abs().to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    if n < 0 { format!("-{out}") } else { out }
}

/// "2026-11-03" → "Nov 3".
pub(crate) fn short_date(iso: &str) -> String {
    let mut p = iso.split('-');
    let (_, m, d) = (
        p.next(),
        p.next().and_then(|m| m.parse::<usize>().ok()),
        p.next().and_then(|d| d.parse::<u32>().ok()),
    );
    const M: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    match (m, d) {
        (Some(m @ 1..=12), Some(d)) => format!("{} {d}", M[m - 1]),
        _ => iso.to_string(),
    }
}

/// Translated label of an audit action (`user.create` → `audit-user-create`), or the raw code.
pub(crate) fn action_label(action: &str) -> String {
    let key = format!("audit-{}", action.replace(['.', '_'], "-"));
    dioxus_i18n::te!(&key).unwrap_or_else(|_| action.to_string())
}

#[component]
pub(crate) fn Panel(
    title: String,
    #[props(into, optional)] description: Option<String>,
    #[props(into, default = "col-span-1 lg:col-span-4".to_string())] class: String,
    children: Element,
) -> Element {
    rsx! {
        Card { class: "{CARD} {class}",
            CardHeader {
                CardTitle { "{title}" }
                if let Some(d) = description { crate::ui::card::CardDescription { "{d}" } }
            }
            CardContent { {children} }
        }
    }
}

/// Horizontal bars with a value (users by role).
#[component]
pub(crate) fn BarList(items: Vec<(String, f64)>, bar_class: &'static str) -> Element {
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
                    div { class: "ps-2 text-xs font-medium tabular-nums", "{value}" }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formatting_helpers() {
        assert_eq!(num(1234567), "1,234,567");
        assert_eq!(num(12), "12");
        assert_eq!(delta(12, 10), "+20.0%");
        assert_eq!(delta(5, 10), "-50.0%");
        assert_eq!(delta(3, 0), "+3");
        assert_eq!(short_date("2026-11-03"), "Nov 3");
    }
}
