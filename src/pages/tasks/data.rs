//! `features/tasks/data/{data.tsx,schema.ts,tasks.ts}`: the task shape, the label / status /
//! priority option lists and the 100 faker-generated demo tasks (same seed as the reference).
use std::sync::LazyLock;

use dioxus_i18n::t;
use serde::{Deserialize, Serialize};

use crate::components::data_table::FilterOption;
use crate::components::ui_extra::icon::IconType;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub status: String,
    pub label: String,
    pub priority: String,
}

pub static TASKS: LazyLock<Vec<Task>> =
    LazyLock::new(|| serde_json::from_str(include_str!("tasks.json")).expect("tasks.json"));

/// (value, i18n key)
pub const LABELS: [(&str, &str); 3] = [
    ("bug", "task-label-bug"),
    ("feature", "task-label-feature"),
    ("documentation", "task-label-documentation"),
];

pub const STATUSES: [(&str, &str, IconType); 5] = [
    ("backlog", "task-status-backlog", IconType::CircleHelp),
    ("todo", "task-status-todo", IconType::Circle),
    ("in progress", "task-status-in-progress", IconType::Timer),
    ("done", "task-status-done", IconType::CircleCheck),
    ("canceled", "task-status-canceled", IconType::CircleOff),
];

pub const PRIORITIES: [(&str, &str, IconType); 4] = [
    ("low", "task-priority-low", IconType::ArrowDown),
    ("medium", "task-priority-medium", IconType::ArrowRight),
    ("high", "task-priority-high", IconType::ArrowUp),
    ("critical", "task-priority-critical", IconType::CircleAlert),
];

pub fn options(list: &[(&'static str, &'static str, IconType)]) -> Vec<FilterOption> {
    list.iter()
        .map(|(value, key, icon)| FilterOption {
            value: value.to_string(),
            label: t!(*key),
            icon: Some(*icon),
        })
        .collect()
}

pub fn find<'a>(
    list: &'a [(&'static str, &'static str, IconType)],
    value: &str,
) -> Option<&'a (&'static str, &'static str, IconType)> {
    list.iter().find(|(v, _, _)| *v == value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_tasks_load() {
        assert_eq!(TASKS.len(), 100);
        assert!(TASKS.iter().all(|t| find(&STATUSES, &t.status).is_some()));
        assert!(
            TASKS
                .iter()
                .all(|t| find(&PRIORITIES, &t.priority).is_some())
        );
        assert!(
            TASKS
                .iter()
                .all(|t| LABELS.iter().any(|(v, _)| *v == t.label))
        );
    }
}
