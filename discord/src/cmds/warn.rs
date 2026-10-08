//! `/warn`: moderator warnings, with optional automatic escalation.
//!
//! Warnings live in `guild_warnings`. A guild may add a policy (`/warn policy set`) that times
//! out, kicks or bans a member once they hold enough warnings; without one, warnings are a
//! record and nothing more.

use std::collections::HashMap;
use std::fmt::Write as _;

use ::serenity::builder::{CreateEmbed, CreateEmbedFooter, CreateMessage, EditMember};
use ::serenity::model::Timestamp;
use ::serenity::model::id::{GuildId, RoleId, UserId};
use ::serenity::model::permissions::Permissions;
use ::serenity::model::user::User;
use poise::CreateReply;
use serenity::all::HttpError;

use crate::audit;
use crate::checks::{self, require_guild};
use crate::db::{self, WarnAction, WarnPolicy, Warning};
use crate::types::{Context, Error};
use crate::ui::{COLOR_WARN, TITLE_MAX};
use crate::utils::ellipsize;

/// Longest reason accepted, leaving room under the 1024-character embed field the DM and the
/// audit-log mirror put it in.
const REASON_MAX: usize = 1000;
/// Each reason is cut to this in `/warn list`, so one long reason can't crowd out the rest.
const LIST_REASON_MAX: usize = 200;
/// `/warn list` stops adding entries past this many characters, under the 4096 description cap.
const LIST_BUDGET: usize = 3800;
/// Discord's longest timeout, 28 days.
const TIMEOUT_MAX_MINUTES: u32 = 28 * 24 * 60;
const DAY_SECS: i64 = 86_400;
/// Discord's JSON error code for "Unknown Member".
const UNKNOWN_MEMBER: isize = 10007;

/// Warn members and review their warnings.
///
/// Subcommands: `add`, `list`, `remove`, `clear`, `policy`. The first four need the Timeout
/// Members permission or this server's mod role; `policy` needs Manage Server.
#[poise::command(
    slash_command,
    guild_only,
    subcommands("warn_add", "warn_list", "warn_remove", "warn_clear", "warn_policy"),
    subcommand_required
)]
pub async fn warn(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Warn a member. They get a DM, and the server's escalation policy runs if one is set.
#[poise::command(
    slash_command,
    guild_only,
    rename = "add",
    check = "crate::checks::moderate_members_check"
)]
pub async fn warn_add(
    ctx: Context<'_>,
    #[description = "Member to warn"] user: User,
    #[description = "Why they are being warned"]
    #[max_length = 1000]
    reason: String,
) -> Result<(), Error> {
    let guild = require_guild(ctx)?;
    if user.id == ctx.author().id {
        return Err("You can't warn yourself.".into());
    }
    if user.id == ctx.framework().bot_id {
        return Err("I can't warn myself.".into());
    }
    if user.bot {
        return Err("Bots can't be warned.".into());
    }
    let reason = reason.trim();
    if reason.is_empty() {
        return Err("Give a reason for the warning.".into());
    }
    if reason.chars().count() > REASON_MAX {
        return Err(format!("Keep the reason under {REASON_MAX} characters.").into());
    }

    // The role lookup, the DM and an escalation are several round-trips; don't race the 3 s
    // interaction window.
    ctx.defer_ephemeral().await?;
    check_hierarchy(ctx, guild, user.id).await?;

    let pool = &ctx.data().pool;
    let now = Timestamp::now().unix_timestamp();
    let id = db::add_warning(pool, guild, user.id, ctx.author().id, reason, now)
        .await
        .map_err(|e| format!("Couldn't save the warning: {e}"))?;
    let total = db::count_warnings(pool, guild, user.id, None)
        .await
        .map_err(|e| format!("Saved warning #{id}, but couldn't count their warnings: {e}"))?;

    let guild_name = guild
        .name(ctx.serenity_context())
        .unwrap_or_else(|| "a server".to_string());
    let dm = dm_embed(&guild_name, reason, total);
    let dm_sent = user
        .direct_message(ctx.http(), CreateMessage::new().embed(dm))
        .await
        .is_ok();

    // Escalation never fails the warning: its outcome, good or bad, is reported alongside.
    let escalation = escalate(ctx, guild, &user, now).await;

    audit::log_warning(
        ctx.http(),
        ctx.data(),
        guild,
        &user,
        ctx.author().id,
        id,
        reason,
        total,
        escalation.as_deref(),
    )
    .await;

    let mut content = format!(
        "Warned <@{}> (warning `#{id}`, {total} in total). {}",
        user.id,
        if dm_sent {
            "They were sent a DM."
        } else {
            "Could not DM them; their DMs may be closed."
        }
    );
    if let Some(outcome) = escalation {
        let _ = write!(content, "\nEscalation: {outcome}");
    }
    ctx.send(CreateReply::default().content(content).ephemeral(true))
        .await?;
    Ok(())
}

/// Refuse to warn a member the invoker doesn't outrank, as Discord does for its own timeouts.
///
/// The server owner can never be warned. The server owner and bot owners may warn anyone else.
/// Otherwise the target's highest role must sit strictly below the invoker's. Without this, a
/// moderator holding only Timeout Members could warn an admin into a ban policy.
async fn check_hierarchy(ctx: Context<'_>, guild: GuildId, target: UserId) -> Result<(), Error> {
    let owner = checks::guild_owner(ctx, guild).await?;
    if target == owner {
        return Err("You can't warn the server owner.".into());
    }
    if ctx.author().id == owner || checks::is_bot_owner(ctx) {
        return Ok(());
    }
    let target_member = match guild.member(ctx, target).await {
        Ok(m) => m,
        // Not in the server: no roles here for a warning to outrank.
        Err(serenity::Error::Http(HttpError::UnsuccessfulRequest(r)))
            if r.error.code == UNKNOWN_MEMBER =>
        {
            return Ok(());
        }
        Err(e) => return Err(format!("Couldn't look up that member: {e}").into()),
    };
    let author_member = ctx
        .author_member()
        .await
        .ok_or("Couldn't read your roles in this server.")?;
    let positions: HashMap<RoleId, u16> = guild
        .roles(ctx.http())
        .await
        .map_err(|e| format!("Couldn't read this server's roles: {e}"))?
        .into_iter()
        .map(|(id, role)| (id, role.position))
        .collect();
    if highest_position(&target_member.roles, &positions)
        >= highest_position(&author_member.roles, &positions)
    {
        return Err("You can't warn someone whose highest role is at or above yours.".into());
    }
    Ok(())
}

/// The highest position among `roles`; 0 (the `@everyone` position) for a member with none.
fn highest_position(roles: &[RoleId], positions: &HashMap<RoleId, u16>) -> u16 {
    roles
        .iter()
        .filter_map(|r| positions.get(r).copied())
        .max()
        .unwrap_or(0)
}

/// Apply the guild's policy to `user` if their warnings have reached its threshold.
///
/// Returns `None` when there is no policy or the threshold isn't reached, else one line saying
/// what happened, including failures such as the member outranking the bot.
async fn escalate(ctx: Context<'_>, guild: GuildId, user: &User, now: i64) -> Option<String> {
    let pool = &ctx.data().pool;
    let policy = match db::get_warn_policy(pool, guild).await {
        Ok(Some(p)) => p,
        Ok(None) => return None,
        Err(e) => return Some(format!("couldn't read the warning policy: {e}")),
    };
    let since = policy
        .window_days
        .map(|days| now - i64::from(days) * DAY_SECS);
    let count = match db::count_warnings(pool, guild, user.id, since).await {
        Ok(c) => c,
        Err(e) => return Some(format!("couldn't count warnings for the policy: {e}")),
    };
    if count < i64::from(policy.threshold) {
        return None;
    }

    let reason = format!("Reached {count} warnings");

    // A kick or ban goes out under the bot's permissions, so the invoker must hold that
    // permission (or the mod role) themselves. A timeout is what `/warn` already gates on.
    let gate = match policy.action {
        WarnAction::Timeout => None,
        WarnAction::Kick => Some((Permissions::KICK_MEMBERS, "Kick Members")),
        WarnAction::Ban => Some((Permissions::BAN_MEMBERS, "Ban Members")),
    };
    if let Some((needed, name)) = gate {
        let action = policy.action.as_db_str();
        match checks::passes_elevated(ctx, needed).await {
            Ok(true) => {}
            Ok(false) => {
                return Some(format!(
                    "{reason}, but the policy's {action} was not applied: you lack the {name} \
                     permission (or this server's mod role)."
                ));
            }
            Err(e) => {
                return Some(format!(
                    "{reason}, but the policy's {action} was not applied: couldn't check your \
                     permissions: {e}"
                ));
            }
        }
    }

    let http = ctx.http();
    let (done, result) = match policy.action {
        WarnAction::Timeout => {
            let secs = policy.timeout_secs.unwrap_or(3600);
            let result = match Timestamp::from_unix_timestamp(now + i64::from(secs)) {
                Ok(until) => guild
                    .edit_member(
                        http,
                        user.id,
                        EditMember::new()
                            .disable_communication_until_datetime(until)
                            .audit_log_reason(&reason),
                    )
                    .await
                    .map(|_| ())
                    .map_err(|e| e.to_string()),
                Err(e) => Err(format!("bad timeout length: {e}")),
            };
            (
                format!("timed out for {}", format_minutes(secs / 60)),
                result,
            )
        }
        WarnAction::Kick => (
            "kicked".to_string(),
            guild
                .kick_with_reason(http, user.id, &reason)
                .await
                .map_err(|e| e.to_string()),
        ),
        WarnAction::Ban => (
            "banned".to_string(),
            guild
                .ban_with_reason(http, user.id, 0, &reason)
                .await
                .map_err(|e| e.to_string()),
        ),
    };
    Some(match result {
        Ok(()) => format!("{reason}, so they were {done}."),
        Err(e) => format!(
            "{reason}, but they could not be {done}: {e}. Check that my role is above theirs \
             and has the permission."
        ),
    })
}

/// List a member's warnings, newest first.
#[poise::command(
    slash_command,
    guild_only,
    rename = "list",
    check = "crate::checks::moderate_members_check"
)]
pub async fn warn_list(
    ctx: Context<'_>,
    #[description = "Member whose warnings to show"] user: User,
) -> Result<(), Error> {
    let guild = require_guild(ctx)?;
    let warnings = db::list_warnings(&ctx.data().pool, guild, user.id)
        .await
        .map_err(|e| format!("Couldn't read warnings: {e}"))?;
    if warnings.is_empty() {
        ctx.send(
            CreateReply::default()
                .content(format!("<@{}> has no warnings.", user.id))
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    }

    let embed = list_embed(&user, &warnings);
    ctx.send(CreateReply::default().embed(embed).ephemeral(true))
        .await?;
    Ok(())
}

/// The DM a warned member receives.
fn dm_embed(guild_name: &str, reason: &str, total: i64) -> CreateEmbed {
    CreateEmbed::new()
        .title(ellipsize(
            &format!("You were warned in {guild_name}"),
            TITLE_MAX,
        ))
        .colour(COLOR_WARN)
        .field("Reason", reason, false)
        .field("Warnings in this server", total.to_string(), true)
        .timestamp(Timestamp::now())
}

/// `/warn list`'s embed: newest first, cut off under the description cap.
fn list_embed(user: &User, warnings: &[Warning]) -> CreateEmbed {
    let mut body = String::new();
    let mut shown = 0;
    for w in warnings {
        let entry = format!(
            "`#{}` <t:{}:d> by <@{}>\n{}\n",
            w.id,
            w.created_at,
            w.moderator_id,
            ellipsize(&w.reason, LIST_REASON_MAX)
        );
        if body.len() + entry.len() > LIST_BUDGET {
            break;
        }
        body.push_str(&entry);
        shown += 1;
    }
    if shown < warnings.len() {
        let _ = write!(
            body,
            "…and {} older warning(s) not shown.",
            warnings.len() - shown
        );
    }

    CreateEmbed::new()
        .title(ellipsize(
            &format!("Warnings for {}", user.tag()),
            TITLE_MAX,
        ))
        .thumbnail(user.face())
        .description(body)
        .colour(COLOR_WARN)
        .footer(CreateEmbedFooter::new(format!(
            "{} warning(s) in total • User ID: {}",
            warnings.len(),
            user.id
        )))
}

/// Remove one warning by its id (shown in `/warn list`).
#[poise::command(
    slash_command,
    guild_only,
    rename = "remove",
    check = "crate::checks::moderate_members_check"
)]
pub async fn warn_remove(
    ctx: Context<'_>,
    #[description = "Warning id, from /warn list"]
    #[min = 1]
    id: i64,
) -> Result<(), Error> {
    let guild = require_guild(ctx)?;
    let removed = db::remove_warning(&ctx.data().pool, guild, id)
        .await
        .map_err(|e| format!("Couldn't remove the warning: {e}"))?;
    let Some(user) = removed else {
        return Err(format!("There is no warning `#{id}` in this server.").into());
    };
    ctx.send(
        CreateReply::default()
            .content(format!("Removed warning `#{id}` from <@{user}>."))
            .ephemeral(true),
    )
    .await?;
    Ok(())
}

/// Remove every warning a member holds in this server.
#[poise::command(
    slash_command,
    guild_only,
    rename = "clear",
    check = "crate::checks::moderate_members_check"
)]
pub async fn warn_clear(
    ctx: Context<'_>,
    #[description = "Member whose warnings to clear"] user: User,
) -> Result<(), Error> {
    let guild = require_guild(ctx)?;
    let removed = db::clear_warnings(&ctx.data().pool, guild, user.id)
        .await
        .map_err(|e| format!("Couldn't clear warnings: {e}"))?;
    let content = if removed == 0 {
        format!("<@{}> had no warnings.", user.id)
    } else {
        format!("Removed {removed} warning(s) from <@{}>.", user.id)
    };
    ctx.send(CreateReply::default().content(content).ephemeral(true))
        .await?;
    Ok(())
}

/// Configure what happens when a member collects too many warnings.
///
/// Subcommands: `set`, `show`, `clear`. Needs the Manage Server permission or this server's
/// mod role. With no policy, warnings never trigger an action.
#[poise::command(
    slash_command,
    guild_only,
    rename = "policy",
    check = "crate::checks::manage_guild_check",
    subcommands("warn_policy_set", "warn_policy_show", "warn_policy_clear"),
    subcommand_required
)]
pub async fn warn_policy(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Set the escalation policy, replacing any previous one.
///
/// When a member's warnings reach `threshold` (counting only the last `window_days` days when
/// given), `action` is applied. Every warning past the threshold applies it again.
#[poise::command(slash_command, guild_only, rename = "set")]
pub async fn warn_policy_set(
    ctx: Context<'_>,
    #[description = "Warnings that trigger the action"]
    #[min = 1]
    #[max = 100]
    threshold: u32,
    #[description = "Action to take at the threshold"] action: WarnAction,
    #[description = "Only count warnings from the last N days (omit to count all)"]
    #[min = 1]
    #[max = 3650]
    window_days: Option<u32>,
    #[description = "Timeout length in minutes (required for Timeout, max 40320)"]
    #[min = 1]
    #[max = 40320]
    timeout_minutes: Option<u32>,
) -> Result<(), Error> {
    let guild = require_guild(ctx)?;
    let timeout_secs = match (action, timeout_minutes) {
        (WarnAction::Timeout, None) => {
            return Err("`timeout_minutes` is required when the action is `Timeout user`.".into());
        }
        (WarnAction::Timeout, Some(m)) if m > TIMEOUT_MAX_MINUTES => {
            return Err(
                format!("Discord timeouts last at most {TIMEOUT_MAX_MINUTES} minutes.").into(),
            );
        }
        (WarnAction::Timeout, Some(m)) => Some(m * 60),
        (_, Some(_)) => {
            return Err("`timeout_minutes` only applies to `Timeout user`.".into());
        }
        (_, None) => None,
    };
    let policy = WarnPolicy {
        threshold,
        window_days,
        action,
        timeout_secs,
    };
    db::set_warn_policy(&ctx.data().pool, guild, &policy)
        .await
        .map_err(|e| format!("Couldn't save the warning policy: {e}"))?;
    ctx.send(
        CreateReply::default()
            .content(format!("Warning policy set.\n{}", format_policy(&policy)))
            .ephemeral(true),
    )
    .await?;
    Ok(())
}

/// Show the escalation policy.
#[poise::command(slash_command, guild_only, rename = "show")]
pub async fn warn_policy_show(ctx: Context<'_>) -> Result<(), Error> {
    let guild = require_guild(ctx)?;
    let content = match db::get_warn_policy(&ctx.data().pool, guild)
        .await
        .map_err(|e| format!("Couldn't read the warning policy: {e}"))?
    {
        Some(p) => format_policy(&p),
        None => "No warning policy configured. Warnings never trigger an action.".to_string(),
    };
    ctx.send(CreateReply::default().content(content).ephemeral(true))
        .await?;
    Ok(())
}

/// Remove the escalation policy. Existing warnings are kept.
#[poise::command(slash_command, guild_only, rename = "clear")]
pub async fn warn_policy_clear(ctx: Context<'_>) -> Result<(), Error> {
    let guild = require_guild(ctx)?;
    let removed = db::clear_warn_policy(&ctx.data().pool, guild)
        .await
        .map_err(|e| format!("Couldn't clear the warning policy: {e}"))?;
    let content = if removed == 0 {
        "No warning policy was configured."
    } else {
        "Warning policy cleared. Warnings no longer trigger an action."
    };
    ctx.send(CreateReply::default().content(content).ephemeral(true))
        .await?;
    Ok(())
}

fn format_policy(p: &WarnPolicy) -> String {
    let window = p.window_days.map_or_else(
        || "ever".to_string(),
        |d| format!("in the last **{d}** day(s)"),
    );
    let mut s = format!("• Threshold: **{}** warning(s) {window}\n", p.threshold);
    let _ = write!(s, "• Action: **{}**", p.action.as_db_str());
    if matches!(p.action, WarnAction::Timeout)
        && let Some(secs) = p.timeout_secs
    {
        let _ = write!(s, " ({})", format_minutes(secs / 60));
    }
    s
}

/// "45 min", "2 h", "1 h 30 min", "3 d".
fn format_minutes(minutes: u32) -> String {
    let (days, hours, mins) = (minutes / 1440, minutes % 1440 / 60, minutes % 60);
    let parts: Vec<String> = [(days, "d"), (hours, "h"), (mins, "min")]
        .into_iter()
        .filter(|(n, _)| *n > 0)
        .map(|(n, unit)| format!("{n} {unit}"))
        .collect();
    if parts.is_empty() {
        "0 min".to_string()
    } else {
        parts.join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn highest_position_ignores_unknown_roles() {
        let (low, high, gone) = (RoleId::new(1), RoleId::new(2), RoleId::new(3));
        let positions = HashMap::from([(low, 3), (high, 9)]);
        assert_eq!(highest_position(&[low, high, gone], &positions), 9);
        assert_eq!(highest_position(&[gone], &positions), 0);
        assert_eq!(highest_position(&[], &positions), 0);
    }

    #[test]
    fn formats_minutes() {
        assert_eq!(format_minutes(45), "45 min");
        assert_eq!(format_minutes(120), "2 h");
        assert_eq!(format_minutes(90), "1 h 30 min");
        assert_eq!(format_minutes(TIMEOUT_MAX_MINUTES), "28 d");
    }
}
