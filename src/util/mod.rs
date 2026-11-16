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

/// sonner's `toast.promise(sleep(ms), { loading, success })` used by the reference's demo bulk
/// actions: a Loading toast that is replaced by a Success toast after `ms`.
pub fn toast_promise(loading: impl Into<String>, success: impl Into<String>, ms: u32) {
    let loading = serde_json::to_string(&loading.into()).unwrap_or_default();
    let success = serde_json::to_string(&success.into()).unwrap_or_default();
    let js = format!(
        r#"(() => {{
            const fire = (variant, title) => {{ const t = document.createElement('button'); t.type = 'button';
                t.dataset.name = 'ToastTrigger'; t.dataset.variant = variant;
                t.dataset.toastTitle = title; t.dataset.toastDescription = '';
                document.body.appendChild(t); t.click(); t.remove(); }};
            fire('Loading', {loading});
            const items = document.querySelectorAll('[data-toast-toast][data-variant=Loading]');
            const el = items[items.length - 1];
            setTimeout(() => {{
                if (el && typeof dismissToast === 'function') dismissToast(el); else if (el) el.remove();
                fire('Success', {success});
            }}, {ms});
        }})()"#
    );
    let _ = dioxus::document::eval(&js);
}
