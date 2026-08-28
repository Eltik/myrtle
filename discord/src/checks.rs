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
    // Copy the id out before any await - the cache lookup hands back a live map guard.
    let cached = ctx
        .serenity_context()
        .cache
        .guild(guild)
        .map(|g| g.owner_id);
    let owner_id = match cached {
        Some(id) => id,
        None => {
            guild
                .to_partial_guild(ctx.serenity_context())
                .await
                .map_err(|e| format!("Couldn't look up the server owner: {e}"))?
                .owner_id
        }
    };
    if ctx.author().id == owner_id {
        Ok(true)
    } else {
        Err("Only the server owner can use this command.".into())
    }
}
