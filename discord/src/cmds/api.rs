use poise::CreateReply;
use poise::serenity_prelude as serenity;

use crate::api;
use crate::config::EndpointsConfig;
use crate::types::{Context, Error};

// Standard Discord brand hex colors; keep them in conventional `0xRRGGBB` form.
#[allow(clippy::unreadable_literal)]
const COLOR_OK: u32 = 0x57F287;
#[allow(clippy::unreadable_literal)]
const COLOR_WARN: u32 = 0xFEE75C;
#[allow(clippy::unreadable_literal)]
const COLOR_BAD: u32 = 0xED4245;

/// Which backend to query for subcommands that hit a single backend (health, stats, leaderboard).
#[derive(Debug, Clone, Copy, poise::ChoiceParameter)]
pub enum Target {
    #[name = "local"]
    Local,
    #[name = "public"]
    Public,
}

impl Target {
    #[must_use]
    pub fn backend(self, e: &EndpointsConfig) -> &str {
        match self {
            Self::Local => &e.local_backend,
            Self::Public => &e.public_backend,
        }
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Public => "public",
        }
    }
}

/// Inspect the configured Myrtle API.
///
/// Subcommands: `status`, `health`, `stats`, `leaderboard`. The `health`, `stats`, and
/// `leaderboard` subcommands target a single backend (defaulting to the public one); pass
/// `target: local` to hit the locally-configured backend instead.
#[poise::command(slash_command, guild_only, subcommands("status"), subcommand_required)]
pub async fn api(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Ping every configured endpoint and report reachability and latency.
#[poise::command(slash_command, guild_only)]
pub async fn status(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;
    let data = ctx.data();
    let result = api::status::status(&data.http_client, &data.config.endpoints).await;

    let endpoints = [
        &result.local_backend,
        &result.local_frontend,
        &result.public_backend,
        &result.public_frontend,
    ];
    let reachable = endpoints.iter().filter(|e| e.reachable).count();
    let color = match reachable {
        n if n == endpoints.len() => COLOR_OK,
        0 => COLOR_BAD,
        _ => COLOR_WARN,
    };

    let embed = serenity::CreateEmbed::new()
        .title(format!("API status - {reachable}/{} up", endpoints.len()))
        .colour(color)
        .field(
            "Local backend",
            format_endpoint(&result.local_backend),
            false,
        )
        .field(
            "Local frontend",
            format_endpoint(&result.local_frontend),
            false,
        )
        .field(
            "Public backend",
            format_endpoint(&result.public_backend),
            false,
        )
        .field(
            "Public frontend",
            format_endpoint(&result.public_frontend),
            false,
        )
        .timestamp(serenity::Timestamp::now());

    ctx.send(CreateReply::default().embed(embed)).await?;
    Ok(())
}

fn format_endpoint(s: &api::status::EndpointStatus) -> String {
    if s.reachable {
        format!(
            "**OK {}** — {} ms\n<{}>",
            s.status_code.unwrap_or(0),
            s.response_time_ms.unwrap_or(0),
            s.url,
        )
    } else {
        format!("**unreachable**\n<{}>", s.url)
    }
}
