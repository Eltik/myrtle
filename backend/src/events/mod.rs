use tokio::sync::broadcast;

use crate::core::authentication::constants::{AuthSession, Server};

pub mod setup_event_listeners;

/// Asset mapping statistics for logging
#[derive(Debug, Clone, Default)]
pub struct AssetStats {
    pub avatars: usize,
    pub skin_portraits: usize,
    pub module_big: usize,
    pub module_small: usize,
    pub skill_icons: usize,
    pub skill_icons_alt: usize,
    pub portraits: usize,
    pub chararts: usize,
    pub item_icons: usize,
    pub item_icons_arts: usize,
    pub enemy_icons: usize,
}

/// Game data statistics for logging
#[derive(Debug, Clone, Default)]
pub struct GameDataStats {
    pub operators: usize,
    pub skills: usize,
    pub modules: usize,
    pub skins: usize,
    pub items: usize,
    pub handbook_entries: usize,
    pub chibis: usize,
    pub ranges: usize,
    pub gacha_pools: usize,
    pub voices: usize,
    pub zones: usize,
    pub stages: usize,
    pub enemies: usize,
}

/// Chibi data statistics
#[derive(Debug, Clone, Default)]
pub struct ChibiStats {
    pub operators: usize,
    pub chararts_processed: usize,
    pub skinpack_processed: usize,
    pub dynchars_processed: usize,
}

#[derive(Debug, Clone)]
pub enum ConfigEvent {
    // Server lifecycle
    ServerStarting,
    ServerStarted {
        port: u16,
    },

    // Configuration loading
    ConfigLoadStarted,
    ConfigLoadComplete,
    ConfigReloaded,

    // Network configuration
    NetworkLoaded(Server),
    NetworkInitiated,
    NetworkLoadError {
        server: Server,
        error: String,
    },
    NetworkTimeout {
        server: Server,
    },

    // Version configuration
    VersionLoaded(Server),
    VersionInitiated,
    VersionLoadError {
        server: Server,
        error: String,
    },
    VersionTimeout {
        server: Server,
    },

    // Device IDs
    DeviceIdsGenerated,

    // Database
    DatabaseConnecting,
    DatabaseConnected,
    DatabaseTablesInitialized,

    // Redis
    RedisConnecting,
    RedisConnected,

    // Asset loading
    AssetScanStarted {
        directory: String,
    },
    AssetScanComplete {
        directory: String,
        count: usize,
    },
    AssetMappingsBuilt(AssetStats),

    // Game data loading
    GameDataLoadStarted {
        data_dir: String,
        assets_dir: String,
    },
    GameDataTableLoaded {
        table: String,
    },
    GameDataTableWarning {
        table: String,
        error: String,
    },
    GameDataEnrichmentStarted,
    GameDataEnrichmentComplete,
    GameDataLoaded(GameDataStats),
    GameDataEmpty,

    // Chibi data
    ChibiDataLoaded(ChibiStats),

    // Authentication
    AuthLoginSuccess(AuthSession),
    AuthLoginError(String),

    // User database operations
    DatabaseUserCreated {
        uid: String,
        server: String,
    },
    DatabaseUserUpdated {
        uid: String,
        server: String,
    },

    // Asset source (S3)
    AssetSourceError(String),
    AssetSourceS3Enabled {
        mode: String,
    },
}

pub struct EventEmitter {
    sender: broadcast::Sender<ConfigEvent>,
}

impl Default for EventEmitter {
    fn default() -> Self {
        Self::new()
    }
}

impl EventEmitter {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(64);
        Self { sender }
    }

    pub fn emit(&self, event: ConfigEvent) {
        let _ = self.sender.send(event); // Ignore if no receivers
    }

    pub fn subscribe(&self) -> broadcast::Receiver<ConfigEvent> {
        self.sender.subscribe()
    }
}

/// Logging utility functions with colors
pub mod log {
    use colored::Colorize;

    /// Print a section header
    pub fn section(title: &str) {
        println!("\n{}", format!("━━━ {title} ━━━").cyan().bold());
    }

    /// Print an info message
    pub fn info(message: &str) {
        println!("{} {}", "│".dimmed(), message);
    }

    /// Print a success message
    pub fn success(message: &str) {
        println!("{} {} {}", "│".dimmed(), "✓".green().bold(), message);
    }

    /// Print a warning message
    pub fn warn(message: &str) {
        eprintln!(
            "{} {} {}",
            "│".dimmed(),
            "⚠".yellow().bold(),
            message.yellow()
        );
    }

    /// Print an error message
    pub fn error(message: &str) {
        eprintln!("{} {} {}", "│".dimmed(), "✗".red().bold(), message.red());
    }

    /// Print a key-value pair
    pub fn kv(key: &str, value: &str) {
        println!("{} {} {}", "│".dimmed(), format!("{key}:").dimmed(), value);
    }

    /// Print a stat line (key: number)
    pub fn stat(key: &str, value: usize) {
        println!(
            "{} {} {}",
            "│".dimmed(),
            format!("{key}:").dimmed(),
            value.to_string().cyan()
        );
    }

    /// Print multiple stats in a compact format
    pub fn stats_inline(items: &[(&str, usize)]) {
        let formatted: Vec<String> = items
            .iter()
            .map(|(k, v)| format!("{} {}", k.dimmed(), v.to_string().cyan()))
            .collect();
        println!("{} {}", "│".dimmed(), formatted.join(" │ "));
    }

    /// Print a completion message with counts
    pub fn complete(message: &str) {
        println!("{} {}", "│".dimmed(), message.green());
    }

    /// Print a loading/processing indicator
    pub fn loading(message: &str) {
        println!("{} {} {}", "│".dimmed(), "→".blue(), message);
    }

    /// Print a server label
    pub fn server_status(server: &str, status: &str, success: bool) {
        let status_formatted = if success {
            status.green()
        } else {
            status.red()
        };
        println!(
            "{} {} {}",
            "│".dimmed(),
            format!("[{server}]").cyan(),
            status_formatted
        );
    }
}
