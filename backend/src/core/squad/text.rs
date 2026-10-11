//! Rich-text helpers for game descriptions.
//!
//! Descriptions carry tags such as `<@ba.kw>Arts damage</>`,
//! `<$ba.stun>Stun</>` and `{atk_scale:0%}` placeholders. The load-time text
//! rules (`index`) match words on the plain text; the analysis keeps the raw
//! text for the page, which renders the tags itself.

use std::sync::LazyLock;

use regex::Regex;

/// Opening tags (`<@...>`, `<$...>`) and the closing `</>`.
static TAG: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<[@$][^>]*>|</>").expect("tag regex"));

/// The text with every rich-text tag removed and the words inside kept.
pub fn plain(text: &str) -> String {
    TAG.replace_all(text, "").into_owned()
}

/// Lowercase plain text, the form every word rule matches against.
pub fn lower_plain(text: &str) -> String {
    plain(text).to_lowercase()
}

/// Whether `text` carries the rich-text tag `tag` (`ba.invisible`), in either
/// the `<@...>` keyword or the `<$...>` glossary form.
pub fn has_tag(text: &str, tag: &str) -> bool {
    text.contains(&format!("<${tag}>")) || text.contains(&format!("<@{tag}>"))
}

/// Whether the lowercase `hay` contains any of `needles` (already lowercase).
pub fn any_of(hay: &str, needles: &[&str]) -> bool {
    needles.iter().any(|n| hay.contains(n))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_tags_and_keeps_words() {
        assert_eq!(
            plain("Deals <@ba.kw>Arts damage</> and <$ba.stun>Stuns</>"),
            "Deals Arts damage and Stuns"
        );
        assert!(has_tag(
            "Reveals <$ba.invisible>Invisible</>",
            "ba.invisible"
        ));
        assert!(!has_tag("Invisible", "ba.invisible"));
    }
}
