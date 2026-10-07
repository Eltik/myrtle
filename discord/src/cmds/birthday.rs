//! `/birthday`: operator birthday announcements and lookups.
//!
//! The announcer itself lives in `crate::birthday`; this module binds its channel and answers
//! "whose birthday is it" on demand.

use std::fmt::Write as _;
use std::sync::Arc;

use ::serenity::builder::CreateEmbed;
use ::serenity::model::Timestamp;
use poise::CreateReply;
use poise::serenity_prelude as serenity;

use crate::api::gamedata::{GameData, Operator};
use crate::birthday::{
    Date, MonthDay, birthday_embed, birthday_roster, celebrates, game_day, headline,
    next_reset_after, reset_instant,
};
use crate::db;
use crate::types::{Context, Error};
use crate::utils::ellipsize;

#[allow(clippy::unreadable_literal)]
const COLOR_BIRTHDAY: u32 = 0xEB459E;

/// Discord's cap on embeds per message.
const EMBEDS_PER_MESSAGE: usize = 10;
const DESCRIPTION_MAX: usize = 4096;

/// Operator birthdays.
///
/// Subcommands: `channel` (daily announcements, Manage Server), `today`, `upcoming`.
#[poise::command(
    slash_command,
    subcommands("birthday_channel", "birthday_today", "birthday_upcoming"),
    subcommand_required
)]
pub async fn birthday(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Configure the channel that gets a post at every daily reset.
///
/// Subcommands: `set`, `clear`, `show`. Needs the Manage Server permission or this server's
/// mod role.
#[poise::command(
    slash_command,
    guild_only,
    rename = "channel",
    check = "crate::checks::manage_guild_check",
    subcommands(
        "birthday_channel_set",
        "birthday_channel_clear",
        "birthday_channel_show"
    ),
    subcommand_required
)]
pub async fn birthday_channel(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Post operator birthdays in a channel at every daily reset (04:00 UTC-7).
///
/// A new binding counts the current game day as already posted, so its first post is the next
/// reset, never an immediate one. Moving an existing binding keeps its schedule.
#[poise::command(slash_command, guild_only, rename = "set")]
pub async fn birthday_channel_set(
    ctx: Context<'_>,
    #[description = "Channel to post birthdays in"] channel: serenity::ChannelId,
) -> Result<(), Error> {
    let guild = ctx
        .guild_id()
        .ok_or("This command must be used in a guild.")?;
    let pool = &ctx.data().pool;
    let existing = db::get_birthday_channel(pool, guild)
        .await
        .map_err(|e| format!("Couldn't read the birthday channel: {e}"))?;
    let now = Timestamp::now().unix_timestamp();
    db::set_birthday_channel(pool, guild, channel, &game_day(now).iso())
        .await
        .map_err(|e| format!("Couldn't save the birthday channel: {e}"))?;
    let next = next_reset_after(now);
    let content = if existing.is_some() {
        format!(
            "Birthday announcements moved to <#{channel}>. They keep posting at each daily \
             reset (04:00 UTC-7)."
        )
    } else {
        format!(
            "Operator birthdays will be posted in <#{channel}> at each daily reset \
             (04:00 UTC-7). The first post is <t:{next}:F>, <t:{next}:R>."
        )
    };
    ctx.send(CreateReply::default().content(content).ephemeral(true))
        .await?;
    Ok(())
}

/// Stop posting operator birthdays.
#[poise::command(slash_command, guild_only, rename = "clear")]
pub async fn birthday_channel_clear(ctx: Context<'_>) -> Result<(), Error> {
    let guild = ctx
        .guild_id()
        .ok_or("This command must be used in a guild.")?;
    let removed = db::clear_birthday_channel(&ctx.data().pool, guild)
        .await
        .map_err(|e| format!("Couldn't clear the birthday channel: {e}"))?;
    let content = if removed > 0 {
        "Birthday announcements disabled."
    } else {
        "No birthday channel was configured."
    };
    ctx.send(CreateReply::default().content(content).ephemeral(true))
        .await?;
    Ok(())
}

/// Show where operator birthdays are posted.
#[poise::command(slash_command, guild_only, rename = "show")]
pub async fn birthday_channel_show(ctx: Context<'_>) -> Result<(), Error> {
    let guild = ctx
        .guild_id()
        .ok_or("This command must be used in a guild.")?;
    let content = match db::get_birthday_channel(&ctx.data().pool, guild)
        .await
        .map_err(|e| format!("Couldn't read the birthday channel: {e}"))?
    {
        Some((channel, _)) => {
            let next = next_reset_after(Timestamp::now().unix_timestamp());
            format!("Birthday announcements: <#{channel}>. Next post <t:{next}:R>.")
        }
        None => "No birthday channel configured.".to_string(),
    };
    ctx.send(CreateReply::default().content(content).ephemeral(true))
        .await?;
    Ok(())
}

/// Whose birthday is it today? Today is the current game day, which turns at 04:00 UTC-7.
#[poise::command(slash_command, rename = "today")]
pub async fn birthday_today(ctx: Context<'_>) -> Result<(), Error> {
    let gamedata = begin(ctx).await?;
    let operators = gamedata.operators().await?;
    let roster = birthday_roster(&operators);
    let today = game_day(Timestamp::now().unix_timestamp());
    let celebrated = on(&roster, today);

    if celebrated.is_empty() {
        let next = (1..=366)
            .map(|i| today.add_days(i))
            .find_map(|date| {
                let ops = on(&roster, date);
                (!ops.is_empty()).then(|| {
                    let names: Vec<&str> = ops.iter().map(|o| o.name.as_str()).collect();
                    format!(" Next up, **{}**: {}.", date.short(), names.join(", "))
                })
            })
            .unwrap_or_default();
        ctx.send(CreateReply::default().content(format!(
            "No operator birthdays today ({}).{next}",
            today.short()
        )))
        .await?;
        return Ok(());
    }

    let frontend = &ctx.data().config.endpoints.public_frontend;
    let mut content = headline(&celebrated);
    if celebrated.len() > EMBEDS_PER_MESSAGE {
        let _ = write!(
            content,
            " Showing the first {EMBEDS_PER_MESSAGE} of {}.",
            celebrated.len()
        );
    }
    let mut reply = CreateReply::default().content(content);
    for op in celebrated.iter().take(EMBEDS_PER_MESSAGE) {
        reply = reply.embed(birthday_embed(op, &gamedata, frontend));
    }
    ctx.send(reply).await?;
    Ok(())
}

/// Operator birthdays over the coming days.
#[poise::command(slash_command, rename = "upcoming")]
pub async fn birthday_upcoming(
    ctx: Context<'_>,
    #[description = "How many days ahead, including today (default 7, max 31)"]
    #[min = 1]
    #[max = 31]
    days: Option<u32>,
) -> Result<(), Error> {
    let days = days.unwrap_or(7).clamp(1, 31);
    let gamedata = begin(ctx).await?;
    let operators = gamedata.operators().await?;
    let roster = birthday_roster(&operators);
    let today = game_day(Timestamp::now().unix_timestamp());

    let mut body = String::new();
    for i in 0..i64::from(days) {
        let date = today.add_days(i);
        let ops = on(&roster, date);
        if ops.is_empty() {
            continue;
        }
        let names: Vec<&str> = ops.iter().map(|o| o.name.as_str()).collect();
        let when = if i == 0 {
            "today".to_string()
        } else {
            format!("<t:{}:R>", reset_instant(date))
        };
        let _ = writeln!(body, "**{}** {when}: {}", date.short(), names.join(", "));
    }
    if body.is_empty() {
        let _ = write!(body, "No operator birthdays in the next {days} day(s).");
    }

    let embed = CreateEmbed::new()
        .title(if days == 1 {
            "Operator birthdays today".to_string()
        } else {
            format!("Operator birthdays, next {days} days")
        })
        .description(ellipsize(&body, DESCRIPTION_MAX))
        .colour(COLOR_BIRTHDAY);
    ctx.send(CreateReply::default().embed(embed)).await?;
    Ok(())
}

/// The roster entries celebrating on `date`, by name.
fn on<'a>(roster: &[(MonthDay, &'a Operator)], date: Date) -> Vec<&'a Operator> {
    roster
        .iter()
        .filter(|(birthday, _)| celebrates(*birthday, date))
        .map(|(_, op)| *op)
        .collect()
}

/// Check the backend is configured, and defer when the operator list hasn't been fetched yet.
async fn begin(ctx: Context<'_>) -> Result<Arc<GameData>, Error> {
    let gamedata = Arc::clone(&ctx.data().gamedata);
    gamedata.base()?;
    if !gamedata.is_warm().await {
        ctx.defer().await?;
    }
    Ok(gamedata)
}
