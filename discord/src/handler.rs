use std::time::{Duration, Instant};

use crate::audit;
use crate::db::{self, AntiSpamAction, AntiSpamPolicy};
use crate::types::{AntiSpamPolicies, Data, Error, PingHistory};
use poise::{FrameworkContext, serenity_prelude::FullEvent::Ready};
use serenity::all::EditMember;
use serenity::model::Timestamp;
use serenity::model::channel::{Message, Reaction};
use serenity::model::id::{GuildId, UserId};
use serenity::{
    all::prelude::Context,
    client::FullEvent::{
        self, GuildAuditLogEntryCreate, GuildBanAddition, GuildBanRemoval, GuildDelete,
        GuildMemberAddition, GuildMemberRemoval, Message as MessageCreate, MessageDelete,
        MessageDeleteBulk, MessageUpdate, ReactionAdd, ReactionRemove,
    },
};

// Dispatcher grows additional event arms over time (reaction roles, antispam, audit log...);
// splitting it into per-event helpers just scatters the wiring without making any of it clearer.
#[allow(clippy::too_many_lines)]
pub async fn event_handler(
    ctx: &Context,
    event: &FullEvent,
    framework: FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    let bot_id = framework.bot_id;
    // dispatcher grows additional arms over time; keep `match` over `if let`
    #[allow(clippy::single_match)]
    match event {
        Ready { data_about_bot, .. } => {
            tracing::info!("{} is connected!", data_about_bot.user.name);
        }
        GuildMemberAddition { new_member, .. } => {
            match db::get_auto_role(&data.pool, new_member.guild_id).await {
                Ok(Some(role_id)) => {
                    if let Err(e) = new_member.add_role(&ctx.http, role_id).await {
                        tracing::error!(
                            "Failed to add auto-role {role_id} to {} in {}: {e}",
                            new_member.user.id,
                            new_member.guild_id
                        );
                    }
                }
                Ok(None) => {}
                Err(e) => {
                    tracing::error!(
                        "Failed to look up auto-role for guild {}: {e}",
                        new_member.guild_id
                    );
                }
            }
            audit::log_member_join(ctx, data, new_member).await;
        }
        GuildMemberRemoval {
            guild_id,
            user,
            member_data_if_available,
        } => {
            audit::log_member_leave(
                ctx,
                data,
                *guild_id,
                user,
                member_data_if_available.as_ref(),
            )
            .await;
        }
        GuildBanAddition {
            guild_id,
            banned_user,
        } => {
            audit::log_member_ban(ctx, data, *guild_id, banned_user).await;
        }
        GuildBanRemoval {
            guild_id,
            unbanned_user,
        } => {
            audit::log_member_unban(ctx, data, *guild_id, unbanned_user).await;
        }
        GuildAuditLogEntryCreate { entry, guild_id } => {
            audit::log_audit_entry(ctx, data, *guild_id, entry).await;
        }
        ReactionAdd { add_reaction } => {
            handle_reaction(ctx, data, bot_id, add_reaction, true).await?;
        }
        ReactionRemove { removed_reaction } => {
            handle_reaction(ctx, data, bot_id, removed_reaction, false).await?;
        }
        MessageCreate { new_message } => {
            if let Err(e) = handle_message(ctx, data, bot_id, new_message).await {
                tracing::error!(
                    "Antispam handler failed for message {}: {e}",
                    new_message.id
                );
            }
        }
        MessageUpdate {
            old_if_available,
            new,
            event,
        } => {
            audit::log_message_update(ctx, data, old_if_available.as_ref(), new.as_ref(), event)
                .await;
        }
        MessageDelete {
            channel_id,
            deleted_message_id,
            guild_id,
        } => {
            // Gate DB writes on cache membership - most deleted messages aren't tracked.
            let was_tracked = data
                .tracked_messages
                .write()
                .await
                .remove(deleted_message_id);
            if was_tracked
                && let Err(e) = db::remove_reaction_message(&data.pool, *deleted_message_id).await
            {
                tracing::error!(
                    "Failed to delete reaction-role rows for message {deleted_message_id}: {e}"
                );
            }
            audit::log_message_delete(ctx, data, *channel_id, *deleted_message_id, *guild_id).await;
        }
        MessageDeleteBulk {
            channel_id,
            multiple_deleted_messages_ids,
            guild_id,
        } => {
            let tracked: Vec<_> = {
                let mut cache = data.tracked_messages.write().await;
                multiple_deleted_messages_ids
                    .iter()
                    .copied()
                    .filter(|id| cache.remove(id))
                    .collect()
            };
            for id in tracked {
                if let Err(e) = db::remove_reaction_message(&data.pool, id).await {
                    tracing::error!("Failed to delete reaction-role rows for message {id}: {e}");
                }
            }
            audit::log_message_bulk_delete(
                ctx,
                data,
                *channel_id,
                multiple_deleted_messages_ids,
                *guild_id,
            )
            .await;
        }
        GuildDelete { incomplete, .. } => {
            // `unavailable` flags a transient outage, not a removal - only purge on real kicks.
            if !incomplete.unavailable
                && let Err(e) = db::remove_reaction_roles_for_guild(&data.pool, incomplete.id).await
            {
                tracing::error!(
                    "Failed to clean reaction-role rows for guild {}: {e}",
                    incomplete.id
                );
            }
        }
        _ => {}
    }
    Ok(())
}

async fn handle_reaction(
    ctx: &Context,
    data: &Data,
    bot_id: UserId,
    r: &Reaction,
    added: bool,
) -> Result<(), Error> {
    let Some(user_id) = r.user_id else {
        return Ok(());
    };
    let Some(guild_id) = r.guild_id else {
        return Ok(());
    };

    if user_id == bot_id {
        return Ok(());
    }

    if !data.tracked_messages.read().await.contains(&r.message_id) {
        return Ok(());
    }

    // `as_data()` percent-encodes Unicode (URL form); use `to_string()` so the storage key
    // matches the human-readable form that `/reactionrole add` parses and writes.
    let key = r.emoji.to_string();
    let Some(role_id) = db::get_role_for_reaction(&data.pool, r.message_id, &key).await? else {
        return Ok(());
    };

    let member = guild_id.member(&ctx.http, user_id).await?;
    if added {
        member.add_role(&ctx.http, role_id).await?;
    } else {
        member.remove_role(&ctx.http, role_id).await?;
    }
    Ok(())
}

/// Run both antispam checks (per-message + rolling window) against `msg` and execute the
/// configured action on a violation. Silently returns for DMs, bots, and guilds with no policy.
//
// `significant_drop_tightening` flags the write guard in the rolling-window block, but `entry`
// borrows from the guard so the suggested rewrite doesn't typecheck. Allow at the fn level
// since attribute placement inside `let` bindings is ignored by this lint.
#[allow(clippy::significant_drop_tightening)]
async fn handle_message(
    ctx: &Context,
    data: &Data,
    bot_id: UserId,
    msg: &Message,
) -> Result<(), Error> {
    if msg.author.bot || msg.author.id == bot_id {
        return Ok(());
    }
    let Some(guild_id) = msg.guild_id else {
        return Ok(());
    };
    let Some(policy) = data.antispam_policies.read().await.get(&guild_id).cloned() else {
        return Ok(());
    };

    if let Some(exempt_role) = policy.exempt_role_id
        && let Some(member) = msg.member.as_ref()
        && member.roles.contains(&exempt_role)
    {
        return Ok(());
    }

    // `msg.mentions` and `msg.mention_roles` are *deduplicated* — pinging the same target
    // five times yields one entry. We want the raw token count so the limit reflects what
    // a human reads as "pings in this message", so parse `msg.content` directly. Requires
    // the MESSAGE_CONTENT intent, which is already requested in main.rs.
    let pings = count_mention_tokens(&msg.content);
    if pings == 0 {
        return Ok(());
    }

    let mut triggered = pings > policy.max_per_message;

    if let (Some(window_secs), Some(window_max)) = (policy.window_secs, policy.window_max_pings) {
        let now = Instant::now();
        let cutoff = Duration::from_secs(u64::from(window_secs));
        let key = (guild_id, msg.author.id);
        // Scope the write guard so it drops before we run the action. The borrow checker
        // forces `hist` to live as long as `entry`, so clippy's significant_drop_tightening
        // suggestion to merge them doesn't typecheck; the block is the tightest we can do.
        let total: u32 = {
            let mut hist = data.ping_history.write().await;
            let entry = hist.entry(key).or_default();
            while let Some(&(t, _)) = entry.front() {
                if now.duration_since(t) > cutoff {
                    entry.pop_front();
                } else {
                    break;
                }
            }
            entry.push_back((now, pings));
            entry.iter().map(|(_, p)| *p).sum()
        };
        if total > window_max {
            triggered = true;
        }
    }

    if !triggered {
        return Ok(());
    }

    tracing::info!(
        "Antispam triggered for {} in {}: pings={pings}, max_per_message={}, action={}",
        msg.author.id,
        guild_id,
        policy.max_per_message,
        policy.action.as_db_str(),
    );
    execute_action(ctx, &policy, guild_id, msg).await;
    Ok(())
}

/// Count raw mention tokens — `<@123>`, `<@!123>`, `<@&123>` — in `content`. Each occurrence
/// counts separately, so `@me @me @me` returns 3 (unlike `msg.mentions.len()` which dedupes).
fn count_mention_tokens(content: &str) -> u32 {
    let bytes = content.as_bytes();
    let mut count: u32 = 0;
    let mut i = 0;
    while i + 2 < bytes.len() {
        if bytes[i] != b'<' || bytes[i + 1] != b'@' {
            i += 1;
            continue;
        }
        // Optional `!` (legacy nickname mention) or `&` (role mention) sigil.
        let mut j = i + 2;
        if j < bytes.len() && (bytes[j] == b'!' || bytes[j] == b'&') {
            j += 1;
        }
        let digits_start = j;
        while j < bytes.len() && bytes[j].is_ascii_digit() {
            j += 1;
        }
        if j > digits_start && j < bytes.len() && bytes[j] == b'>' {
            count = count.saturating_add(1);
            i = j + 1;
        } else {
            i += 1;
        }
    }
    count
}

/// Apply the configured action against `msg.author`. Failures are logged but not propagated
/// - one missed enforcement shouldn't kill the event handler.
async fn execute_action(ctx: &Context, policy: &AntiSpamPolicy, guild_id: GuildId, msg: &Message) {
    let reason = "Antispam: ping limit exceeded";
    match policy.action {
        AntiSpamAction::Delete => {
            if let Err(e) = msg.delete(&ctx.http).await {
                tracing::warn!("Antispam delete failed for {}: {e}", msg.id);
            }
        }
        AntiSpamAction::Warn => {
            let content = format!(
                "<@{}> please slow down on the pings (limit: {} per message).",
                msg.author.id, policy.max_per_message,
            );
            if let Err(e) = msg.channel_id.say(&ctx.http, content).await {
                tracing::warn!("Antispam warn failed for {}: {e}", msg.author.id);
            }
        }
        AntiSpamAction::Timeout => {
            let secs = policy.timeout_secs.unwrap_or(60);
            let until = Timestamp::now().unix_timestamp() + i64::from(secs);
            match Timestamp::from_unix_timestamp(until) {
                Ok(ts) => {
                    let edit = EditMember::new().disable_communication_until_datetime(ts);
                    if let Err(e) = guild_id.edit_member(&ctx.http, msg.author.id, edit).await {
                        tracing::warn!("Antispam timeout failed for {}: {e}", msg.author.id);
                    }
                }
                Err(e) => {
                    tracing::warn!("Antispam: bad timeout target ({secs}s): {e}");
                }
            }
        }
        AntiSpamAction::Kick => {
            if let Err(e) = guild_id
                .kick_with_reason(&ctx.http, msg.author.id, reason)
                .await
            {
                tracing::warn!("Antispam kick failed for {}: {e}", msg.author.id);
            }
        }
        AntiSpamAction::Ban => {
            if let Err(e) = guild_id
                .ban_with_reason(&ctx.http, msg.author.id, 0, reason)
                .await
            {
                tracing::warn!("Antispam ban failed for {}: {e}", msg.author.id);
            }
        }
    }
}

/// Sweep interval for `run_ping_history_sweep`. Kept fast enough to keep memory bounded but
/// slow enough to avoid contending with the hot path on every message.
const SWEEP_INTERVAL: Duration = Duration::from_mins(1);

/// Periodically prune `ping_history` so users who go quiet don't keep their deques alive
/// forever. Runs until the task is aborted at shutdown.
///
/// Per-message pruning in `handle_message` already trims expired entries from the front of
/// each deque, but only when that user sends another message. This sweep catches the
/// long-tail case where a user sends one message and never returns.
pub async fn run_ping_history_sweep(history: PingHistory, policies: AntiSpamPolicies) {
    let mut ticker = tokio::time::interval(SWEEP_INTERVAL);
    ticker.tick().await;
    loop {
        ticker.tick().await;
        let removed = sweep_once(&history, &policies).await;
        if removed > 0 {
            tracing::debug!("Antispam sweep dropped {removed} stale ping-history entries");
        }
    }
}

/// One pass of the sweep. Returned for tracing; the loop above is the only caller.
async fn sweep_once(history: &PingHistory, policies: &AntiSpamPolicies) -> usize {
    // Snapshot the policy map so we don't hold both locks at once.
    let policy_snapshot: std::collections::HashMap<_, _> = policies
        .read()
        .await
        .iter()
        .map(|(g, p)| (*g, p.window_secs))
        .collect();

    let now = Instant::now();
    let mut hist = history.write().await;
    let before = hist.len();
    hist.retain(|(guild_id, _), entries| {
        // If the guild's policy disappeared (or its window was cleared) the entries are
        // orphaned - drop them outright.
        let Some(Some(window_secs)) = policy_snapshot.get(guild_id).copied() else {
            return false;
        };
        let cutoff = Duration::from_secs(u64::from(window_secs));
        while let Some(&(t, _)) = entries.front() {
            if now.duration_since(t) > cutoff {
                entries.pop_front();
            } else {
                break;
            }
        }
        !entries.is_empty()
    });
    before - hist.len()
}
