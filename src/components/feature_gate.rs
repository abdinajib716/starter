//! `FeatureGate` (§27, M7.8): render children only when the feature flag is on for the
//! signed-in user. The server re-checks with `require_flag` where data is involved.
use dioxus::prelude::*;

use crate::components::layout::use_current_user;

#[component]
pub fn FeatureGate(
    flag: &'static str,
    #[props(optional)] fallback: Option<Element>,
    children: Element,
) -> Element {
    if use_current_user().is_some_and(|u| u.flag(flag)) {
        rsx! { {children} }
    } else {
        fallback.unwrap_or_else(|| rsx! {})
    }
}
