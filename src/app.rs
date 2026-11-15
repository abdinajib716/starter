//! Root component, route table and document head.
use dioxus::prelude::*;

use crate::pages::home::Home;
use crate::ui::toast::ToastToaster;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[route("/")]
    Home {},
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");
// Rust/UI runtimes (copied from rust-ui/ui @ 4f5a516)
const TOAST_RUNTIME_JS: Asset = asset!("/public/app_components/toast.js");
const TOAST_RUNTIME_CSS: Asset = asset!("/public/app_components/toast.css");
const APEXCHARTS_JS: Asset = asset!("/public/cdn/apexcharts.5.3.6.min.js");
const CHART_INIT_JS: Asset = asset!("/public/app_components/chart_init.js");

#[component]
pub fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        // Dioxus mounts the app inside <div id="main">; give it full height so
        // Rust/UI's html(100dvh) -> body(h-full) chain keeps working.
        document::Style { "#main {{ height: 100%; overflow-y: auto; }}" }
        document::Stylesheet { href: TAILWIND_CSS }
        document::Stylesheet { href: TOAST_RUNTIME_CSS }
        document::Script { src: APEXCHARTS_JS }
        document::Script { src: CHART_INIT_JS }
        ToastToaster {}
        script { src: TOAST_RUNTIME_JS }
        Router::<Route> {}
    }
}
