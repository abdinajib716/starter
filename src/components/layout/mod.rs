//! App shell, ported from shadcn-admin `src/components/layout/*` (SPEC-02 P2–P3).
mod app_shell;
mod config_drawer;
mod header;
#[allow(dead_code)]
mod locale_switch;
mod main_area;
mod nav_config;
mod nav_group;
mod nav_user;
mod navigation_progress;
mod profile_dropdown;
mod search;
mod sidebar;
mod sign_out_dialog;
mod skip_to_main;
mod team_switcher;
mod theme_switch;

pub use app_shell::{AppShell, UserResource, use_current_user, use_tz};
pub use config_drawer::ConfigDrawer;
pub use header::Header;
// Shown again when the `i18n.enabled` setting exists (M7); the reference has no switcher.
#[allow(unused_imports)]
pub use locale_switch::LocaleSwitch;
pub use main_area::Main;
pub use navigation_progress::NavigationProgress;
pub use profile_dropdown::ProfileDropdown;
pub use search::Search;
pub use theme_switch::ThemeSwitch;
