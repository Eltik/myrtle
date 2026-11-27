use myrtle_db::sql::{contains_pattern, escape_like};

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
    assert_eq!(contains_pattern(""), "%%");
}
