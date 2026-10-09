use std::collections::HashMap;
use std::fmt;

use ::serenity::all::HttpError;
use poise::serenity_prelude as serenity;
use serenity::model::id::{GuildId, RoleId, UserId};
use serenity::model::permissions::Permissions;

use crate::db;
use crate::types::{Context, Error};

/// Bot-owner gate: the accounts listed in the application's owner set.
pub async fn owner_check(ctx: Context<'_>) -> Result<bool, Error> {
    Ok(is_bot_owner(ctx))
}

/// Guild-owner gate. Bot owners pass too, since they operate the pipeline these commands
/// drive and would otherwise be locked out of guilds they don't own.
///
/// Attached to the top-level command, this covers every subcommand: poise runs the checks of
/// each parent in the chain before the invoked one.
pub async fn guild_owner_check(ctx: Context<'_>) -> Result<bool, Error> {
    if is_bot_owner(ctx) {
        return Ok(true);
    }
    let guild = require_guild(ctx)?;
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

pub async fn manage_nicknames_check(ctx: Context<'_>) -> Result<bool, Error> {
    elevated(ctx, Permissions::MANAGE_NICKNAMES).await
}

pub async fn moderate_members_check(ctx: Context<'_>) -> Result<bool, Error> {
    elevated(ctx, Permissions::MODERATE_MEMBERS).await
}

/// The gate behind every elevated command: the caller passes if they are a bot owner, if they
/// hold the guild's mod role, or if they hold `needed` in the invoking channel.
///
/// This replaces poise's `required_permissions`, which cannot express that "or". Poise ANDs it
/// in `check_permissions_and_cooldown` *before* any check runs, so a mod-role holder without
/// the permission would be refused before this function ever executed.
async fn elevated(ctx: Context<'_>, needed: Permissions) -> Result<bool, Error> {
    if passes_elevated(ctx, needed).await? {
        return Ok(true);
    }
    Err(format!(
        "You need the {} permission, or this server's mod role, to use this command.",
        needed.get_permission_names().join(" and ")
    )
    .into())
}

/// Whether the invoker would pass [`elevated`] for `needed`, as an answer, not a check failure.
///
/// For commands that act on a second permission partway through, such as `/warn add`
/// escalating to a kick or ban, so the invoker can't borrow the bot's permission.
pub async fn passes_elevated(ctx: Context<'_>, needed: Permissions) -> Result<bool, Error> {
    if is_bot_owner(ctx) {
        return Ok(true);
    }
    let guild = require_guild(ctx)?;

    if has_mod_role(ctx, guild).await? {
        return Ok(true);
    }

    let permissions = author_permissions(ctx)
        .await
        .ok_or("Couldn't read your permissions in this channel.")?;
    Ok(permissions.contains(needed))
}

/// The invoking guild, or the error a guild-only command answers with outside one.
pub fn require_guild(ctx: Context<'_>) -> Result<GuildId, Error> {
    Ok(ctx
        .guild_id()
        .ok_or("This command must be used in a guild.")?)
}

/// Whether the invoker is one of the bot's owners.
#[must_use]
pub fn is_bot_owner(ctx: Context<'_>) -> bool {
    ctx.framework().options().owners.contains(&ctx.author().id)
}

/// Whether the invoking member holds the guild's configured mod role.
///
/// Returns `false` when no mod role is set, which is the state every guild starts in: the
/// permission check in [`passes_elevated`] is then the only way through.
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
pub async fn guild_owner(ctx: Context<'_>, guild: GuildId) -> Result<serenity::UserId, Error> {
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

/// Discord's JSON error code for "Unknown Member".
const UNKNOWN_MEMBER: isize = 10007;

/// Refuse a moderation action on a member the invoker doesn't outrank, as Discord does for its
/// own timeouts. `verb` names the action in the reply ("warn", "ban", "kick").
///
/// The server owner can never be targeted. The server owner and bot owners may target anyone
/// else. Otherwise the target's highest role must sit strictly below the invoker's. Without
/// this, a holder of the mod role (which passes every moderation check with no permission of its
/// own) could warn an admin into a ban policy, or ban or kick them outright: Discord enforces
/// hierarchy against the bot's top role, not the invoker's. A target who isn't in the server has
/// no roles to outrank and passes.
pub async fn check_hierarchy(
    ctx: Context<'_>,
    guild: GuildId,
    target: UserId,
    verb: &str,
) -> Result<(), Error> {
    let owner = guild_owner(ctx, guild).await?;
    if target == owner {
        return Err(format!("You can't {verb} the server owner.").into());
    }
    if ctx.author().id == owner || is_bot_owner(ctx) {
        return Ok(());
    }
    let target_member = match guild.member(ctx, target).await {
        Ok(m) => m,
        // Not in the server: no roles here to outrank.
        Err(::serenity::Error::Http(HttpError::UnsuccessfulRequest(r)))
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
    let positions = role_positions(ctx, guild).await?;
    if highest_position(&target_member.roles, &positions)
        >= highest_position(&author_member.roles, &positions)
    {
        return Err(
            format!("You can't {verb} someone whose highest role is at or above yours.").into(),
        );
    }
    Ok(())
}

/// Every role's position in `guild`, fetched once so all comparisons use one snapshot.
async fn role_positions(ctx: Context<'_>, guild: GuildId) -> Result<HashMap<RoleId, u16>, Error> {
    Ok(guild
        .roles(ctx.http())
        .await
        .map_err(|e| format!("Couldn't read this server's roles: {e}"))?
        .into_iter()
        .map(|(id, role)| (id, role.position))
        .collect())
}

/// Permissions a role bound by `/autorole set` or `/reactionrole add` may not carry. Either
/// binding hands the role to anyone who joins or reacts, so one of these would let any member
/// take over, reconfigure or moderate the server.
const UNBINDABLE: Permissions = Permissions::ADMINISTRATOR
    .union(Permissions::MANAGE_GUILD)
    .union(Permissions::MANAGE_ROLES)
    .union(Permissions::BAN_MEMBERS)
    .union(Permissions::KICK_MEMBERS);

/// Why a role can't be bound to joining or to a reaction.
#[derive(Debug, PartialEq, Eq)]
enum RoleBindRefusal {
    /// The role carries these permissions from [`UNBINDABLE`].
    Dangerous(Permissions),
    /// The role sits at or above the invoker's highest role.
    NotBelowInvoker,
    /// The role sits at or above the bot's highest role, so Discord would refuse the grant.
    NotBelowBot,
}

impl fmt::Display for RoleBindRefusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Dangerous(perms) => write!(
                f,
                "That role carries {}, which a binding would hand to anyone who joins or reacts. \
                 Pick a role without it.",
                perms.get_permission_names().join(", ")
            ),
            Self::NotBelowInvoker => {
                f.write_str("You can only bind a role that sits below your highest role.")
            }
            Self::NotBelowBot => f.write_str(
                "That role sits at or above my highest role, so Discord won't let me grant it. \
                 Move my role above it first.",
            ),
        }
    }
}

/// The decision behind [`check_role_binding`], kept free of Discord so it can be tested.
///
/// `invoker_top` is `None` for an invoker exempt from the hierarchy rule (the server owner or a
/// bot owner). The permission rule applies to everyone, since the role reaches every member who
/// joins or reacts no matter who set the binding up.
fn role_binding_refusal(
    role_position: u16,
    role_permissions: Permissions,
    invoker_top: Option<u16>,
    bot_top: u16,
) -> Option<RoleBindRefusal> {
    let dangerous = role_permissions & UNBINDABLE;
    if !dangerous.is_empty() {
        return Some(RoleBindRefusal::Dangerous(dangerous));
    }
    if invoker_top.is_some_and(|top| role_position >= top) {
        return Some(RoleBindRefusal::NotBelowInvoker);
    }
    if role_position >= bot_top {
        return Some(RoleBindRefusal::NotBelowBot);
    }
    None
}

/// Refuse to bind `role` to joining (`/autorole set`) or to a reaction (`/reactionrole add`)
/// unless the invoker could grant it themselves.
///
/// The role must carry none of [`UNBINDABLE`], must sit strictly below the invoker's highest
/// role (the server owner and bot owners are exempt from that rule, as in [`check_hierarchy`]),
/// and must sit strictly below the bot's highest role. Without this, a holder of the mod role,
/// which passes `manage_roles_check` with no permission of its own, could bind a role above
/// their own, such as an Administrator role, to a reaction and grant it to themselves.
pub async fn check_role_binding(
    ctx: Context<'_>,
    guild: GuildId,
    role: &serenity::Role,
) -> Result<(), Error> {
    let positions = role_positions(ctx, guild).await?;
    let role_position = positions.get(&role.id).copied().unwrap_or(role.position);
    let invoker_top = if is_bot_owner(ctx) || ctx.author().id == guild_owner(ctx, guild).await? {
        None
    } else {
        let author_member = ctx
            .author_member()
            .await
            .ok_or("Couldn't read your roles in this server.")?;
        Some(highest_position(&author_member.roles, &positions))
    };
    let bot_member = guild
        .member(ctx, ctx.framework().bot_id)
        .await
        .map_err(|e| format!("Couldn't read my roles in this server: {e}"))?;
    let bot_top = highest_position(&bot_member.roles, &positions);
    match role_binding_refusal(role_position, role.permissions, invoker_top, bot_top) {
        Some(refusal) => Err(refusal.to_string().into()),
        None => Ok(()),
    }
}

/// The highest position among `roles`; 0 (the `@everyone` position) for a member with none.
fn highest_position(roles: &[RoleId], positions: &HashMap<RoleId, u16>) -> u16 {
    roles
        .iter()
        .filter_map(|r| positions.get(r).copied())
        .max()
        .unwrap_or(0)
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
    fn role_binding_needs_a_role_strictly_below_the_invoker() {
        let none = Permissions::empty();
        assert_eq!(role_binding_refusal(4, none, Some(5), 10), None);
        assert_eq!(
            role_binding_refusal(5, none, Some(5), 10),
            Some(RoleBindRefusal::NotBelowInvoker)
        );
        assert_eq!(
            role_binding_refusal(7, none, Some(5), 10),
            Some(RoleBindRefusal::NotBelowInvoker)
        );
    }

    #[test]
    fn role_binding_exempt_invoker_still_needs_a_role_below_the_bot() {
        let none = Permissions::empty();
        assert_eq!(role_binding_refusal(9, none, None, 10), None);
        assert_eq!(
            role_binding_refusal(10, none, None, 10),
            Some(RoleBindRefusal::NotBelowBot)
        );
        assert_eq!(
            role_binding_refusal(4, none, Some(5), 4),
            Some(RoleBindRefusal::NotBelowBot)
        );
    }

    #[test]
    fn role_binding_refuses_dangerous_permissions_for_everyone() {
        let perms = Permissions::ADMINISTRATOR | Permissions::SEND_MESSAGES;
        assert_eq!(
            role_binding_refusal(1, perms, None, 10),
            Some(RoleBindRefusal::Dangerous(Permissions::ADMINISTRATOR))
        );
        for p in [
            Permissions::MANAGE_GUILD,
            Permissions::MANAGE_ROLES,
            Permissions::BAN_MEMBERS,
            Permissions::KICK_MEMBERS,
        ] {
            assert_eq!(
                role_binding_refusal(1, p, Some(5), 10),
                Some(RoleBindRefusal::Dangerous(p))
            );
        }
        let ordinary =
            Permissions::SEND_MESSAGES | Permissions::ADD_REACTIONS | Permissions::CHANGE_NICKNAME;
        assert_eq!(role_binding_refusal(1, ordinary, Some(5), 10), None);
    }

    #[test]
    fn role_binding_refusal_names_the_dangerous_permissions() {
        let msg = RoleBindRefusal::Dangerous(Permissions::ADMINISTRATOR | Permissions::BAN_MEMBERS)
            .to_string();
        assert!(msg.contains("Administrator"), "{msg}");
        assert!(msg.contains("Ban Members"), "{msg}");
    }
}
