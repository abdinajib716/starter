//! Small helpers shared by server and client (port of shadcn-admin `src/lib/*`).
pub mod cookies;

/// `lib/show-submitted-data.tsx`: a toast with the submitted values as JSON (used by the
/// reference's demo forms).
pub fn show_submitted_data(data: &serde_json::Value, title: &str) {
    let title = serde_json::to_string(title).unwrap_or_default();
    let desc = serde_json::to_string(&serde_json::to_string_pretty(data).unwrap_or_default())
        .unwrap_or_default();
    let js = format!(
        r#"(() => {{ const t = document.createElement('button'); t.type = 'button';
            t.dataset.name = 'ToastTrigger'; t.dataset.variant = 'Default';
            t.dataset.toastTitle = {title}; t.dataset.toastDescription = {desc};
            document.body.appendChild(t); t.click(); t.remove(); }})()"#
    );
    let _ = dioxus::document::eval(&js);
}
