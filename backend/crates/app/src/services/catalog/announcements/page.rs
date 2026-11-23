use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

use ammonia::{Builder, Url, UrlRelative};
use regex::Regex;

use super::banner_url;
use crate::services::catalog::maintenance::parse::head_title;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageParts {
    pub headline: Option<String>,
    pub banner_url: Option<String>,
    pub body_html: String,
}

static RE_CONTENT_OPEN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?is)<div\s+class="content"\s*>"#).expect("static regex is valid")
});
static RE_BODY_END: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?is)<script\b|</body>").expect("static regex is valid"));
static RE_STYLED_SPAN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?is)<span\s+style\s*=\s*"([^"]*)"\s*>"#).expect("static regex is valid")
});
static RE_COLOR: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(?:^|;)\s*color\s*:\s*#([0-9a-f]{6}|[0-9a-f]{3})\b")
        .expect("static regex is valid")
});
static RE_EMPTY_P: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<p>\s*</p>").expect("static regex is valid"));

const GREY_SPREAD: u8 = 40;

#[must_use]
pub fn is_coloured(hex: &str) -> bool {
    let full: String = if hex.len() == 3 {
        hex.chars().flat_map(|c| [c, c]).collect()
    } else {
        hex.to_owned()
    };
    let channel = |i: usize| {
        full.get(i..i + 2)
            .and_then(|c| u8::from_str_radix(c, 16).ok())
            .unwrap_or(0)
    };
    let (r, g, b) = (channel(0), channel(2), channel(4));
    r.max(g).max(b) - r.min(g).min(b) > GREY_SPREAD
}

fn mark_accents(html: &str) -> String {
    RE_STYLED_SPAN
        .replace_all(html, |caps: &regex::Captures<'_>| {
            let coloured = RE_COLOR
                .captures(&caps[1])
                .is_some_and(|c| is_coloured(&c[1]));
            if coloured {
                r#"<span class="accent">"#.to_owned()
            } else {
                "<span>".to_owned()
            }
        })
        .into_owned()
}

fn sanitizer(base: Option<Url>) -> Builder<'static> {
    let tags: HashSet<&str> = [
        "p",
        "br",
        "hr",
        "strong",
        "b",
        "em",
        "i",
        "u",
        "s",
        "del",
        "sup",
        "sub",
        "span",
        "div",
        "a",
        "img",
        "ul",
        "ol",
        "li",
        "h1",
        "h2",
        "h3",
        "h4",
        "h5",
        "h6",
        "blockquote",
        "table",
        "thead",
        "tbody",
        "tfoot",
        "tr",
        "th",
        "td",
        "caption",
        "figure",
        "figcaption",
    ]
    .into_iter()
    .collect();
    let tag_attributes: HashMap<&str, HashSet<&str>> = HashMap::from([
        ("a", HashSet::from(["href"])),
        ("img", HashSet::from(["src", "alt", "width", "height"])),
        ("td", HashSet::from(["colspan", "rowspan"])),
        ("th", HashSet::from(["colspan", "rowspan"])),
    ]);
    let classes: HashMap<&str, HashSet<&str>> =
        HashMap::from([("span", HashSet::from(["accent"]))]);
    let mut b = Builder::empty();
    b.tags(tags)
        .tag_attributes(tag_attributes)
        .allowed_classes(classes)
        .url_schemes(HashSet::from(["http", "https"]))
        .link_rel(Some("noopener noreferrer"))
        .clean_content_tags(HashSet::from(["script", "style"]))
        .strip_comments(true)
        .set_tag_attribute_value("img", "loading", "lazy"); // one only: ammonia emits these in `HashMap` order
    b.url_relative(base.map_or(UrlRelative::Deny, UrlRelative::RewriteWithBase));
    b
}

#[must_use]
pub fn extract(html: &str, page_url: &str) -> PageParts {
    let base = Url::parse(page_url).ok();
    let headline = head_title(html).filter(|t| !t.is_empty());
    let banner_url = banner_url(html, page_url);
    let body_html = RE_CONTENT_OPEN.find(html).map_or_else(String::new, |open| {
        let rest = &html[open.end()..];
        let end = RE_BODY_END.find(rest).map_or(rest.len(), |m| m.start());
        let cleaned = sanitizer(base)
            .clean(&mark_accents(&rest[..end]))
            .to_string();
        RE_EMPTY_P.replace_all(&cleaned, "").trim().to_owned()
    });
    PageParts {
        headline,
        banner_url,
        body_html,
    }
}
