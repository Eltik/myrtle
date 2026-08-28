use crate::types::{Context, Data, Error};
use poise::CreateReply;

pub async fn pre_command(ctx: Context<'_>) {
    let name = ctx.command().qualified_name.clone();
    tracing::info!("Got command '{name}' from user '{}'", ctx.author().name);

    // Hold the lock only long enough to bump the counter, then log without it.
    let mut counter = ctx.data().command_counter.lock().await;
    let entry = counter.entry(name).or_insert(0);
    *entry += 1;
    let count = *entry;
    drop(counter);
    tracing::info!("Command used {count} time(s)");
}

pub async fn post_command(ctx: Context<'_>) {
    tracing::info!("Finished command '{}'", ctx.command().qualified_name);
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
        // A failed check is silent in poise's builtin handler, which leaves the interaction
        // unacknowledged ("application did not respond"). Say why instead.
        poise::FrameworkError::CommandCheckFailed { error, ctx, .. } => {
            tracing::info!(
                "Check denied '{}' for {}: {error:?}",
                ctx.command().qualified_name,
                ctx.author().name
            );
            let content = error.map_or_else(
                || "You can't use this command.".to_string(),
                |e| format!("{e}"),
            );
            let reply = CreateReply::default().content(content).ephemeral(true);
            if let Err(e) = ctx.send(reply).await {
                tracing::error!("Failed to send check-failure reply: {e}");
            }
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                tracing::error!("Error while handling error: {e}");
            }
        }
    }
}
