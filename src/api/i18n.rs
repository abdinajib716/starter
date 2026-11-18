use dioxus::prelude::*;

/// Persist the chosen locale in a cookie (session-less users). Public endpoint.
/// From M9 the Profile → Language tab also saves `users.locale`.
#[post("/api/i18n/locale")]
pub async fn set_locale(locale: String) -> Result<(), ServerFnError> {
    use dioxus::fullstack::{FullstackContext, HeaderValue, http::header::SET_COOKIE};
    if !crate::i18n::SUPPORTED.contains(&locale.as_str()) {
        return Err(ServerFnError::new("unsupported locale"));
    }
    let ctx = FullstackContext::current().ok_or_else(|| ServerFnError::new("no request"))?;
    let cookie = format!(
        "{}={locale}; Path=/; Max-Age=31536000; SameSite=Lax",
        crate::i18n::LOCALE_COOKIE
    );
    ctx.add_response_header(
        SET_COOKIE,
        HeaderValue::from_str(&cookie).map_err(|e| ServerFnError::new(e.to_string()))?,
    );
    Ok(())
}

/// Session required (own account, M3.8): save `users.locale` (`None` = follow the default)
/// and the cookie, so the next page load renders in that language.
#[post("/api/profile/locale")]
pub async fn set_my_locale(locale: Option<String>) -> Result<(), ServerFnError> {
    let ctx = crate::server::ctx::Ctx::current().await?;
    let user = ctx.require_user()?;
    let st = crate::server::state::get()?;
    crate::server::services::auth::set_locale(&st.pool, &ctx, user, locale.clone()).await?;
    if let Some(l) = locale {
        set_locale(l).await?;
    }
    Ok(())
}
