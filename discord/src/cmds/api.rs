use base64::Engine as _;
use resvg::tiny_skia;
use resvg::usvg::{Options, Tree};
use std::fmt::Write as _;

use ::serenity::builder::CreateEmbed;
use ::serenity::builder::CreateEmbedAuthor;
use ::serenity::model::Timestamp;
use poise::CreateReply;
use serenity::builder::CreateAttachment;

use crate::api;
use crate::api::user::UserProfile;
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
    subcommands("status", "stats", "user"),
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

/// Fetches information about an user
#[poise::command(slash_command, guild_only)]
pub async fn user(
    ctx: Context<'_>,
    #[description = "User ID to check"] user_id: String,
) -> Result<(), Error> {
    ctx.defer().await?;
    let data = ctx.data();
    let target = Target::Local;
    let base_url = target.backend(&data.config.endpoints);

    let s = match api::user::user(&data.http_client, base_url, &user_id).await {
        Ok(s) => s,
        Err(err) => {
            let embed = CreateEmbed::new()
                .author(CreateEmbedAuthor::new("myrtle.moe").url("https://myrtle.moe"))
                .title("User unavailable")
                .description(format!(
                    "Couldn't reach the **{}** backend.\n\nError: `{}`",
                    target.label(),
                    err,
                ))
                .colour(COLOR_BAD)
                .timestamp(Timestamp::now());

            ctx.send(CreateReply::default().embed(embed)).await?;
            return Ok(());
        }
    };

    let png_bytes = build_profile_card_png(&data.http_client, base_url, &s).await?;
    let embed = CreateEmbed::new()
        .author(CreateEmbedAuthor::new("myrtle.moe").url("https://myrtle.moe"))
        .title(format!("User {user_id}"))
        .image("attachment://profile.png")
        .timestamp(Timestamp::now());

    let attachment = CreateAttachment::bytes(png_bytes, "profile.png");
    ctx.send(CreateReply::default().embed(embed).attachment(attachment))
        .await?;

    Ok(())
}

async fn build_profile_card_png(
    client: &reqwest::Client,
    api_base: &str,
    user: &UserProfile,
) -> Result<Vec<u8>, Error> {
    let avatar_href = fetch_avatar_data_url(
        client,
        resolve_avatar_url(api_base, user.avatar_id.as_deref()),
    )
    .await;
    let svg = build_profile_svg(user, avatar_href.as_deref()); // build your card SVG string here

    let mut options = Options::default();
    options.fontdb_mut().load_system_fonts();
    let tree = Tree::from_str(&svg, &options)?;
    let size = tree.size().to_int_size();
    let mut pixmap =
        tiny_skia::Pixmap::new(size.width(), size.height()).ok_or("failed to allocate pixmap")?;

    resvg::render(
        &tree,
        tiny_skia::Transform::identity(),
        &mut pixmap.as_mut(),
    );
    let png = pixmap.encode_png()?;
    Ok(png)
}

fn resolve_avatar_url(api_base: &str, avatar_id: Option<&str>) -> Option<String> {
    let id = avatar_id?.trim();
    if id.is_empty() {
        return None;
    }
    if id.starts_with("http://") || id.starts_with("https://") {
        return Some(id.to_string());
    }

    let mut url = reqwest::Url::parse(api_base).ok()?;
    {
        let mut segs = url.path_segments_mut().ok()?;
        segs.pop_if_empty();
        segs.push("api");
        segs.push("avatar");
        segs.push(id);
    }
    Some(url.into())
}

async fn fetch_avatar_data_url(client: &reqwest::Client, url: Option<String>) -> Option<String> {
    let url = url?;
    let res = client.get(url).send().await.ok()?;
    if !res.status().is_success() {
        return None;
    }
    let mime = res
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(';').next())
        .filter(|value| {
            matches!(
                *value,
                "image/jpeg" | "image/jpg" | "image/png" | "image/gif" | "image/webp"
            )
        })
        .unwrap_or("image/png")
        .to_owned();
    let bytes = res.bytes().await.ok()?;
    let b64 = base64::engine::general_purpose::STANDARD.encode(bytes);
    Some(format!("data:{mime};base64,{b64}"))
}

// This builds one SVG template out of many display-only numeric fields; splitting it
// apart would scatter the markup without making anything clearer, and the casts below
// are all on values already clamped/rounded to fit their target range (grades, levels,
// UI pixel sizes) — precision loss / truncation / wrap there is expected, not a bug.
#[allow(
    clippy::too_many_lines,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_possible_wrap
)]
fn build_profile_svg(user: &UserProfile, avatar_href: Option<&str>) -> String {
    const WIDTH: i32 = 1200;
    const HEIGHT: i32 = 520;
    const MAX_LEVEL: f64 = 120.0;

    const SURFACE: &str = "#0d0d10";
    const BG_BOTTOM: &str = "#070709";
    const BORDER: &str = "#2d2d32";
    const TILE: &str = "#17171a";
    const ACCENT: &str = "#ff847d";
    const ACCENT_SOFT: &str = "rgba(255,132,125,0.14)";
    const TEXT: &str = "#f5f5f8";
    const MUTED: &str = "#8e8e96";
    const FAINT: &str = "#5d5d64";
    const TRACK: &str = "rgba(255,255,255,0.08)";
    const ONLINE: &str = "#20c45f";
    const IDLE: &str = "#e0a63b";
    const FONT: &str = "'Inter', 'Inter Variable', 'Segoe UI', system-ui, sans-serif";

    let uid = user.uid.trim();
    let nickname = user.nickname.as_deref().unwrap_or("").trim();
    let nickname = if nickname.is_empty() {
        format!("Doctor_{uid}")
    } else {
        nickname.to_string()
    };
    let nick_number = user
        .nick_number
        .as_deref()
        .map(|s| format!("#{s}"))
        .unwrap_or_default();
    let role = if user.role.trim().is_empty() {
        "Doctor"
    } else {
        user.role.as_str()
    };
    let server = if user.server.trim().is_empty() {
        "—"
    } else {
        user.server.as_str()
    };
    let level = i32::from(user.level.unwrap_or(0).max(0));
    let total_score = user.total_score.unwrap_or(0.0).max(0.0).round() as i64;
    let grade = user.grade.as_deref().unwrap_or("").trim().to_uppercase();
    let resume = user.resume.as_deref().unwrap_or("No signature set.").trim();

    let sanity = i64::from(user.sanity.unwrap_or(0).max(0));
    let max_sanity = i64::from(user.max_sanity.unwrap_or(0).max(0));
    let operator_count = user.operator_count.unwrap_or(0).max(0);
    let skin_count = user.skin_count.unwrap_or(0).max(0);
    let non_default_skin_count = user.non_default_skin_count.unwrap_or(0).max(0);
    let item_count = user.item_count.unwrap_or(0).max(0);
    let signin = i64::from(user.cumulative_signin.unwrap_or(0).max(0));
    let orundum = i64::from(user.orundum.unwrap_or(0).max(0));
    let lmd = i64::from(user.lmd.unwrap_or(0).max(0));

    let esc = |s: &str| {
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&apos;")
    };
    let trunc = |s: &str, max: usize| {
        let c = s.chars().collect::<Vec<_>>();
        if c.len() > max {
            c[..max - 1].iter().collect::<String>() + "…"
        } else {
            s.to_string()
        }
    };
    let comma = |n: i64| {
        let s = n.to_string();
        let mut out = String::new();
        for (i, ch) in s.chars().rev().enumerate() {
            if i != 0 && i % 3 == 0 {
                out.push(',');
            }
            out.push(ch);
        }
        out.chars().rev().collect::<String>()
    };
    let compact = |n: i64| match n {
        0..=999 => n.to_string(),
        1_000..=999_999 => format!("{:.1}K", n as f64 / 1_000.0),
        1_000_000..=999_999_999 => format!("{:.1}M", n as f64 / 1_000_000.0),
        _ => format!("{:.1}B", n as f64 / 1_000_000_000.0),
    };
    let relative = |ts: Option<i64>| {
        let Some(ts) = ts else {
            return "unknown".to_string();
        };
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .ok()
            .map_or(0, |d| d.as_secs() as i64);
        let d = now - ts;
        if d <= 0 {
            "just now".to_string()
        } else if d < 3600 {
            format!("{}m ago", d / 60)
        } else if d < 86_400 {
            format!("{}h ago", d / 3600)
        } else if d < 2_592_000 {
            format!("{}d ago", d / 86_400)
        } else if d < 31_536_000 {
            format!("{}mo ago", d / 2_592_000)
        } else {
            format!("{}y ago", d / 31_536_000)
        }
    };
    let presence = {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .ok()
            .map_or(0, |d| d.as_secs() as i64);
        match user.last_online_ts {
            Some(ts) if ts > 0 => {
                let m = (now - ts) / 60;
                if m <= 10 {
                    ONLINE
                } else if m <= 1440 {
                    IDLE
                } else {
                    FAINT
                }
            }
            _ => FAINT,
        }
    };
    // `S` is spelled out explicitly (even though it matches the `_` fallback) so the
    // rank->color mapping stays readable as a complete table, not S-falls-through-by-accident.
    #[allow(clippy::match_same_arms)]
    let grade_color = match grade.chars().next() {
        Some('S') => ACCENT,
        Some('A') => "#4ade80",
        Some('B') => "#38bdf8",
        Some('C' | 'D') => "#94a3b8",
        _ => ACCENT,
    };

    let id_x = 248;
    let content_right = WIDTH - 56;
    let usable_w = WIDTH - 112;
    let level_ratio = (f64::from(level) / MAX_LEVEL).clamp(0.0, 1.0);
    let fill_w = (f64::from(usable_w) * level_ratio)
        .round()
        .max(if level > 0 { 12.0 } else { 0.0 }) as i32;
    let at_max = f64::from(level) >= MAX_LEVEL;
    let level_right = if at_max {
        "MAX".to_string()
    } else {
        format!("Lv {level} / {}", MAX_LEVEL as i32)
    };

    let avatar_image = avatar_href.map_or_else(|| r##"<rect x="56" y="84" width="152" height="152" rx="20" fill="#17171a"/>"##.to_string(), |h| format!(r#"<image href="{}" x="56" y="84" width="152" height="152" preserveAspectRatio="xMidYMid slice" clip-path="url(#avatarClip)"/>"#, esc(h)));

    let grade_badge = if grade.is_empty() {
        String::new()
    } else {
        format!(
            r#"<g>
  <rect x="{}" y="86" width="56" height="56" rx="14" fill="{}" stroke="{}" stroke-width="1.5"/>
  <text x="{}" y="126" text-anchor="middle" fill="{}" font-size="30" font-weight="800" font-family="{}">{}</text>
</g>"#,
            content_right - 56,
            ACCENT_SOFT,
            grade_color,
            content_right - 28,
            grade_color,
            FONT,
            esc(&grade)
        )
    };
    let score_right = if grade.is_empty() {
        content_right
    } else {
        content_right - 72
    };

    let footer = format!(
        "Registered {}   ·   Last online {}   ·   Sanity {} / {}",
        user.register_ts
            .map_or_else(|| "—".to_string(), |v| v.to_string()),
        relative(user.last_online_ts),
        comma(sanity),
        comma(max_sanity)
    );

    format!(
        r#"<svg width="{WIDTH}" height="{HEIGHT}" viewBox="0 0 {WIDTH} {HEIGHT}" xmlns="http://www.w3.org/2000/svg">
  <defs>
    <clipPath id="avatarClip"><rect x="56" y="84" width="152" height="152" rx="20"/></clipPath>
  </defs>

  <rect x="0" y="0" width="{WIDTH}" height="{HEIGHT}" rx="28" fill="{SURFACE}"/>
  <rect x="0.75" y="0.75" width="{w1}" height="{h1}" rx="27.5" fill="none" stroke="{BORDER}"/>
  <rect x="28" y="0" width="{w2}" height="3" fill="{ACCENT}"/>

  <g>
    <rect x="56" y="34" width="10" height="10" rx="3" fill="{ACCENT}"/>
    <text x="74" y="44" fill="{TEXT}" font-size="16" font-weight="700" font-family="{FONT}">myrtle.moe<tspan fill="{FAINT}" font-weight="500"> · operator profile</tspan></text>
  </g>
  <text x="{content_right}" y="44" text-anchor="end" fill="{FAINT}" font-size="13" font-weight="500" letter-spacing="0.5" font-family="{FONT}">UID {uid}</text>

  <rect x="53" y="81" width="158" height="158" rx="23" fill="{TILE}" stroke="{ACCENT}" stroke-width="2"/>
  {avatar_image}

  <circle cx="194" cy="222" r="14" fill="{BG_BOTTOM}"/>
  <circle cx="194" cy="222" r="8" fill="{presence}"/>

  <text x="{id_x}" y="176" fill="{TEXT}" font-size="50" font-weight="800" font-family="{FONT}">{name}<tspan fill="{FAINT}" font-size="30" font-weight="600"> {nick}</tspan></text>
  <text x="{id_x}" y="210" fill="{MUTED}" font-size="19" font-style="italic" font-family="{FONT}">{resume}</text>

  <text x="{score_right}" y="76" text-anchor="end" fill="{MUTED}" font-size="12" font-weight="600" letter-spacing="1.5" font-family="{FONT}">TOTAL SCORE</text>
  <text x="{score_right}" y="122" text-anchor="end" fill="{TEXT}" font-size="40" font-weight="800" font-family="{FONT}">{total_score}</text>
  {grade_badge}

  <line x1="56" y1="270" x2="{content_right}" y2="270" stroke="{BORDER}" stroke-width="1"/>

  <text x="56" y="300" fill="{MUTED}" font-size="13" font-weight="600" letter-spacing="1" font-family="{FONT}">LEVEL PROGRESS</text>
  <text x="{content_right}" y="300" text-anchor="end" fill="{lvl_color}" font-size="14" font-weight="600" font-family="{FONT}">{level_right}</text>
  <rect x="56" y="312" width="{usable_w}" height="10" rx="5" fill="{TRACK}"/>
  <rect x="56" y="312" width="{fill_w}" height="10" rx="5" fill="{ACCENT}"/>

  <text x="56" y="346" fill="{FAINT}" font-size="13" font-family="{FONT}">{footer}</text>

  <text x="74" y="414" fill="{MUTED}" font-size="12" font-weight="600" letter-spacing="1" font-family="{FONT}">OPERATORS</text>
  <text x="74" y="452" fill="{ACCENT}" font-size="30" font-weight="700" font-family="{FONT}">{operators}</text>

  <text x="270" y="414" fill="{MUTED}" font-size="12" font-weight="600" letter-spacing="1" font-family="{FONT}">SKINS</text>
  <text x="270" y="452" fill="{TEXT}" font-size="30" font-weight="700" font-family="{FONT}">{skins}</text>
  <text x="270" y="474" fill="{FAINT}" font-size="12" font-family="{FONT}">{nonskins} non-default</text>

  <text x="466" y="414" fill="{MUTED}" font-size="12" font-weight="600" letter-spacing="1" font-family="{FONT}">ITEMS</text>
  <text x="466" y="452" fill="{TEXT}" font-size="30" font-weight="700" font-family="{FONT}">{items}</text>

  <text x="662" y="414" fill="{MUTED}" font-size="12" font-weight="600" letter-spacing="1" font-family="{FONT}">SIGN-IN DAYS</text>
  <text x="662" y="452" fill="{TEXT}" font-size="30" font-weight="700" font-family="{FONT}">{signin}</text>

  <text x="858" y="414" fill="{MUTED}" font-size="12" font-weight="600" letter-spacing="1" font-family="{FONT}">ORUNDUM</text>
  <text x="858" y="452" fill="{TEXT}" font-size="30" font-weight="700" font-family="{FONT}">{orundum}</text>

  <text x="1054" y="414" fill="{MUTED}" font-size="12" font-weight="600" letter-spacing="1" font-family="{FONT}">LMD</text>
  <text x="1054" y="452" fill="{TEXT}" font-size="30" font-weight="700" font-family="{FONT}">{lmd}</text>

  <text x="{id_x}" y="126" fill="{ACCENT}" font-size="14" font-weight="600" font-family="{FONT}">Lv {level}</text>
  <text x="{id_x}" y="146" fill="{MUTED}" font-size="14" font-weight="600" font-family="{FONT}">{role} · Server {server}</text>
</svg>"#,
        w1 = WIDTH as f32 - 1.5,
        h1 = HEIGHT as f32 - 1.5,
        w2 = WIDTH - 56,
        uid = esc(uid),
        avatar_image = avatar_image,
        presence = presence,
        id_x = id_x,
        name = esc(&trunc(&nickname, 20)),
        nick = esc(&nick_number),
        resume = esc(&trunc(resume, 74)),
        total_score = comma(total_score),
        grade_badge = grade_badge,
        score_right = score_right,
        content_right = content_right,
        lvl_color = if at_max { ACCENT } else { TEXT },
        level_right = esc(&level_right),
        fill_w = fill_w,
        usable_w = usable_w,
        footer = esc(&footer),
        operators = comma(operator_count),
        skins = comma(skin_count),
        nonskins = comma(non_default_skin_count),
        items = comma(item_count),
        signin = comma(signin),
        orundum = compact(orundum),
        lmd = compact(lmd),
        role = esc(role),
        server = esc(server),
    )
}
