//! Thin server functions, one file per module (§4, §31.2).
//! Rule (§3): session → permission check → service → audit → return. No logic here.
pub mod i18n;
pub mod system;
