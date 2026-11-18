//! Thin server functions, one file per module (§4, §31.2).
//! Rule (§3): session → permission check → service → audit → return. No logic here.
pub mod account;
pub mod audit;
pub mod auth;
pub mod backups;
pub mod dashboard;
pub mod i18n;
pub mod notifications;
pub mod orgs;
pub mod roles;
pub mod security;
pub mod settings;
pub mod system;
pub mod tokens;
pub mod users;

/// M2.12: every server function in `api/` must check access — `require(` (permission),
/// `require_user(` (own account, session only) or `require_flag(` — or be on the explicit
/// public allow-list below. A forgotten check fails this test (and CI).
#[cfg(all(test, feature = "server"))]
mod convention {
    const PUBLIC: &[&str] = &[
        "login",
        "verify_2fa",
        "forgot_password",
        "set_password_with_token",
        "verify_email",
        "current_user",
        "server_info",
        "set_locale",
    ];

    #[test]
    fn every_server_fn_checks_access() {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/src/api");
        let mut missing = vec![];
        let mut seen = 0;
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            let src = std::fs::read_to_string(&path).unwrap();
            // Split at each route attribute; the chunk after it is that function.
            for chunk in src.split("\n#[").skip(1) {
                let attr = chunk.split(']').next().unwrap_or("");
                if !["get(", "post(", "put(", "delete(", "patch(", "server"]
                    .iter()
                    .any(|a| attr.starts_with(a))
                {
                    continue;
                }
                seen += 1;
                let name = chunk
                    .split("fn ")
                    .nth(1)
                    .and_then(|s| s.split('(').next())
                    .unwrap_or("?")
                    .trim();
                let body = chunk.split("\n#[").next().unwrap_or(chunk);
                let checked = ["require(", "require_user(", "require_flag("]
                    .iter()
                    .any(|c| body.contains(c));
                if !checked && !PUBLIC.contains(&name) {
                    missing.push(format!("{}: {name}", path.display()));
                }
            }
        }
        assert!(seen >= 4, "parser found only {seen} server fns");
        assert!(
            missing.is_empty(),
            "server fns without an access check: {missing:#?}"
        );
    }
}
