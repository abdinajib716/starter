//! `features/errors/*` + `routes/(errors)/*` + `routes/_authenticated/errors/$error.tsx`.
use dioxus::prelude::*;
use dioxus_i18n::t;
use tw_merge::tw_merge;

use crate::app::Route;
use crate::components::layout::{ConfigDrawer, Header, ProfileDropdown, Search, ThemeSwitch};
use crate::ui::button::{Button, ButtonVariant};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ErrorKind {
    Unauthorized,
    Forbidden,
    NotFound,
    General,
    Maintenance,
}

impl ErrorKind {
    /// `/errors/$error` slug → page; unknown slugs show 404 (reference `errorMap`).
    pub fn from_slug(s: &str) -> Self {
        match s {
            "unauthorized" => Self::Unauthorized,
            "forbidden" => Self::Forbidden,
            "internal-server-error" => Self::General,
            "maintenance-error" => Self::Maintenance,
            _ => Self::NotFound,
        }
    }
    fn texts(self) -> (&'static str, &'static str, &'static str, &'static str) {
        match self {
            Self::Unauthorized => ("401", "err401-title", "err401-desc-1", "err401-desc-2"),
            Self::Forbidden => ("403", "err403-title", "err403-desc-1", "err403-desc-2"),
            Self::NotFound => ("404", "err404-title", "err404-desc-1", "err404-desc-2"),
            Self::General => ("500", "err500-title", "err500-desc-1", "err500-desc-2"),
            Self::Maintenance => ("503", "err503-title", "err503-desc-1", "err503-desc-2"),
        }
    }
}

/// One error screen. `minimal` = `GeneralError minimal` (no code, no buttons).
#[component]
pub fn ErrorView(
    kind: ErrorKind,
    #[props(default)] minimal: bool,
    #[props(into, optional)] class: Option<String>,
) -> Element {
    let (code, title, d1, d2) = kind.texts();
    rsx! {
        div { class: tw_merge!("h-svh w-full", class.as_deref().unwrap_or("")),
            div { class: "m-auto flex h-full w-full flex-col items-center justify-center gap-2",
                if !minimal { h1 { class: "text-[7rem] leading-tight font-bold", "{code}" } }
                span { class: "font-medium", {t!(title)} }
                p { class: "text-center text-muted-foreground", {t!(d1)} br {} {t!(d2)} }
                if !minimal {
                    div { class: "mt-6 flex gap-4",
                        if kind == ErrorKind::Maintenance {
                            Button { variant: ButtonVariant::Outline, {t!("err-learn-more")} }
                        } else {
                            Button { variant: ButtonVariant::Outline, onclick: move |_| navigator().go_back(), {t!("err-go-back")} }
                            Button { onclick: move |_| { navigator().push(Route::Dashboard {}); }, {t!("err-back-home")} }
                        }
                    }
                }
            }
        }
    }
}

// Standalone routes (/401 … /503), public like the reference's `(errors)` group.
#[component]
pub fn Error401() -> Element {
    rsx! {
        ErrorView { kind: ErrorKind::Unauthorized }
    }
}
#[component]
pub fn Error403() -> Element {
    rsx! {
        ErrorView { kind: ErrorKind::Forbidden }
    }
}
#[component]
pub fn Error404() -> Element {
    rsx! {
        ErrorView { kind: ErrorKind::NotFound }
    }
}
#[component]
pub fn Error500() -> Element {
    rsx! {
        ErrorView { kind: ErrorKind::General }
    }
}
#[component]
pub fn Error503() -> Element {
    rsx! {
        ErrorView { kind: ErrorKind::Maintenance }
    }
}

/// `/errors/$error` inside the app shell.
#[component]
pub fn ShellError(error: String) -> Element {
    rsx! {
        Header { fixed: true, class: "border-b",
            Search { class: "me-auto" }
            ThemeSwitch {}
            ConfigDrawer {}
            ProfileDropdown {}
        }
        div { class: "flex-1 [&>div]:h-full", ErrorView { kind: ErrorKind::from_slug(&error) } }
    }
}

/// `coming-soon.tsx` (Help Center).
#[component]
pub fn ComingSoon() -> Element {
    rsx! {
        div { class: "h-svh",
            div { class: "m-auto flex h-full w-full flex-col items-center justify-center gap-2",
                crate::components::ui_extra::icon::Icon { icon: crate::components::ui_extra::icon::IconType::Telescope, class: "size-[72px]" }
                h1 { class: "text-4xl leading-tight font-bold", {t!("coming-soon-title")} }
                p { class: "text-center text-muted-foreground", {t!("coming-soon-desc-1")} br {} {t!("coming-soon-desc-2")} }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ErrorKind;

    #[test]
    fn slugs() {
        assert_eq!(ErrorKind::from_slug("forbidden"), ErrorKind::Forbidden);
        assert_eq!(
            ErrorKind::from_slug("internal-server-error"),
            ErrorKind::General
        );
        assert_eq!(ErrorKind::from_slug("nope"), ErrorKind::NotFound);
    }
}
