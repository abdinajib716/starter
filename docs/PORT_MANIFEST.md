# Port manifest — shadcn-admin → Dioxus/Rust/UI

SPEC-02 v3 §3. One row per file of `reference/shadcn-admin` (commit `e16c87f`). Checked by `just manifest-check`;
`just manifest-check final` also fails while any row is `todo` or `in-progress`.

**Status:** `todo` · `in-progress` · `done` · `n/a` (reason required; only for files without runtime meaning, or Clerk per §5).

| Reference path | Kind | Our target | Status | Notes |
|---|---|---|---|---|
| `.env.example` | config | — | n/a | Clerk key only (§5: Clerk removed) |
| `.github/CODE_OF_CONDUCT.md` | config | — | n/a | repo tooling (CI / editor settings) |
| `.github/CONTRIBUTING.md` | config | — | n/a | repo tooling (CI / editor settings) |
| `.github/FUNDING.yml` | config | — | n/a | repo tooling (CI / editor settings) |
| `.github/ISSUE_TEMPLATE/config.yml` | config | — | n/a | repo tooling (CI / editor settings) |
| `.github/ISSUE_TEMPLATE/✨-feature-request.md` | config | — | n/a | repo tooling (CI / editor settings) |
| `.github/ISSUE_TEMPLATE/🐞-bug-report.md` | config | — | n/a | repo tooling (CI / editor settings) |
| `.github/PULL_REQUEST_TEMPLATE.md` | config | — | n/a | repo tooling (CI / editor settings) |
| `.github/workflows/ci.yml` | config | — | n/a | repo tooling (CI / editor settings) |
| `.github/workflows/stale.yml` | config | — | n/a | repo tooling (CI / editor settings) |
| `.gitignore` | config | — | n/a | git config of the reference |
| `.prettierignore` | config | — | n/a | Prettier config |
| `.prettierrc` | config | — | n/a | Prettier config |
| `.vscode/extensions.json` | config | — | n/a | repo tooling (CI / editor settings) |
| `.vscode/settings.json` | config | — | n/a | repo tooling (CI / editor settings) |
| `CHANGELOG.md` | config | — | n/a | reference changelog |
| `LICENSE` | config | — | n/a | kept; MIT attribution in NOTICE |
| `README.md` | config | — | n/a | reference docs |
| `components.json` | config | — | n/a | shadcn CLI config (we vendor Rust/UI by hand) |
| `cz.yaml` | config | — | n/a | commitizen config |
| `eslint.config.js` | config | — | n/a | ESLint config |
| `index.html` | config | index.html | todo | meta, title, favicon links; fonts are self-hosted |
| `knip.config.ts` | config | — | n/a | knip config |
| `netlify.toml` | config | — | n/a | Netlify deploy config |
| `package.json` | config | — | n/a | npm manifest; its UI dependencies are mapped by the rows that use them |
| `pnpm-lock.yaml` | config | — | n/a | lockfile |
| `public/images/favicon.png` | asset | assets/brand/favicon.png | todo | replaced by our branding (§1.3) |
| `public/images/favicon.svg` | asset | assets/brand/favicon.svg | todo | replaced by our branding (§1.3) |
| `public/images/favicon_light.png` | asset | assets/brand/favicon_light.png | todo | replaced by our branding (§1.3) |
| `public/images/favicon_light.svg` | asset | assets/brand/favicon_light.svg | todo | replaced by our branding (§1.3) |
| `public/images/shadcn-admin.png` | asset | — | n/a | README screenshot |
| `src/assets/brand-icons/icon-discord.tsx` | asset | src/components/icons/brand.rs | todo | inline SVG components |
| `src/assets/brand-icons/icon-docker.tsx` | asset | src/components/icons/brand.rs | todo | inline SVG components |
| `src/assets/brand-icons/icon-facebook.tsx` | asset | src/pages/auth/mod.rs (SocialButtons) | done | inline path in SocialButtons |
| `src/assets/brand-icons/icon-figma.tsx` | asset | src/components/icons/brand.rs | todo | inline SVG components |
| `src/assets/brand-icons/icon-github.tsx` | asset | src/pages/auth/mod.rs (SocialButtons) | done | inline path in SocialButtons |
| `src/assets/brand-icons/icon-gitlab.tsx` | asset | src/components/icons/brand.rs | todo | inline SVG components |
| `src/assets/brand-icons/icon-gmail.tsx` | asset | src/components/icons/brand.rs | todo | inline SVG components |
| `src/assets/brand-icons/icon-medium.tsx` | asset | src/components/icons/brand.rs | todo | inline SVG components |
| `src/assets/brand-icons/icon-notion.tsx` | asset | src/components/icons/brand.rs | todo | inline SVG components |
| `src/assets/brand-icons/icon-skype.tsx` | asset | src/components/icons/brand.rs | todo | inline SVG components |
| `src/assets/brand-icons/icon-slack.tsx` | asset | src/components/icons/brand.rs | todo | inline SVG components |
| `src/assets/brand-icons/icon-stripe.tsx` | asset | src/components/icons/brand.rs | todo | inline SVG components |
| `src/assets/brand-icons/icon-telegram.tsx` | asset | src/components/icons/brand.rs | todo | inline SVG components |
| `src/assets/brand-icons/icon-trello.tsx` | asset | src/components/icons/brand.rs | todo | inline SVG components |
| `src/assets/brand-icons/icon-whatsapp.tsx` | asset | src/components/icons/brand.rs | todo | inline SVG components |
| `src/assets/brand-icons/icon-zoom.tsx` | asset | src/components/icons/brand.rs | todo | inline SVG components |
| `src/assets/brand-icons/index.ts` | asset | src/components/icons/brand.rs | todo | inline SVG components |
| `src/assets/clerk-full-logo.tsx` | asset | — | n/a | Clerk branding (§5: Clerk removed) |
| `src/assets/clerk-logo.tsx` | asset | — | n/a | Clerk branding (§5: Clerk removed) |
| `src/assets/custom/icon-dir.tsx` | asset | src/components/icons/custom.rs | todo | inline SVG components |
| `src/assets/custom/icon-layout-compact.tsx` | asset | src/components/icons/custom.rs | todo | inline SVG components |
| `src/assets/custom/icon-layout-default.tsx` | asset | src/components/icons/custom.rs | todo | inline SVG components |
| `src/assets/custom/icon-layout-full.tsx` | asset | src/components/icons/custom.rs | todo | inline SVG components |
| `src/assets/custom/icon-sidebar-floating.tsx` | asset | src/components/icons/custom.rs | todo | inline SVG components |
| `src/assets/custom/icon-sidebar-inset.tsx` | asset | src/components/icons/custom.rs | todo | inline SVG components |
| `src/assets/custom/icon-sidebar-sidebar.tsx` | asset | src/components/icons/custom.rs | todo | inline SVG components |
| `src/assets/custom/icon-theme-dark.tsx` | asset | src/components/icons/custom.rs | todo | inline SVG components |
| `src/assets/custom/icon-theme-light.tsx` | asset | src/components/icons/custom.rs | todo | inline SVG components |
| `src/assets/custom/icon-theme-system.tsx` | asset | src/components/icons/custom.rs | todo | inline SVG components |
| `src/assets/logo.tsx` | asset | src/pages/auth/mod.rs (Logo) | done | our branding: command glyph + app name |
| `src/components/coming-soon.tsx` | component | src/pages/errors/mod.rs (ComingSoon) | done |  |
| `src/components/command-menu.tsx` | component | src/components/layout/search.rs | done | Ctrl/Cmd+K, filter, ↑↓/Enter, pages + theme |
| `src/components/config-drawer.test.tsx` | test | — | n/a | DOM test; manual check in P11, end-to-end later (§4.6) |
| `src/components/config-drawer.tsx` | component | src/components/config_drawer.rs | in-progress | options + behaviour done; illustrated option cards (assets/custom/*) in P5 |
| `src/components/confirm-dialog.test.tsx` | test | — | n/a | DOM test; manual check in P11, end-to-end later (§4.6) |
| `src/components/confirm-dialog.tsx` | component | src/components/ui_extra/dialog.rs | done | ConfirmDialog |
| `src/components/data-table/bulk-actions.tsx` | component | src/components/data_table/bulk_actions.rs | todo |  |
| `src/components/data-table/column-header.tsx` | component | src/components/data_table/column_header.rs | todo |  |
| `src/components/data-table/faceted-filter.tsx` | component | src/components/data_table/faceted_filter.rs | todo |  |
| `src/components/data-table/index.ts` | component | src/components/data_table/mod.rs | todo |  |
| `src/components/data-table/pagination.tsx` | component | src/components/data_table/pagination.rs | todo |  |
| `src/components/data-table/toolbar.tsx` | component | src/components/data_table/toolbar.rs | todo |  |
| `src/components/data-table/view-options.tsx` | component | src/components/data_table/view_options.rs | todo |  |
| `src/components/date-picker.tsx` | component | src/components/ui_extra/form.rs | done | `ui_extra::form::DatePicker` (month/year dropdowns, future disabled) |
| `src/components/layout/app-sidebar.tsx` | layout | src/components/layout/sidebar.rs | done | variant/collapsible from layout state; mobile copy via `mobile` slot |
| `src/components/layout/app-title.tsx` | layout | src/components/layout/sidebar_top.rs | todo |  |
| `src/components/layout/authenticated-layout.tsx` | layout | src/components/layout/app_shell.rs | done | guard + search state + SidebarProvider/Inset |
| `src/components/layout/data/sidebar-data.ts` | data | src/components/layout/nav_config.rs | done | reference tree minus Clerk; permission filter; demo items marked for flags (M7) |
| `src/components/layout/header.tsx` | layout | src/components/layout/header.rs | done | sticky + blur when `fixed` |
| `src/components/layout/main.tsx` | layout | src/components/layout/main_area.rs | done | container queries, fixed/fluid |
| `src/components/layout/nav-group.tsx` | layout | src/components/layout/nav_group.rs | done | collapsible sections; dropdown when icons-only |
| `src/components/layout/nav-user.tsx` | layout | src/components/layout/nav_user.rs | done | real user (current_user) |
| `src/components/layout/team-switcher.tsx` | layout | src/components/layout/team_switcher.rs | done | single tenant: app name + org; multi-tenant list in M16 |
| `src/components/layout/top-nav.tsx` | layout | src/components/layout/top_nav.rs | done | menu below lg |
| `src/components/layout/types.ts` | data | src/components/layout/nav_config.rs | done | typed NavGroup/NavItem/NavLink |
| `src/components/learn-more.tsx` | component | src/components/learn_more.rs | todo |  |
| `src/components/long-text.tsx` | component | src/components/long_text.rs | todo |  |
| `src/components/navigation-progress.tsx` | component | src/components/navigation_progress.rs | todo |  |
| `src/components/password-input.test.tsx` | test | src/components/password_input.rs #[cfg(test)] | todo | logic test → Rust #[test] |
| `src/components/password-input.tsx` | component | src/components/password_input.rs | done |  |
| `src/components/profile-dropdown.tsx` | component | src/components/profile_dropdown.rs | done | real user |
| `src/components/search.tsx` | component | src/components/layout/search.rs | done |  |
| `src/components/select-dropdown.tsx` | component | src/components/select_dropdown.rs | todo |  |
| `src/components/sign-out-dialog.test.tsx` | test | — | n/a | DOM test; manual check in P11, end-to-end later (§4.6) |
| `src/components/sign-out-dialog.tsx` | component | src/components/layout/sign_out_dialog.rs | done | calls logout, keeps `next` |
| `src/components/skip-to-main.tsx` | component | src/components/layout/skip_to_main.rs | done |  |
| `src/components/theme-switch.tsx` | component | src/components/layout/theme_switch.rs | done | light/dark/system, cookie `theme`, no-flash script in index.html |
| `src/components/ui/alert-dialog.tsx` | ui-primitive | src/components/ui_extra/dialog.rs | done | `ui_extra::dialog` |
| `src/components/ui/alert.tsx` | ui-primitive | src/ui/alert.rs | done | Rust/UI equivalent vendored; visual parity checked in P11 |
| `src/components/ui/avatar.tsx` | ui-primitive | src/ui/avatar.rs | done | Rust/UI equivalent vendored; visual parity checked in P11 |
| `src/components/ui/badge.tsx` | ui-primitive | src/ui/badge.rs | done | Rust/UI equivalent vendored; visual parity checked in P11 |
| `src/components/ui/button.tsx` | ui-primitive | src/ui/button.rs | done | Rust/UI equivalent vendored; visual parity checked in P11 |
| `src/components/ui/calendar.tsx` | ui-primitive | src/components/ui_extra/form.rs | done | ui_extra::form::DatePicker calendar |
| `src/components/ui/card.tsx` | ui-primitive | src/ui/card.rs | done | Rust/UI equivalent vendored; visual parity checked in P11 |
| `src/components/ui/checkbox.tsx` | ui-primitive | src/components/ui_extra/form.rs | done | ui_extra::form::CheckBox |
| `src/components/ui/collapsible.tsx` | ui-primitive | src/ui/collapsible.rs | done | Rust/UI equivalent vendored; visual parity checked in P11 |
| `src/components/ui/command.tsx` | ui-primitive | src/components/ui_extra/form.rs | done | ui_extra::form::Combobox + layout/search.rs palette |
| `src/components/ui/dialog.tsx` | ui-primitive | src/ui/dialog.rs | done | Rust/UI equivalent vendored; visual parity checked in P11 |
| `src/components/ui/dropdown-menu.tsx` | ui-primitive | src/components/ui_extra/menu.rs | done | `ui_extra::menu` (fixed-position, no inline script); Rust/UI version kept vendored |
| `src/components/ui/form.tsx` | ui-primitive | src/components/ui_extra/form.rs | done | ui_extra::form (FormItem/Label/Description/Message) |
| `src/components/ui/input-otp.tsx` | ui-primitive | src/pages/auth/otp.rs (OtpInput) | done | single transparent input over 6 slots |
| `src/components/ui/input.tsx` | ui-primitive | src/components/ui_extra/form.rs | done | ui_extra::form::TextInput (controlled) |
| `src/components/ui/label.tsx` | ui-primitive | src/ui/label.rs | done | Rust/UI equivalent vendored; visual parity checked in P11 |
| `src/components/ui/popover.tsx` | ui-primitive | src/components/ui_extra/form.rs | done | ui_extra::menu (popover content) |
| `src/components/ui/radio-group.tsx` | ui-primitive | src/components/ui_extra/form.rs | done | ui_extra::form::Radio |
| `src/components/ui/scroll-area.tsx` | ui-primitive | src/ui/scroll_area.rs | done | added from Rust/UI 4f5a516 in P1; visual parity checked in P11 |
| `src/components/ui/select.tsx` | ui-primitive | src/components/ui_extra/form.rs | done | ui_extra::form::SelectBox |
| `src/components/ui/separator.tsx` | ui-primitive | src/ui/separator.rs | done | Rust/UI equivalent vendored; visual parity checked in P11 |
| `src/components/ui/sheet.tsx` | ui-primitive | src/components/ui_extra/sheet.rs | done | `ui_extra::sheet::SideSheet` (signal-controlled) |
| `src/components/ui/sidebar.tsx` | ui-primitive | src/components/ui_extra/sidebar.rs | done | faithful port (Rust/UI sidenav lacks icon mode, mobile sheet, cookie, Ctrl+B) |
| `src/components/ui/skeleton.tsx` | ui-primitive | src/ui/skeleton.rs | done | Rust/UI equivalent vendored; visual parity checked in P11 |
| `src/components/ui/sonner.tsx` | ui-primitive | src/ui/toast.rs | done | Rust/UI equivalent vendored; visual parity checked in P11 |
| `src/components/ui/switch.tsx` | ui-primitive | src/components/ui_extra/form.rs | done | ui_extra::form::Switch (Rust/UI one has no change handler) |
| `src/components/ui/table.tsx` | ui-primitive | src/ui/table.rs | done | Rust/UI equivalent vendored; visual parity checked in P11 |
| `src/components/ui/tabs.tsx` | ui-primitive | src/ui/tabs.rs | done | Rust/UI equivalent vendored; visual parity checked in P11 |
| `src/components/ui/textarea.tsx` | ui-primitive | src/components/ui_extra/form.rs | done | ui_extra::form::TextArea (Rust/UI one cannot bind state) |
| `src/components/ui/tooltip.tsx` | ui-primitive | src/ui/tooltip.rs | done | sidebar tooltip in `ui_extra::sidebar` (fixed, collapsed only); general tooltip = Rust/UI |
| `src/config/fonts.ts` | config | src/state/font.rs + self-hosted fonts in assets/fonts/ | done | inter/manrope/system; cookie `font`; pre-paint class |
| `src/context/direction-provider.tsx` | context | src/state/direction.rs | in-progress | cookie `dir` + html dir; full RTL review in P11 |
| `src/context/font-provider.tsx` | context | src/state/prefs.rs | done | `Ui.font` / `set_font` |
| `src/context/layout-provider.tsx` | context | src/state/prefs.rs | done | Ui.variant / Ui.collapsible (cookies) |
| `src/context/search-provider.test.tsx` | test | src/state/search.rs #[cfg(test)] | todo | logic test → Rust #[test] |
| `src/context/search-provider.tsx` | context | src/state/search.rs + app_shell.rs | done | SearchOpen + Ctrl/Cmd+K |
| `src/context/theme-provider.tsx` | context | src/state/prefs.rs | done | Ui.theme + apply_theme |
| `src/features/apps/data/apps.tsx` | data | src/pages/apps/data/apps.rs | todo | demo seed data, flag `demo.apps` (§5) |
| `src/features/apps/index.tsx` | feature | src/pages/apps/mod.rs | todo | demo page behind flag `demo.apps`, default off (§5) |
| `src/features/auth/auth-layout.tsx` | feature | src/pages/auth/mod.rs (AuthLayout) | done |  |
| `src/features/auth/forgot-password/components/forgot-password-form.test.tsx` | test | — | n/a | DOM test; manual check in P11, end-to-end later (§4.6) |
| `src/features/auth/forgot-password/components/forgot-password-form.tsx` | feature | src/pages/auth/forgot_password.rs | done |  |
| `src/features/auth/forgot-password/index.tsx` | feature | src/pages/auth/forgot_password.rs | done | reset email in M13; honest toast |
| `src/features/auth/otp/components/otp-form.test.tsx` | test | — | n/a | DOM test; manual check in P11, end-to-end later (§4.6) |
| `src/features/auth/otp/components/otp-form.tsx` | feature | src/pages/auth/otp.rs (OtpInput) | done |  |
| `src/features/auth/otp/index.tsx` | feature | src/pages/auth/otp.rs | done | verification in M9 (2FA) |
| `src/features/auth/sign-in/assets/dashboard-dark.png` | asset | assets/auth/dashboard-dark.png | done |  |
| `src/features/auth/sign-in/assets/dashboard-light.png` | asset | assets/auth/dashboard-light.png | done |  |
| `src/features/auth/sign-in/components/user-auth-form.test.tsx` | test | — | n/a | DOM test; manual check in P11, end-to-end later (§4.6) |
| `src/features/auth/sign-in/components/user-auth-form.tsx` | feature | src/pages/auth/sign_in.rs (UserAuthForm) | done | reference checks + server login, toasts |
| `src/features/auth/sign-in/index.tsx` | feature | src/pages/auth/sign_in.rs | done | wired to our login (M1), redirect param |
| `src/features/auth/sign-in/sign-in-2.tsx` | feature | src/pages/auth/sign_in.rs (SignIn2) | done |  |
| `src/features/auth/sign-up/components/sign-up-form.test.tsx` | test | — | n/a | DOM test; manual check in P11, end-to-end later (§4.6) |
| `src/features/auth/sign-up/components/sign-up-form.tsx` | feature | src/pages/auth/sign_up.rs | done |  |
| `src/features/auth/sign-up/index.tsx` | feature | src/pages/auth/sign_up.rs | done | self sign-up off: submit explains (setting `auth.self_signup`, M7) |
| `src/features/chats/components/new-chat.tsx` | feature | src/pages/chats/components/new_chat.rs | todo | demo page behind flag `demo.chats`, default off (§5) |
| `src/features/chats/data/chat-types.ts` | data | src/pages/chats/data/chat_types.rs | todo | demo seed data, flag `demo.chats` (§5) |
| `src/features/chats/data/convo.json` | data | src/pages/chats/data/convo.rs | todo | demo seed data, flag `demo.chats` (§5) |
| `src/features/chats/index.tsx` | feature | src/pages/chats/mod.rs | todo | demo page behind flag `demo.chats`, default off (§5) |
| `src/features/dashboard/components/analytics-chart.tsx` | feature | src/components/ui_extra/chart.rs | done | SVG monotone area chart |
| `src/features/dashboard/components/analytics.tsx` | feature | src/pages/dashboard/analytics.rs | done |  |
| `src/features/dashboard/components/overview.tsx` | feature | src/pages/dashboard/overview.rs | done | SVG bar chart, Recharts ticks |
| `src/features/dashboard/components/recent-sales.tsx` | feature | src/pages/dashboard/recent_sales.rs | done |  |
| `src/features/dashboard/index.tsx` | feature | src/pages/dashboard/mod.rs | done | UI only; sample figures until M10 |
| `src/features/errors/forbidden.tsx` | feature | src/pages/errors/mod.rs | done | `ErrorView` + `ErrorKind` |
| `src/features/errors/general-error.tsx` | feature | src/pages/errors/mod.rs | done | `ErrorView` + `ErrorKind` |
| `src/features/errors/maintenance-error.tsx` | feature | src/pages/errors/mod.rs | done | `ErrorView` + `ErrorKind` |
| `src/features/errors/not-found-error.tsx` | feature | src/pages/errors/mod.rs | done | `ErrorView` + `ErrorKind` |
| `src/features/errors/unauthorized-error.tsx` | feature | src/pages/errors/mod.rs | done | `ErrorView` + `ErrorKind` |
| `src/features/settings/account/account-form.tsx` | feature | src/pages/settings/account.rs | done | demo form as in the reference (submit → toast); real data M9 |
| `src/features/settings/account/index.tsx` | feature | src/pages/settings/account.rs | done | demo form as in the reference (submit → toast); real data M9 |
| `src/features/settings/appearance/appearance-form.tsx` | feature | src/pages/settings/appearance.rs | done | demo form as in the reference (submit → toast); real data M9 |
| `src/features/settings/appearance/index.tsx` | feature | src/pages/settings/appearance.rs | done | demo form as in the reference (submit → toast); real data M9 |
| `src/features/settings/components/content-section.tsx` | feature | src/pages/settings/mod.rs (ContentSection) | done | demo form as in the reference (submit → toast); real data M9 |
| `src/features/settings/components/sidebar-nav.tsx` | feature | src/pages/settings/mod.rs (SidebarNav) | done | demo form as in the reference (submit → toast); real data M9 |
| `src/features/settings/display/display-form.tsx` | feature | src/pages/settings/display.rs | done | demo form as in the reference (submit → toast); real data M9 |
| `src/features/settings/display/index.tsx` | feature | src/pages/settings/display.rs | done | demo form as in the reference (submit → toast); real data M9 |
| `src/features/settings/index.tsx` | feature | src/pages/settings/mod.rs | done | demo form as in the reference (submit → toast); real data M9 |
| `src/features/settings/notifications/index.tsx` | feature | src/pages/settings/notifications.rs | done | demo form as in the reference (submit → toast); real data M9 |
| `src/features/settings/notifications/notifications-form.tsx` | feature | src/pages/settings/notifications.rs | done | demo form as in the reference (submit → toast); real data M9 |
| `src/features/settings/profile/index.tsx` | feature | src/pages/settings/profile.rs | done | demo form as in the reference (submit → toast); real data M9 |
| `src/features/settings/profile/profile-form.tsx` | feature | src/pages/settings/profile.rs | done | demo form as in the reference (submit → toast); real data M9 |
| `src/features/tasks/components/data-table-bulk-actions.tsx` | feature | src/pages/tasks/components/data_table_bulk_actions.rs | todo | demo page behind flag `demo.tasks`, default off (§5) |
| `src/features/tasks/components/data-table-row-actions.tsx` | feature | src/pages/tasks/components/data_table_row_actions.rs | todo | demo page behind flag `demo.tasks`, default off (§5) |
| `src/features/tasks/components/tasks-columns.tsx` | feature | src/pages/tasks/components/tasks_columns.rs | todo | demo page behind flag `demo.tasks`, default off (§5) |
| `src/features/tasks/components/tasks-dialogs.tsx` | feature | src/pages/tasks/components/tasks_dialogs.rs | todo | demo page behind flag `demo.tasks`, default off (§5) |
| `src/features/tasks/components/tasks-import-dialog.test.tsx` | test | — | n/a | DOM test; manual check in P11, end-to-end later (§4.6) |
| `src/features/tasks/components/tasks-import-dialog.tsx` | feature | src/pages/tasks/components/tasks_import_dialog.rs | todo | demo page behind flag `demo.tasks`, default off (§5) |
| `src/features/tasks/components/tasks-multi-delete-dialog.test.tsx` | test | — | n/a | DOM test; manual check in P11, end-to-end later (§4.6) |
| `src/features/tasks/components/tasks-multi-delete-dialog.tsx` | feature | src/pages/tasks/components/tasks_multi_delete_dialog.rs | todo | demo page behind flag `demo.tasks`, default off (§5) |
| `src/features/tasks/components/tasks-mutate-drawer.test.tsx` | test | — | n/a | DOM test; manual check in P11, end-to-end later (§4.6) |
| `src/features/tasks/components/tasks-mutate-drawer.tsx` | feature | src/pages/tasks/components/tasks_mutate_drawer.rs | todo | demo page behind flag `demo.tasks`, default off (§5) |
| `src/features/tasks/components/tasks-primary-buttons.tsx` | feature | src/pages/tasks/components/tasks_primary_buttons.rs | todo | demo page behind flag `demo.tasks`, default off (§5) |
| `src/features/tasks/components/tasks-provider.tsx` | feature | src/pages/tasks/components/tasks_provider.rs | todo | demo page behind flag `demo.tasks`, default off (§5) |
| `src/features/tasks/components/tasks-table.tsx` | feature | src/pages/tasks/components/tasks_table.rs | todo | demo page behind flag `demo.tasks`, default off (§5) |
| `src/features/tasks/data/data.tsx` | data | src/pages/tasks/data/data.rs | todo | demo seed data, flag `demo.tasks` (§5) |
| `src/features/tasks/data/schema.ts` | data | src/pages/tasks/data/schema.rs | todo | demo seed data, flag `demo.tasks` (§5) |
| `src/features/tasks/data/tasks.ts` | data | src/pages/tasks/data/tasks.rs | todo | demo seed data, flag `demo.tasks` (§5) |
| `src/features/tasks/index.tsx` | feature | src/pages/tasks/mod.rs | todo | demo page behind flag `demo.tasks`, default off (§5) |
| `src/features/users/components/data-table-bulk-actions.tsx` | feature | src/pages/users/components/data_table_bulk_actions.rs | todo |  |
| `src/features/users/components/data-table-row-actions.tsx` | feature | src/pages/users/components/data_table_row_actions.rs | todo |  |
| `src/features/users/components/users-action-dialog.test.tsx` | test | — | n/a | DOM test; manual check in P11, end-to-end later (§4.6) |
| `src/features/users/components/users-action-dialog.tsx` | feature | src/pages/users/components/users_action_dialog.rs | todo |  |
| `src/features/users/components/users-columns.tsx` | feature | src/pages/users/components/users_columns.rs | todo |  |
| `src/features/users/components/users-delete-dialog.test.tsx` | test | — | n/a | DOM test; manual check in P11, end-to-end later (§4.6) |
| `src/features/users/components/users-delete-dialog.tsx` | feature | src/pages/users/components/users_delete_dialog.rs | todo |  |
| `src/features/users/components/users-dialogs.tsx` | feature | src/pages/users/components/users_dialogs.rs | todo |  |
| `src/features/users/components/users-invite-dialog.test.tsx` | test | — | n/a | DOM test; manual check in P11, end-to-end later (§4.6) |
| `src/features/users/components/users-invite-dialog.tsx` | feature | src/pages/users/components/users_invite_dialog.rs | todo |  |
| `src/features/users/components/users-multi-delete-dialog.test.tsx` | test | — | n/a | DOM test; manual check in P11, end-to-end later (§4.6) |
| `src/features/users/components/users-multi-delete-dialog.tsx` | feature | src/pages/users/components/users_multi_delete_dialog.rs | todo |  |
| `src/features/users/components/users-primary-buttons.tsx` | feature | src/pages/users/components/users_primary_buttons.rs | todo |  |
| `src/features/users/components/users-provider.tsx` | feature | src/pages/users/components/users_provider.rs | todo |  |
| `src/features/users/components/users-table.tsx` | feature | src/pages/users/components/users_table.rs | todo |  |
| `src/features/users/data/data.ts` | data | src/pages/users/data/data.rs | todo | real DB (M5/M10), no mock data (§5) |
| `src/features/users/data/schema.ts` | data | src/pages/users/data/schema.rs | todo | real DB (M5/M10), no mock data (§5) |
| `src/features/users/data/users.ts` | data | src/pages/users/data/users.rs | todo | real DB (M5/M10), no mock data (§5) |
| `src/features/users/index.tsx` | feature | src/pages/users/mod.rs | todo |  |
| `src/hooks/use-dialog-state.tsx` | hook | src/app_hooks/dialog_state.rs | todo |  |
| `src/hooks/use-mobile.tsx` | hook | src/components/ui_extra/sidebar.rs (SidebarProvider) | done | matchMedia(<768px) via eval |
| `src/hooks/use-table-url-state.test.ts` | test | src/app_hooks/table_url_state.rs #[cfg(test)] | todo | logic test → Rust #[test] |
| `src/hooks/use-table-url-state.ts` | hook | src/app_hooks/table_url_state.rs | todo |  |
| `src/lib/cookies.test.ts` | test | src/util/cookies.rs #[cfg(test)] | todo | logic test → Rust #[test] |
| `src/lib/cookies.ts` | lib/util | src/util/cookies.rs | done | request (SSR) + set/remove (browser) |
| `src/lib/handle-server-error.test.ts` | test | src/i18n.rs (error_text) #[cfg(test)] | todo | logic test → Rust #[test] |
| `src/lib/handle-server-error.ts` | lib/util | src/i18n.rs (error_text) + domain::errors | todo |  |
| `src/lib/show-submitted-data.tsx` | lib/util | src/util/mod.rs | done | toast with JSON description |
| `src/lib/utils.test.ts` | test | src/util/mod.rs #[cfg(test)] | todo | logic test → Rust #[test] |
| `src/lib/utils.ts` | lib/util | src/util/mod.rs | todo |  |
| `src/main.tsx` | config | src/main.rs + src/app.rs | in-progress | providers: i18n, Ui prefs, toaster, router; font provider in P5 |
| `src/routeTree.gen.ts` | route | src/app.rs (Route enum) | n/a | generated by TanStack Router; the Routable enum replaces it |
| `src/routes/(auth)/forgot-password.tsx` | route | src/app.rs | done | public route |
| `src/routes/(auth)/otp.tsx` | route | src/app.rs | done | public route |
| `src/routes/(auth)/sign-in-2.tsx` | route | src/app.rs | done | public route |
| `src/routes/(auth)/sign-in.tsx` | route | src/app.rs | done | public route |
| `src/routes/(auth)/sign-up.tsx` | route | src/app.rs | done | public route |
| `src/routes/(errors)/401.tsx` | route | src/app.rs (Route::Error401) | done | public route |
| `src/routes/(errors)/403.tsx` | route | src/app.rs (Route::Error403) | done | public route |
| `src/routes/(errors)/404.tsx` | route | src/app.rs (Route::Error404) | done | public route |
| `src/routes/(errors)/500.tsx` | route | src/app.rs (Route::Error500) | done | public route |
| `src/routes/(errors)/503.tsx` | route | src/app.rs (Route::Error503) | done | public route |
| `src/routes/__root.tsx` | route | src/app.rs (Route::…) | in-progress | root = App; navigation progress bar in P3 |
| `src/routes/_authenticated/apps/index.tsx` | route | src/app.rs (Route::…) | todo | _authenticated/apps/index |
| `src/routes/_authenticated/chats/index.tsx` | route | src/app.rs (Route::…) | todo | _authenticated/chats/index |
| `src/routes/_authenticated/errors/$error.tsx` | route | src/app.rs (Route::ShellError) | done | `ShellError` with fixed header |
| `src/routes/_authenticated/help-center/index.tsx` | route | src/app.rs (Route::ComingSoon) | done |  |
| `src/routes/_authenticated/index.tsx` | route | src/app.rs (Route::Dashboard) | done |  |
| `src/routes/_authenticated/route.tsx` | route | src/app.rs (#[layout(AppShell)]) | done |  |
| `src/routes/_authenticated/settings/account.tsx` | route | src/app.rs (#[nest("/settings")]) | done |  |
| `src/routes/_authenticated/settings/appearance.tsx` | route | src/app.rs (#[nest("/settings")]) | done |  |
| `src/routes/_authenticated/settings/display.tsx` | route | src/app.rs (#[nest("/settings")]) | done |  |
| `src/routes/_authenticated/settings/index.tsx` | route | src/app.rs (#[nest("/settings")]) | done |  |
| `src/routes/_authenticated/settings/notifications.tsx` | route | src/app.rs (#[nest("/settings")]) | done |  |
| `src/routes/_authenticated/settings/route.tsx` | route | src/app.rs (#[nest("/settings")]) | done |  |
| `src/routes/_authenticated/tasks/index.tsx` | route | src/app.rs (Route::…) | todo | _authenticated/tasks/index |
| `src/routes/_authenticated/users/index.tsx` | route | src/app.rs (Route::…) | todo | _authenticated/users/index |
| `src/routes/clerk/(auth)/route.tsx` | route | — | n/a | §5: Clerk routes removed; their screens map to our own auth/user pages |
| `src/routes/clerk/(auth)/sign-in.tsx` | route | — | n/a | §5: Clerk routes removed; their screens map to our own auth/user pages |
| `src/routes/clerk/(auth)/sign-up.tsx` | route | — | n/a | §5: Clerk routes removed; their screens map to our own auth/user pages |
| `src/routes/clerk/_authenticated/route.tsx` | route | — | n/a | §5: Clerk routes removed; their screens map to our own auth/user pages |
| `src/routes/clerk/_authenticated/user-management.tsx` | route | — | n/a | §5: Clerk routes removed; their screens map to our own auth/user pages |
| `src/routes/clerk/route.tsx` | route | — | n/a | §5: Clerk routes removed; their screens map to our own auth/user pages |
| `src/stores/auth-store.test.ts` | test | src/components/layout/app_layout.rs (UserResource) #[cfg(test)] | todo | logic test → Rust #[test] |
| `src/stores/auth-store.ts` | context | src/components/layout/app_layout.rs (UserResource) | todo | client auth state = current_user() resource (M1) |
| `src/styles/index.css` | config | tailwind.css | done | base layer (body scroll, min-h-svh, inset bg), no-scrollbar, faded-bottom (inset-s → start for Tailwind 4.1); Radix CollapsibleContent rules n/a |
| `src/styles/theme.css` | config | tailwind.css | done | tokens match the reference in light + dark (47 colours checked in the browser); sidenav-* follows sidebar-* |
| `src/tanstack-table.d.ts` | config | — | n/a | TypeScript type declarations |
| `src/test-utils/cookies.ts` | test | — | n/a | JS test helpers; Rust tests need none |
| `src/test-utils/tanstack-table.ts` | test | — | n/a | JS test helpers; Rust tests need none |
| `src/vite-env.d.ts` | config | — | n/a | TypeScript type declarations |
| `tsconfig.app.json` | config | — | n/a | TypeScript config |
| `tsconfig.json` | config | — | n/a | TypeScript config |
| `tsconfig.node.json` | config | — | n/a | TypeScript config |
| `vite.config.ts` | config | — | n/a | Vite build config (dx builds ours) |
