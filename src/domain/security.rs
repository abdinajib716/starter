//! Sessions and two-factor DTOs (§21, §22).
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

/// One signed-in device (own Sessions list, admin view of a user).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionRow {
    pub id: Uuid,
    /// "Chrome on Windows" (from the User-Agent).
    pub device: String,
    pub ip: Option<String>,
    pub created_at: OffsetDateTime,
    pub last_seen_at: OffsetDateTime,
    /// The session this request came from.
    pub current: bool,
    pub mfa: bool,
}

/// What the UI needs to know about the signed-in user's second factor.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct TwoFaInfo {
    pub enrolled: bool,
    /// The 2FA mode makes it mandatory for this user.
    pub required: bool,
    /// Required, not enrolled: enrol before this moment (banner), after it everything but
    /// the Security tab is blocked.
    pub deadline: Option<OffsetDateTime>,
    pub blocked: bool,
}

/// Security tab state.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct TwoFaStatus {
    pub info: TwoFaInfo,
    /// The organization allows 2FA at all (`security.2fa_mode != off`).
    pub available: bool,
    pub recovery_left: i64,
}

/// Enrolment step 1: scan the QR or type the key, then confirm with a code.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnrollStart {
    pub secret: String,
    pub qr_svg: String,
}

/// Parse a User-Agent into "Browser on OS" (good enough for a sessions list).
#[cfg_attr(not(feature = "server"), allow(dead_code))]
pub fn device_label(ua: &str) -> String {
    let has = |s: &str| ua.contains(s);
    let browser = if has("Edg/") {
        "Edge"
    } else if has("OPR/") || has("Opera") {
        "Opera"
    } else if has("Firefox/") {
        "Firefox"
    } else if has("Chrome/") || has("CriOS/") {
        "Chrome"
    } else if has("Safari/") {
        "Safari"
    } else if has("curl/") {
        "curl"
    } else {
        "Browser"
    };
    let os = if has("iPhone") || has("iPad") {
        "iOS"
    } else if has("Android") {
        "Android"
    } else if has("Windows") {
        "Windows"
    } else if has("Mac OS X") || has("Macintosh") {
        "macOS"
    } else if has("CrOS") {
        "ChromeOS"
    } else if has("Linux") {
        "Linux"
    } else {
        "unknown OS"
    };
    format!("{browser} on {os}")
}

#[cfg(test)]
mod tests {
    #[test]
    fn device_labels() {
        use super::device_label as d;
        assert_eq!(
            d(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0 Safari/537.36"
            ),
            "Chrome on Windows"
        );
        assert_eq!(
            d(
                "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 Version/17.0 Mobile/15E148 Safari/604.1"
            ),
            "Safari on iOS"
        );
        assert_eq!(
            d("Mozilla/5.0 (X11; Linux x86_64; rv:131.0) Gecko/20100101 Firefox/131.0"),
            "Firefox on Linux"
        );
        assert_eq!(
            d(
                "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 Chrome/130 Safari/537.36 Edg/130"
            ),
            "Edge on macOS"
        );
    }
}
