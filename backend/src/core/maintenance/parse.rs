//! Reading a maintenance window out of a Yostar announcement page.
//!
//! The notices are hand-written HTML, so the window is pulled out of the prose.
//! Every EN maintenance notice from May to September 2026 (ids 1543, 1559,
//! 1567, 1574, 1585, 1592, 1609, 1613, 1616, 1621, 1625, 1626) carries it the
//! same way: `June 22, 2026, 04:00-08:00 (UTC-7)`, once in the opening sentence
//! and once under a `Maintenance Date and Time` heading. The spacing around the
//! zone drifts (`08:00(UTC-7)`, `(UTC-7 )`, a trailing full stop) and the page
//! headline drops the year, so the grammar here is looser than any one notice.
//!
//! Formats handled: a same-day range (`May 8, 2026, 08:00-08:10`), a range that
//! names its end date (`May 8, 2026, 23:00 - May 9, 2026, 01:00`), a start
//! with no end (`starting May 8, 2026, 08:00`), a missing year (inferred from
//! the reference time), `24:00` as the next midnight, and the zones `UTC±H`,
//! `UTC±HH:MM`, `GMT±H`, bare `UTC`/`GMT`, `PDT`, `PST`, `JST` and `KST`. An
//! extension notice (`extended to May 8, 2026, 09:00 (UTC-7)`) replaces the end.

use std::sync::LazyLock;

use chrono::{DateTime, Datelike, Duration, FixedOffset, NaiveDate, TimeZone, Utc};
use regex::Regex;

/// One maintenance window as one notice states it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoticeWindow {
    pub starts_at: i64,
    /// `None` when the notice gives a start and no end.
    pub ends_at: Option<i64>,
    /// The notice moves the end of a window that was already announced.
    pub extended: bool,
    /// The page headline, whitespace collapsed.
    pub title: String,
}

const MONTHS: &str = "January|February|March|April|May|June|July|August|September|October|November|December|Jan|Feb|Mar|Apr|Jun|Jul|Aug|Sept|Sep|Oct|Nov|Dec";
const ZONE: &str = r"(?:UTC|GMT)\s*[+\-−]\s*\d{1,2}(?::?\d{2})?|UTC|GMT|PDT|PST|JST|KST";

fn date_time(prefix: &str, year_required: bool) -> String {
    let year = if year_required {
        format!(r"(?P<{prefix}y>\d{{4}}),?\s*")
    } else {
        format!(r"(?:(?P<{prefix}y>\d{{4}}),?\s*)?")
    };
    format!(
        r"(?P<{prefix}mon>{MONTHS})\.?\s+(?P<{prefix}d>\d{{1,2}})(?:st|nd|rd|th)?,?\s*{year}(?P<{prefix}h>\d{{1,2}}):(?P<{prefix}mi>\d{{2}})"
    )
}

/// `<date>, HH:MM - [<date>,] HH:MM (zone)`.
static RE_RANGE: LazyLock<Regex> = LazyLock::new(|| {
    let end_date = format!(
        r"(?:(?P<emon>{MONTHS})\.?\s+(?P<ed>\d{{1,2}})(?:st|nd|rd|th)?,?\s*(?:(?P<ey>\d{{4}}),?\s*)?)?"
    );
    Regex::new(&format!(
        r"(?i){start}\s*(?:-|\x{{2013}}|\x{{2014}}|~|to)\s*{end_date}(?P<eh>\d{{1,2}}):(?P<emi>\d{{2}})\s*\(?\s*(?P<tz>{ZONE})\s*\)?",
        start = date_time("s", false),
    ))
    .expect("range regex compiles")
});

/// `<date>, HH:MM (zone)`, a single instant.
static RE_INSTANT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
        r"(?i){start}\s*\(?\s*(?P<tz>{ZONE})\s*\)?",
        start = date_time("s", false)
    ))
    .expect("instant regex compiles")
});

static RE_EXTENDED: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\b(?:extended|postponed|prolonged)\s+(?:to|until|till)\b")
        .expect("extension regex compiles")
});

static RE_HEAD_TITLE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?is)class="head-title"[^>]*>(.*?)</"#).expect("title regex compiles")
});
static RE_CONTENT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?is)<div class="content">(.*)</div>"#).expect("content regex compiles")
});
static RE_BREAK: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)<br\s*/?>|</p>|</div>").expect("break regex compiles"));
static RE_TAG: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<[^>]*>").expect("tag regex compiles"));
static RE_SPACE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[ \t\u{a0}]+").expect("space regex compiles"));
static RE_MAINT_WORD: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\bmaintenance\b").expect("maintenance regex compiles"));
/// The phrases a maintenance notice uses and an event notice does not. An
/// event page can name a `[Maintenance Checklist]` mission (1544), so the bare
/// word in the body is not enough.
static RE_MAINT_BODY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)server maintenance|maintenance date and time|maintenance (?:has been|will be|is) (?:extended|postponed)|emergency maintenance")
        .expect("maintenance body regex compiles")
});

/// Tags to text: line breaks kept as `\n`, entities decoded, runs of spaces
/// collapsed.
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

fn decode_entities(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(amp) = rest.find('&') {
        out.push_str(&rest[..amp]);
        let tail = &rest[amp..];
        // Bytes, not a `str` slice: the 10th byte can fall inside a
        // multi-byte character (`&#39;임계` on KR 1391 panicked here).
        let Some(semi) = tail.as_bytes()[..tail.len().min(10)]
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
    let z: String = zone
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
    let lower = name.to_ascii_lowercase();
    let key = lower.get(..3)?;
    let n = [
        "jan", "feb", "mar", "apr", "may", "jun", "jul", "aug", "sep", "oct", "nov", "dec",
    ]
    .iter()
    .position(|m| *m == key)?;
    u32::try_from(n + 1).ok()
}

/// The instant `month day [year] hh:mm` names in `offset`. A missing year is
/// the one that puts the date closest to `reference`, so a December notice
/// read in January lands in the right year.
fn instant(
    month: u32,
    day: u32,
    year: Option<i32>,
    hour: u32,
    minute: u32,
    offset: FixedOffset,
    reference: i64,
) -> Option<i64> {
    let build = |y: i32| -> Option<DateTime<FixedOffset>> {
        let date = NaiveDate::from_ymd_opt(y, month, day)?;
        // `24:00` is the midnight that ends the day.
        let (date, hour) = if hour == 24 && minute == 0 {
            (date.succ_opt()?, 0)
        } else {
            (date, hour)
        };
        let naive = date.and_hms_opt(hour, minute, 0)?;
        offset.from_local_datetime(&naive).single()
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

fn num<T: std::str::FromStr>(caps: &regex::Captures<'_>, name: &str) -> Option<T> {
    caps.name(name).and_then(|m| m.as_str().parse().ok())
}

/// A range read out of `text`: `(start, end)` as unix seconds.
fn find_range(text: &str, reference: i64) -> Option<(i64, i64)> {
    for caps in RE_RANGE.captures_iter(text) {
        let Some(offset) = caps.name("tz").and_then(|m| zone_offset(m.as_str())) else {
            continue;
        };
        let Some(month) = caps.name("smon").and_then(|m| month_number(m.as_str())) else {
            continue;
        };
        let (Some(day), Some(hour), Some(minute)) =
            (num(&caps, "sd"), num(&caps, "sh"), num(&caps, "smi"))
        else {
            continue;
        };
        let year: Option<i32> = num(&caps, "sy");
        let Some(start) = instant(month, day, year, hour, minute, offset, reference) else {
            continue;
        };
        let end_month = caps
            .name("emon")
            .and_then(|m| month_number(m.as_str()))
            .unwrap_or(month);
        let end_day = num(&caps, "ed").unwrap_or(day);
        let end_year = num(&caps, "ey").or(year);
        let (Some(eh), Some(emi)) = (num(&caps, "eh"), num(&caps, "emi")) else {
            continue;
        };
        // Anchor a yearless end on the start, not on `reference`.
        let Some(mut end) = instant(end_month, end_day, end_year, eh, emi, offset, start) else {
            continue;
        };
        // `23:00-01:00` with no end date crosses midnight.
        if end <= start && caps.name("emon").is_none() {
            end += Duration::days(1).num_seconds();
        }
        if end > start {
            return Some((start, end));
        }
    }
    None
}

fn find_instant(text: &str, reference: i64) -> Option<i64> {
    RE_INSTANT.captures_iter(text).find_map(|caps| {
        let offset = zone_offset(caps.name("tz")?.as_str())?;
        let month = month_number(caps.name("smon")?.as_str())?;
        instant(
            month,
            num(&caps, "sd")?,
            num(&caps, "sy"),
            num(&caps, "sh")?,
            num(&caps, "smi")?,
            offset,
            reference,
        )
    })
}

/// Read a maintenance window out of one announcement page.
///
/// Returns `None` for anything that is not a maintenance notice, and for a
/// maintenance notice no time could be read from. `reference` (unix seconds)
/// settles a missing year.
pub fn parse_notice(html: &str, reference: i64) -> Option<NoticeWindow> {
    let title = RE_HEAD_TITLE
        .captures(html)
        .and_then(|c| c.get(1))
        .map(|m| html_to_text(m.as_str()).replace('\n', " "))
        .unwrap_or_default();
    let body_html = RE_CONTENT
        .captures(html)
        .and_then(|c| c.get(1))
        .map_or(html, |m| m.as_str());
    let body = html_to_text(body_html);

    if !RE_MAINT_WORD.is_match(&title) && !RE_MAINT_BODY.is_match(&body) {
        return None;
    }

    // An extension names the new end after "extended to"; the start stays
    // whatever the notice's range or first instant says.
    let extension_end = RE_EXTENDED
        .find(&body)
        .and_then(|m| find_instant(&body[m.end()..], reference));

    // The dedicated heading is the most deliberate statement of the window;
    // fall back to the whole body, then to the headline.
    let heading_tail = body
        .to_ascii_lowercase()
        .find("maintenance date and time")
        .map(|i| &body[i..]);
    let range = heading_tail
        .and_then(|t| find_range(t, reference))
        .or_else(|| find_range(&body, reference))
        .or_else(|| find_range(&title, reference));

    let (starts_at, planned_end) = match range {
        Some((s, e)) => (s, Some(e)),
        None => {
            let start = heading_tail
                .and_then(|t| find_instant(t, reference))
                .or_else(|| find_instant(&body, reference))
                .or_else(|| find_instant(&title, reference))?;
            (start, None)
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    /// 2026-10-11T00:00:00Z, the day the fixtures were fetched.
    const FETCHED: i64 = 1_791_676_800;

    fn ts(s: &str) -> i64 {
        DateTime::parse_from_rfc3339(s)
            .expect("valid rfc3339")
            .timestamp()
    }

    fn parse(html: &str) -> NoticeWindow {
        parse_notice(html, FETCHED).expect("a maintenance notice")
    }

    #[test]
    fn fixed_reference_is_the_fetch_day() {
        assert_eq!(FETCHED, ts("2026-10-11T00:00:00Z"));
    }

    #[test]
    fn four_hour_update_with_a_yearless_headline() {
        let w = parse(include_str!("fixtures/1567.html"));
        assert_eq!(w.title, "June 22, 04:00 (UTC-7) MAINTENANCE");
        assert_eq!(w.starts_at, ts("2026-06-22T04:00:00-07:00"));
        assert_eq!(w.ends_at, Some(ts("2026-06-22T08:00:00-07:00")));
        assert!(!w.extended);
    }

    #[test]
    fn ten_minute_window() {
        let w = parse(include_str!("fixtures/1543.html"));
        assert_eq!(w.starts_at, ts("2026-05-08T08:00:00-07:00"));
        assert_eq!(w.ends_at, Some(ts("2026-05-08T08:10:00-07:00")));
    }

    #[test]
    fn zone_with_a_stray_space_inside_the_parenthesis() {
        // "08:00-08:10 (UTC-7 )."
        let w = parse(include_str!("fixtures/1585.html"));
        assert_eq!(w.starts_at, ts("2026-07-16T08:00:00-07:00"));
        assert_eq!(w.ends_at, Some(ts("2026-07-16T08:10:00-07:00")));
    }

    #[test]
    fn headline_with_no_space_before_the_zone() {
        // Headline "July 30, 2026, 08:00(UTC-7) MAINTENANCE".
        let w = parse(include_str!("fixtures/1592.html"));
        assert_eq!(w.title, "July 30, 2026, 08:00(UTC-7) MAINTENANCE");
        assert_eq!(w.starts_at, ts("2026-07-30T08:00:00-07:00"));
        assert_eq!(w.ends_at, Some(ts("2026-07-30T08:10:00-07:00")));
    }

    #[test]
    fn trailing_full_stop_after_the_zone() {
        let w = parse(include_str!("fixtures/1616.html"));
        assert_eq!(w.starts_at, ts("2026-09-02T08:00:00-07:00"));
        assert_eq!(w.ends_at, Some(ts("2026-09-02T08:10:00-07:00")));
    }

    #[test]
    fn event_notice_naming_a_maintenance_checklist_is_not_maintenance() {
        assert_eq!(
            parse_notice(include_str!("fixtures/1544.html"), FETCHED),
            None
        );
    }

    #[test]
    fn monthly_card_notice_with_dated_ranges_is_not_maintenance() {
        assert_eq!(
            parse_notice(include_str!("fixtures/1627.html"), FETCHED),
            None
        );
    }

    fn page(title: &str, body: &str) -> String {
        format!(
            r#"<span class="head-title">{title}</span><div class="content"><p>{body}</p></div>"#
        )
    }

    #[test]
    fn range_that_crosses_midnight_with_explicit_end_date() {
        let html = page(
            "MAINTENANCE",
            "We plan to perform server maintenance on May 8, 2026, 23:00 – May 9, 2026, 01:30 (UTC-7).",
        );
        let w = parse(&html);
        assert_eq!(w.starts_at, ts("2026-05-08T23:00:00-07:00"));
        assert_eq!(w.ends_at, Some(ts("2026-05-09T01:30:00-07:00")));
    }

    #[test]
    fn range_that_crosses_midnight_without_an_end_date() {
        let w = parse(&page(
            "MAINTENANCE",
            "Server maintenance: Dec 31, 2026, 23:00-01:00 (UTC-8)",
        ));
        assert_eq!(w.starts_at, ts("2026-12-31T23:00:00-08:00"));
        assert_eq!(w.ends_at, Some(ts("2027-01-01T01:00:00-08:00")));
    }

    #[test]
    fn named_zones_and_minute_offsets() {
        let pdt = parse(&page("Maintenance", "October 14, 2026, 04:00 to 08:00 PDT"));
        assert_eq!(pdt.starts_at, ts("2026-10-14T04:00:00-07:00"));
        let pst = parse(&page(
            "Maintenance",
            "November 4, 2026, 04:00 - 08:00 (PST)",
        ));
        assert_eq!(pst.ends_at, Some(ts("2026-11-04T08:00:00-08:00")));
        let plus = parse(&page(
            "Maintenance",
            "Nov 4th, 2026, 10:00-12:00 (UTC+05:30)",
        ));
        assert_eq!(plus.starts_at, ts("2026-11-04T10:00:00+05:30"));
        let utc = parse(&page("Maintenance", "Nov 4, 2026, 10:00-12:00 UTC"));
        assert_eq!(utc.starts_at, ts("2026-11-04T10:00:00Z"));
    }

    #[test]
    fn twenty_four_hundred_is_the_next_midnight() {
        let w = parse(&page("Maintenance", "July 16, 2026, 20:00 - 24:00 (UTC-7)"));
        assert_eq!(w.ends_at, Some(ts("2026-07-17T00:00:00-07:00")));
    }

    #[test]
    fn missing_year_resolves_to_the_nearest_one() {
        // Read in early January, a December 30 notice is last year's.
        let jan = ts("2027-01-02T00:00:00Z");
        let w = parse_notice(
            &page("Maintenance", "December 30, 04:00-08:00 (UTC-7)"),
            jan,
        )
        .expect("parses");
        assert_eq!(w.starts_at, ts("2026-12-30T04:00:00-07:00"));
    }

    #[test]
    fn extension_notice_moves_the_end() {
        let w = parse(&page(
            "MAINTENANCE EXTENSION",
            "The server maintenance on October 14, 2026, 04:00-08:00 (UTC-7) has been extended to October 14, 2026, 10:30 (UTC-7).",
        ));
        assert_eq!(w.starts_at, ts("2026-10-14T04:00:00-07:00"));
        assert_eq!(w.ends_at, Some(ts("2026-10-14T10:30:00-07:00")));
        assert!(w.extended);
    }

    #[test]
    fn start_with_no_end() {
        let w = parse(&page(
            "EMERGENCY MAINTENANCE",
            "Emergency maintenance started on October 14, 2026, 09:12 (UTC-7). We will announce the end time later.",
        ));
        assert_eq!(w.starts_at, ts("2026-10-14T09:12:00-07:00"));
        assert_eq!(w.ends_at, None);
    }

    #[test]
    fn entity_next_to_multibyte_text_does_not_panic() {
        assert_eq!(html_to_text("&#39;임계 상전이&#39;"), "'임계 상전이'");
        assert_eq!(html_to_text("A & Pozëmka"), "A & Pozëmka");
    }

    #[test]
    fn entities_are_decoded() {
        assert_eq!(
            html_to_text("a&#x27;b &amp; c&nbsp;d<br/>e"),
            "a'b & c d\ne"
        );
    }
}
