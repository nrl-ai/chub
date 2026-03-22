//! Shared utility functions used across the crate.

use crate::error::{Error, Result};

/// Convert days since Unix epoch to (year, month, day).
/// Algorithm from <http://howardhinnant.github.io/date_algorithms.html>.
pub fn days_to_date(days: u64) -> (u64, u64, u64) {
    let z = days + 719468;
    let era = z / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

/// Get current time as ISO 8601 string (e.g. `2026-03-21T14:30:00.000Z`).
pub fn now_iso8601() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let days = secs / 86400;
    let tod = secs % 86400;
    let (y, m, d) = days_to_date(days);
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.000Z",
        y,
        m,
        d,
        tod / 3600,
        (tod % 3600) / 60,
        tod % 60
    )
}

/// Get current date as ISO 8601 date string (e.g. `2026-03-21`).
pub fn today_date() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let days = secs / 86400;
    let (y, m, d) = days_to_date(days);
    format!("{:04}-{:02}-{:02}", y, m, d)
}

/// Sanitize an entry ID for use in filenames.
/// Replaces `/` and `\` with `--` and strips `..` to prevent path traversal.
pub fn sanitize_entry_id(entry_id: &str) -> String {
    entry_id.replace(['/', '\\'], "--").replace("..", "")
}

/// Validate that a name is safe for use as a filename (no path traversal).
pub fn validate_filename(name: &str, kind: &str) -> Result<()> {
    if name.is_empty()
        || name.contains('/')
        || name.contains('\\')
        || name.contains("..")
        || name.starts_with('.')
    {
        return Err(Error::Config(format!(
            "Invalid {} name \"{}\": must not contain path separators or \"..\"",
            kind, name
        )));
    }
    Ok(())
}
