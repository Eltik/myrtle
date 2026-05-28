use std::{fs, path::Path};

use serde::Deserialize;
use serenity::all::{GuildId, RoleId};

use crate::types::Error;

pub const DEFAULT_CONFIG_PATH: &str = "config.json";

/// Top-level config loaded once at startup.
///
/// All sections default to empty so partial configs still parse - validate per-section
/// where the values are actually used.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default)]
    pub registration: RegistrationConfig,
    #[serde(default)]
    pub endpoints: EndpointsConfig,
    #[serde(default)]
    pub roles: RolesConfig,
}

/// Command-registration target.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RegistrationConfig {
    /// When `true`, commands are registered in `guild_id` only (fast iteration, no global cache).
    #[serde(default)]
    pub use_guild_commands: bool,
    /// Required when `use_guild_commands` is `true`. Accepts either a JSON number or quoted string.
    #[serde(default)]
    pub guild_id: Option<GuildId>,
}

/// HTTP endpoints the status check pings.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EndpointsConfig {
    #[serde(default)]
    pub local_backend: String,
    #[serde(default)]
    pub local_frontend: String,
    #[serde(default)]
    pub public_backend: String,
    #[serde(default)]
    pub public_frontend: String,
}

/// Role-related constants. Add new role ids here as features need them.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RolesConfig {
    /// Role automatically granted to new members on join. `None` disables auto-role.
    #[serde(default)]
    pub auto_role_id: Option<RoleId>,
}

impl Config {
    /// Load and parse `path`, then validate cross-field invariants.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, Error> {
        let path = path.as_ref();
        let bytes = fs::read(path)
            .map_err(|e| format!("Failed to read config at {}: {e}", path.display()))?;
        let config: Self = serde_json::from_slice(&bytes)
            .map_err(|e| format!("Failed to parse config at {}: {e}", path.display()))?;
        config.validate()?;
        Ok(config)
    }

    /// Load from `$DISCORD_CONFIG_PATH` if set, else `config.json` in the working dir.
    pub fn load_default() -> Result<Self, Error> {
        let path =
            std::env::var("DISCORD_CONFIG_PATH").unwrap_or_else(|_| DEFAULT_CONFIG_PATH.to_string());
        Self::load(path)
    }

    fn validate(&self) -> Result<(), Error> {
        if self.registration.use_guild_commands && self.registration.guild_id.is_none() {
            return Err(
                "registration.guild_id is required when registration.use_guild_commands is true"
                    .into(),
            );
        }
        Ok(())
    }
}
