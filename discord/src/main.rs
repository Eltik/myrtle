use discord::{cmds, config::Config, handler, hooks, types::Data};
use dotenvy::dotenv;
use std::env;

use serenity::{all::ClientBuilder, prelude::*};

#[tokio::main]
async fn main() {
    dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "discord=info".into()),
        )
        .init();

    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let config = Config::load_default().expect("Failed to load config");
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_PRESENCES;

    let options = poise::FrameworkOptions {
        commands: cmds::all(),
        prefix_options: poise::PrefixFrameworkOptions {
            prefix: Some("-".into()),
            ..Default::default()
        },
        pre_command: |ctx| Box::pin(hooks::pre_command(ctx)),
        post_command: |ctx| Box::pin(hooks::post_command(ctx)),
        on_error: |error| Box::pin(hooks::on_error(error)),
        event_handler: |ctx, event, framework, data| {
            Box::pin(handler::event_handler(ctx, event, framework, data))
        },
        ..Default::default()
    };

    let framework = poise::Framework::builder()
        .options(options)
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                let commands = &framework.options().commands;

                if config.registration.use_guild_commands {
                    // validate() guarantees guild_id is Some when use_guild_commands is true
                    let guild_id = config
                        .registration
                        .guild_id
                        .expect("guild_id validated at config load");
                    poise::builtins::register_in_guild(ctx, commands, guild_id).await?;
                    tracing::info!(
                        "Registered {} command(s) in guild {guild_id}",
                        commands.len()
                    );
                } else {
                    poise::builtins::register_globally(ctx, commands).await?;
                    tracing::info!("Registered {} command(s) globally", commands.len());
                }

                Ok(Data {
                    command_counter: Default::default(),
                    config,
                })
            })
        })
        .build();

    let mut client = ClientBuilder::new(token, intents)
        .framework(framework)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
