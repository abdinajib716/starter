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
| `src/assets/brand-icons/icon-facebook.tsx` | asset | src/components/icons/brand.rs | todo | inline SVG components |
| `src/assets/brand-icons/icon-figma.tsx` | asset | src/components/icons/brand.rs | todo | inline SVG components |
| `src/assets/brand-icons/icon-github.tsx` | asset | src/components/icons/brand.rs | todo | inline SVG components |
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
| `src/assets/logo.tsx` | asset | src/components/icons/logo.rs | todo | our logo (branding) |
| `src/components/coming-soon.tsx` | component | src/components/coming_soon.rs | todo |  |
| `src/components/command-menu.tsx` | component | src/components/command_menu.rs | todo |  |
| `src/components/config-drawer.test.tsx` | test | — | n/a | DOM test; manual check in P11, end-to-end later (§4.6) |
| `src/components/config-drawer.tsx` | component | src/components/config_drawer.rs | todo |  |
| `src/components/confirm-dialog.test.tsx` | test | — | n/a | DOM test; manual check in P11, end-to-end later (§4.6) |
| `src/components/confirm-dialog.tsx` | component | src/components/confirm_dialog.rs | todo |  |
| `src/components/data-table/bulk-actions.tsx` | component | src/components/data_table/bulk_actions.rs | todo |  |
| `src/components/data-table/column-header.tsx` | component | src/components/data_table/column_header.rs | todo |  |
| `src/components/data-table/faceted-filter.tsx` | component | src/components/data_table/faceted_filter.rs | todo |  |
| `src/components/data-table/index.ts` | component | src/components/data_table/mod.rs | todo |  |
| `src/components/data-table/pagination.tsx` | component | src/components/data_table/pagination.rs | todo |  |
| `src/components/data-table/toolbar.tsx` | component | src/components/data_table/toolbar.rs | todo |  |
| `src/components/data-table/view-options.tsx` | component | src/components/data_table/view_options.rs | todo |  |
| `src/components/date-picker.tsx` | component | src/components/date_picker.rs | todo |  |
| `src/components/layout/app-sidebar.tsx` | layout | src/components/layout/sidebar.rs | todo |  |
| `src/components/layout/app-title.tsx` | layout | src/components/layout/sidebar_top.rs | todo |  |
| `src/components/layout/authenticated-layout.tsx` | layout | src/components/layout/app_shell.rs | todo |  |
| `src/components/layout/data/sidebar-data.ts` | data | src/components/layout/nav_config.rs | todo | our menu tree, permission-filtered |
| `src/components/layout/header.tsx` | layout | src/components/layout/header.rs | todo |  |
| `src/components/layout/main.tsx` | layout | src/components/layout/main_area.rs | todo |  |
| `src/components/layout/nav-group.tsx` | layout | src/components/layout/nav_group.rs | todo |  |
| `src/components/layout/nav-user.tsx` | layout | src/components/layout/nav_user.rs | todo |  |
| `src/components/layout/team-switcher.tsx` | layout | src/components/layout/team_switcher.rs | todo |  |
| `src/components/layout/top-nav.tsx` | layout | src/components/layout/top_nav.rs | todo |  |
| `src/components/layout/types.ts` | data | src/components/layout/nav_config.rs | todo | our menu tree, permission-filtered |
| `src/components/learn-more.tsx` | component | src/components/learn_more.rs | todo |  |
| `src/components/long-text.tsx` | component | src/components/long_text.rs | todo |  |
| `src/components/navigation-progress.tsx` | component | src/components/navigation_progress.rs | todo |  |
| `src/components/password-input.test.tsx` | test | src/components/password_input.rs #[cfg(test)] | todo | logic test → Rust #[test] |
| `src/components/password-input.tsx` | component | src/components/password_input.rs | todo |  |
| `src/components/profile-dropdown.tsx` | component | src/components/profile_dropdown.rs | todo |  |
| `src/components/search.tsx` | component | src/components/search.rs | todo |  |
| `src/components/select-dropdown.tsx` | component | src/components/select_dropdown.rs | todo |  |
| `src/components/sign-out-dialog.test.tsx` | test | — | n/a | DOM test; manual check in P11, end-to-end later (§4.6) |
| `src/components/sign-out-dialog.tsx` | component | src/components/sign_out_dialog.rs | todo |  |
| `src/components/skip-to-main.tsx` | component | src/components/skip_to_main.rs | todo |  |
| `src/components/theme-switch.tsx` | component | src/components/theme_switch.rs | todo |  |
| `src/components/ui/alert-dialog.tsx` | ui-primitive | src/ui/alert_dialog.rs | done | Rust/UI equivalent vendored; visual parity checked in P11 |
| `src/components/ui/alert.tsx` | ui-primitive | src/ui/alert.rs | done | Rust/UI equivalent vendored; visual parity checked in P11 |
| `src/components/ui/avatar.tsx` | ui-primitive | src/ui/avatar.rs | done | Rust/UI equivalent vendored; visual parity checked in P11 |
| `src/components/ui/badge.tsx` | ui-primitive | src/ui/badge.rs | done | Rust/UI equivalent vendored; visual parity checked in P11 |
| `src/components/ui/button.tsx` | ui-primitive | src/ui/button.rs | done | Rust/UI equivalent vendored; visual parity checked in P11 |
| `src/components/ui/calendar.tsx` | ui-primitive | src/ui/date_picker.rs | done | added from Rust/UI 4f5a516 in P1; visual parity checked in P11 |
| `src/components/ui/card.tsx` | ui-primitive | src/ui/card.rs | done | Rust/UI equivalent vendored; visual parity checked in P11 |
| `src/components/ui/checkbox.tsx` | ui-primitive | src/ui/checkbox.rs | done | Rust/UI equivalent vendored; visual parity checked in P11 |
| `src/components/ui/collapsible.tsx` | ui-primitive | src/ui/collapsible.rs | done | Rust/UI equivalent vendored; visual parity checked in P11 |
| `src/components/ui/command.tsx` | ui-primitive | src/ui/command.rs | done | added from Rust/UI 4f5a516 in P1; visual parity checked in P11 |
| `src/components/ui/dialog.tsx` | ui-primitive | src/ui/dialog.rs | done | Rust/UI equivalent vendored; visual parity checked in P11 |
| `src/components/ui/dropdown-menu.tsx` | ui-primitive | src/ui/dropdown_menu.rs | done | Rust/UI equivalent vendored; visual parity checked in P11 |
| `src/components/ui/form.tsx` | ui-primitive | src/ui/form.rs | done | Rust/UI equivalent vendored; visual parity checked in P11 |
| `src/components/ui/input-otp.tsx` | ui-primitive | src/ui/input_otp.rs | done | added from Rust/UI 4f5a516 in P1; visual parity checked in P11 |
| `src/components/ui/input.tsx` | ui-primitive | src/ui/input.rs | done | Rust/UI equivalent vendored; visual parity checked in P11 |
| `src/components/ui/label.tsx` | ui-primitive | src/ui/label.rs | done | Rust/UI equivalent vendored; visual parity checked in P11 |
| `src/components/ui/popover.tsx` | ui-primitive | src/ui/popover.rs | done | Rust/UI equivalent vendored; visual parity checked in P11 |
| `src/components/ui/radio-group.tsx` | ui-primitive | src/ui/radio_button_group.rs | done | added from Rust/UI 4f5a516 in P1; visual parity checked in P11 |
| `src/components/ui/scroll-area.tsx` | ui-primitive | src/ui/scroll_area.rs | done | added from Rust/UI 4f5a516 in P1; visual parity checked in P11 |
| `src/components/ui/select.tsx` | ui-primitive | src/ui/select.rs | done | Rust/UI equivalent vendored; visual parity checked in P11 |
| `src/components/ui/separator.tsx` | ui-primitive | src/ui/separator.rs | done | Rust/UI equivalent vendored; visual parity checked in P11 |
| `src/components/ui/sheet.tsx` | ui-primitive | src/ui/sheet.rs | done | Rust/UI equivalent vendored; visual parity checked in P11 |
| `src/components/ui/sidebar.tsx` | ui-primitive | src/ui/sidenav.rs | done | Rust/UI equivalent vendored; visual parity checked in P11 |
| `src/components/ui/skeleton.tsx` | ui-primitive | src/ui/skeleton.rs | done | Rust/UI equivalent vendored; visual parity checked in P11 |
| `src/components/ui/sonner.tsx` | ui-primitive | src/ui/toast.rs | done | Rust/UI equivalent vendored; visual parity checked in P11 |
| `src/components/ui/switch.tsx` | ui-primitive | src/ui/switch.rs | done | Rust/UI equivalent vendored; visual parity checked in P11 |
| `src/components/ui/table.tsx` | ui-primitive | src/ui/table.rs | done | Rust/UI equivalent vendored; visual parity checked in P11 |
| `src/components/ui/tabs.tsx` | ui-primitive | src/ui/tabs.rs | done | Rust/UI equivalent vendored; visual parity checked in P11 |
| `src/components/ui/textarea.tsx` | ui-primitive | src/ui/textarea.rs | done | Rust/UI equivalent vendored; visual parity checked in P11 |
| `src/components/ui/tooltip.tsx` | ui-primitive | src/ui/tooltip.rs | done | Rust/UI equivalent vendored; visual parity checked in P11 |
| `src/config/fonts.ts` | config | src/state/font.rs + self-hosted fonts in assets/fonts/ | in-progress | Inter + Manrope self-hosted in public/fonts, `font-inter`/`font-manrope`/`font-system` utilities; switcher in P5 |
| `src/context/direction-provider.tsx` | context | src/state/direction.rs | todo |  |
| `src/context/font-provider.tsx` | context | src/state/font.rs | todo |  |
| `src/context/layout-provider.tsx` | context | src/state/layout.rs | todo |  |
| `src/context/search-provider.test.tsx` | test | src/state/search.rs #[cfg(test)] | todo | logic test → Rust #[test] |
| `src/context/search-provider.tsx` | context | src/state/search.rs | todo |  |
| `src/context/theme-provider.tsx` | context | src/state/theme.rs | todo |  |
| `src/features/apps/data/apps.tsx` | data | src/pages/apps/data/apps.rs | todo | demo seed data, flag `demo.apps` (§5) |
| `src/features/apps/index.tsx` | feature | src/pages/apps/mod.rs | todo | demo page behind flag `demo.apps`, default off (§5) |
| `src/features/auth/auth-layout.tsx` | feature | src/pages/auth/auth_layout.rs | todo |  |
| `src/features/auth/forgot-password/components/forgot-password-form.test.tsx` | test | — | n/a | DOM test; manual check in P11, end-to-end later (§4.6) |
| `src/features/auth/forgot-password/components/forgot-password-form.tsx` | feature | src/pages/auth/forgot_password/components/forgot_password_form.rs | todo |  |
| `src/features/auth/forgot-password/index.tsx` | feature | src/pages/auth/forgot_password/mod.rs | todo |  |
| `src/features/auth/otp/components/otp-form.test.tsx` | test | — | n/a | DOM test; manual check in P11, end-to-end later (§4.6) |
| `src/features/auth/otp/components/otp-form.tsx` | feature | src/pages/auth/otp/components/otp_form.rs | todo |  |
| `src/features/auth/otp/index.tsx` | feature | src/pages/auth/otp/mod.rs | todo |  |
| `src/features/auth/sign-in/assets/dashboard-dark.png` | asset | assets/auth/dashboard-dark.png | todo | sign-in-2 preview image (MIT) |
| `src/features/auth/sign-in/assets/dashboard-light.png` | asset | assets/auth/dashboard-light.png | todo | sign-in-2 preview image (MIT) |
| `src/features/auth/sign-in/components/user-auth-form.test.tsx` | test | — | n/a | DOM test; manual check in P11, end-to-end later (§4.6) |
| `src/features/auth/sign-in/components/user-auth-form.tsx` | feature | src/pages/auth/sign_in/components/user_auth_form.rs | todo |  |
| `src/features/auth/sign-in/index.tsx` | feature | src/pages/auth/sign_in/mod.rs | todo |  |
| `src/features/auth/sign-in/sign-in-2.tsx` | feature | src/pages/auth/sign_in/sign_in_2.rs | todo |  |
| `src/features/auth/sign-up/components/sign-up-form.test.tsx` | test | — | n/a | DOM test; manual check in P11, end-to-end later (§4.6) |
| `src/features/auth/sign-up/components/sign-up-form.tsx` | feature | src/pages/auth/sign_up/components/sign_up_form.rs | todo |  |
| `src/features/auth/sign-up/index.tsx` | feature | src/pages/auth/sign_up/mod.rs | todo |  |
| `src/features/chats/components/new-chat.tsx` | feature | src/pages/chats/components/new_chat.rs | todo | demo page behind flag `demo.chats`, default off (§5) |
| `src/features/chats/data/chat-types.ts` | data | src/pages/chats/data/chat_types.rs | todo | demo seed data, flag `demo.chats` (§5) |
| `src/features/chats/data/convo.json` | data | src/pages/chats/data/convo.rs | todo | demo seed data, flag `demo.chats` (§5) |
| `src/features/chats/index.tsx` | feature | src/pages/chats/mod.rs | todo | demo page behind flag `demo.chats`, default off (§5) |
| `src/features/dashboard/components/analytics-chart.tsx` | feature | src/pages/dashboard/components/analytics_chart.rs | todo |  |
| `src/features/dashboard/components/analytics.tsx` | feature | src/pages/dashboard/components/analytics.rs | todo |  |
| `src/features/dashboard/components/overview.tsx` | feature | src/pages/dashboard/components/overview.rs | todo |  |
| `src/features/dashboard/components/recent-sales.tsx` | feature | src/pages/dashboard/components/recent_sales.rs | todo |  |
| `src/features/dashboard/index.tsx` | feature | src/pages/dashboard/mod.rs | todo |  |
| `src/features/errors/forbidden.tsx` | feature | src/pages/errors/forbidden.rs | todo |  |
| `src/features/errors/general-error.tsx` | feature | src/pages/errors/general_error.rs | todo |  |
| `src/features/errors/maintenance-error.tsx` | feature | src/pages/errors/maintenance_error.rs | todo |  |
| `src/features/errors/not-found-error.tsx` | feature | src/pages/errors/not_found_error.rs | todo |  |
| `src/features/errors/unauthorized-error.tsx` | feature | src/pages/errors/unauthorized_error.rs | todo |  |
| `src/features/settings/account/account-form.tsx` | feature | src/pages/settings/account/account_form.rs | todo |  |
| `src/features/settings/account/index.tsx` | feature | src/pages/settings/account/mod.rs | todo |  |
| `src/features/settings/appearance/appearance-form.tsx` | feature | src/pages/settings/appearance/appearance_form.rs | todo |  |
| `src/features/settings/appearance/index.tsx` | feature | src/pages/settings/appearance/mod.rs | todo |  |
| `src/features/settings/components/content-section.tsx` | feature | src/pages/settings/components/content_section.rs | todo |  |
| `src/features/settings/components/sidebar-nav.tsx` | feature | src/pages/settings/components/sidebar_nav.rs | todo |  |
| `src/features/settings/display/display-form.tsx` | feature | src/pages/settings/display/display_form.rs | todo |  |
| `src/features/settings/display/index.tsx` | feature | src/pages/settings/display/mod.rs | todo |  |
| `src/features/settings/index.tsx` | feature | src/pages/settings/mod.rs | todo |  |
| `src/features/settings/notifications/index.tsx` | feature | src/pages/settings/notifications/mod.rs | todo |  |
| `src/features/settings/notifications/notifications-form.tsx` | feature | src/pages/settings/notifications/notifications_form.rs | todo |  |
| `src/features/settings/profile/index.tsx` | feature | src/pages/settings/profile/mod.rs | todo |  |
| `src/features/settings/profile/profile-form.tsx` | feature | src/pages/settings/profile/profile_form.rs | todo |  |
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
| `src/hooks/use-mobile.tsx` | hook | src/app_hooks/mobile.rs | todo |  |
| `src/hooks/use-table-url-state.test.ts` | test | src/app_hooks/table_url_state.rs #[cfg(test)] | todo | logic test → Rust #[test] |
| `src/hooks/use-table-url-state.ts` | hook | src/app_hooks/table_url_state.rs | todo |  |
| `src/lib/cookies.test.ts` | test | src/util/cookies.rs #[cfg(test)] | todo | logic test → Rust #[test] |
| `src/lib/cookies.ts` | lib/util | src/util/cookies.rs | todo |  |
| `src/lib/handle-server-error.test.ts` | test | src/i18n.rs (error_text) #[cfg(test)] | todo | logic test → Rust #[test] |
| `src/lib/handle-server-error.ts` | lib/util | src/i18n.rs (error_text) + domain::errors | todo |  |
| `src/lib/show-submitted-data.tsx` | lib/util | src/util/mod.rs (toast helper) | todo |  |
| `src/lib/utils.test.ts` | test | src/util/mod.rs #[cfg(test)] | todo | logic test → Rust #[test] |
| `src/lib/utils.ts` | lib/util | src/util/mod.rs | todo |  |
| `src/main.tsx` | config | src/main.rs + src/app.rs | todo | providers (theme, font, direction, layout, search), toaster, router |
| `src/routeTree.gen.ts` | route | src/app.rs (Route enum) | n/a | generated by TanStack Router; the Routable enum replaces it |
| `src/routes/(auth)/forgot-password.tsx` | route | src/app.rs (Route::…) | todo | (auth)/forgot-password |
| `src/routes/(auth)/otp.tsx` | route | src/app.rs (Route::…) | todo | (auth)/otp |
| `src/routes/(auth)/sign-in-2.tsx` | route | src/app.rs (Route::…) | todo | (auth)/sign-in-2 |
| `src/routes/(auth)/sign-in.tsx` | route | src/app.rs (Route::…) | todo | (auth)/sign-in |
| `src/routes/(auth)/sign-up.tsx` | route | src/app.rs (Route::…) | todo | (auth)/sign-up |
| `src/routes/(errors)/401.tsx` | route | src/app.rs (Route::…) | todo | (errors)/401 |
| `src/routes/(errors)/403.tsx` | route | src/app.rs (Route::…) | todo | (errors)/403 |
| `src/routes/(errors)/404.tsx` | route | src/app.rs (Route::…) | todo | (errors)/404 |
| `src/routes/(errors)/500.tsx` | route | src/app.rs (Route::…) | todo | (errors)/500 |
| `src/routes/(errors)/503.tsx` | route | src/app.rs (Route::…) | todo | (errors)/503 |
| `src/routes/__root.tsx` | route | src/app.rs (Route::…) | todo | __root |
| `src/routes/_authenticated/apps/index.tsx` | route | src/app.rs (Route::…) | todo | _authenticated/apps/index |
| `src/routes/_authenticated/chats/index.tsx` | route | src/app.rs (Route::…) | todo | _authenticated/chats/index |
| `src/routes/_authenticated/errors/$error.tsx` | route | src/app.rs (Route::…) | todo | _authenticated/errors/$error |
| `src/routes/_authenticated/help-center/index.tsx` | route | src/app.rs (Route::…) | todo | _authenticated/help-center/index |
| `src/routes/_authenticated/index.tsx` | route | src/app.rs (Route::…) | todo | _authenticated/index |
| `src/routes/_authenticated/route.tsx` | route | src/app.rs (Route::…) | todo | _authenticated/route |
| `src/routes/_authenticated/settings/account.tsx` | route | src/app.rs (Route::…) | todo | _authenticated/settings/account |
| `src/routes/_authenticated/settings/appearance.tsx` | route | src/app.rs (Route::…) | todo | _authenticated/settings/appearance |
| `src/routes/_authenticated/settings/display.tsx` | route | src/app.rs (Route::…) | todo | _authenticated/settings/display |
| `src/routes/_authenticated/settings/index.tsx` | route | src/app.rs (Route::…) | todo | _authenticated/settings/index |
| `src/routes/_authenticated/settings/notifications.tsx` | route | src/app.rs (Route::…) | todo | _authenticated/settings/notifications |
| `src/routes/_authenticated/settings/route.tsx` | route | src/app.rs (Route::…) | todo | _authenticated/settings/route |
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
