//! A maintenance window read out of the prose of one Yostar notice page.

use std::sync::LazyLock;

use chrono::{DateTime, Datelike, Duration, FixedOffset, NaiveDate, TimeZone, Utc};
use regex::{Captures, Regex};
use serde::{Deserialize, Serialize};

/// One maintenance window as one notice states it
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NoticeWindow {
    pub starts_at: i64,
    pub ends_at: Option<i64>,
    /// The notice moves the end of a window already announced
    pub extended: bool,
    pub title: String,
}

const MONTHS: &str = "January|February|March|April|May|June|July|August|September|October|November|December|Jan|Feb|Mar|Apr|Jun|Jul|Aug|Sept|Sep|Oct|Nov|Dec";
const ZONE: &str = r"(?:UTC|GMT)\s*[+\-−]\s*\d{1,2}(?::?\d{2})?|UTC|GMT|PDT|PST|JST|KST";

fn regex(pattern: &str) -> Regex {
    Regex::new(pattern).expect("static regex is valid")
}

fn date(prefix: &str) -> String {
    format!(
        r"(?P<{prefix}mon>{MONTHS})\.?\s+(?P<{prefix}d>\d{{1,2}})(?:st|nd|rd|th)?,?\s*(?:(?P<{prefix}y>\d{{4}}),?\s*)?"
    )
}

fn clock(prefix: &str) -> String {
    format!(r"(?P<{prefix}h>\d{{1,2}}):(?P<{prefix}mi>\d{{2}})")
}

static RE_RANGE: LazyLock<Regex> = LazyLock::new(|| {
    regex(&format!(
        r"(?i){}{}\s*(?:-|\x{{2013}}|\x{{2014}}|~|to)\s*(?:{})?{}\s*\(?\s*(?P<tz>{ZONE})\s*\)?",
        date("s"),
        clock("s"),
        date("e"),
        clock("e"),
    ))
});
static RE_INSTANT: LazyLock<Regex> = LazyLock::new(|| {
    regex(&format!(
        r"(?i){}{}\s*\(?\s*(?P<tz>{ZONE})\s*\)?",
        date("s"),
        clock("s"),
    ))
});
static RE_EXTENDED: LazyLock<Regex> =
    LazyLock::new(|| regex(r"(?i)\b(?:extended|postponed|prolonged)\s+(?:to|until|till)\b"));
static RE_HEAD_TITLE: LazyLock<Regex> =
    LazyLock::new(|| regex(r#"(?is)class="head-title"[^>]*>(.*?)</"#));
static RE_CONTENT: LazyLock<Regex> =
    LazyLock::new(|| regex(r#"(?is)<div class="content">(.*)</div>"#));
static RE_BREAK: LazyLock<Regex> = LazyLock::new(|| regex(r"(?i)<br\s*/?>|</p>|</div>"));
static RE_TAG: LazyLock<Regex> = LazyLock::new(|| regex(r"<[^>]*>"));
static RE_SPACE: LazyLock<Regex> = LazyLock::new(|| regex(r"[ \t\u{a0}]+"));
static RE_MAINT_WORD: LazyLock<Regex> = LazyLock::new(|| regex(r"(?i)\bmaintenance\b"));
/// An event page can name a `[Maintenance Checklist]` mission, so the bare word in the body is not enough
static RE_MAINT_BODY: LazyLock<Regex> = LazyLock::new(|| {
    regex(
        r"(?i)server maintenance|maintenance date and time|maintenance (?:has been|will be|is) (?:extended|postponed)|emergency maintenance",
    )
});

/// Tags to text: line breaks kept, entities decoded, runs of spaces collapsed
pub fn html_to_text(html: &str) -> String {
    let broken = RE_BREAK.replace_all(html, "\n");
    let stripped = RE_TAG.replace_all(&broken, " ");
    let decoded = decode_entities(&stripped);
    let spaced = RE_SPACE.replace_all(&decoded, " ");
    spaced
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

/// The page's `head-title` on one line; empty on a cover-only notice
pub fn head_title(html: &str) -> Option<String> {
    RE_HEAD_TITLE
        .captures(html)
        .and_then(|c| c.get(1))
        .map(|m| html_to_text(m.as_str()).replace('\n', " "))
}

fn decode_entities(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(amp) = rest.find('&') {
        out.push_str(&rest[..amp]);
        let tail = &rest[amp..];
        let Some(semi) = tail.as_bytes()[..tail.len().min(10)] // bytes: the 10th can fall inside a multi-byte character
            .iter()
            .position(|&b| b == b';')
        else {
            out.push('&');
            rest = &tail[1..];
            continue;
        };
        let entity = &tail[1..semi];
        let decoded = match entity {
            "amp" => Some('&'),
            "lt" => Some('<'),
            "gt" => Some('>'),
            "quot" => Some('"'),
            "apos" => Some('\''),
            "nbsp" => Some(' '),
            "ndash" => Some('\u{2013}'),
            "mdash" => Some('\u{2014}'),
            _ => entity
                .strip_prefix("#x")
                .or_else(|| entity.strip_prefix("#X"))
                .and_then(|hex| u32::from_str_radix(hex, 16).ok())
                .or_else(|| entity.strip_prefix('#').and_then(|d| d.parse().ok()))
                .and_then(char::from_u32),
        };
        if let Some(c) = decoded {
            out.push(c);
            rest = &tail[semi + 1..];
        } else {
            out.push('&');
            rest = &tail[1..];
        }
    }
    out.push_str(rest);
    out
}

fn zone_offset(zone: &str) -> Option<FixedOffset> {
    let z = zone
        .chars()
        .filter(|c| !c.is_whitespace())
        .map(|c| if c == '−' { '-' } else { c })
        .collect::<String>()
        .to_ascii_uppercase();
    let hours = |h: i32| FixedOffset::east_opt(h * 3600);
    match z.as_str() {
        "UTC" | "GMT" => return hours(0),
        "PDT" => return hours(-7),
        "PST" => return hours(-8),
        "JST" | "KST" => return hours(9),
        _ => {}
    }
    let signed = z.strip_prefix("UTC").or_else(|| z.strip_prefix("GMT"))?;
    let (sign, digits) = match signed.as_bytes().first()? {
        b'+' => (1, &signed[1..]),
        b'-' => (-1, &signed[1..]),
        _ => return None,
    };
    let (h, m) = match digits.split_once(':') {
        Some((h, m)) => (h.parse::<i32>().ok()?, m.parse::<i32>().ok()?),
        None if digits.len() > 2 => (
            digits[..digits.len() - 2].parse().ok()?,
            digits[digits.len() - 2..].parse().ok()?,
        ),
        None => (digits.parse().ok()?, 0),
    };
    if h > 14 || m > 59 {
        return None;
    }
    FixedOffset::east_opt(sign * (h * 3600 + m * 60))
}

fn month_number(name: &str) -> Option<u32> {
    let key = name.to_ascii_lowercase();
    let n = [
        "jan", "feb", "mar", "apr", "may", "jun", "jul", "aug", "sep", "oct", "nov", "dec",
    ]
    .iter()
    .position(|m| key.get(..3) == Some(*m))?;
    u32::try_from(n + 1).ok()
}

/// A missing year is the one that puts the date closest to `reference`
fn instant(
    (month, day, year): (u32, u32, Option<i32>),
    (hour, minute): (u32, u32),
    offset: FixedOffset,
    reference: i64,
) -> Option<i64> {
    let build = |y: i32| -> Option<DateTime<FixedOffset>> {
        let date = NaiveDate::from_ymd_opt(y, month, day)?;
        let (date, hour) = if hour == 24 && minute == 0 {
            (date.succ_opt()?, 0)
        } else {
            (date, hour)
        };
        offset
            .from_local_datetime(&date.and_hms_opt(hour, minute, 0)?)
            .single()
    };
    if let Some(y) = year {
        return build(y).map(|d| d.timestamp());
    }
    let ref_year = Utc.timestamp_opt(reference, 0).single()?.year();
    [ref_year - 1, ref_year, ref_year + 1]
        .into_iter()
        .filter_map(build)
        .map(|d| d.timestamp())
        .min_by_key(|t| (t - reference).abs())
}

fn num<T: std::str::FromStr>(caps: &Captures<'_>, name: &str) -> Option<T> {
    caps.name(name).and_then(|m| m.as_str().parse().ok())
}

fn range_of(caps: &Captures<'_>, reference: i64) -> Option<(i64, i64)> {
    let offset = zone_offset(caps.name("tz")?.as_str())?;
    let month = month_number(caps.name("smon")?.as_str())?;
    let day = num(caps, "sd")?;
    let year = num(caps, "sy");
    let start = instant(
        (month, day, year),
        (num(caps, "sh")?, num(caps, "smi")?),
        offset,
        reference,
    )?;
    let end_month = caps.name("emon").and_then(|m| month_number(m.as_str()));
    let end_date = (
        end_month.unwrap_or(month),
        num(caps, "ed").unwrap_or(day),
        num(caps, "ey").or(year),
    );
    let mut end = instant(
        end_date,
        (num(caps, "eh")?, num(caps, "emi")?),
        offset,
        start,
    )?;
    if end <= start && caps.name("emon").is_none() {
        end += Duration::days(1).num_seconds(); // `23:00-01:00` crosses midnight
    }
    (end > start).then_some((start, end))
}

fn find_range(text: &str, reference: i64) -> Option<(i64, i64)> {
    RE_RANGE
        .captures_iter(text)
        .find_map(|caps| range_of(&caps, reference))
}

fn find_instant(text: &str, reference: i64) -> Option<i64> {
    RE_INSTANT.captures_iter(text).find_map(|caps| {
        instant(
            (
                month_number(caps.name("smon")?.as_str())?,
                num(&caps, "sd")?,
                num(&caps, "sy"),
            ),
            (num(&caps, "sh")?, num(&caps, "smi")?),
            zone_offset(caps.name("tz")?.as_str())?,
            reference,
        )
    })
}

/// `None` for a page that is not a maintenance notice or names no time; `reference` settles a missing year
pub fn parse_notice(html: &str, reference: i64) -> Option<NoticeWindow> {
    let title = head_title(html).unwrap_or_default();
    let body_html = RE_CONTENT
        .captures(html)
        .and_then(|c| c.get(1))
        .map_or(html, |m| m.as_str());
    let body = html_to_text(body_html);
    if !RE_MAINT_WORD.is_match(&title) && !RE_MAINT_BODY.is_match(&body) {
        return None;
    }

    let extension_end = RE_EXTENDED
        .find(&body)
        .and_then(|m| find_instant(&body[m.end()..], reference));
    let heading_tail = body
        .to_ascii_lowercase()
        .find("maintenance date and time")
        .map(|i| &body[i..]);
    let sources = [heading_tail, Some(body.as_str()), Some(title.as_str())];
    let (starts_at, planned_end) = if let Some((s, e)) = sources
        .iter()
        .flatten()
        .find_map(|t| find_range(t, reference))
    {
        (s, Some(e))
    } else {
        let start = sources
            .iter()
            .flatten()
            .find_map(|t| find_instant(t, reference))?;
        (start, None)
    };
    let (ends_at, extended) = match extension_end {
        Some(end) if end > starts_at => (Some(end), true),
        _ => (planned_end, false),
    };
    Some(NoticeWindow {
        starts_at,
        ends_at,
        extended,
        title,
    })
}
