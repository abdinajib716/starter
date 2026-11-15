mod hooks;
mod ui;
use dioxus::prelude::*;
use ui::avatar::{Avatar, AvatarFallback, AvatarSize};
use ui::button::{Button, ButtonSize, ButtonVariant};
use ui::theme_toggle::ThemeToggle;
use ui::toast::{ToastToaster, ToastType, show_toast};
use ui::card::{Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle};

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(Navbar)]
    #[route("/")]
    Home {},
    #[route("/blog/:id")]
    Blog { id: i32 },
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const HEADER_SVG: Asset = asset!("/assets/header.svg");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");
// Rust/UI runtimes (copied from rust-ui/ui @ 4f5a516)
const TOAST_RUNTIME_JS: Asset = asset!("/public/app_components/toast.js");
const TOAST_RUNTIME_CSS: Asset = asset!("/public/app_components/toast.css");
const APEXCHARTS_JS: Asset = asset!("/public/cdn/apexcharts.5.3.6.min.js");
const CHART_INIT_JS: Asset = asset!("/public/app_components/chart_init.js");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
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

#[component]
pub fn Hero() -> Element {
    rsx! {
        div {
            id: "hero",
            img { src: HEADER_SVG, id: "header" }
            div { id: "links",
                a { href: "https://dioxuslabs.com/learn/0.7/", "📚 Learn Dioxus" }
                a { href: "https://dioxuslabs.com/awesome", "🚀 Awesome Dioxus" }
                a { href: "https://github.com/dioxus-community/", "📡 Community Libraries" }
                a { href: "https://github.com/DioxusLabs/sdk", "⚙️ Dioxus Development Kit" }
                a { href: "https://marketplace.visualstudio.com/items?itemName=DioxusLabs.dioxus", "💫 VSCode Extension" }
                a { href: "https://discord.gg/XgGxMSkvUM", "👋 Community Discord" }
            }
        }
    }
}

/// Home page: Rust/UI smoke test
#[component]
fn Home() -> Element {
    let mut count = use_signal(|| 0);

    rsx! {
        div { class: "min-h-screen bg-background text-foreground p-8 flex items-start justify-center",
            Card { class: "w-full max-w-md",
                CardHeader {
                    div { class: "flex items-center gap-3",
                        Avatar { size: AvatarSize::Lg,
                            AvatarFallback { "AM" }
                        }
                        div { class: "flex-1",
                            CardTitle { "Rust/UI is working" }
                            CardDescription { "Dioxus 0.7 + Tailwind v4 theme" }
                        }
                        ThemeToggle {}
                    }
                }
                CardContent {
                    p { "Clicked {count} times" }
                }
                CardFooter { class: "gap-2",
                    Button { onclick: move |_| count += 1, "Increment" }
                    Button { variant: ButtonVariant::Outline, onclick: move |_| count.set(0), "Reset" }
                    Button { variant: ButtonVariant::Destructive, size: ButtonSize::Sm,
                        onclick: move |_| show_toast(ToastType::Error, "Delete clicked (toast works)"),
                        "Delete"
                    }
                }
            }
        }
    }
}

/// Blog page
#[component]
pub fn Blog(id: i32) -> Element {
    rsx! {
        div {
            id: "blog",

            // Content
            h1 { "This is blog #{id}!" }
            p { "In blog #{id}, we show how the Dioxus router works and how URL parameters can be passed as props to our route components." }

            // Navigation links
            Link {
                to: Route::Blog { id: id - 1 },
                "Previous"
            }
            span { " <---> " }
            Link {
                to: Route::Blog { id: id + 1 },
                "Next"
            }
        }
    }
}

/// Shared navbar component.
#[component]
fn Navbar() -> Element {
    rsx! {
        div {
            id: "navbar",
            Link {
                to: Route::Home {},
                "Home"
            }
            Link {
                to: Route::Blog { id: 1 },
                "Blog"
            }
        }

        Outlet::<Route> {}
    }
}

/// Echo component that demonstrates fullstack server functions.
#[component]
fn Echo() -> Element {
    let mut response = use_signal(|| String::new());

    rsx! {
        div {
            id: "echo",
            h4 { "ServerFn Echo" }
            input {
                placeholder: "Type here to echo...",
                oninput:  move |event| async move {
                    let data = echo_server(event.value()).await.unwrap();
                    response.set(data);
                },
            }

            if !response().is_empty() {
                p {
                    "Server echoed: "
                    i { "{response}" }
                }
            }
        }
    }
}

/// Echo the user input on the server.
#[post("/api/echo")]
async fn echo_server(input: String) -> Result<String, ServerFnError> {
    Ok(input)
}
