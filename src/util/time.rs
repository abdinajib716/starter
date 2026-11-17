//! Dates and times for display (M3.11): stored in UTC, shown in the viewer's time zone —
//! the user's own (`users.timezone`) or the organization default (`app.timezone`, System
//! Setting → General). The IANA database is bundled (`jiff`), so the server render and the
//! browser give the same text.
use jiff::{Timestamp, tz::TimeZone};
use time::OffsetDateTime;

/// Is `name` a known IANA zone (`Africa/Mogadishu`, `UTC`)?
#[cfg_attr(not(feature = "server"), allow(dead_code))]
pub fn valid_zone(name: &str) -> bool {
    TimeZone::get(name).is_ok()
}

fn zoned(t: OffsetDateTime, tz: &str) -> Option<jiff::Zoned> {
    let ts = Timestamp::from_second(t.unix_timestamp()).ok()?;
    let zone = TimeZone::get(tz).unwrap_or(TimeZone::UTC);
    Some(ts.to_zoned(zone))
}

/// "Nov 17, 2026, 12:05" in `tz`.
pub fn datetime(t: OffsetDateTime, tz: &str) -> String {
    zoned(t, tz)
        .map(|z| z.strftime("%b %-d, %Y, %H:%M").to_string())
        .unwrap_or_default()
}

/// "Nov 17, 2026, 12:05:09" in `tz` (audit log).
pub fn datetime_secs(t: OffsetDateTime, tz: &str) -> String {
    zoned(t, tz)
        .map(|z| z.strftime("%b %-d, %Y, %H:%M:%S").to_string())
        .unwrap_or_default()
}

/// "Nov 17, 2026" in `tz`.
pub fn date(t: OffsetDateTime, tz: &str) -> String {
    zoned(t, tz)
        .map(|z| z.strftime("%b %-d, %Y").to_string())
        .unwrap_or_default()
}

/// The zone's current abbreviation or offset for column headers: "EAT", "UTC", "+03".
pub fn zone_label(tz: &str) -> String {
    let zone = TimeZone::get(tz).unwrap_or(TimeZone::UTC);
    Timestamp::now_or_epoch()
        .to_zoned(zone)
        .strftime("%Z")
        .to_string()
}

trait NowOrEpoch {
    fn now_or_epoch() -> Timestamp;
}
impl NowOrEpoch for Timestamp {
    /// `Timestamp::now()` needs a clock (unavailable in some wasm setups); the label only
    /// depends on DST, so the current time from `time` is enough.
    fn now_or_epoch() -> Timestamp {
        Timestamp::from_second(crate::util::time::unix_now()).unwrap_or(Timestamp::UNIX_EPOCH)
    }
}

pub fn unix_now() -> i64 {
    #[cfg(target_arch = "wasm32")]
    {
        (js_sys::Date::now() / 1000.0) as i64
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        OffsetDateTime::now_utc().unix_timestamp()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_to_the_zone() {
        let t = time::macros::datetime!(2026-11-17 09:05:09 UTC);
        assert_eq!(datetime(t, "UTC"), "Nov 17, 2026, 09:05");
        assert_eq!(datetime(t, "Africa/Mogadishu"), "Nov 17, 2026, 12:05");
        assert_eq!(
            datetime_secs(t, "America/New_York"),
            "Nov 17, 2026, 04:05:09"
        );
        assert_eq!(
            date(time::macros::datetime!(2026-11-17 23:30 UTC), "Asia/Tokyo"),
            "Nov 18, 2026"
        );
        assert_eq!(
            datetime(t, "Not/AZone"),
            "Nov 17, 2026, 09:05",
            "unknown zone falls back to UTC"
        );
        assert!(valid_zone("Africa/Mogadishu") && valid_zone("UTC") && !valid_zone("drop table"));
    }
}
