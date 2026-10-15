//! User text placed inside a LIKE or ILIKE pattern must not read `%` or `_`
//! as wildcards. Every caller pairs these patterns with `ESCAPE '\'`.

/// `s` with `\`, `%` and `_` escaped by a backslash, for use inside a LIKE or
/// ILIKE pattern that declares `ESCAPE '\'`.
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

/// A substring pattern: `%<escaped s>%`.
pub fn contains_pattern(s: &str) -> String {
    format!("%{}%", escape_like(s))
}

#[cfg(test)]
mod tests {
    use super::{contains_pattern, escape_like};

    #[test]
    fn like_wildcards_are_escaped() {
        assert_eq!(escape_like("123"), "123");
        assert_eq!(escape_like("1%_\\"), "1\\%\\_\\\\");
    }

    #[test]
    fn contains_pattern_wraps_the_escaped_text() {
        assert_eq!(contains_pattern("ab"), "%ab%");
        assert_eq!(contains_pattern("_"), r"%\_%");
        assert_eq!(contains_pattern("50%"), r"%50\%%");
        assert_eq!(contains_pattern(r"a\b"), r"%a\\b%");
        // Callers only search with Some(q); an empty q still matches everything.
        assert_eq!(contains_pattern(""), "%%");
    }
}
