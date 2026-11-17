//! Roles and permission catalog DTOs (§9.3, §9.4).
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RoleSummary {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub is_system: bool,
    pub require_2fa: bool,
    /// Non-deleted users holding the role.
    pub user_count: i64,
    /// Granted keys (Super Admin: the whole registry).
    pub permission_count: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RoleDetail {
    pub role: RoleSummary,
    pub keys: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct RoleInput {
    pub name: String,
    pub description: String,
    pub require_2fa: bool,
    pub keys: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PermissionInfo {
    pub key: String,
    pub module: String,
    pub action: String,
    pub description: String,
    /// Roles holding it (Super Admin always included).
    pub roles: Vec<String>,
}

/// The catalog plus what the signed-in user may grant (role editor, no-escalation rule).
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct PermissionCatalog {
    pub permissions: Vec<PermissionInfo>,
}

pub const ROLE_NAME_MAX: usize = 60;
pub const ROLE_DESC_MAX: usize = 300;

/// Name / description rules shared by the form and the server. Returns the error code.
pub fn validate_role(i: &RoleInput) -> Option<&'static str> {
    let name = i.name.trim();
    if name.is_empty() {
        Some("roles.name_required")
    } else if name.chars().count() > ROLE_NAME_MAX {
        Some("roles.name_too_long")
    } else if i.description.chars().count() > ROLE_DESC_MAX {
        Some("roles.description_too_long")
    } else {
        None
    }
}

/// Modules in catalog order, each with its permissions (`users` → [view, create, …]).
pub fn group_by_module(perms: &[PermissionInfo]) -> Vec<(String, Vec<PermissionInfo>)> {
    let mut out: Vec<(String, Vec<PermissionInfo>)> = Vec::new();
    for p in perms {
        match out.iter_mut().find(|(m, _)| *m == p.module) {
            Some((_, v)) => v.push(p.clone()),
            None => out.push((p.module.clone(), vec![p.clone()])),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn role_rules() {
        let ok = RoleInput {
            name: "Editor".into(),
            ..Default::default()
        };
        assert_eq!(validate_role(&ok), None);
        assert_eq!(
            validate_role(&RoleInput::default()),
            Some("roles.name_required")
        );
        let long = RoleInput {
            name: "x".repeat(61),
            ..Default::default()
        };
        assert_eq!(validate_role(&long), Some("roles.name_too_long"));
    }

    #[test]
    fn grouping_keeps_order() {
        let p = |k: &str| PermissionInfo {
            key: k.into(),
            module: k.split('.').next().unwrap().into(),
            action: k.split('.').nth(1).unwrap().into(),
            description: String::new(),
            roles: vec![],
        };
        let g = group_by_module(&[p("users.view"), p("roles.view"), p("users.create")]);
        assert_eq!(
            g.iter()
                .map(|x| (x.0.as_str(), x.1.len()))
                .collect::<Vec<_>>(),
            [("users", 2), ("roles", 1)]
        );
    }
}
