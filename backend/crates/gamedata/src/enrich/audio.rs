use std::sync::LazyLock;

use regex::Regex;

static CHAR_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"char_\d+_[a-z0-9]+").expect("valid regex"));
static SK_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?:skchr|tachr)_([a-z0-9]+)_(\d+)").expect("valid regex"));

fn codename(id: &str) -> Option<&str> {
    id.splitn(3, '_').nth(2)
}
