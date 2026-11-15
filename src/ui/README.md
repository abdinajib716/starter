# Rust/UI components (vendored)

Copied verbatim from [rust-ui/ui](https://github.com/rust-ui/ui) at commit **`4f5a516`**
(`app_crates/registry/src/ui/` → `src/ui/`, `app_crates/registry/src/hooks/` → `src/hooks/`).
MIT licensed — see [LICENSE-rust-ui](LICENSE-rust-ui) and the root `NOTICE`.

Rules:
- Keep files byte-identical to upstream so updates are a plain re-copy + diff. Customise in
  `src/components/` (wrappers), not here.
- `src/ui` and `src/hooks` stay at the crate root: upstream files import `crate::ui::…` / `crate::hooks::…`.
- Adding a component: copy `<name>.rs` (plus any `crate::hooks::…` it uses), add `pub mod <name>;`
  to `mod.rs`, and list it in `TRACKER.md`.
