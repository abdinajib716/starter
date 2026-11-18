//! Module switches (§17): typed reads of the master-switch settings. Server functions of a
//! module start with `Modules::get().require_email()?`; the UI receives the same booleans as
//! `ModuleFlags` in `current_user()` and hides menus accordingly.
use serde_json::Value;

use super::error::AppError;
use super::settings;
use crate::domain::{ModuleFlags, TwoFaMode};

/// A snapshot of the switches, read from the settings cache.
pub struct Modules {
    get: Box<dyn Fn(&str) -> Value + Send + Sync>,
}

impl Modules {
    /// From the process settings cache (registry defaults before it starts).
    pub fn get() -> Self {
        Self {
            get: Box::new(|k| match settings::cache() {
                Some(c) => c.get(k),
                None => settings::def(k)
                    .map(|d| (d.default)())
                    .unwrap_or(Value::Null),
            }),
        }
    }

    #[cfg(test)]
    pub fn from_map(map: std::collections::HashMap<String, Value>) -> Self {
        Self {
            get: Box::new(move |k| {
                map.get(k)
                    .cloned()
                    .or_else(|| settings::def(k).map(|d| (d.default)()))
                    .unwrap_or(Value::Null)
            }),
        }
    }

    fn flag(&self, key: &str) -> bool {
        (self.get)(key).as_bool().unwrap_or(false)
    }

    pub fn email_enabled(&self) -> bool {
        self.flag("email.enabled")
    }

    /// Password reset, invites and email verification need email (§17 dependency rules).
    pub fn password_reset(&self) -> bool {
        self.email_enabled() && self.flag("auth.password_reset_enabled")
    }
    pub fn invites(&self) -> bool {
        self.email_enabled() && self.flag("auth.invites_enabled")
    }
    pub fn email_verification(&self) -> bool {
        self.email_enabled() && self.flag("auth.email_verification")
    }

    pub fn twofa_grace_days(&self) -> i64 {
        (self.get)("security.2fa_grace_days").as_i64().unwrap_or(7)
    }

    pub fn twofa_issuer(&self) -> String {
        match (self.get)("security.2fa_issuer").as_str() {
            Some(s) if !s.trim().is_empty() => s.trim().to_string(),
            _ => (self.get)("app.name")
                .as_str()
                .unwrap_or("Starter")
                .to_string(),
        }
    }

    pub fn twofa_mode(&self) -> TwoFaMode {
        TwoFaMode::parse((self.get)("security.2fa_mode").as_str().unwrap_or(""))
    }
    pub fn public_api(&self) -> bool {
        self.flag("api.public_enabled")
    }
    pub fn notifications(&self) -> bool {
        self.flag("notifications.enabled")
    }
    /// Enabled locales and the default (M3.9). With `i18n.enabled = false` only the default.
    pub fn locales(&self) -> (Vec<String>, String) {
        let default = (self.get)("i18n.default_locale")
            .as_str()
            .filter(|l| crate::i18n::SUPPORTED.contains(l))
            .unwrap_or(crate::i18n::DEFAULT_LOCALE)
            .to_string();
        if !self.i18n() {
            return (vec![default.clone()], default);
        }
        let mut list: Vec<String> = (self.get)("i18n.enabled_locales")
            .as_str()
            .unwrap_or("en")
            .split(',')
            .map(|l| l.trim().to_string())
            .filter(|l| crate::i18n::SUPPORTED.contains(&l.as_str()))
            .collect();
        if !list.contains(&default) {
            list.push(default.clone());
        }
        (list, default)
    }

    pub fn i18n(&self) -> bool {
        self.flag("i18n.enabled")
    }
    /// Organization default zone (System Setting → General → Time zone).
    pub fn app_timezone() -> String {
        Self::get()
            .get_str("app.timezone")
            .unwrap_or_else(|| "UTC".into())
    }

    fn get_str(&self, key: &str) -> Option<String> {
        (self.get)(key).as_str().map(str::to_owned)
    }

    pub fn app_name(&self) -> String {
        (self.get)("app.name")
            .as_str()
            .unwrap_or("Starter")
            .to_string()
    }

    #[allow(dead_code)] // callers arrive with their modules (M9, M13, M15, M8)
    pub fn require_email(&self) -> Result<(), AppError> {
        self.email_enabled()
            .then_some(())
            .ok_or(AppError::Disabled("email.disabled"))
    }
    #[allow(dead_code)] // callers arrive with their modules (M9, M13, M15, M8)
    pub fn require_public_api(&self) -> Result<(), AppError> {
        self.public_api()
            .then_some(())
            .ok_or(AppError::Disabled("api.disabled"))
    }
    #[allow(dead_code)] // callers arrive with their modules (M9, M13, M15, M8)
    pub fn require_notifications(&self) -> Result<(), AppError> {
        self.notifications()
            .then_some(())
            .ok_or(AppError::Disabled("notifications.disabled"))
    }

    /// What the UI needs to show or hide.
    pub fn flags(&self) -> ModuleFlags {
        ModuleFlags {
            app_name: self.app_name(),
            email: self.email_enabled(),
            password_reset: self.password_reset(),
            invites: self.invites(),
            twofa_mode: self.twofa_mode(),
            public_api: self.public_api(),
            notifications: self.notifications(),
            i18n: self.i18n(),
            locales: self.locales().0,
            multi_tenant: crate::server::state::get()
                .is_ok_and(|s| s.config.tenancy_mode == crate::domain::TenancyMode::Multi),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn m(pairs: &[(&str, Value)]) -> Modules {
        Modules::from_map(
            pairs
                .iter()
                .map(|(k, v)| (k.to_string(), v.clone()))
                .collect(),
        )
    }

    #[test]
    fn defaults_match_the_blueprint() {
        let f = m(&[]).flags();
        assert!(!f.email && !f.public_api && !f.i18n);
        assert!(f.notifications);
        assert_eq!(f.twofa_mode, TwoFaMode::Optional);
    }

    #[test]
    fn email_features_force_off_without_email() {
        let off = m(&[("auth.password_reset_enabled", json!(true))]);
        assert!(!off.password_reset() && !off.invites());
        assert!(matches!(
            off.require_email(),
            Err(AppError::Disabled("email.disabled"))
        ));
        let on = m(&[("email.enabled", json!(true))]);
        assert!(on.password_reset() && on.invites() && !on.email_verification());
        assert!(on.require_email().is_ok());
    }
}
