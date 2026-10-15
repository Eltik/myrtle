//! One Yostar notice page, cut into the parts the site shows: the headline,
//! the banner, and the body run through an allowlist sanitizer.
//!
//! Every page fetched on 2026-10-11 (EN 1540 to 1640, JP 1427, KR 1391) has
//! the same skeleton: an optional `<img class="banner-image">` (inside an
//! `<a href="uniwebview://...">` on a cover-only notice like EN 1608), a
//! `<span class="head-title">`, then `<div class="content">` and a sizing
//! `<script>`. The body uses `p`, `br`, `strong`, `span style="color:..."`,
//! `div.media-wrap > img` and `hr`, and nothing else in that sample.
//!
//! The sanitizer is `ammonia` (html5ever underneath): an allowlist of tags and
//! attributes, `http`/`https` URLs only (so the in-game `uniwebview://` jumps
//! are dropped), relative URLs rewritten against the page's own URL, and
//! every link marked `noopener noreferrer`. Inline styles are dropped: the
//! pages paint text `#000000` for a white page, which disappears on a dark
//! one. A span the notice coloured (not grey) keeps the emphasis as
//! `class="accent"`, which the site paints in its own accent colour.

use std::{
    collections::{HashMap, HashSet},
    sync::LazyLock,
};

use ammonia::{Builder, Url, UrlRelative};
use regex::Regex;

use crate::core::maintenance::parse::head_title;

/// What one notice page holds, ready to serve.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageParts {
    /// The page's own headline, whitespace collapsed; `None` on a cover-only
    /// notice that has none.
    pub headline: Option<String>,
    /// Absolute URL of the banner image, if the page has one.
    pub banner_url: Option<String>,
    /// The sanitized body; empty on a cover-only notice.
    pub body_html: String,
}

static RE_IMG: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?is)<img\b[^>]*>").expect("img regex compiles"));
static RE_SRC: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"(?is)\bsrc\s*=\s*"([^"]*)""#).expect("src regex compiles"));
static RE_CONTENT_OPEN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?is)<div\s+class="content"\s*>"#).expect("content regex compiles")
});
static RE_BODY_END: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?is)<script\b|</body>").expect("body end regex compiles"));
static RE_STYLED_SPAN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?is)<span\s+style\s*=\s*"([^"]*)"\s*>"#).expect("span regex compiles")
});
static RE_COLOR: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(?:^|;)\s*color\s*:\s*#([0-9a-f]{6}|[0-9a-f]{3})\b")
        .expect("color regex compiles")
});
static RE_EMPTY_P: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<p>\s*</p>").expect("empty p regex compiles"));

/// Channels further apart than this make a colour, not a grey. `#333333` and
/// `#7e8c8d` (the notices' greys) stay plain; `#c0392b`, `#d35400`, `#003ba5`
/// (their reds, orange and blue) become accents.
const GREY_SPREAD: u8 = 40;

fn is_coloured(hex: &str) -> bool {
    let full: String = if hex.len() == 3 {
        hex.chars().flat_map(|c| [c, c]).collect()
    } else {
        hex.to_owned()
    };
    let channel = |i: usize| u8::from_str_radix(&full[i..i + 2], 16).unwrap_or(0);
    let (r, g, b) = (channel(0), channel(2), channel(4));
    r.max(g).max(b) - r.min(g).min(b) > GREY_SPREAD
}

/// `<span style="color:#c0392b">` to `<span class="accent">`; every other
/// styled span loses its style.
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
        // Most notices are a stack of full-width images; only the first is on
        // screen when the page opens.
        // One attribute only: ammonia keeps these in a `HashMap`, so two would
        // come out in a random order and move the `ETag` from run to run.
        .set_tag_attribute_value("img", "loading", "lazy");
    b.url_relative(base.map_or(UrlRelative::Deny, UrlRelative::RewriteWithBase));
    b
}

/// The page's banner image, made absolute against `base`.
fn banner(html: &str, base: Option<&Url>) -> Option<String> {
    RE_IMG
        .find_iter(html)
        .map(|m| m.as_str())
        .find(|tag| tag.contains("banner-image"))
        .and_then(|tag| RE_SRC.captures(tag))
        .and_then(|c| absolute(&c[1].replace("&amp;", "&"), base))
}

fn absolute(src: &str, base: Option<&Url>) -> Option<String> {
    let url = match Url::parse(src) {
        Ok(u) => u,
        Err(_) => base?.join(src).ok()?,
    };
    matches!(url.scheme(), "http" | "https").then(|| url.to_string())
}

/// Cut `html`, fetched from `page_url`, into its parts.
pub fn extract(html: &str, page_url: &str) -> PageParts {
    let base = Url::parse(page_url).ok();
    let headline = head_title(html).filter(|t| !t.is_empty());
    let banner_url = banner(html, base.as_ref());
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

#[cfg(test)]
mod tests {
    use super::*;

    const EN_URL: &str = "https://ark-us-static-online.yo-star.com/announce/Android/announcement/1627.html?t=1791543600";

    #[test]
    fn en_event_page_keeps_text_images_and_accents() {
        let p = extract(include_str!("../maintenance/fixtures/1627.html"), EN_URL);
        assert_eq!(
            p.headline.as_deref(),
            Some("RECOMMENDED MONTHLY CARD AVAILABLE")
        );
        assert_eq!(p.banner_url, None);
        assert_eq!(p.body_html.matches("<img ").count(), 2);
        assert!(p.body_html.contains(
            r#"<img src="https://ark-us-static-online.yo-star.com/announce/images/20260927/e0ae4f38c73fe4570bfab5b30224c0fa.jpg" loading="lazy">"#
        ));
        // `#c0392b` twice is an accent; `#000000` five times is not.
        assert_eq!(p.body_html.matches(r#"class="accent""#).count(), 2);
        assert!(!p.body_html.contains("style="));
        assert!(!p.body_html.contains("<script"));
        assert!(!p.body_html.contains("media-wrap"));
        assert!(p.body_html.contains("<strong>DURATION: </strong>"));
    }

    #[test]
    fn cover_only_page_is_a_banner_with_no_body() {
        let url =
            "https://ark-us-static-online.yo-star.com/announce/Android/announcement/1608.html";
        let p = extract(include_str!("../maintenance/fixtures/1608.html"), url);
        assert_eq!(p.headline, None);
        assert_eq!(p.body_html, "");
        assert_eq!(
            p.banner_url.as_deref(),
            Some(
                "https://ark-us-static-online.yo-star.com/announce/images/20260804/2712dbbcbd6206c2f3cc6ad94de47668.jpg"
            )
        );
    }

    #[test]
    fn jp_page_with_banner_and_rules() {
        let url =
            "https://ark-jp-static-online.yo-star.com/announce/Android/announcement/1427.html";
        let p = extract(include_str!("../maintenance/fixtures/jp_1427.html"), url);
        assert_eq!(
            p.headline.as_deref(),
            Some("メインテーマ 第十七章「相変臨界」公開")
        );
        assert!(
            p.banner_url
                .as_deref()
                .is_some_and(|u| u.ends_with("/20260923/0d4ade920ccb4638dd7a4db8debb365b.png"))
        );
        assert!(p.body_html.contains("<hr>"));
        assert!(p.body_html.contains("2026年10月14日"));
        assert!(p.body_html.contains(r#"<span class="accent">"#));
    }

    #[test]
    fn kr_page_is_one_image() {
        let url = "https://ark-kr-static-online-1300509597.yo-star.com/announce/Android/announcement/1391.html";
        let p = extract(include_str!("../maintenance/fixtures/kr_1391.html"), url);
        assert_eq!(p.headline.as_deref(), Some("'임계 상전이' 이벤트 안내"));
        assert_eq!(p.body_html.matches("<img ").count(), 1);
        assert!(p.banner_url.is_some());
    }

    #[test]
    fn hostile_markup_is_stripped_and_relative_urls_made_absolute() {
        let html = r#"<div class="content"><p onclick="x()">hi<img src="../../images/a.png" onerror="x()"></p>
            <a href="javascript:alert(1)">bad</a><a href="uniwebview://move?target=shop">jump</a>
            <a href="/news/1">rel</a><iframe src="https://evil.invalid"></iframe><style>p{}</style>
            <span style="color:#333333">grey</span><span style="font-size:1.25rem">big</span></div><script>evil()</script>"#;
        let p = extract(html, EN_URL);
        let b = &p.body_html;
        assert!(!b.contains("onclick") && !b.contains("onerror"), "{b}");
        assert!(
            !b.contains("javascript") && !b.contains("uniwebview"),
            "{b}"
        );
        assert!(
            !b.contains("iframe") && !b.contains("evil") && !b.contains("p{}"),
            "{b}"
        );
        assert!(
            b.contains(
                r#"<img src="https://ark-us-static-online.yo-star.com/announce/images/a.png" loading="lazy">"#
            ),
            "{b}"
        );
        assert!(b.contains(
            r#"<a href="https://ark-us-static-online.yo-star.com/news/1" rel="noopener noreferrer">rel</a>"#
        ), "{b}");
        assert!(b.contains("<span>grey</span><span>big</span>"), "{b}");
    }

    #[test]
    fn grey_and_colour() {
        assert!(!is_coloured("000000"));
        assert!(!is_coloured("7e8c8d"));
        assert!(!is_coloured("fff"));
        assert!(is_coloured("c0392b"));
        assert!(is_coloured("003ba5"));
        assert!(is_coloured("d35400"));
    }
}
