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

/// Validate that a URL uses an allowed scheme (https, or http for localhost/127.0.0.1 in dev).
/// Returns the validated URL or an error.
pub fn validate_url(url: &str, context: &str) -> Result<String> {
    let trimmed = url.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        return Err(Error::Config(format!("{}: URL must not be empty", context)));
    }

    // Parse scheme
    let lower = trimmed.to_lowercase();
    if lower.starts_with("https://") {
        return Ok(trimmed.to_string());
    }

    // Allow http:// only for localhost / 127.0.0.1 (local dev servers)
    if let Some(host_part) = lower.strip_prefix("http://") {
        let host = host_part.split('/').next().unwrap_or("");
        let host_no_port = host.split(':').next().unwrap_or("");
        if host_no_port == "localhost" || host_no_port == "127.0.0.1" || host_no_port == "[::1]" {
            return Ok(trimmed.to_string());
        }
        return Err(Error::Config(format!(
            "{}: HTTP URLs are only allowed for localhost. Use HTTPS for remote servers: \"{}\"",
            context, trimmed
        )));
    }

    Err(Error::Config(format!(
        "{}: URL must use HTTPS (got \"{}\")",
        context, trimmed
    )))
}

/// Validate that a file path stays within a given base directory (no path traversal).
/// Returns the canonicalized path on success.
pub fn validate_path_within(
    base: &std::path::Path,
    target: &std::path::Path,
    context: &str,
) -> Result<std::path::PathBuf> {
    // Check for obvious traversal patterns in the raw path
    let target_str = target.to_string_lossy();
    if target_str.contains("..") {
        return Err(Error::Config(format!(
            "{}: path traversal not allowed: \"{}\"",
            context, target_str
        )));
    }

    // Resolve the path and verify containment
    // Use the canonical base if available, otherwise normalize manually
    let resolved_base = base.canonicalize().unwrap_or_else(|_| base.to_path_buf());
    let resolved = if target.exists() {
        target
            .canonicalize()
            .unwrap_or_else(|_| base.join(target.file_name().unwrap_or_default()))
    } else {
        // For non-existent files, just join and normalize
        resolved_base.join(
            target.strip_prefix(&resolved_base).unwrap_or(
                target
                    .file_name()
                    .map(std::path::Path::new)
                    .unwrap_or(target),
            ),
        )
    };

    if !resolved.starts_with(&resolved_base) {
        return Err(Error::Config(format!(
            "{}: path escapes allowed directory: \"{}\"",
            context, target_str
        )));
    }

    Ok(resolved)
}

/// Write data to a file atomically using a temp file + rename.
/// On failure, falls back to a direct write.
pub fn atomic_write(
    path: &std::path::Path,
    data: &[u8],
) -> std::result::Result<(), std::io::Error> {
    let parent = path.parent().unwrap_or(std::path::Path::new("."));
    let _ = std::fs::create_dir_all(parent);

    // Write to a temp file in the same directory, then rename
    let tmp_name = format!(
        ".{}.tmp.{}",
        path.file_name().unwrap_or_default().to_string_lossy(),
        std::process::id()
    );
    let tmp_path = parent.join(&tmp_name);

    std::fs::write(&tmp_path, data)?;

    // Rename is atomic on most filesystems
    if std::fs::rename(&tmp_path, path).is_err() {
        // Fallback: on Windows, rename can fail if target exists
        let _ = std::fs::remove_file(path);
        if let Err(_e) = std::fs::rename(&tmp_path, path) {
            // Last resort: clean up temp and do direct write
            let _ = std::fs::remove_file(&tmp_path);
            return std::fs::write(path, data);
        }
    }

    Ok(())
}
