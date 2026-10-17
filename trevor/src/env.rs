//! Environment variables, read one way each. Every `TREVOR_*` switch and setting goes through one of these
//! readers, so the rule a value follows is the reader's name:
//!
//! - [`on_unless_0`]: a default-on kill switch, off only for exactly `0`.
//! - [`is`] / [`is_0`]: the value is exactly this string (an opt-in `1`, or a named mode).
//! - [`debug_on`]: a debug or opt-in switch, on for `1` or `true` (any case).
//! - [`var`], [`non_empty`]: the raw value, or the value when it is not empty.
//! - [`parse_or`]: a number, or the default when unset or unparsable.
//!
//! A variable that is set but not valid Unicode reads as unset in every reader.

use std::str::FromStr;

/// The raw value of `key`, or `None` when it is unset (or not Unicode).
#[must_use]
pub fn var(key: &str) -> Option<String> {
    std::env::var(key).ok()
}

/// The value of `key` when it is set and not empty.
#[must_use]
pub fn non_empty(key: &str) -> Option<String> {
    var(key).filter(|v| !v.is_empty())
}

/// Whether `key` is set to exactly `value`.
#[must_use]
pub fn is(key: &str, value: &str) -> bool {
    std::env::var(key).as_deref() == Ok(value)
}

/// Whether `key` is set to exactly `0`: the off value of a default-on switch.
#[must_use]
pub fn is_0(key: &str) -> bool {
    is(key, "0")
}

/// A default-on kill switch: on unless `key` is exactly `0`.
#[must_use]
pub fn on_unless_0(key: &str) -> bool {
    !is_0(key)
}

/// `key` parsed as a `T`, or `default` when it is unset or does not parse.
#[must_use]
pub fn parse_or<T: FromStr>(key: &str, default: T) -> T {
    var(key).and_then(|v| v.parse().ok()).unwrap_or(default)
}

/// Whether a debug or opt-in environment switch is on: `1` or `true` (any case). Unset, empty, `0`, `false` and
/// every other value are off, so `KEY=0` turns a switch off instead of on.
#[must_use]
pub fn debug_on(key: &str) -> bool {
    switch_on(var(key).as_deref())
}

/// The rule of [`debug_on`] for one value (`None` = unset).
#[must_use]
pub fn switch_on(value: Option<&str>) -> bool {
    value.is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_switch_is_on_only_for_1_or_true() {
        for on in ["1", "true", "TRUE", "True"] {
            assert!(switch_on(Some(on)), "{on}");
        }
        for off in [None, Some(""), Some("0"), Some("false"), Some("no"), Some("yes"), Some(" 1"), Some("2")] {
            assert!(!switch_on(off), "{off:?}");
        }
    }
}
