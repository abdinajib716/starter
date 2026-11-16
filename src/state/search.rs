use dioxus::prelude::*;

/// Command palette open state (search-provider.tsx); toggled by the search button and
/// by Ctrl/Cmd+K (registered in `AppShell`).
#[derive(Clone, Copy)]
pub struct SearchOpen(pub Signal<bool>);
