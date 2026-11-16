use dioxus::prelude::*;

use crate::ui::avatar::{Avatar, AvatarFallback};

/// `recent-sales.tsx` sample rows (the reference's avatar images are not shipped; it shows
/// the initials fallback, as here).
const SALES: [(&str, &str, &str, &str); 5] = [
    (
        "OM",
        "Olivia Martin",
        "olivia.martin@email.com",
        "+$1,999.00",
    ),
    ("JL", "Jackson Lee", "jackson.lee@email.com", "+$39.00"),
    (
        "IN",
        "Isabella Nguyen",
        "isabella.nguyen@email.com",
        "+$299.00",
    ),
    ("WK", "William Kim", "will@email.com", "+$99.00"),
    ("SD", "Sofia Davis", "sofia.davis@email.com", "+$39.00"),
];

#[component]
pub fn RecentSales() -> Element {
    rsx! {
        div { class: "space-y-8",
            for (i, (initials, name, email, amount)) in SALES.iter().enumerate() {
                div { class: "flex items-center gap-4",
                    Avatar { class: if i == 1 { "flex h-9 w-9 items-center justify-center space-y-0 border" } else { "h-9 w-9" },
                        AvatarFallback { class: "text-foreground", "{initials}" }
                    }
                    div { class: "flex flex-1 flex-wrap items-center justify-between",
                        div { class: "space-y-1",
                            p { class: "text-sm leading-none font-medium", "{name}" }
                            p { class: "text-sm text-muted-foreground", "{email}" }
                        }
                        div { class: "font-medium", "{amount}" }
                    }
                }
            }
        }
    }
}
