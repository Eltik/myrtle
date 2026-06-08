/// Formats an integer with thousands separators (e.g. 12345 -> "12,345")
#[must_use]
pub fn commafy(n: u64) -> String {
    let s = n.to_string();
    let mut out = String::with_capacity(s.len() + s.len() / 3);
    for (i, ch) in s.chars().rev().enumerate() {
        if i != 0 && i % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    out.chars().rev().collect()
}

/// Percentage of `part` out of `whole`, to one decimal place (guards div-by-zero)
#[must_use]
pub fn pct(part: u64, whole: u64) -> String {
    if whole == 0 {
        return "0.0".to_owned();
    }
    format!("{:.1}", part as f64 / whole as f64 * 100.0)
}
