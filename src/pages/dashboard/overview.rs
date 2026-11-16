use dioxus::prelude::*;

use crate::components::ui_extra::chart::BarChart;

/// `overview.tsx`: monthly totals (the reference randomises 1,000–6,000 per load; fixed
/// here so server and client render the same chart).
#[component]
pub fn Overview() -> Element {
    rsx! {
        BarChart {
            labels: vec!["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"],
            values: vec![4210.0, 3150.0, 5620.0, 2840.0, 4930.0, 3610.0, 5180.0, 4420.0, 1920.0, 5790.0, 2960.0, 4710.0],
            height: 350.0,
            y_prefix: "$",
        }
    }
}
