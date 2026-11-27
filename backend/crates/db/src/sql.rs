//! Text that goes inside a LIKE or ILIKE pattern declared with `ESCAPE '\'`.

/// `s` with `\`, `%` and `_` escaped by a backslash
#[must_use]
pub fn escape_like(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        if matches!(c, '\\' | '%' | '_') {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

/// `%<escaped s>%`
#[must_use]
pub fn contains_pattern(s: &str) -> String {
    format!("%{}%", escape_like(s))
}
