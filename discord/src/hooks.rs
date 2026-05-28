use crate::types::{Context, Data, Error};
use poise::CreateReply;

pub async fn pre_command(_ctx: Context<'_>) {
    // let name = ctx.command().qualified_name.clone();
    // tracing::info!("Got command '{name}' from user '{}'", ctx.author().name);

    // let mut counter = ctx.data().command_counter.lock().await;
    // let entry = counter.entry(name).or_insert(0);
    // *entry += 1;
    // tracing::info!("Command used {} time(s)", *entry);
}

pub async fn post_command(_ctx: Context<'_>) {
    //tracing::info!("Finished command '{}'", ctx.command().qualified_name);
}

pub async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    match error {
        poise::FrameworkError::Setup { error, .. } => {
            panic!("Failed to start bot: {error:?}");
        }
        poise::FrameworkError::Command { error, ctx, .. } => {
            tracing::error!(
                "Error in command '{}': {error:?}",
                ctx.command().qualified_name
            );
            let reply = CreateReply::default()
                .content(format!("{error}"))
                .ephemeral(true);
            if let Err(e) = ctx.send(reply).await {
                tracing::error!("Failed to send error reply: {e}");
            }
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                tracing::error!("Error while handling error: {e}");
            }
        }
    }
}
