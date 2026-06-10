use discord::{
    cmds,
    config::Config,
    db, handler, hooks,
    types::Data,
    watcher::{self, AssetStatus, AssetsState},
};
use dotenvy::dotenv;
use std::collections::{HashMap, HashSet};
use std::env;
use std::sync::Arc;

use serenity::{all::ClientBuilder, cache::Settings, prelude::*};
use tokio::sync::{Mutex as TokioMutex, mpsc};

// Bot bootstrap touches many subsystems (config, db, watcher, framework); splitting it
// into helpers would just scatter the wiring without making any of it clearer.
#[allow(clippy::too_many_lines)]
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
    let database_url =
        env::var("DATABASE_URL").unwrap_or_else(|_| db::DEFAULT_DATABASE_URL.to_string());
    let pool = db::init_pool(&database_url)
        .await
        .expect("Failed to initialize database");
    let http_client = reqwest::Client::builder()
        .user_agent(concat!("myrtle-discord/", env!("CARGO_PKG_VERSION")))
        .build()
        .expect("Failed to build HTTP client");
    // One state + WS watcher per configured asset server (EN, CN, ...).
    let mut assets_states: HashMap<String, Arc<AssetsState>> = HashMap::new();
    let mut assets_watchers: Vec<(String, String, mpsc::UnboundedReceiver<String>)> = Vec::new();
    for srv in config.assets.resolved_servers() {
        let (tx, rx) = mpsc::unbounded_channel::<String>();
        assets_states.insert(
            srv.label.clone(),
            Arc::new(AssetsState {
                status: RwLock::new(AssetStatus::default()),
                pending_resource_list: TokioMutex::new(None),
                tx,
            }),
        );
        assets_watchers.push((srv.label, srv.ws_url, rx));
    }
    let assets_states = Arc::new(assets_states);

    let intents = GatewayIntents::GUILDS
        | GatewayIntents::GUILD_MEMBERS
        | GatewayIntents::GUILD_MODERATION
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_PRESENCES
        | GatewayIntents::GUILD_MESSAGE_REACTIONS;

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

    let setup_states = assets_states.clone();
    let mut assets_watchers_slot = Some(assets_watchers);

    let framework = poise::Framework::builder()
        .options(options)
        .setup(move |ctx, _ready, _framework| {
            let assets_states = setup_states;
            let assets_watchers = assets_watchers_slot
                .take()
                .expect("framework setup runs once");
            Box::pin(async move {
                ctx.set_presence(
                    Some(serenity::all::ActivityData::playing("Arknights")),
                    serenity::all::OnlineStatus::Online,
                );

                let tracked: HashSet<_> = db::list_tracked_message_ids(&pool)
                    .await?
                    .into_iter()
                    .collect();

                let antispam_policies: std::collections::HashMap<_, _> =
                    db::list_antispam_policies(&pool)
                        .await?
                        .into_iter()
                        .collect();
                tracing::info!(
                    "Hydrated antispam policies for {} guild(s)",
                    antispam_policies.len()
                );

                let audit_log_channels: std::collections::HashMap<_, _> =
                    db::list_audit_log_channels(&pool)
                        .await?
                        .into_iter()
                        .collect();
                tracing::info!(
                    "Hydrated audit-log bindings for {} guild(s)",
                    audit_log_channels.len()
                );

                let reconnect_secs = config.assets.reconnect_secs;
                for (label, ws_url, rx) in assets_watchers {
                    let watcher_http = ctx.http.clone();
                    let watcher_pool = pool.clone();
                    let watcher_state = assets_states
                        .get(&label)
                        .cloned()
                        .expect("state exists for each watcher label");
                    tokio::spawn(async move {
                        watcher::run(
                            watcher_http,
                            watcher_pool,
                            label,
                            ws_url,
                            reconnect_secs,
                            watcher_state,
                            rx,
                        )
                        .await;
                    });
                }

                let ping_history = Arc::new(RwLock::new(std::collections::HashMap::new()));
                let antispam_policies = Arc::new(RwLock::new(antispam_policies));

                let sweep_history = ping_history.clone();
                let sweep_policies = antispam_policies.clone();
                tokio::spawn(async move {
                    handler::run_ping_history_sweep(sweep_history, sweep_policies).await;
                });

                Ok(Data {
                    command_counter: Mutex::default(),
                    config,
                    http_client,
                    pool,
                    tracked_messages: Arc::new(RwLock::new(tracked)),
                    assets: assets_states,
                    ping_history,
                    antispam_policies,
                    audit_log_channels: Arc::new(RwLock::new(audit_log_channels)),
                })
            })
        })
        .build();

    let mut cache_settings = Settings::default();
    cache_settings.max_messages = 1000;

    let mut client = ClientBuilder::new(token, intents)
        .framework(framework)
        .cache_settings(cache_settings)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
