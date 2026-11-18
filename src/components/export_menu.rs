//! Export button with a format menu (CSV for data, PDF for printing / sharing).
use dioxus::prelude::*;
use dioxus_i18n::t;

use crate::components::ui_extra::icon::{Icon, IconType};
use crate::components::ui_extra::menu::{Align, ITEM, Menu};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Format {
    Csv,
    Pdf,
}

impl Format {
    pub fn ext(self) -> &'static str {
        match self {
            Self::Csv => "csv",
            Self::Pdf => "pdf",
        }
    }
}

#[component]
pub fn ExportMenu(on_export: EventHandler<Format>) -> Element {
    rsx! {
        Menu {
            align: Align::End,
            class: "w-44",
            trigger: rsx! {
                button { r#type: "button", class: "inline-flex h-9 items-center justify-center gap-2 rounded-md border bg-background px-4 py-2 text-sm font-medium shadow-xs transition-all hover:bg-accent hover:text-accent-foreground dark:border-input dark:bg-input/30",
                    span { {t!("users-export")} }
                    Icon { icon: IconType::Download, class: "size-[18px]" }
                }
            },
            button { r#type: "button", class: ITEM, onclick: move |_| on_export.call(Format::Csv),
                Icon { icon: IconType::FileSpreadsheet, class: "size-4" }
                {t!("export-csv")}
            }
            button { r#type: "button", class: ITEM, onclick: move |_| on_export.call(Format::Pdf),
                Icon { icon: IconType::FileText, class: "size-4" }
                {t!("export-pdf")}
            }
        }
    }
}
