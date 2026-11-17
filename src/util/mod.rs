//! Small helpers shared by server and client (port of shadcn-admin `src/lib/*`).
pub mod cookies;
pub mod query;

/// Toast that also works right before navigation. Rust/UI's `show_toast` runs its script in a
/// task owned by the calling component; when the component unmounts (e.g. login navigates
/// away) the task is dropped and the toast never appears. `eval` here runs immediately.
pub fn toast(variant: crate::ui::toast::ToastType, title: impl Into<String>) {
    let title = serde_json::to_string(&title.into()).unwrap_or_default();
    let js = format!(
        r#"(() => {{ const t = document.createElement('button'); t.type = 'button';
            t.dataset.name = 'ToastTrigger'; t.dataset.variant = {variant:?};
            t.dataset.toastTitle = {title}; t.dataset.toastDescription = '';
            document.body.appendChild(t); t.click(); t.remove(); }})()"#,
        variant = variant.to_string()
    );
    let _ = dioxus::document::eval(&js);
}

/// Wait `ms` milliseconds in the browser (debounce); returns at once during SSR.
pub async fn sleep_ms(ms: u32) {
    #[cfg(target_arch = "wasm32")]
    {
        let mut e = dioxus::document::eval(&format!(
            "await new Promise(r => setTimeout(r, {ms})); dioxus.send(true);"
        ));
        let _ = e.recv::<bool>().await;
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _ = ms;
}

/// Toast for a failed server call: the translated error code (§26 rule 2).
pub fn toast_error(e: &dioxus::prelude::ServerFnError) {
    toast(
        crate::ui::toast::ToastType::Error,
        crate::i18n::error_text(e),
    );
}
