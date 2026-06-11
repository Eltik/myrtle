/// Formats an integer with thousands separators (e.g. 12345 -> "12,345")
#[must_use]
pub fn commafy(n: i32) -> String {
    let s = n.unsigned_abs().to_string();
    let mut out = String::with_capacity(s.len() + s.len() / 3 + 1);
    for (i, ch) in s.chars().rev().enumerate() {
        if i != 0 && i % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    if n < 0 {
        out.push('-');
    }
    out.chars().rev().collect()
}

/// Percentage of `part` out of `whole`, to one decimal place (guards div-by-zero)
#[must_use]
pub fn pct(part: i32, whole: i32) -> String {
    if whole == 0 {
        return "0.0".to_owned();
    }
    format!("{:.1}", f64::from(part) / f64::from(whole) * 100.0)
}
