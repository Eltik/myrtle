//! Operator birthday announcements.
//!
//! The day turns over at the EN game reset, 04:00 UTC-7, which is 11:00 UTC all year (the game
//! clock observes no DST). The birthday "day" is the UTC-7 calendar date at that instant, so
//! between 07:00 and 11:00 UTC the UTC-7 date has already moved on while the game day has not.
//!
//! The dates come from each operator's profile (`dateOfBirth` in `/api/operators/index`), which
//! is free text: "Dec. 23", "Sept 17th", "July 1?", "Unknown". [`parse_birthday`] turns the
//! readable ones into a [`MonthDay`] and drops the rest.
//!
//! Dates are plain std arithmetic (civil-from-days) rather than a `chrono`/`time` dependency:
//! the bot needs one fixed offset, day arithmetic and an ISO string, nothing a calendar crate
//! would make shorter.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use serenity::all::{ChannelId, CreateEmbed, CreateMessage, Http, HttpError, Timestamp};
use sqlx::SqlitePool;

use crate::api::gamedata::{GameData, Operator};
use crate::db;

#[allow(clippy::unreadable_literal)]
const COLOR_BIRTHDAY: u32 = 0xEB459E;

/// Seconds after UTC midnight at which the game day turns over (04:00 UTC-7).
pub const RESET_UTC_SECS: i64 = 11 * 3600;
/// The game clock's fixed offset from UTC.
const GAME_UTC_OFFSET_SECS: i64 = -7 * 3600;
const DAY_SECS: i64 = 86_400;
/// Longest the scheduler sleeps between checks, so a channel bound since the last tick, or a
/// post that failed transiently, is picked up without waiting a whole day.
const MAX_SLEEP: Duration = Duration::from_mins(10);
/// Discord's cap on embeds per message.
const EMBEDS_PER_MESSAGE: usize = 10;

const MONTHS: [&str; 12] = [
    "january",
    "february",
    "march",
    "april",
    "may",
    "june",
    "july",
    "august",
    "september",
    "october",
    "november",
    "december",
];
const MONTH_ABBR: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

/// A month and day with no year: what an operator profile gives.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MonthDay {
    pub month: u32,
    pub day: u32,
}

/// A proleptic Gregorian calendar date.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Date {
    pub year: i64,
    pub month: u32,
    pub day: u32,
}

impl Date {
    /// The date `days` days after 1970-01-01 (Howard Hinnant's `civil_from_days`).
    #[must_use]
    // Every intermediate is bounded by the algorithm: month is 1..=12 and day 1..=31.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub const fn from_days(days: i64) -> Self {
        let z = days + 719_468;
        let era = z.div_euclid(146_097);
        let doe = z.rem_euclid(146_097);
        let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
        let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
        let year = yoe + era * 400 + if month <= 2 { 1 } else { 0 };
        Self { year, month, day }
    }

    /// Days since 1970-01-01, the inverse of [`Date::from_days`].
    #[must_use]
    pub const fn to_days(self) -> i64 {
        let month = self.month as i64;
        let year = if month <= 2 { self.year - 1 } else { self.year };
        let era = year.div_euclid(400);
        let yoe = year - era * 400;
        let mp = (month + 9) % 12;
        let doy = (153 * mp + 2) / 5 + self.day as i64 - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        era * 146_097 + doe - 719_468
    }

    #[must_use]
    pub const fn add_days(self, n: i64) -> Self {
        Self::from_days(self.to_days() + n)
    }

    /// `YYYY-MM-DD`, the form `guild_birthday_channel.last_posted_date` stores.
    #[must_use]
    pub fn iso(self) -> String {
        format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }

    /// "Oct 9".
    #[must_use]
    pub fn short(self) -> String {
        format!("{} {}", MONTH_ABBR[(self.month - 1) as usize], self.day)
    }
}

#[must_use]
pub const fn is_leap(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

const fn days_in_month(month: u32) -> u32 {
    match month {
        2 => 29,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

/// The game day in effect at `unix`: the UTC-7 date of the most recent reset.
#[must_use]
pub const fn game_day(unix: i64) -> Date {
    Date::from_days((unix - RESET_UTC_SECS).div_euclid(DAY_SECS))
}

/// The UTC-7 calendar date at `unix`.
#[must_use]
pub const fn utc7_date(unix: i64) -> Date {
    Date::from_days((unix + GAME_UTC_OFFSET_SECS).div_euclid(DAY_SECS))
}

/// The unix time of the reset that starts `date`'s game day.
#[must_use]
pub const fn reset_instant(date: Date) -> i64 {
    date.to_days() * DAY_SECS + RESET_UTC_SECS
}

/// The first reset strictly after `unix`.
#[must_use]
pub const fn next_reset_after(unix: i64) -> i64 {
    reset_instant(game_day(unix)) + DAY_SECS
}

/// Parse a profile's `dateOfBirth` into a month and day.
///
/// Accepts a month (full name, or any prefix of three letters or more, so "Sept" works) and a
/// day (optionally "st"/"nd"/"rd"/"th"), in either order, with dots, commas, spacing and case
/// ignored. A trailing "?" is dropped: "July 1?" is the profile's own best guess and still names
/// a day. Anything else ("Unknown", "Undisclosed", empty) is `None`.
#[must_use]
pub fn parse_birthday(raw: &str) -> Option<MonthDay> {
    let text = raw.trim().trim_end_matches('?').to_lowercase();
    let tokens: Vec<&str> = text
        .split(|c: char| c.is_whitespace() || c == '.' || c == ',')
        .filter(|t| !t.is_empty())
        .collect();
    let [first, second] = tokens.as_slice() else {
        return None;
    };
    let (month, day) = match month_of(first) {
        Some(m) => (m, day_of(second)?),
        None => (month_of(second)?, day_of(first)?),
    };
    (1..=days_in_month(month))
        .contains(&day)
        .then_some(MonthDay { month, day })
}

fn month_of(token: &str) -> Option<u32> {
    if token.len() < 3 || !token.chars().all(|c| c.is_ascii_alphabetic()) {
        return None;
    }
    let index = MONTHS.iter().position(|m| m.starts_with(token))?;
    u32::try_from(index + 1).ok()
}

fn day_of(token: &str) -> Option<u32> {
    let digits = ["st", "nd", "rd", "th"]
        .iter()
        .find_map(|suffix| token.strip_suffix(suffix))
        .unwrap_or(token);
    if digits.is_empty() || !digits.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    digits.parse().ok()
}

/// Whether a `birthday` falls on `date`. A Feb 29 birthday is celebrated on Feb 28 in years
/// without a Feb 29.
#[must_use]
pub const fn celebrates(birthday: MonthDay, date: Date) -> bool {
    if birthday.month == date.month && birthday.day == date.day {
        return true;
    }
    birthday.month == 2
        && birthday.day == 29
        && date.month == 2
        && date.day == 28
        && !is_leap(date.year)
}

/// Every operator with a readable birthday, one entry per character.
///
/// Some characters have several ids (Amiya's Caster, Guard and Medic forms). Those collapse to
/// one by name, keeping an obtainable form over an unobtainable one, then the shortest id, which
/// is the base form ("`char_002_amiya`"). The index holds no tokens or traps (every entry has a
/// real operator class), so nothing else is filtered out.
#[must_use]
pub fn birthday_roster(operators: &[Operator]) -> Vec<(MonthDay, &Operator)> {
    let mut by_name: HashMap<&str, (MonthDay, &Operator)> = HashMap::new();
    for op in operators {
        let Some(birthday) = parse_birthday(&op.date_of_birth) else {
            continue;
        };
        by_name
            .entry(op.name.as_str())
            .and_modify(|kept| {
                if form_rank(op) < form_rank(kept.1) {
                    *kept = (birthday, op);
                }
            })
            .or_insert((birthday, op));
    }
    let mut roster: Vec<_> = by_name.into_values().collect();
    roster.sort_by(|a, b| a.1.name.cmp(&b.1.name));
    roster
}

/// Which of a character's forms speaks for it: obtainable first, then the shortest id.
const fn form_rank(op: &Operator) -> (bool, usize, &str) {
    (op.is_not_obtainable, op.id.len(), op.id.as_str())
}

/// The operators whose birthday is `date`, by name.
#[must_use]
pub fn birthdays_on(operators: &[Operator], date: Date) -> Vec<&Operator> {
    birthday_roster(operators)
        .into_iter()
        .filter(|(birthday, _)| celebrates(*birthday, date))
        .map(|(_, op)| op)
        .collect()
}

/// One birthday card. The line under the title only restates profile fields.
pub fn birthday_embed(op: &Operator, gamedata: &GameData, frontend: &str) -> CreateEmbed {
    // Branch ("Core Caster"), else class, else nothing more specific than "operator".
    let branch = op.branch().or_else(|| op.class()).unwrap_or("operator");
    let line = match op.factions().last() {
        Some(faction) => format!(
            "Celebrating our {}★ {branch} of {faction} today.",
            op.rarity
        ),
        None => format!("Celebrating our {}★ {branch} today.", op.rarity),
    };
    let mut embed = CreateEmbed::new()
        .title(format!("Happy birthday, {}!", op.name))
        .description(line)
        .colour(COLOR_BIRTHDAY)
        .image(gamedata.api_url(&format!("/charart/{}", op.id)));
    if !frontend.is_empty() {
        embed = embed.url(format!(
            "{}/operators/{}",
            frontend.trim_end_matches('/'),
            op.id
        ));
    }
    embed
}

/// The messages announcing `operators`: one embed each, ten embeds per message.
#[must_use]
pub fn birthday_messages(
    operators: &[&Operator],
    gamedata: &GameData,
    frontend: &str,
) -> Vec<CreateMessage> {
    operators
        .chunks(EMBEDS_PER_MESSAGE)
        .enumerate()
        .map(|(i, chunk)| {
            let mut msg = CreateMessage::new().embeds(
                chunk
                    .iter()
                    .map(|op| birthday_embed(op, gamedata, frontend))
                    .collect(),
            );
            if i == 0 {
                msg = msg.content(headline(operators));
            }
            msg
        })
        .collect()
}

/// "Today's operator birthdays: **Amiya**, **Kal'tsit**."
#[must_use]
pub fn headline(operators: &[&Operator]) -> String {
    let names: Vec<String> = operators
        .iter()
        .map(|o| format!("**{}**", o.name))
        .collect();
    if names.len() == 1 {
        format!("Today's operator birthday: {}.", names[0])
    } else {
        format!("Today's operator birthdays: {}.", names.join(", "))
    }
}

fn unix_now() -> i64 {
    Timestamp::now().unix_timestamp()
}

/// The announcer: posts each bound guild's birthdays once per game day.
///
/// Wakes at every reset and at least every [`MAX_SLEEP`]. Whether a guild is due is read from
/// `last_posted_date`, never from the wake-up itself, so a restart can't double-post and a bot
/// that was down at 11:00 UTC catches up on its first tick after.
pub async fn run(http: Arc<Http>, pool: SqlitePool, gamedata: Arc<GameData>, frontend: String) {
    loop {
        tick(&http, &pool, &gamedata, &frontend).await;
        let now = unix_now();
        let until_reset = u64::try_from(next_reset_after(now) - now).unwrap_or(1);
        tokio::time::sleep(
            Duration::from_secs(until_reset).clamp(Duration::from_secs(1), MAX_SLEEP),
        )
        .await;
    }
}

async fn tick(http: &Arc<Http>, pool: &SqlitePool, gamedata: &GameData, frontend: &str) {
    let now = unix_now();
    let today = utc7_date(now);
    // Between 07:00 and 11:00 UTC the UTC-7 date has turned but its reset hasn't happened.
    if now < reset_instant(today) {
        return;
    }
    let date = today.iso();
    let due = match db::list_birthday_channels_due(pool, &date).await {
        Ok(d) => d,
        Err(e) => {
            tracing::error!("birthdays: list due channels: {e}");
            return;
        }
    };
    if due.is_empty() {
        return;
    }
    let operators = match gamedata.operators().await {
        Ok(o) => o,
        Err(e) => {
            // Nothing is marked posted, so the next tick tries again.
            tracing::warn!("birthdays: operator list unavailable: {e}");
            return;
        }
    };
    let Some(celebrated) = celebrations(&operators, today) else {
        // An empty index is a backend hiccup, not a day without birthdays: posting nothing and
        // marking every guild done would silently drop the day.
        tracing::warn!("birthdays: operator list is empty; retrying next tick");
        return;
    };
    tracing::info!(
        "birthdays: {date}: {} operator(s), {} guild(s) due",
        celebrated.len(),
        due.len()
    );

    for (guild, channel) in due {
        let delivered = if celebrated.is_empty() {
            true
        } else {
            post(http, channel, &celebrated, gamedata, frontend).await
        };
        if delivered && let Err(e) = db::mark_birthday_posted(pool, guild, &date).await {
            tracing::error!("birthdays: mark {guild} posted for {date}: {e}");
        }
    }
}

/// Send the day's messages to `channel`. Returns whether the day counts as done for this guild:
/// true on success, and also when the binding itself is broken (see [`SendFailure::Permanent`]),
/// so it is not retried every ten minutes. Anything else returns false and the next tick tries
/// again.
async fn post(
    http: &Arc<Http>,
    channel: ChannelId,
    operators: &[&Operator],
    gamedata: &GameData,
    frontend: &str,
) -> bool {
    let announce = crate::watcher::is_announcement_channel(http, channel).await;
    for msg in birthday_messages(operators, gamedata, frontend) {
        match channel.send_message(http.as_ref(), msg).await {
            Ok(sent) => {
                if announce && let Err(e) = sent.crosspost(http.as_ref()).await {
                    tracing::warn!("birthdays: crosspost {} in {channel}: {e}", sent.id);
                }
            }
            Err(e) => {
                return match classify(&e) {
                    SendFailure::Permanent => {
                        tracing::warn!("birthdays: {channel} is unusable, skipping today: {e}");
                        true
                    }
                    SendFailure::BadPayload => {
                        tracing::error!(
                            "birthdays: Discord rejected the message for {channel}: {e}"
                        );
                        false
                    }
                    SendFailure::Transient => {
                        tracing::warn!("birthdays: send to {channel} failed, will retry: {e}");
                        false
                    }
                };
            }
        }
    }
    true
}

/// The operators celebrating on `date`, or `None` when the operator list itself is empty:
/// that is a failed fetch in disguise, never a real day without birthdays.
#[must_use]
pub fn celebrations(operators: &[Operator], date: Date) -> Option<Vec<&Operator>> {
    (!operators.is_empty()).then(|| birthdays_on(operators, date))
}

/// Why a send failed, which decides whether the day is retried.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SendFailure {
    /// The channel is gone or the bot can't use it: "Unknown Channel" (10003), "Missing
    /// Access" (50001), "Missing Permissions" (50013). Retrying won't help until an admin acts.
    Permanent,
    /// Discord rejected the message itself (HTTP 400). Our bug: retried, and logged as an error.
    BadPayload,
    /// Anything else: rate limits, 5xx, network trouble.
    Transient,
}

const UNKNOWN_CHANNEL: isize = 10_003;
const MISSING_ACCESS: isize = 50_001;
const MISSING_PERMISSIONS: isize = 50_013;

const fn classify(e: &serenity::Error) -> SendFailure {
    match e {
        serenity::Error::Http(HttpError::UnsuccessfulRequest(r)) => {
            classify_response(r.status_code.as_u16(), r.error.code)
        }
        _ => SendFailure::Transient,
    }
}

/// [`classify`] on the parts of an HTTP error response: its status and Discord's JSON code.
const fn classify_response(status: u16, code: isize) -> SendFailure {
    match code {
        UNKNOWN_CHANNEL | MISSING_ACCESS | MISSING_PERMISSIONS => SendFailure::Permanent,
        _ if status == 400 => SendFailure::BadPayload,
        _ => SendFailure::Transient,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn md(month: u32, day: u32) -> Option<MonthDay> {
        Some(MonthDay { month, day })
    }

    #[test]
    fn empty_operator_list_is_transient() {
        let date = Date {
            year: 2026,
            month: 12,
            day: 23,
        };
        assert!(celebrations(&[], date).is_none());
        let ops: Vec<Operator> = serde_json::from_str(
            r#"[{"id":"char_002_amiya","name":"Amiya","rarity":5,"position":"RANGED",
                "professionName":"Caster","subProfessionName":null,"nationName":null,
                "groupName":null,"teamName":null,"dateOfBirth":"Dec. 23"}]"#,
        )
        .unwrap();
        assert_eq!(celebrations(&ops, date).map(|c| c.len()), Some(1));
        assert_eq!(
            celebrations(&ops, date.add_days(1)).map(|c| c.len()),
            Some(0)
        );
    }

    #[test]
    fn classifies_send_failures() {
        assert_eq!(classify_response(404, 10_003), SendFailure::Permanent);
        assert_eq!(classify_response(403, 50_001), SendFailure::Permanent);
        assert_eq!(classify_response(403, 50_013), SendFailure::Permanent);
        assert_eq!(classify_response(400, 50_035), SendFailure::BadPayload);
        assert_eq!(classify_response(429, 0), SendFailure::Transient);
        assert_eq!(classify_response(500, 0), SendFailure::Transient);
        // A 403 with some other code (e.g. a temporary block) is not proof the binding is dead.
        assert_eq!(classify_response(403, 40_002), SendFailure::Transient);
    }

    #[test]
    fn parses_profile_formats() {
        assert_eq!(parse_birthday("Dec. 23"), md(12, 23));
        assert_eq!(parse_birthday("Dec 23"), md(12, 23));
        assert_eq!(parse_birthday("December 23"), md(12, 23));
        assert_eq!(parse_birthday("23 Dec"), md(12, 23));
        assert_eq!(parse_birthday("Dec. 23rd"), md(12, 23));
        assert_eq!(parse_birthday("Mar 25  "), md(3, 25));
        assert_eq!(parse_birthday("  oCtObEr 4"), md(10, 4));
        assert_eq!(parse_birthday("Sept 17th"), md(9, 17));
        assert_eq!(parse_birthday("July 1?"), md(7, 1));
        assert_eq!(parse_birthday("Jan 1st"), md(1, 1));
        assert_eq!(parse_birthday("Feb 29"), md(2, 29));
    }

    #[test]
    fn rejects_non_dates() {
        for raw in [
            "",
            "   ",
            "Unknown",
            "Undisclosed",
            "Reported as Unknown",
            "Claims to have forgotten",
            "No exact date provided",
            "Individual exhibits memory loss regarding this information",
            "Feb 30",
            "Apr 31",
            "Dec 0",
            "Ju 4",
            "Dec",
            "23",
            "Dec 23 1990",
            "Dec x23",
        ] {
            assert_eq!(parse_birthday(raw), None, "{raw:?}");
        }
    }

    #[test]
    fn civil_round_trip() {
        assert_eq!(
            Date::from_days(0),
            Date {
                year: 1970,
                month: 1,
                day: 1
            }
        );
        let d = Date {
            year: 2024,
            month: 2,
            day: 29,
        };
        assert_eq!(Date::from_days(d.to_days()), d);
        assert_eq!(d.add_days(1).iso(), "2024-03-01");
        for days in -800_000..800_000 {
            if days % 997 == 0 {
                assert_eq!(Date::from_days(days).to_days(), days);
            }
        }
    }

    #[test]
    fn game_day_turns_at_eleven_utc() {
        // 2026-10-07 10:59:59 UTC: the 10-07 reset hasn't happened, so the game day is 10-06,
        // though the UTC-7 date is already 10-07.
        let before = Date {
            year: 2026,
            month: 10,
            day: 7,
        }
        .to_days()
            * DAY_SECS
            + RESET_UTC_SECS
            - 1;
        assert_eq!(game_day(before).iso(), "2026-10-06");
        assert_eq!(utc7_date(before).iso(), "2026-10-07");
        assert!(before < reset_instant(utc7_date(before)));
        assert_eq!(game_day(before + 1).iso(), "2026-10-07");
        assert_eq!(next_reset_after(before), before + 1);
        assert_eq!(next_reset_after(before + 1), before + 1 + DAY_SECS);
        // 06:59 UTC on 10-08 is still 23:59 on 10-07 in UTC-7.
        let late = before + 1 + 20 * 3600 - 60;
        assert_eq!(utc7_date(late).iso(), "2026-10-07");
        assert_eq!(game_day(late).iso(), "2026-10-07");
    }

    #[test]
    fn leap_day_birthdays_move_to_feb_28() {
        let leap = MonthDay { month: 2, day: 29 };
        let date = |year, day| Date {
            year,
            month: 2,
            day,
        };
        assert!(celebrates(leap, date(2027, 28)));
        assert!(!celebrates(leap, date(2028, 28)));
        assert!(celebrates(leap, date(2028, 29)));
        assert!(!celebrates(MonthDay { month: 2, day: 28 }, date(2028, 29)));
    }
}
