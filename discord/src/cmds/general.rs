use crate::types::{Context, Error};

/// Ping
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    check = "crate::checks::owner_check"
)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("Pong!").await?;
    Ok(())
}

/// Say
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    check = "crate::checks::owner_check"
)]
pub async fn say(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("Pong!").await?;
    Ok(())
}
