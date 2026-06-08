use std::fmt::Write as _;

use ::serenity::builder::CreateEmbed;
use ::serenity::builder::CreateEmbedAuthor;
use ::serenity::model::Timestamp;
use poise::CreateReply;

use crate::api;
use crate::config::EndpointsConfig;
use crate::types::{Context, Error};
use crate::utils::commafy;
use crate::utils::pct;

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
#[poise::command(
    slash_command,
    guild_only,
    subcommands("status", "stats"),
    subcommand_required
)]
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

    let embed = CreateEmbed::new()
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
        .timestamp(Timestamp::now());

    ctx.send(CreateReply::default().embed(embed)).await?;
    Ok(())
}

/// Shows the public website statistics.
#[poise::command(slash_command, guild_only)]
pub async fn stats(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;
    let data = ctx.data();
    let target = Target::Local;
    let base_url = target.backend(&data.config.endpoints);

    let Ok(s) = api::stats::stats(&data.http_client, base_url).await else {
        let embed = CreateEmbed::new()
            .author(CreateEmbedAuthor::new("myrtle.moe").url("https://myrtle.moe"))
            .title("Stats unavailable")
            .description(format!(
                "Couldn't reach the **{}** backend. Please try again shortly.",
                target.label()
            ))
            .colour(COLOR_BAD)
            .timestamp(Timestamp::now());
        ctx.send(CreateReply::default().embed(embed)).await?;
        return Ok(());
    };

    let by_server = {
        let rows = [
            ("EN", s.users.by_server.en),
            ("JP", s.users.by_server.jp),
            ("KR", s.users.by_server.kr),
            ("CN", s.users.by_server.cn),
            ("BILI", s.users.by_server.bili),
            ("TW", s.users.by_server.tw),
        ];
        let mut t = format!("```\n{:<6}{:>11}\n", "Server", "Players");
        for (name, count) in rows {
            let _ = writeln!(t, "{name:<6}{:>11}", commafy(count));
        }
        t.push_str("```");
        t
    };

    let game_data = {
        let g = &s.game_data;
        let rows = [
            ("Operators", g.operators),
            ("Skills", g.skills),
            ("Modules", g.modules),
            ("Skins", g.skins),
            ("Stages", g.stages),
            ("Zones", g.zones),
            ("Enemies", g.enemies),
        ];
        let mut t = String::from("```\n");
        for (name, count) in rows {
            let _ = writeln!(t, "{name:<10}{:>9}", commafy(count));
        }
        t.push_str("```");
        t
    };

    let embed = CreateEmbed::new()
        .author(CreateEmbedAuthor::new("myrtle.moe").url("https://myrtle.moe"))
        .title("Statistics")
        .colour(COLOR_OK)
        .field(
            "Users",
            format!(
                "**{}** total\n`{}` public profiles\n`+{}` this week\n`+{}` this month",
                commafy(s.users.total),
                commafy(s.users.public_profiles),
                commafy(s.users.signups7d),
                commafy(s.users.signups30d),
            ),
            true,
        )
        .field(
            "Gacha",
            format!(
                "**{}** pulls · {} contributors\n★6: {} (`{}%`)\n★5: {} (`{}%`)\n★4: {} (`{}%`)",
                commafy(s.gacha.total_pulls),
                commafy(s.gacha.contributing_users),
                commafy(s.gacha.six_star_count),
                pct(s.gacha.six_star_count, s.gacha.total_pulls),
                commafy(s.gacha.five_star_count),
                pct(s.gacha.five_star_count, s.gacha.total_pulls),
                commafy(s.gacha.four_star_count),
                pct(s.gacha.four_star_count, s.gacha.total_pulls),
            ),
            true,
        )
        .field(
            "Tier Lists & Rosters",
            format!(
                "**{}** tier lists (`{}` active)\n`{}` versions · `{}` placements\n**{}** rosters",
                commafy(s.tier_lists.total),
                commafy(s.tier_lists.active),
                commafy(s.tier_lists.total_versions),
                commafy(s.tier_lists.total_placements),
                commafy(s.rosters.total),
            ),
            true,
        )
        .field("Registrations by server", by_server, false)
        .field("Game database", game_data, false)
        .timestamp(Timestamp::now());

    ctx.send(CreateReply::default().embed(embed)).await?;
    Ok(())
}

fn format_endpoint(s: &api::status::EndpointStatus) -> String {
    if s.reachable {
        format!(
            "**OK {}** - {} ms\n<{}>",
            s.status_code.unwrap_or(0),
            s.response_time_ms.unwrap_or(0),
            s.url,
        )
    } else {
        format!("**unreachable**\n<{}>", s.url)
    }
}
