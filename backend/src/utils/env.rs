//! Kill switches read from the environment.

/// Whether the kill switch `name` is set to `0`, surrounding whitespace
/// ignored. Unset, or any other value, leaves the feature on, so a deploy that
/// never heard of the switch keeps the default.
pub fn switched_off(name: &str) -> bool {
    std::env::var(name).is_ok_and(|v| v.trim() == "0")
}
