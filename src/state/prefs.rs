use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

use crate::util::cookies;

const YEAR: u32 = 60 * 60 * 24 * 365;
const WEEK: u32 = 60 * 60 * 24 * 7;

pub const THEME_COOKIE: &str = "theme";
pub const SIDEBAR_COOKIE: &str = "sidebar_state";
pub const VARIANT_COOKIE: &str = "layout_variant";
pub const COLLAPSIBLE_COOKIE: &str = "layout_collapsible";
pub const DIR_COOKIE: &str = "dir";

/// Theme choice (theme-provider.tsx). `System` follows `prefers-color-scheme`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Theme {
    Light,
    Dark,
    #[default]
    System,
}

impl Theme {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Light => "light",
            Self::Dark => "dark",
            Self::System => "system",
        }
    }
    pub fn parse(s: &str) -> Self {
        match s {
            "light" => Self::Light,
            "dark" => Self::Dark,
            _ => Self::System,
        }
    }
}

/// Sidebar look (layout-provider.tsx, default `inset`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Variant {
    #[default]
    Inset,
    Sidebar,
    Floating,
}

impl Variant {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Inset => "inset",
            Self::Sidebar => "sidebar",
            Self::Floating => "floating",
        }
    }
    pub fn parse(s: &str) -> Self {
        match s {
            "sidebar" => Self::Sidebar,
            "floating" => Self::Floating,
            _ => Self::Inset,
        }
    }
}

/// How the sidebar collapses (layout-provider.tsx, default `icon`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Collapsible {
    #[default]
    Icon,
    Offcanvas,
    None,
}

impl Collapsible {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Icon => "icon",
            Self::Offcanvas => "offcanvas",
            Self::None => "none",
        }
    }
    pub fn parse(s: &str) -> Self {
        match s {
            "offcanvas" => Self::Offcanvas,
            "none" => Self::None,
            _ => Self::Icon,
        }
    }
}

/// Text direction (direction-provider.tsx, default `ltr`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Dir {
    #[default]
    Ltr,
    Rtl,
}

impl Dir {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ltr => "ltr",
            Self::Rtl => "rtl",
        }
    }
    pub fn parse(s: &str) -> Self {
        if s == "rtl" { Self::Rtl } else { Self::Ltr }
    }
}

/// Preferences read from the request cookies during SSR.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct UiPrefs {
    pub theme: Theme,
    pub variant: Variant,
    pub collapsible: Collapsible,
    pub sidebar_open: bool,
    pub dir: Dir,
}

impl UiPrefs {
    pub fn from_request() -> Self {
        let c = |n| cookies::request(n).unwrap_or_default();
        Self {
            theme: Theme::parse(&c(THEME_COOKIE)),
            variant: Variant::parse(&c(VARIANT_COOKIE)),
            collapsible: Collapsible::parse(&c(COLLAPSIBLE_COOKIE)),
            sidebar_open: c(SIDEBAR_COOKIE) != "false",
            dir: Dir::parse(&c(DIR_COOKIE)),
        }
    }
}

/// Shared, reactive UI preferences. Provided once by `App`.
#[derive(Clone, Copy)]
pub struct Ui {
    pub theme: Signal<Theme>,
    pub variant: Signal<Variant>,
    pub collapsible: Signal<Collapsible>,
    pub dir: Signal<Dir>,
    pub sidebar_open: Signal<bool>,
}

impl Ui {
    pub fn provide(prefs: UiPrefs) -> Self {
        let ui = Self {
            theme: Signal::new(prefs.theme),
            variant: Signal::new(prefs.variant),
            collapsible: Signal::new(prefs.collapsible),
            dir: Signal::new(prefs.dir),
            sidebar_open: Signal::new(prefs.sidebar_open),
        };
        provide_context(ui)
    }

    pub fn get() -> Self {
        use_context::<Ui>()
    }

    pub fn set_theme(mut self, t: Theme) {
        cookies::set(THEME_COOKIE, t.as_str(), YEAR);
        self.theme.set(t);
        apply_theme(t);
    }

    pub fn set_variant(mut self, v: Variant) {
        cookies::set(VARIANT_COOKIE, v.as_str(), WEEK);
        self.variant.set(v);
    }

    pub fn set_collapsible(mut self, c: Collapsible) {
        cookies::set(COLLAPSIBLE_COOKIE, c.as_str(), WEEK);
        self.collapsible.set(c);
    }

    pub fn set_dir(mut self, d: Dir) {
        cookies::set(DIR_COOKIE, d.as_str(), YEAR);
        self.dir.set(d);
        let _ = document::eval(&format!("document.documentElement.dir = '{}'", d.as_str()));
    }

    pub fn set_sidebar_open(mut self, open: bool) {
        cookies::set(SIDEBAR_COOKIE, if open { "true" } else { "false" }, WEEK);
        self.sidebar_open.set(open);
    }

    /// Back to the defaults (config drawer "Reset").
    pub fn reset(self) {
        for name in [THEME_COOKIE, VARIANT_COOKIE, COLLAPSIBLE_COOKIE, DIR_COOKIE] {
            cookies::remove(name);
        }
        let d = UiPrefs {
            theme: Theme::default(),
            variant: Variant::default(),
            collapsible: Collapsible::default(),
            sidebar_open: true,
            dir: Dir::default(),
        };
        let mut s = self;
        s.theme.set(d.theme);
        s.variant.set(d.variant);
        s.collapsible.set(d.collapsible);
        s.dir.set(d.dir);
        self.set_sidebar_open(true);
        apply_theme(d.theme);
        let _ = document::eval("document.documentElement.dir = 'ltr'");
    }
}

/// Put `dark`/`light` on `<html>`; `System` follows the OS setting (and its changes).
/// The inline script in `index.html` does the same before first paint.
pub fn apply_theme(t: Theme) {
    let js = format!(
        "(() => {{ const t = '{}'; const mq = matchMedia('(prefers-color-scheme: dark)');
           const apply = () => {{ const dark = t === 'dark' || (t === 'system' && mq.matches);
             document.documentElement.classList.remove('light', 'dark');
             document.documentElement.classList.add(dark ? 'dark' : 'light'); }};
           apply(); if (window.__themeMq) window.__themeMq(); 
           if (t === 'system') {{ mq.addEventListener('change', apply); window.__themeMq = () => mq.removeEventListener('change', apply); }}
           else window.__themeMq = null; }})()",
        t.as_str()
    );
    let _ = document::eval(&js);
}
