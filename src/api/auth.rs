//! Auth & profile server functions (§9.1). Thin: context → service → response.
use dioxus::prelude::*;

use crate::domain::{AfterLogin, UserInfo};

/// Public. Sets the session cookie on success.
#[post("/api/auth/login")]
pub async fn login(email: String, password: String) -> Result<AfterLogin, ServerFnError> {
    use crate::server::{auth::cookie, ctx, services::auth, state};
    let ctx = ctx::Ctx::current().await?;
    let st = state::get()?;
    let ok = auth::login(&st.pool, &ctx, &email, password).await?;
    if ok.next == AfterLogin::TwoFactor {
        let name = cookie::pending_name(&st.config);
        ctx::set_cookie(&cookie::set_named(
            &st.config,
            &name,
            &ok.token,
            ok.max_age_secs,
        ))?;
    } else {
        ctx::set_cookie(&cookie::set(&st.config, &ok.token, ok.max_age_secs))?;
    }
    Ok(ok.next)
}

/// Public (second sign-in step): the pending cookie from `login` + a TOTP or recovery code.
#[post("/api/auth/2fa")]
pub async fn verify_2fa(code: String) -> Result<AfterLogin, ServerFnError> {
    use crate::server::{auth::cookie, ctx, services::security, state};
    use dioxus::fullstack::{FullstackContext, http::HeaderMap};
    let ctx = ctx::Ctx::current().await?;
    let st = state::get()?;
    let headers = FullstackContext::extract::<HeaderMap, _>()
        .await
        .map_err(|_| ServerFnError::new("headers"))?;
    let name = cookie::pending_name(&st.config);
    let pending = cookie::read(&headers, &name)
        .ok_or_else(|| crate::domain::errors::coded(401, "twofa.expired"))?
        .to_string();
    let ok = security::verify_login(&st.pool, &ctx, &pending, &code).await?;
    ctx::set_cookie(&cookie::set_named(&st.config, &name, "", 0))?;
    ctx::set_cookie(&cookie::set(&st.config, &ok.token, ok.max_age_secs))?;
    Ok(ok.next)
}

/// Session required. Deletes the session and clears the cookie.
#[post("/api/auth/logout")]
pub async fn logout() -> Result<(), ServerFnError> {
    use crate::server::{auth::cookie, ctx, services::auth, state};
    let ctx = ctx::Ctx::current().await?;
    let st = state::get()?;
    let user = ctx.require_user()?;
    auth::logout(&st.pool, &ctx, user).await?;
    ctx::set_cookie(&cookie::clear(&st.config))?;
    Ok(())
}

/// Public (returns `None` when signed out). Feeds the app shell and permission gates (§8).
#[get("/api/auth/me")]
pub async fn current_user() -> Result<Option<UserInfo>, ServerFnError> {
    let ctx = crate::server::ctx::Ctx::current().await?;
    Ok(ctx.user.map(|u| {
        let flags = crate::server::flags::for_user(&u);
        let twofa = u.twofa_info();
        let mut permissions: Vec<String> = u.permissions.into_iter().collect();
        permissions.sort();
        UserInfo {
            id: u.id,
            email: u.email,
            full_name: u.full_name,
            role_name: u.role_name,
            is_super_admin: u.is_super_admin,
            permissions,
            must_change_pw: u.must_change_pw,
            modules: crate::server::modules::Modules::get().flags(),
            flags,
            timezone: u
                .timezone
                .clone()
                .unwrap_or_else(crate::server::modules::Modules::app_timezone),
            twofa,
            locale: u.locale.clone(),
            own_timezone: u.timezone,
        }
    }))
}

/// Session required (own account, no permission key — §8).
#[post("/api/profile/password")]
pub async fn change_password(current: String, new: String) -> Result<(), ServerFnError> {
    use crate::server::{ctx, services::auth, state};
    let ctx = ctx::Ctx::current().await?;
    let st = state::get()?;
    let user = ctx.require_user()?;
    auth::change_password(&st.pool, &ctx, user, current, new).await?;
    Ok(())
}

/// Session required (own account): change the display name (§9.1).
#[post("/api/profile/update")]
pub async fn update_profile(full_name: String, timezone: String) -> Result<(), ServerFnError> {
    use crate::server::{ctx, services::auth, state};
    let ctx = ctx::Ctx::current().await?;
    let st = state::get()?;
    let user = ctx.require_user()?;
    auth::update_profile(&st.pool, &ctx, user, full_name, timezone).await?;
    Ok(())
}
