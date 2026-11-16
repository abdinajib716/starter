//! App-wide UI state (port of shadcn-admin `src/context/*`): theme, layout, direction,
//! search. Initial values come from cookies on the server (`UiPrefs`), so SSR and hydration
//! agree; changes write the cookie and update a signal.
pub mod prefs;
pub mod search;

pub use prefs::{Collapsible, Dir, Theme, Ui, UiPrefs, Variant};
pub use search::SearchOpen;
