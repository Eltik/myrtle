use std::sync::LazyLock;

use regex::Regex;

static TAG: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<[@$][^>]*>|</>").expect("tag regex"));

pub fn plain(text: &str) -> String {
    TAG.replace_all(text, "").into_owned()
}

#[must_use]
pub fn lower_plain(text: &str) -> String {
    plain(text).to_lowercase()
}

#[must_use]
pub fn has_tag(text: &str, tag: &str) -> bool {
    text.contains(&format!("<${tag}>")) || text.contains(&format!("<@{tag}>"))
}

#[must_use]
pub fn any_of(hay: &str, needles: &[&str]) -> bool {
    needles.iter().any(|n| hay.contains(n))
}
