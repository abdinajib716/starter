# English — source of truth (§26). Keys: <area>-<name>.
app-name = Starter

home-title = Rust/UI is working
home-welcome = Welcome, { $name }
home-subtitle = Dioxus 0.7 + Tailwind v4 theme
home-clicks = { $count ->
    [one] Clicked once
   *[other] Clicked { $count } times
}
action-increment = Increment
action-reset = Reset
action-delete = Delete
toast-delete = Delete clicked (toast works)
status-loading = loading…
lang-label = Language

nav-logout = Log out
nav-profile = Profile

login-title = Sign in
login-subtitle = Use your work email and password.
login-submit = Sign in
login-submitting = Signing in…
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

notfound-title = Page not found
notfound-back = Back to home

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
teams-label = Teams
skip-to-main = Skip to Main
action-cancel = Cancel

nav-group-general = General
nav-group-pages = Pages
nav-group-other = Other
nav-dashboard = Dashboard
nav-tasks = Tasks
nav-apps = Apps
nav-chats = Chats
nav-users = Users
nav-auth = Auth
nav-sign-in = Sign In
nav-sign-in-2 = Sign In (2 Col)
nav-sign-up = Sign Up
nav-forgot-password = Forgot Password
nav-otp = OTP
nav-errors = Errors
nav-unauthorized = Unauthorized
nav-forbidden = Forbidden
nav-not-found = Not Found
nav-internal-error = Internal Server Error
nav-maintenance = Maintenance Error
nav-settings = Settings
nav-account = Account
nav-appearance = Appearance
nav-notifications = Notifications
nav-display = Display
nav-help-center = Help Center

menu-upgrade = Upgrade to Pro
menu-billing = Billing
menu-new-team = New Team
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
dash-download = Download
dash-nav-overview = Overview
dash-nav-customers = Customers
dash-nav-products = Products
dash-tab-overview = Overview
dash-tab-analytics = Analytics
dash-tab-reports = Reports
dash-tab-notifications = Notifications
dash-revenue = Total Revenue
dash-revenue-note = +20.1% from last month
dash-subscriptions = Subscriptions
dash-subscriptions-note = +180.1% from last month
dash-sales = Sales
dash-sales-note = +19% from last month
dash-active = Active Now
dash-active-note = +201 since last hour
dash-overview = Overview
dash-recent-sales = Recent Sales
dash-recent-sales-desc = You made { $count } sales this month.
dash-traffic = Traffic Overview
dash-traffic-desc = Weekly clicks and unique visitors
dash-clicks = Total Clicks
dash-clicks-note = +12.4% vs last week
dash-uniques = Unique Visitors
dash-uniques-note = +5.8% vs last week
dash-bounce = Bounce Rate
dash-bounce-note = -3.2% vs last week
dash-session = Avg. Session
dash-session-note = +18s vs last week
dash-referrers = Referrers
dash-referrers-desc = Top sources driving traffic
dash-devices = Devices
dash-devices-desc = How users access your app
dash-desktop = Desktop
dash-mobile = Mobile
dash-tablet = Tablet
