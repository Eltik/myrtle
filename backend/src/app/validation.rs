use crate::app::error::ApiError;

pub const TIER_NAME_MAX: usize = 24;
pub const TIER_DESCRIPTION_MAX: usize = 1000;
pub const LIST_NAME_MAX: usize = 80;
pub const LIST_DESCRIPTION_MAX: usize = 4000;
pub const PLACEMENT_DESCRIPTION_MAX: usize = 1000;

/// Reject anything that isn't a 6-digit `#rrggbb` hex color. Mirrors the
/// frontend's `HEX_RE` so user-submitted values can't be smuggled into inline
/// CSS (e.g. `red url('https://attacker/pixel')`).
pub fn validate_hex_color(value: Option<&str>) -> Result<(), ApiError> {
    let Some(color) = value else { return Ok(()) };
    let bytes = color.as_bytes();
    let valid =
        bytes.len() == 7 && bytes[0] == b'#' && bytes[1..].iter().all(u8::is_ascii_hexdigit);
    if valid {
        Ok(())
    } else {
        Err(ApiError::BadRequest("invalid color".into()))
    }
}

pub fn validate_length(field: &str, value: &str, max: usize) -> Result<(), ApiError> {
    if value.chars().count() > max {
        return Err(ApiError::BadRequest(format!("{field} max {max} chars")));
    }
    Ok(())
}

pub fn validate_opt_length(field: &str, value: Option<&str>, max: usize) -> Result<(), ApiError> {
    match value {
        Some(v) => validate_length(field, v, max),
        None => Ok(()),
    }
}

/// Whether a movement/standing interval is one of the supported time windows.
pub fn is_valid_interval(interval: &str) -> bool {
    matches!(interval, "1 day" | "7 days" | "30 days")
}
