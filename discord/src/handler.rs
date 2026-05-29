use crate::db;
use crate::types::{Data, Error};
use poise::{FrameworkContext, serenity_prelude::FullEvent::Ready};
use serenity::model::channel::Reaction;
use serenity::model::id::UserId;
use serenity::{
    all::prelude::Context,
    client::FullEvent::{
        self, GuildDelete, GuildMemberAddition, MessageDelete, ReactionAdd, ReactionRemove,
    },
};

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
        }
        ReactionAdd { add_reaction } => {
            handle_reaction(ctx, data, bot_id, add_reaction, true).await?;
        }
        ReactionRemove { removed_reaction } => {
            handle_reaction(ctx, data, bot_id, removed_reaction, false).await?;
        }
        MessageDelete {
            deleted_message_id, ..
        } => {
            // Gate DB writes on cache membership — most deleted messages aren't tracked.
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
        }
        GuildDelete { incomplete, .. } => {
            // `unavailable` flags a transient outage, not a removal — only purge on real kicks.
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
