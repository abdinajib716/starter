//! App shell. M1: a minimal signed-in layout (top bar + guard). M3 replaces it with the
//! full sidenav shell (§12).
mod app_layout;
mod locale_switch;

pub use app_layout::{AppLayout, UserResource};
pub use locale_switch::LocaleSwitch;
