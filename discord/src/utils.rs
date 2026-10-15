/// Formats an integer with thousands separators (e.g. 12345 -> "12,345")
#[must_use]
pub fn commafy(n: i32) -> String {
    let grouped = group_thousands(u64::from(n.unsigned_abs()));
    if n < 0 {
        format!("-{grouped}")
    } else {
        grouped
    }
}

/// `n` with thousands separators: 1234567 -> "1,234,567".
#[must_use]
pub fn group_thousands(n: u64) -> String {
    let digits = n.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

/// Percentage of `part` out of `whole`, to one decimal place (guards div-by-zero)
#[must_use]
pub fn pct(part: i32, whole: i32) -> String {
    if whole == 0 {
        return "0.0".to_owned();
    }
    format!("{:.1}", f64::from(part) / f64::from(whole) * 100.0)
}

/// `s` trimmed, or `None` when that leaves nothing.
#[must_use]
pub fn non_blank(s: Option<&str>) -> Option<&str> {
    s.map(str::trim).filter(|s| !s.is_empty())
}

/// Cut `s` to at most `max` characters, ending in "…" when anything was dropped.
///
/// Counts chars, not bytes: Discord's embed limits are in characters, and a byte cut could
/// split a UTF-8 sequence.
#[must_use]
pub fn ellipsize(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    let kept: String = s.chars().take(max.saturating_sub(1)).collect();
    format!("{}…", kept.trim_end())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ellipsize_counts_chars() {
        assert_eq!(ellipsize("short", 10), "short");
        assert_eq!(ellipsize("Wiš'adel", 8), "Wiš'adel");
        assert_eq!(ellipsize("Wiš'adel", 5), "Wiš'…");
        assert_eq!(ellipsize("one two", 5), "one…");
        assert_eq!(ellipsize("abc", 0), "…");
    }

    #[test]
    fn commafy_groups_thousands() {
        assert_eq!(commafy(12345), "12,345");
        assert_eq!(commafy(-1000), "-1,000");
    }
}
