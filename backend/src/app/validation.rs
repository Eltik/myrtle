use crate::app::error::ApiError;

/// Reject anything that isn't a 6-digit `#rrggbb` hex color. Mirrors the
/// frontend's `HEX_RE` so user-submitted values can't be smuggled into inline
/// CSS (e.g. `red url('https://attacker/pixel')`).
pub fn validate_hex_color(value: Option<&str>) -> Result<(), ApiError> {
    let Some(color) = value else { return Ok(()) };
    let bytes = color.as_bytes();
    let valid =
        bytes.len() == 7 && bytes[0] == b'#' && bytes[1..].iter().all(|b| b.is_ascii_hexdigit());
    if valid {
        Ok(())
    } else {
        Err(ApiError::BadRequest("invalid color".into()))
    }
}
