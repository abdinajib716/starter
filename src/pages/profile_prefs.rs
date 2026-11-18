//! Profile → Notifications (per-kind preferences, §23) and Language (M3.8).
use crate::t;
use dioxus::prelude::*;
use dioxus_i18n::prelude::i18n;

use crate::api::i18n::set_my_locale;
use crate::api::notifications::{notification_prefs, set_notification_pref};
use crate::components::layout::{UserResource, use_current_user};
use crate::domain::notifications::NotificationPref;
use crate::ui::card::{Card, CardContent, CardDescription, CardHeader, CardTitle};
use crate::ui::toast::ToastType;
use crate::util::{toast, toast_error};

const CHECK: &str = "size-4 rounded border-input accent-primary";

#[component]
pub fn NotificationsTab() -> Element {
    let me = use_current_user();
    let email_on = me.as_ref().is_some_and(|u| u.modules.email);
    let enabled = me.as_ref().is_some_and(|u| u.modules.notifications);
    let mut prefs = use_resource(notification_prefs);
    let save = move |p: NotificationPref| async move {
        match set_notification_pref(p).await {
            Ok(()) => {
                toast(ToastType::Success, t!("prefs-saved"));
                prefs.restart();
            }
            Err(e) => toast_error(&e),
        }
    };
    let rows = prefs
        .read()
        .clone()
        .and_then(Result::ok)
        .unwrap_or_default();
    rsx! {
        Card { class: "max-w-3xl",
            CardHeader {
                CardTitle { {t!("profile-tab-notifications")} }
                CardDescription {
                    if enabled { {t!("prefs-desc")} } else { {t!("prefs-module-off")} }
                }
            }
            CardContent {
                table { class: "w-full text-sm",
                    thead {
                        tr { class: "border-b text-start text-muted-foreground",
                            th { class: "py-2 text-start font-medium", {t!("prefs-kind")} }
                            th { class: "w-24 py-2 font-medium", {t!("prefs-in-app")} }
                            th { class: "w-24 py-2 font-medium", {t!("prefs-email")} }
                        }
                    }
                    tbody {
                        for p in rows {
                            tr { key: "{p.kind}", class: "border-b last:border-0",
                                td { class: "py-2.5", {crate::i18n::tr(&format!("prefs-kind-{}", p.kind.replace('_', "-")), &Default::default())} }
                                td { class: "text-center",
                                    input { r#type: "checkbox", class: CHECK, checked: p.in_app, disabled: !enabled,
                                        "aria-label": "in-app {p.kind}",
                                        onchange: { let p = p.clone(); move |e: FormEvent| save(NotificationPref { in_app: e.checked(), ..p.clone() }) } }
                                }
                                td { class: "text-center",
                                    input { r#type: "checkbox", class: CHECK, checked: p.email, disabled: !email_on,
                                        "aria-label": "email {p.kind}",
                                        onchange: { let p = p.clone(); move |e: FormEvent| save(NotificationPref { email: e.checked(), ..p.clone() }) } }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn LanguageTab() -> Element {
    let mut user = use_context::<UserResource>();
    let me = use_current_user();
    let locales = me
        .as_ref()
        .map(|u| u.modules.locales.clone())
        .unwrap_or_default();
    let own = me.as_ref().and_then(|u| u.locale.clone());
    let mut i18n = i18n();
    let choose = move |l: Option<String>| async move {
        match set_my_locale(l.clone()).await {
            Ok(()) => {
                if let Some(code) = &l
                    && let Ok(id) = code.parse()
                {
                    i18n.set_language(id);
                }
                toast(ToastType::Success, t!("prefs-saved"));
                user.restart();
            }
            Err(e) => toast_error(&e),
        }
    };
    rsx! {
        Card { class: "max-w-xl",
            CardHeader {
                CardTitle { {t!("profile-tab-language")} }
                CardDescription { {t!("language-desc")} }
            }
            CardContent { class: "grid gap-2",
                label { class: "flex items-center gap-3 rounded-md border px-3 py-2.5 text-sm",
                    input { r#type: "radio", name: "locale", class: CHECK, checked: own.is_none(), onchange: move |_| choose(None) }
                    {t!("language-default")}
                }
                for code in locales {
                    label { key: "{code}", class: "flex items-center gap-3 rounded-md border px-3 py-2.5 text-sm",
                        input { r#type: "radio", name: "locale", id: "locale-{code}", class: CHECK, checked: own.as_deref() == Some(code.as_str()),
                            onchange: { let c = code.clone(); move |_| choose(Some(c.clone())) } }
                        {crate::i18n::tr(&format!("language-name-{code}"), &Default::default())}
                    }
                }
            }
        }
    }
}
