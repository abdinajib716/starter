//! `PermissionGate` (§8, M2.6): render children only when the signed-in user holds `perm`.
//! Cosmetic only — every server function checks again.
use dioxus::prelude::*;

use crate::components::layout::use_current_user;

#[component]
pub fn PermissionGate(
    perm: &'static str,
    /// Shown instead when the permission is missing (default: nothing).
    #[props(optional)]
    fallback: Option<Element>,
    children: Element,
) -> Element {
    let allowed = use_current_user().is_some_and(|u| u.can(perm));
    if allowed {
        rsx! { {children} }
    } else {
        fallback.unwrap_or_else(|| rsx! {})
    }
}
