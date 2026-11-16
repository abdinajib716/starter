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
coming-soon-title = Coming Soon!
coming-soon-desc-1 = This page has not been created yet.
coming-soon-desc-2 = Stay tuned though!

# ---- Settings (shadcn-admin demo forms) ----
settings-desc = Manage your account settings and set e-mail preferences.
settings-profile-desc = This is how others will see you on the site.
settings-account-desc = Update your account settings. Set your preferred language and timezone.
settings-appearance-desc = Customize the appearance of the app. Automatically switch between day and night themes.
settings-notifications-desc = Configure how you receive notifications.
settings-display-desc = Turn items on or off to control what's displayed in the app.
submitted-title = You submitted the following values:

f-username = Username
f-username-desc = This is your public display name. It can be your real name or a pseudonym. You can only change this once every 30 days.
f-email-placeholder = Select a verified email to display
f-email-desc-1 = You can manage verified email addresses in your
f-email-desc-link = email settings
f-bio = Bio
f-bio-placeholder = Tell us a little bit about yourself
f-bio-desc-1 = You can
f-bio-desc-2 = other users and organizations to link to them.
f-urls = URLs
f-urls-desc = Add links to your website, blog, or social media profiles.
f-add-url = Add URL
f-update-profile = Update profile

f-name = Name
f-name-placeholder = Your name
f-name-desc = This is the name that will be displayed on your profile and in emails.
f-dob = Date of birth
f-dob-placeholder = Pick a date
f-dob-desc = Your date of birth is used to calculate your age.
f-language = Language
f-language-placeholder = Select language
f-language-search = Search language...
f-language-empty = No language found.
f-language-desc = This is the language that will be used in the dashboard.
f-update-account = Update account

f-font = Font
f-font-desc = Set the font you want to use in the dashboard.
f-theme-desc = Select the theme for the dashboard.
f-update-preferences = Update preferences

f-notify-about = Notify me about...
f-notify-all = All new messages
f-notify-mentions = Direct messages and mentions
f-notify-none = Nothing
f-email-notifications = Email Notifications
f-comm-emails = Communication emails
f-comm-emails-desc = Receive emails about your account activity.
f-marketing-emails = Marketing emails
f-marketing-emails-desc = Receive emails about new products, features, and more.
f-social-emails = Social emails
f-social-emails-desc = Receive emails for friend requests, follows, and more.
f-security-emails = Security emails
f-security-emails-desc = Receive emails about your account activity and security.
f-mobile = Use different settings for my mobile devices
f-mobile-desc-1 = You can manage your mobile notifications in the
f-mobile-desc-link = mobile settings
f-mobile-desc-2 = page.
f-update-notifications = Update notifications

f-display-sidebar = Sidebar
f-display-sidebar-desc = Select the items you want to display in the sidebar.
f-update-display = Update display

v-username-required = Please enter your username.
v-username-min = Username must be at least 2 characters.
v-username-max = Username must not be longer than 30 characters.
v-email-select = Please select an email to display.
v-bio-min = Too small: expected string to have >=4 characters
v-bio-max = Too big: expected string to have <=160 characters
v-url = Please enter a valid URL.
v-name-required = Please enter your name.
v-name-min = Name must be at least 2 characters.
v-name-max = Name must not be longer than 30 characters.
v-dob = Please select your date of birth.
v-language = Please select a language.
v-notify-type = Please select a notification type.
v-display-items = You have to select at least one item.

# ---- Auth screens ----
auth-sign-in-desc-1 = Enter your email and password below to log into
auth-sign-in-desc-2 = your account. Don't have an account?
auth-forgot = Forgot password?
auth-continue-with = Or continue with
auth-terms-lead-sign-in = By clicking sign in, you agree to our
auth-terms-lead-sign-up = By creating an account, you agree to our
auth-terms = Terms of Service
auth-and = and
auth-privacy = Privacy Policy
auth-welcome = Welcome back, { $email }!
auth-signup-title = Create an account
auth-signup-desc = Enter your email and password to create an account.
auth-have-account = Already have an account?
auth-confirm-password = Confirm Password
auth-create-account = Create Account
auth-signup-disabled = Self sign-up is turned off. Ask an administrator to invite you.
auth-forgot-desc-1 = Enter your registered email and
auth-forgot-desc-2 = we will send you a link to reset your password.
auth-continue = Continue
auth-reset-unavailable = Password reset by email is not enabled yet. An administrator can reset your password.
auth-no-account = Don't have an account?
auth-sign-up-lower = Sign up
auth-otp-title = Two-factor Authentication
auth-otp-desc-1 = Please enter the authentication code.
auth-otp-desc-2 = We have sent the authentication code to your email.
auth-otp-label = One-Time Password
auth-verify = Verify
auth-otp-resend-lead = Haven't received it?
auth-otp-resend = Resend a new code.
pw-show = Show password
pw-hide = Hide password
v-email-enter = Please enter your email.
v-email-invalid = Invalid email address
v-password-enter = Please enter your password.
v-password-min7 = Password must be at least 7 characters long.
v-password-confirm = Please confirm your password.
v-password-match = Passwords don't match.
