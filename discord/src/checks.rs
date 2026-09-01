use poise::serenity_prelude as serenity;
use serenity::model::id::GuildId;
use serenity::model::permissions::Permissions;

use crate::db;
use crate::types::{Context, Error};

/// Bot-owner gate: the accounts listed in the application's owner set.
pub async fn owner_check(ctx: Context<'_>) -> Result<bool, Error> {
    Ok(ctx.framework().options().owners.contains(&ctx.author().id))
}

/// Guild-owner gate. Bot owners pass too, since they operate the pipeline these commands
/// drive and would otherwise be locked out of guilds they don't own.
///
/// Attached to the top-level command, this covers every subcommand: poise runs the checks of
/// each parent in the chain before the invoked one.
pub async fn guild_owner_check(ctx: Context<'_>) -> Result<bool, Error> {
    if ctx.framework().options().owners.contains(&ctx.author().id) {
        return Ok(true);
    }
    let guild = ctx
        .guild_id()
        .ok_or("This command must be used in a guild.")?;
    if ctx.author().id == guild_owner(ctx, guild).await? {
        Ok(true)
    } else {
        Err("Only the server owner can use this command.".into())
    }
}

pub async fn manage_messages_check(ctx: Context<'_>) -> Result<bool, Error> {
    elevated(ctx, Permissions::MANAGE_MESSAGES).await
}

pub async fn manage_roles_check(ctx: Context<'_>) -> Result<bool, Error> {
    elevated(ctx, Permissions::MANAGE_ROLES).await
}

pub async fn manage_guild_check(ctx: Context<'_>) -> Result<bool, Error> {
    elevated(ctx, Permissions::MANAGE_GUILD).await
}

pub async fn ban_members_check(ctx: Context<'_>) -> Result<bool, Error> {
    elevated(ctx, Permissions::BAN_MEMBERS).await
}

pub async fn kick_members_check(ctx: Context<'_>) -> Result<bool, Error> {
    elevated(ctx, Permissions::KICK_MEMBERS).await
}

/// The gate behind every elevated command: the caller passes if they are a bot owner, if they
/// hold the guild's mod role, or if they hold `needed` in the invoking channel.
///
/// This replaces poise's `required_permissions`, which cannot express that "or". Poise ANDs it
/// in `check_permissions_and_cooldown` *before* any check runs, so a mod-role holder without
/// the permission would be refused before this function ever executed.
async fn elevated(ctx: Context<'_>, needed: Permissions) -> Result<bool, Error> {
    if ctx.framework().options().owners.contains(&ctx.author().id) {
        return Ok(true);
    }
    let guild = ctx
        .guild_id()
        .ok_or("This command must be used in a guild.")?;

    if has_mod_role(ctx, guild).await? {
        return Ok(true);
    }

    let permissions = author_permissions(ctx)
        .await
        .ok_or("Couldn't read your permissions in this channel.")?;
    if permissions.contains(needed) {
        return Ok(true);
    }
    Err(format!(
        "You need the {} permission, or this server's mod role, to use this command.",
        needed.get_permission_names().join(" and ")
    )
    .into())
}

/// Whether the invoking member holds the guild's configured mod role.
///
/// Returns `false` when no mod role is set, which is the state every guild starts in - the
/// permission path below is then the only way through, exactly as it was before.
async fn has_mod_role(ctx: Context<'_>, guild: GuildId) -> Result<bool, Error> {
    let Some(mod_role) = db::get_mod_role(&ctx.data().pool, guild)
        .await
        .map_err(|e| format!("Couldn't read this server's mod role: {e}"))?
    else {
        return Ok(false);
    };
    let Some(member) = ctx.author_member().await else {
        return Ok(false);
    };
    Ok(member.roles.contains(&mod_role))
}

/// The author's effective permissions in the invoking channel, or `None` when they can't be
/// determined - callers deny in that case, as poise does.
///
/// Slash commands carry Discord's own resolved permissions on the interaction member, which
/// costs nothing. Prefix invocations have no such field, so they fall back to computing them
/// from the guild, the channel and the member the way `poise::dispatch` does.
async fn author_permissions(ctx: Context<'_>) -> Option<Permissions> {
    if let Some(member) = ctx.author_member().await
        && let Some(permissions) = member.permissions
    {
        return Some(permissions);
    }

    let guild_id = ctx.guild_id()?;
    let guild = guild_id
        .to_partial_guild(ctx.serenity_context())
        .await
        .ok()?;
    let serenity::Channel::Guild(channel) = ctx
        .channel_id()
        .to_channel(ctx.serenity_context())
        .await
        .ok()?
    else {
        return None;
    };
    let member = guild
        .member(ctx.serenity_context(), ctx.author().id)
        .await
        .ok()?;
    Some(guild.user_permissions_in(&channel, &member))
}

/// The guild's owner id, from cache when it's there and over HTTP when it isn't.
async fn guild_owner(ctx: Context<'_>, guild: GuildId) -> Result<serenity::UserId, Error> {
    // Copy the id out before any await - the cache lookup hands back a live map guard.
    let cached = ctx
        .serenity_context()
        .cache
        .guild(guild)
        .map(|g| g.owner_id);
    match cached {
        Some(id) => Ok(id),
        None => Ok(guild
            .to_partial_guild(ctx.serenity_context())
            .await
            .map_err(|e| format!("Couldn't look up the server owner: {e}"))?
            .owner_id),
    }
}
