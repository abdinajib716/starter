use dioxus::prelude::*;
pub use icons::common::IconType;
use icons::common::icon_registry_getter::get_icon_elements;

/// A Lucide icon by enum value (so menus can store icons as data). Unlike `icons`' own
/// component it renders no `<title>`, which browsers would show as a hover tooltip.
#[component]
pub fn Icon(icon: IconType, #[props(into, optional)] class: Option<String>) -> Element {
    let elements = get_icon_elements(icon).unwrap_or(&[]);
    rsx! {
        svg {
            xmlns: "http://www.w3.org/2000/svg",
            width: "24",
            height: "24",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            "stroke-width": "2",
            "stroke-linecap": "round",
            "stroke-linejoin": "round",
            "aria-hidden": "true",
            class: class.unwrap_or_default(),
            for element in elements.iter() {
                {element.to_dioxus_element()}
            }
        }
    }
}
