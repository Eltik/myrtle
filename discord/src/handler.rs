use crate::types::{Data, Error};
use poise::{FrameworkContext, serenity_prelude::FullEvent::Ready};
use serenity::{all::prelude::Context, client::FullEvent::{self, GuildMemberAddition}};

pub async fn event_handler(
    ctx: &Context,
    event: &FullEvent,
    _framework: FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    // dispatcher grows additional arms over time; keep `match` over `if let`
    #[allow(clippy::single_match)]
    match event {
        Ready { data_about_bot, .. } => {
            tracing::info!("{} is connected!", data_about_bot.user.name);
        }
        GuildMemberAddition { new_member, .. } => {
            if let Some(role_id) = data.config.roles.auto_role_id
                && let Err(e) = new_member.add_role(&ctx.http, role_id).await {
                    tracing::error!(
                        "Failed to add auto-role {role_id} to {} in {}: {e}",
                        new_member.user.id, new_member.guild_id
                    );
                }
        }
        // add more arms as needed: Message { .. }, GuildMemberAddition { .. }, etc.
        _ => {}
    }
    Ok(())
}
