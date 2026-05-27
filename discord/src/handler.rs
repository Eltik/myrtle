use crate::types::{Data, Error};
use poise::serenity_prelude as serenity;

pub async fn event_handler(
    _ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    _data: &Data,
) -> Result<(), Error> {
    // dispatcher grows additional arms over time; keep `match` over `if let`
    #[allow(clippy::single_match)]
    match event {
        serenity::FullEvent::Ready { data_about_bot, .. } => {
            tracing::info!("{} is connected!", data_about_bot.user.name);
        }
        // add more arms as needed: Message { .. }, GuildMemberAddition { .. }, etc.
        _ => {}
    }
    Ok(())
}
