# English — source of truth (§26). Keys: <area>-<name>.
app-name = Starter


nav-profile = Profile

login-title = Sign in
login-submit = Sign in
field-email = Email
field-password = Password

profile-change-password = Change password
profile-must-change = You must set a new password before continuing.
profile-save = Save password
profile-password-changed = Password changed. Other sessions were signed out.
field-current-password = Current password
field-new-password = New password
field-confirm-password = Confirm new password
password-hint = At least 10 characters. Common passwords are rejected.
password-mismatch = The new passwords do not match.

err-generic = Something went wrong. Please try again.
err-internal = Something went wrong on the server. Please try again.
err-auth-invalid-credentials = Incorrect email or password.
err-auth-rate-limited = Too many attempts. Please wait a few minutes and try again.
err-auth-wrong-password = The current password is incorrect.
err-auth-unauthenticated = Your session has ended. Please sign in again.
err-password-too-short = The new password is too short.
err-password-too-long = The new password is too long (maximum 256 characters).
err-password-too-common = This password is too common. Choose another one.

# ---- App shell (SPEC-02, shadcn-admin) ----
app-plan = Admin Dashboard
skip-to-main = Skip to Main
action-cancel = Cancel

nav-group-general = General
nav-dashboard = Dashboard
nav-users = User
nav-appearance = Appearance

menu-sign-out = Sign out
signout-title = Sign out
signout-desc = Are you sure you want to sign out? You will need to sign in again to access your account.
signout-confirm = Sign out

search-placeholder = Search
search-command-placeholder = Type a command or search...
search-empty = No results found.

theme-toggle = Toggle theme
theme-label = Theme
theme-light = Light
theme-dark = Dark
theme-system = System

config-open = Open theme settings
config-title = Theme Settings
config-desc = Adjust the appearance and layout to suit your preferences.
config-sidebar = Sidebar
config-inset = Inset
config-floating = Floating
config-sidebar-plain = Sidebar
config-layout = Layout
config-layout-default = Default
config-layout-compact = Compact
config-layout-full = Full layout
config-direction = Direction
config-ltr = Left to Right
config-rtl = Right to Left
config-reset = Reset

# ---- Dashboard ----
dash-tab-overview = Overview

# ---- Error pages / coming soon ----
err-go-back = Go Back
err-back-home = Back to Home
err-learn-more = Learn more
err401-title = Unauthorized Access
err401-desc-1 = Please log in with the appropriate credentials
err401-desc-2 = to access this resource.
err403-title = Access Forbidden
err403-desc-1 = You don't have necessary permission
err403-desc-2 = to view this resource.
err404-title = Oops! Page Not Found!
err404-desc-1 = It seems like the page you're looking for
err404-desc-2 = does not exist or might have been removed.
err500-title = Oops! Something went wrong :')
err500-desc-1 = We apologize for the inconvenience.
err500-desc-2 = Please try again later.
err503-title = Website is under maintenance!
err503-desc-1 = The site is not available at the moment.
err503-desc-2 = We'll be back online shortly.

# ---- Settings (shadcn-admin demo forms) ----

f-font = Font
f-font-desc = Set the font you want to use in the dashboard.

# ---- Auth screens ----
auth-sign-in-desc-1 = Enter your email and password below to log into
auth-welcome = Welcome back, { $email }!
pw-show = Show password
pw-hide = Hide password
v-email-enter = Please enter your email.
v-email-invalid = Invalid email address
v-password-enter = Please enter your password.
v-password-min7 = Password must be at least 7 characters long.
v-password-match = Passwords don't match.

# ---- Apps / Chats (demo pages) ----

## data table (components/data-table/*)
dt-page-of = Page { $page } of { $total }
dt-rows-per-page = Rows per page
dt-first = Go to first page
dt-previous = Go to previous page
dt-next = Go to next page
dt-last = Go to last page
dt-go-page = Go to page { $page }
dt-asc = Asc
dt-desc = Desc
dt-hide = Hide
dt-view = View
dt-toggle-columns = Toggle columns
dt-reset = Reset
dt-n-selected = { $count } selected
dt-clear-filters = Clear filters
dt-no-results = No results.
dt-select-all = Select all
dt-select-row = Select row
dt-open-menu = Open menu
dt-edit = Edit
dt-delete = Delete
dt-selected = selected
dt-clear-selection = Clear selection
dt-clear-selection-esc = Clear selection (Escape)
dt-bulk-label = Bulk actions for { $count } selected { $entity }
dt-bulk-announce = { $count } { $entity } selected. Bulk actions toolbar is available.
dt-cannot-undo = This action cannot be undone.
dt-multi-delete-sure = Are you sure you want to delete the selected { $entity }?
dt-confirm-typing = Confirm by typing "{ $word }":
dt-type-to-confirm = Type "{ $word }" to confirm.
dt-type-to-confirm-error = Please type "{ $word }" to confirm.
dt-warning = Warning!
dt-warning-desc = Please be careful, this operation can not be rolled back.
action-close = Close
action-save-changes = Save changes

## tasks (features/tasks/*)
tasks-save-hint = Click save when you're done.

## users (features/users/*)
users-title = User List
users-desc = Manage your users and their roles here.
users-add = Add User
users-search = Filter users...
users-col-name = Name
users-col-email = Email
users-col-status = Status
users-col-role = Role
users-entity = user
users-entity-plural = users
user-status-active = Active
user-status-inactive = Inactive
user-status-invited = Invited
user-status-suspended = Suspended
users-bulk-activate = Activate selected users
users-bulk-deactivate = Deactivate selected users
users-bulk-delete = Delete selected users
users-activated = Activated { $count } { $entity }
users-deactivated = Deactivated { $count } { $entity }
users-deleted = Deleted { $count } { $entity }
users-delete-n = Delete { $count } { $entity }
users-add-title = Add New User
users-edit-title = Edit User
users-add-desc = Create new user here.
users-edit-desc = Update the user here.
users-password = Password
users-confirm-password = Confirm Password
users-password-ph = e.g., S3cur3P@ssw0rd
users-select-role = Select a role
users-delete-title = Delete User
users-delete-sure = Are you sure you want to delete
users-delete-role = This removes the user with the role of

## appearance drawer cards
config-reset-theme = Reset theme preference to default
config-reset-sidebar = Reset sidebar style to default
config-reset-layout = Reset layout options to default
config-reset-dir = Reset text direction to default
config-reset-all = Reset all settings to default values
config-select-theme = Select theme preference
config-select-sidebar = Select sidebar style
config-select-layout = Select layout style
config-select-dir = Select site direction
config-select-option = Select { $option }
sidebar-toggle = Toggle Sidebar

## users on real data (M5)
users-export = Export
users-tab-active = Users
users-tab-deleted = Deleted
users-col-last-login = Last sign-in
users-col-deleted = Deleted
users-never = Never
users-deleted-by = { $date } by { $by }
user-status-deleted = Deleted
users-name-ph = John Doe
users-new-password = New password
users-password-keep = Leave empty to keep the current password
users-temp-password-note = The user must change this temporary password at the next sign-in.
users-email-label = Email:
users-email-confirm-ph = Type the email to confirm.
users-delete-soft-tail = and sign them out. You can restore them from the Deleted tab.
users-purge = Delete permanently
users-purge-title = Delete permanently
users-purge-sure = This permanently deletes
users-purge-tail = The audit history keeps their email. This cannot be undone.
users-purged = User permanently deleted.
users-restore = Restore
users-restored = User restored. Activate them to allow sign-in.
users-restored-n = Restored { $count } { $entity }
users-bulk-restore = Restore selected users
users-bulk-export = Export selected users
users-activate = Activate
users-deactivate = Deactivate
users-activated-one = User activated.
users-deactivated-one = User deactivated and signed out.
users-deleted-one = User deleted.
users-created = User created.
users-updated = User updated.
users-skipped = Skipped { $count }: { $reason }
## error codes (server → client, §26)
err-auth-forbidden = You don't have permission to do that.
err-not-found = That item no longer exists.
err-users-self-action = You can't do that to your own account.
err-users-last-super-admin = The last active Super Admin can't be removed, deactivated or demoted.
err-users-email-taken = This email is already used by another user.
err-users-email-invalid = Enter a valid email address.
err-users-name-required = Enter a name.
err-users-name-too-long = The name is too long (max 120 characters).
err-users-role-required = Select a role.
err-users-password-required = Enter a temporary password.
err-users-purge-confirm = The email you typed doesn't match.
err-users-purge-not-deleted = Only deleted users can be deleted permanently.
err-roles-not-found = That role no longer exists.
err-permissions-unknown = Unknown permission.
err-bulk-too-many = Too many items selected for one action.
err-settings-unknown-key = Unknown setting.
err-settings-invalid-value = That value isn't allowed for this setting.
err-email-disabled = Email is turned off in System Settings.
err-api-disabled = The public API is turned off.
err-notifications-disabled = Notifications are turned off.

## dashboard (real data, §10)
dash-tab-activity = Activity
dash-total-users = Total users
dash-total-users-note = +{ $n } in the last 30 days
dash-active-30d = Active users (30 days)
dash-vs-prev-30d = { $delta } from the previous 30 days
dash-online = Online now
dash-online-note = Signed in, active in the last 5 minutes
dash-logins-today = Sign-ins today
dash-vs-yesterday = { $n } yesterday
dash-signins-14d = Sign-ins
dash-signins-14d-desc = Successful and failed sign-ins, last 14 days
dash-legend-ok = Successful
dash-legend-failed = Failed
dash-recent = Recent activity
dash-recent-desc = The latest entries in the audit log
dash-recent-empty = Nothing recorded yet.
dash-system = System
dash-failed-24h = Failed sign-ins (24 h)
dash-vs-prev-24h = { $delta } from the previous 24 hours
dash-last-backup = Last backup
dash-backup-none = None yet
dash-backup-none-note = Backups arrive with the Backup module
dash-backup-completed = Completed
dash-backup-running = Running
dash-backup-failed = Failed
dash-events-7d = Audit events (7 days)
dash-modules-n = Across { $n } modules
dash-roles = Roles
dash-roles-note = Defined for this organization
dash-by-module = Activity by module
dash-by-module-desc = Audit log entries per module, last 7 days
dash-by-role = Users by role
dash-by-role-desc = Users that are not deleted
dash-new-users-12w = New users per week
dash-new-users-12w-desc = Accounts created, last 12 weeks
## audit actions (M4.6)
audit-auth-login = Signed in
audit-auth-login-failed = Failed sign-in
audit-auth-logout = Signed out
audit-auth-account-locked = Account locked
audit-profile-password-change = Changed own password
audit-user-create = Created a user
audit-user-update = Updated a user
audit-user-activate = Activated a user
audit-user-deactivate = Deactivated a user
audit-user-delete = Deleted a user
audit-user-restore = Restored a user
audit-user-purge = Permanently deleted a user
audit-user-reset-password = Reset a user's password
audit-user-bulk-activate = Activated users (bulk)
audit-user-bulk-deactivate = Deactivated users (bulk)
audit-user-bulk-delete = Deleted users (bulk)
audit-user-bulk-restore = Restored users (bulk)
audit-export-users = Exported users
audit-export-audit = Exported the audit log
audit-role-create = Created a role
audit-role-update = Updated a role
audit-role-delete = Deleted a role
audit-settings-update = Changed settings
audit-flag-update = Changed a feature flag
## audit log page (M4)
nav-audit = Audit
audit-title = Audit log
audit-desc = Every sign-in and change, who made it and when.
audit-search = Search action, user or ID...
audit-col-time = Time (UTC)
audit-col-actor = User
audit-col-action = Action
audit-col-entity = Record
audit-col-result = Result
audit-col-module = Module
audit-col-ip = IP address
audit-success = Success
audit-failed = Failed
audit-from = From date
audit-to = To date
audit-newer = Newer
audit-older = Older
audit-channel = Channel
audit-agent = Browser
audit-no-changes = No field changes recorded for this entry.
audit-field = Field
audit-before = Before
audit-after = After

## sidebar (§8)
nav-access-control = Access Control
nav-system-settings = System Setting
nav-roles = Role
nav-permissions = Permission
nav-backup = Backup
profile-desc = Your account, password and appearance.
profile-card-title = Profile
profile-card-desc = Your name as other users see it.
profile-email-note = Ask an administrator to change your email address.
profile-saved = Profile saved.
profile-appearance-desc = Theme and font for this browser.
