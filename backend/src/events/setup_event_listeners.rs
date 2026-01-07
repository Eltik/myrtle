use colored::Colorize;
use std::sync::Arc;

use crate::events::{AssetStats, ChibiStats, ConfigEvent, EventEmitter, GameDataStats, log};

pub fn setup_event_listeners(events: &Arc<EventEmitter>) {
    let mut rx = events.subscribe();

    tokio::spawn(async move {
        while let Ok(event) = rx.recv().await {
            match event {
                // Server lifecycle
                ConfigEvent::ServerStarting => {
                    println!(
                        "\n{}",
                        "╔══════════════════════════════════════════╗".cyan().bold()
                    );
                    println!(
                        "{}",
                        "║         🌸 Myrtle Backend Server         ║".cyan().bold()
                    );
                    println!(
                        "{}",
                        "╚══════════════════════════════════════════╝".cyan().bold()
                    );
                }
                ConfigEvent::ServerStarted { port } => {
                    log::section("Server Ready");
                    log::success(&format!("Listening on http://0.0.0.0:{port}"));
                    println!();
                }

                // Configuration loading
                ConfigEvent::ConfigLoadStarted => {
                    log::section("Configuration");
                    log::loading("Loading remote configuration...");
                }
                ConfigEvent::ConfigLoadComplete => {
                    log::success("Configuration loaded successfully");
                }
                ConfigEvent::ConfigReloaded => {
                    println!(
                        "{} {} Configuration reloaded",
                        "│".dimmed(),
                        "↻".blue().bold()
                    );
                }

                // Network configuration
                ConfigEvent::NetworkLoaded(server) => {
                    println!(
                        "{} {} {} network config loaded",
                        "│".dimmed(),
                        "✓".green(),
                        format!("{server:?}").cyan()
                    );
                }
                ConfigEvent::NetworkInitiated => {
                    log::success("All network configurations loaded");
                }
                ConfigEvent::NetworkLoadError { server, error } => {
                    log::warn(&format!("{server:?} network error: {error}"));
                }
                ConfigEvent::NetworkTimeout { server } => {
                    log::warn(&format!("{server:?} network timeout"));
                }

                // Version configuration
                ConfigEvent::VersionLoaded(server) => {
                    println!(
                        "{} {} {} version config loaded",
                        "│".dimmed(),
                        "✓".green(),
                        format!("{server:?}").cyan()
                    );
                }
                ConfigEvent::VersionInitiated => {
                    log::success("All version configurations loaded");
                }
                ConfigEvent::VersionLoadError { server, error } => {
                    log::warn(&format!("{server:?} version error: {error}"));
                }
                ConfigEvent::VersionTimeout { server } => {
                    log::warn(&format!("{server:?} version timeout"));
                }

                // Device IDs
                ConfigEvent::DeviceIdsGenerated => {
                    log::success("Device IDs generated");
                }

                // Database
                ConfigEvent::DatabaseConnecting => {
                    log::section("Database");
                    log::loading("Connecting to PostgreSQL...");
                }
                ConfigEvent::DatabaseConnected => {
                    log::success("PostgreSQL connected");
                }
                ConfigEvent::DatabaseTablesInitialized => {
                    log::success("Database tables initialized");
                }

                // Redis
                ConfigEvent::RedisConnecting => {
                    log::loading("Connecting to Redis...");
                }
                ConfigEvent::RedisConnected => {
                    log::success("Redis connected");
                }

                // Asset loading
                ConfigEvent::AssetScanStarted { directory } => {
                    log::loading(&format!("Scanning {}", directory.dimmed()));
                }
                ConfigEvent::AssetScanComplete { directory, count } => {
                    println!(
                        "{} {} {} {} assets found",
                        "│".dimmed(),
                        "✓".green(),
                        directory.dimmed(),
                        count.to_string().cyan()
                    );
                }
                ConfigEvent::AssetMappingsBuilt(stats) => {
                    log_asset_stats(&stats);
                }

                // Game data loading
                ConfigEvent::GameDataLoadStarted {
                    data_dir,
                    assets_dir,
                } => {
                    log::section("Game Data");
                    log::kv("Data directory", &data_dir);
                    log::kv("Assets directory", &assets_dir);
                }
                ConfigEvent::GameDataTableLoaded { table } => {
                    println!("{} {} {} loaded", "│".dimmed(), "✓".green(), table.dimmed());
                }
                ConfigEvent::GameDataTableWarning { table, error } => {
                    log::warn(&format!("Failed to load {table}: {error}"));
                }
                ConfigEvent::GameDataEnrichmentStarted => {
                    log::loading("Enriching game data...");
                }
                ConfigEvent::GameDataEnrichmentComplete => {
                    log::success("Data enrichment complete");
                }
                ConfigEvent::GameDataLoaded(stats) => {
                    log_game_data_stats(&stats);
                }
                ConfigEvent::GameDataEmpty => {
                    log::warn("Running with empty game data");
                }

                // Chibi data
                ConfigEvent::ChibiDataLoaded(stats) => {
                    log_chibi_stats(&stats);
                }

                // Authentication
                ConfigEvent::AuthLoginSuccess(session) => {
                    println!(
                        "{} {} Login: {}",
                        "│".dimmed(),
                        "✓".green(),
                        session.uid.cyan()
                    );
                }
                ConfigEvent::AuthLoginError(error) => {
                    log::error(&format!("Login failed: {error}"));
                }

                // User database operations
                ConfigEvent::DatabaseUserCreated { uid, server } => {
                    println!(
                        "{} {} User created: {} ({})",
                        "│".dimmed(),
                        "+".green().bold(),
                        uid.cyan(),
                        server.dimmed()
                    );
                }
                ConfigEvent::DatabaseUserUpdated { uid, server } => {
                    println!(
                        "{} {} User updated: {} ({})",
                        "│".dimmed(),
                        "~".blue(),
                        uid.cyan(),
                        server.dimmed()
                    );
                }
            }
        }
    });
}

fn log_asset_stats(stats: &AssetStats) {
    log::section("Asset Mappings");

    // Group stats into rows for cleaner display
    println!(
        "{} {} {} │ {} {} │ {} {} │ {} {}",
        "│".dimmed(),
        "Avatars".dimmed(),
        stats.avatars.to_string().cyan(),
        "Portraits".dimmed(),
        stats.portraits.to_string().cyan(),
        "Chararts".dimmed(),
        stats.chararts.to_string().cyan(),
        "Skins".dimmed(),
        stats.skin_portraits.to_string().cyan(),
    );

    println!(
        "{} {} {} │ {} {} │ {} {} │ {} {}",
        "│".dimmed(),
        "Skills".dimmed(),
        stats.skill_icons.to_string().cyan(),
        "Skills (alt)".dimmed(),
        stats.skill_icons_alt.to_string().cyan(),
        "Mod (lg)".dimmed(),
        stats.module_big.to_string().cyan(),
        "Mod (sm)".dimmed(),
        stats.module_small.to_string().cyan(),
    );

    println!(
        "{} {} {} │ {} {}",
        "│".dimmed(),
        "Items".dimmed(),
        stats.item_icons.to_string().cyan(),
        "Items (arts)".dimmed(),
        stats.item_icons_arts.to_string().cyan(),
    );

    let total = stats.avatars
        + stats.skin_portraits
        + stats.module_big
        + stats.module_small
        + stats.skill_icons
        + stats.skill_icons_alt
        + stats.portraits
        + stats.chararts
        + stats.item_icons
        + stats.item_icons_arts;

    log::success(&format!(
        "Total: {} asset mappings",
        total.to_string().cyan()
    ));
}

fn log_game_data_stats(stats: &GameDataStats) {
    log::section("Game Data Summary");

    println!(
        "{} {} {} │ {} {} │ {} {} │ {} {}",
        "│".dimmed(),
        "Operators".dimmed(),
        stats.operators.to_string().cyan(),
        "Skills".dimmed(),
        stats.skills.to_string().cyan(),
        "Modules".dimmed(),
        stats.modules.to_string().cyan(),
        "Skins".dimmed(),
        stats.skins.to_string().cyan(),
    );

    println!(
        "{} {} {} │ {} {} │ {} {} │ {} {}",
        "│".dimmed(),
        "Items".dimmed(),
        stats.items.to_string().cyan(),
        "Handbook".dimmed(),
        stats.handbook_entries.to_string().cyan(),
        "Chibis".dimmed(),
        stats.chibis.to_string().cyan(),
        "Voices".dimmed(),
        stats.voices.to_string().cyan(),
    );

    println!(
        "{} {} {} │ {} {} │ {} {} │ {} {}",
        "│".dimmed(),
        "Ranges".dimmed(),
        stats.ranges.to_string().cyan(),
        "Gacha pools".dimmed(),
        stats.gacha_pools.to_string().cyan(),
        "Zones".dimmed(),
        stats.zones.to_string().cyan(),
        "Stages".dimmed(),
        stats.stages.to_string().cyan(),
    );

    log::success("Game data loaded successfully");
}

fn log_chibi_stats(stats: &ChibiStats) {
    println!(
        "{} {} Chibi data: {} operators ({} chararts, {} skinpack, {} dynchars)",
        "│".dimmed(),
        "✓".green(),
        stats.operators.to_string().cyan(),
        stats.chararts_processed.to_string().dimmed(),
        stats.skinpack_processed.to_string().dimmed(),
        stats.dynchars_processed.to_string().dimmed(),
    );
}
