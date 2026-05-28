use crate::types::{Context, Error};

/// Ping
// Everyone can see the cmd, but only owner can use it
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

/// Say a message
#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    default_member_permissions = "MANAGE_MESSAGES",
    required_permissions = "MANAGE_MESSAGES"
)]
pub async fn say(
    ctx: Context<'_>,
    #[description = "What to say"] message: String,
) -> Result<(), Error> {
    ctx.say(message).await?;
    Ok(())
}
