//! Audit log DTOs (§9.5).
use serde::{Deserialize, Serialize};
use time::{Date, OffsetDateTime};

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct AuditQuery {
    /// Free text: action, actor email, entity id.
    pub q: String,
    /// Modules (`auth`, `user`, …) — the part of the action before the first dot.
    pub modules: Vec<String>,
    /// `Some(true)` succeeded only, `Some(false)` failed only.
    pub success: Option<bool>,
    /// Inclusive day range (UTC).
    pub from: Option<Date>,
    pub to: Option<Date>,
    /// Keyset cursor: rows strictly older than `(at, id)`.
    pub before: Option<AuditCursor>,
    pub limit: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AuditCursor {
    #[serde(with = "time::serde::rfc3339")]
    pub at: OffsetDateTime,
    pub id: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuditRow {
    pub id: i64,
    #[serde(with = "time::serde::rfc3339")]
    pub at: OffsetDateTime,
    pub actor_email: Option<String>,
    pub channel: String,
    pub action: String,
    pub entity_type: Option<String>,
    pub entity_id: Option<String>,
    /// JSON text (already masked when written).
    pub before: Option<String>,
    pub after: Option<String>,
    pub ip: Option<String>,
    pub user_agent: Option<String>,
    pub success: bool,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct AuditPage {
    pub rows: Vec<AuditRow>,
    /// Present when older rows exist.
    pub next: Option<AuditCursor>,
    /// Every module seen in the log, for the filter menu.
    pub modules: Vec<String>,
}

/// One line of the before/after diff: key, old value, new value (JSON text).
pub fn diff(before: Option<&str>, after: Option<&str>) -> Vec<(String, String, String)> {
    let parse = |s: Option<&str>| {
        s.and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok())
            .and_then(|v| v.as_object().cloned())
            .unwrap_or_default()
    };
    let (b, a) = (parse(before), parse(after));
    let mut keys: Vec<&String> = b.keys().chain(a.keys()).collect();
    keys.sort();
    keys.dedup();
    let show = |v: Option<&serde_json::Value>| match v {
        None => String::new(),
        Some(serde_json::Value::String(s)) => s.clone(),
        Some(v) => v.to_string(),
    };
    keys.into_iter()
        .filter(|k| b.get(*k) != a.get(*k) || before.is_none() || after.is_none())
        .map(|k| (k.clone(), show(b.get(k)), show(a.get(k))))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::diff;

    #[test]
    fn diff_shows_changed_keys_only() {
        let d = diff(
            Some(r#"{"name":"A","role":"X"}"#),
            Some(r#"{"name":"B","role":"X"}"#),
        );
        assert_eq!(d, vec![("name".into(), "A".into(), "B".into())]);
        let created = diff(None, Some(r#"{"name":"B"}"#));
        assert_eq!(created, vec![("name".into(), String::new(), "B".into())]);
    }
}
