//! `/collection`: operator, enemy, stage and story lookups against the public backend.
//!
//! Each subcommand autocompletes against the cached lists in `api::gamedata` (the autocomplete
//! value is the entity id) and also accepts free text, resolved with the same matcher.

use std::collections::HashSet;
use std::fmt::Write as _;
use std::sync::Arc;

use ::serenity::builder::{CreateEmbed, CreateEmbedFooter};
use poise::CreateReply;
use poise::serenity_prelude as serenity;

use crate::api::gamedata::{Enemy, GameData, Operator, Stage, StageDetail, StoryGroup};
use crate::cmds::operator::{
    self,
    state::{Page, ViewState},
};
use crate::gametext::strip_rich_text;
use crate::search::search;
use crate::types::{Context, Error};
use crate::utils::{commafy, ellipsize};

#[allow(clippy::unreadable_literal)]
const COLOR_INFO: u32 = 0x5865F2;
#[allow(clippy::unreadable_literal)]
const COLOR_WARN: u32 = 0xFEE75C;
#[allow(clippy::unreadable_literal)]
const COLOR_BAD: u32 = 0xED4245;

/// Discord caps an autocomplete response at 25 choices of at most 100 characters each.
const AUTOCOMPLETE_MAX: usize = 25;
const CHOICE_NAME_MAX: usize = 100;
const TITLE_MAX: usize = 256;
const FIELD_MAX: usize = 1024;
/// Enemy descriptions run to a paragraph; the embed keeps the first few lines' worth.
const DESCRIPTION_MAX: usize = 400;

/// Look up Arknights game data.
///
/// Subcommands: `operator`, `enemy`, `stage`, `story`. Pick from the autocomplete list, or type
/// a name and the closest match is shown.
#[poise::command(
    slash_command,
    subcommands(
        "collection_operator",
        "collection_enemy",
        "collection_stage",
        "collection_story"
    ),
    subcommand_required
)]
pub async fn collection(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// The pages `/collection operator` can open on. Pages an operator doesn't have fall back to
/// the overview.
#[derive(Debug, Clone, Copy, poise::ChoiceParameter)]
pub enum PageChoice {
    Overview,
    Skills,
    Summons,
    Modules,
    #[name = "Base skills"]
    Base,
    #[name = "Upgrade costs"]
    Costs,
    Outfits,
    Lore,
    #[name = "Voice lines"]
    Voice,
    #[name = "Paradox Simulation"]
    Paradox,
}

impl PageChoice {
    const fn page(self) -> Page {
        match self {
            Self::Overview => Page::Overview,
            Self::Skills => Page::Skills,
            Self::Summons => Page::Summons,
            Self::Modules => Page::Modules,
            Self::Base => Page::Base,
            Self::Costs => Page::Costs,
            Self::Outfits => Page::Outfits,
            Self::Lore => Page::Lore,
            Self::Voice => Page::Voice,
            Self::Paradox => Page::Paradox,
        }
    }
}

/// Show an operator page by page: stats, skills, modules, costs, outfits, lore and more.
///
/// Pages: Overview, Skills, Summons, Modules, Base skills, Upgrade costs, Outfits, Lore, Voice
/// lines and Paradox Simulation, each listed only when the operator has it.
/// Switch pages and levels with the selects under the reply. Only the person who ran the
/// command moves the public reply; anyone else gets their own private copy.
#[poise::command(slash_command, rename = "operator")]
pub async fn collection_operator(
    ctx: Context<'_>,
    #[description = "Operator name"]
    #[autocomplete = "autocomplete_operator"]
    query: String,
    #[description = "Page to open on (default Overview)"] page: Option<PageChoice>,
) -> Result<(), Error> {
    let gamedata = begin(ctx).await?;
    // Log the reason; the reply says only that it failed, never a URL or an error chain.
    let operators = gamedata.operators().await.map_err(|e| {
        tracing::warn!("collection: operator list: {e}");
        "Couldn't load the operator list right now. Try again in a minute."
    })?;
    let op = resolve(
        &operators,
        &query,
        |o| o.id.as_str(),
        operator_names,
        operator_priority,
    )
    .ok_or_else(|| no_match("operator", &query))?;
    // The record and the page's extras are more requests: defer past the 3 s window.
    ctx.defer().await?;
    let page = page.map_or(Page::Overview, PageChoice::page);
    let state = ViewState::new(ctx.author().id.get(), &op.id, page);
    // After the public defer every failure is answered here, in public and in plain words, so
    // nothing reaches `hooks::on_error` with the deferred response still open.
    let sent = match operator::render(
        &gamedata,
        &ctx.data().operator_images,
        &ctx.data().http_client,
        frontend(ctx),
        op,
        &state,
    )
    .await
    {
        Ok(view) => ctx
            .send(view.into_reply())
            .await
            .map_err(|e| tracing::error!("collection: sending operator {}: {e}", op.id)),
        Err(e) => {
            tracing::warn!("collection: rendering operator {}: {e}", op.id);
            Err(())
        }
    };
    if sent.is_err() {
        let notice = format!(
            "Couldn't load {} right now. Try again in a minute.",
            op.name
        );
        if let Err(e) = ctx.send(CreateReply::default().content(notice)).await {
            tracing::error!("collection: operator {} failure notice: {e}", op.id);
        }
    }
    Ok(())
}

/// Show an enemy's handbook entry.
#[poise::command(slash_command, rename = "enemy")]
pub async fn collection_enemy(
    ctx: Context<'_>,
    #[description = "Enemy name"]
    #[autocomplete = "autocomplete_enemy"]
    query: String,
) -> Result<(), Error> {
    let gamedata = begin(ctx).await?;
    let enemies = gamedata.enemies().await?;
    let enemy = resolve(
        &enemies,
        &query,
        |e| e.id.as_str(),
        enemy_names,
        enemy_priority,
    )
    .ok_or_else(|| no_match("enemy", &query))?;
    let embed = enemy_embed(enemy, &gamedata, frontend(ctx));
    ctx.send(CreateReply::default().embed(embed)).await?;
    Ok(())
}

/// Show a stage: zone, sanity, enemies and drops.
#[poise::command(slash_command, rename = "stage")]
pub async fn collection_stage(
    ctx: Context<'_>,
    #[description = "Stage code or name, e.g. 1-7 or CE-6"]
    #[autocomplete = "autocomplete_stage"]
    query: String,
) -> Result<(), Error> {
    let gamedata = begin(ctx).await?;
    let stages = gamedata.stages().await?;
    let stage = resolve(
        &stages,
        &query,
        |s| s.stage_id.as_str(),
        stage_names,
        stage_priority,
    )
    .ok_or_else(|| no_match("stage", &query))?;
    // The detail (enemies, drops, danger level) is a second request: defer so it can't run the
    // interaction past Discord's 3 s window. A no-match above has already answered ephemerally.
    ctx.defer().await?;
    let detail = match gamedata.stage_detail(&stage.stage_id).await {
        Ok(d) => Some(d),
        Err(e) => {
            tracing::warn!("collection: stage detail {}: {e}", stage.stage_id);
            None
        }
    };
    let embed = stage_embed(stage, detail.as_ref(), &gamedata, frontend(ctx));
    ctx.send(CreateReply::default().embed(embed)).await?;
    Ok(())
}

/// Show a story group: an episode, event story, vignette or operator record.
#[poise::command(slash_command, rename = "story")]
pub async fn collection_story(
    ctx: Context<'_>,
    #[description = "Story, event or operator name"]
    #[autocomplete = "autocomplete_story"]
    query: String,
) -> Result<(), Error> {
    let gamedata = begin(ctx).await?;
    let stories = gamedata.stories().await?;
    let group = resolve(&stories, &query, |g| g.id.as_str(), story_names, |_| 0)
        .ok_or_else(|| no_match("story", &query))?;
    let embed = story_embed(group, &gamedata, frontend(ctx));
    ctx.send(CreateReply::default().embed(embed)).await?;
    Ok(())
}

/// Shared start of every lookup: check the backend is configured and, when a list still has to
/// be fetched for the first time, defer so the fetch can't time the interaction out.
///
/// Deferring is public, so a no-match after a cold start answers publicly too; once the lists
/// are cached (they are warmed at startup) every no-match is ephemeral.
async fn begin(ctx: Context<'_>) -> Result<Arc<GameData>, Error> {
    let gamedata = Arc::clone(&ctx.data().gamedata);
    gamedata.base()?;
    if !gamedata.is_warm().await {
        ctx.defer().await?;
    }
    Ok(gamedata)
}

fn frontend(ctx: Context<'_>) -> &str {
    ctx.data()
        .config
        .endpoints
        .public_frontend
        .trim_end_matches('/')
}

fn no_match(kind: &str, query: &str) -> Error {
    format!(
        "No {kind} matches `{}`. Try picking one from the suggestions.",
        ellipsize(query.trim(), 100)
    )
    .into()
}

/// The item whose id is exactly `query` (an autocomplete pick), else the best name match.
fn resolve<'a, T>(
    items: &'a [T],
    query: &str,
    id: impl Fn(&'a T) -> &'a str,
    names: impl Fn(&'a T) -> Vec<&'a str>,
    priority: impl Fn(&'a T) -> u8,
) -> Option<&'a T> {
    let query = query.trim();
    items
        .iter()
        .find(|item| id(item) == query)
        .or_else(|| search(items, query, 1, names, priority).into_iter().next())
}

/// Autocomplete choices for `items`: the best matches for `partial`, or the first few items
/// when nothing has been typed yet.
fn choices<'a, T>(
    items: &'a [T],
    partial: &str,
    names: impl Fn(&'a T) -> Vec<&'a str>,
    priority: impl Fn(&'a T) -> u8,
    choice: impl Fn(&T) -> (String, String),
) -> Vec<serenity::AutocompleteChoice> {
    let hits: Vec<&T> = if partial.trim().is_empty() {
        items.iter().take(AUTOCOMPLETE_MAX).collect()
    } else {
        search(items, partial, AUTOCOMPLETE_MAX, names, priority)
    };
    hits.into_iter()
        .map(|item| {
            let (label, value) = choice(item);
            serenity::AutocompleteChoice::new(ellipsize(&label, CHOICE_NAME_MAX), value)
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Operators
// ---------------------------------------------------------------------------

fn operator_names(op: &Operator) -> Vec<&str> {
    let mut names = vec![op.name.as_str()];
    names.extend(op.appellation());
    names
}

/// Obtainable operators first: "Sharp" should find the one players can recruit.
fn operator_priority(op: &Operator) -> u8 {
    u8::from(op.is_not_obtainable)
}

async fn autocomplete_operator(
    ctx: Context<'_>,
    partial: &str,
) -> Vec<serenity::AutocompleteChoice> {
    // Never waits on a fetch: an uncached list answers empty and loads in the background.
    let Some(operators) = ctx.data().gamedata.cached_operators().await else {
        return Vec::new();
    };
    choices(
        &operators,
        partial,
        operator_names,
        operator_priority,
        |op| {
            (
                match op.class() {
                    Some(class) => format!("{} ({}★ {class})", op.name, op.rarity),
                    None => format!("{} ({}★)", op.name, op.rarity),
                },
                op.id.clone(),
            )
        },
    )
}

// ---------------------------------------------------------------------------
// Enemies
// ---------------------------------------------------------------------------

fn enemy_names(enemy: &Enemy) -> Vec<&str> {
    let mut names = vec![enemy.name.as_str()];
    names.extend(enemy.index.as_deref());
    names
}

/// Entries hidden from the in-game handbook (mostly alternate forms) sort after the rest.
fn enemy_priority(enemy: &Enemy) -> u8 {
    u8::from(enemy.hide_in_handbook)
}

async fn autocomplete_enemy(ctx: Context<'_>, partial: &str) -> Vec<serenity::AutocompleteChoice> {
    // Never waits on a fetch: an uncached list answers empty and loads in the background.
    let Some(enemies) = ctx.data().gamedata.cached_enemies().await else {
        return Vec::new();
    };
    choices(&enemies, partial, enemy_names, enemy_priority, |e| {
        let mut tags: Vec<&str> = Vec::new();
        tags.extend(e.level.as_deref().map(enemy_level_label));
        tags.extend(e.index.as_deref());
        let label = if tags.is_empty() {
            e.name.clone()
        } else {
            format!("{} ({})", e.name, tags.join(", "))
        };
        (label, e.id.clone())
    })
}

fn enemy_embed(enemy: &Enemy, gamedata: &GameData, frontend: &str) -> CreateEmbed {
    let colour = match enemy.level.as_deref() {
        Some("BOSS") => COLOR_BAD,
        Some("ELITE") => COLOR_WARN,
        _ => COLOR_INFO,
    };
    let mut embed = CreateEmbed::new()
        .title(ellipsize(&enemy.name, TITLE_MAX))
        .colour(colour)
        .thumbnail(gamedata.api_url(&format!("/enemy-icon/{}", enemy.id)));
    if !frontend.is_empty() {
        embed = embed.url(format!("{frontend}/enemies/{}", enemy.id));
    }
    if let Some(description) = enemy.description.as_deref().map(str::trim)
        && !description.is_empty()
    {
        embed = embed.description(ellipsize(&strip_rich_text(description), DESCRIPTION_MAX));
    }

    if let Some(level) = enemy.level.as_deref() {
        embed = embed.field("Class", enemy_level_label(level), true);
    }
    if let Some(way) = enemy.apply_way.as_deref() {
        embed = embed.field("Attack", apply_way_label(way), true);
    }
    if !enemy.damage_types.is_empty() {
        let damage: Vec<&str> = enemy
            .damage_types
            .iter()
            .map(|d| damage_type_label(d))
            .collect();
        embed = embed.field("Damage", damage.join(" / "), true);
    }
    if let Some(s) = enemy.stats {
        embed = embed.field(
            "Stats (level 0)",
            format!(
                "HP **{}** · ATK **{}** · DEF **{}** · RES **{}**",
                number(s.hp),
                number(s.atk),
                number(s.def),
                number(s.res)
            ),
            false,
        );
    }
    let abilities = enemy
        .abilities
        .iter()
        .map(|(text, format)| {
            let text = strip_rich_text(text);
            if format == "TITLE" {
                format!("**{text}**")
            } else {
                format!("• {text}")
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    if !abilities.is_empty() {
        embed = embed.field("Abilities", field(&abilities), false);
    }
    let footer = match enemy.index.as_deref() {
        Some(index) => format!("{index} · {}", enemy.id),
        None => enemy.id.clone(),
    };
    embed.footer(CreateEmbedFooter::new(footer))
}

/// The handbook's labels, as the frontend's English catalog has them.
fn enemy_level_label(level: &str) -> &str {
    match level {
        "NORMAL" => "Normal",
        "ELITE" => "Elite",
        "BOSS" => "Boss",
        other => other,
    }
}

fn apply_way_label(way: &str) -> &str {
    match way {
        "MELEE" => "Melee",
        "RANGED" => "Ranged",
        "ALL" => "Melee and ranged",
        "NONE" => "None",
        other => other,
    }
}

fn damage_type_label(damage: &str) -> &str {
    match damage {
        "PHYSIC" => "Physical",
        "MAGIC" => "Arts",
        "HEAL" => "Heal",
        "NO_DAMAGE" => "No damage",
        other => other,
    }
}

// ---------------------------------------------------------------------------
// Stages
// ---------------------------------------------------------------------------

fn stage_names(stage: &Stage) -> Vec<&str> {
    vec![stage.code.as_str(), stage.name.as_str()]
}

/// The plain version of a stage beats its Challenge, Story and Adverse variants.
fn stage_priority(stage: &Stage) -> u8 {
    u8::from(stage.mode().is_some())
}

async fn autocomplete_stage(ctx: Context<'_>, partial: &str) -> Vec<serenity::AutocompleteChoice> {
    // Never waits on a fetch: an uncached list answers empty and loads in the background.
    let Some(stages) = ctx.data().gamedata.cached_stages().await else {
        return Vec::new();
    };
    choices(&stages, partial, stage_names, stage_priority, |s| {
        let mut label = format!("{} {}", s.code, s.name);
        for extra in [s.zone_name.as_deref(), s.mode()].into_iter().flatten() {
            let _ = write!(label, " · {extra}");
        }
        (label, s.stage_id.clone())
    })
}

fn stage_embed(
    stage: &Stage,
    detail: Option<&StageDetail>,
    gamedata: &GameData,
    frontend: &str,
) -> CreateEmbed {
    let mut embed = CreateEmbed::new()
        .title(ellipsize(
            &format!("{} {}", stage.code, stage.name),
            TITLE_MAX,
        ))
        .colour(COLOR_INFO);
    if !frontend.is_empty() {
        embed = embed.url(format!(
            "{frontend}/stages/{}",
            crate::api::gamedata::encode_path_segment(&stage.stage_id)
        ));
    }
    if let Some(preview) = stage.preview.as_deref() {
        embed = embed.image(gamedata.asset_url(preview));
    }

    // The detail's zone names ("Episode 1 · Evil Time Part 2") say more than the index's.
    let zone = detail
        .and_then(|d| d.zone.as_ref())
        .map(|z| {
            [z.zone_name_first.as_deref(), z.zone_name_second.as_deref()]
                .into_iter()
                .flatten()
                .filter(|n| !n.trim().is_empty())
                .collect::<Vec<_>>()
                .join(" · ")
        })
        .filter(|z| !z.is_empty())
        .or_else(|| stage.zone_name.clone());
    let mut description: Vec<String> = zone.into_iter().collect();
    description.extend(stage.mode().map(|m| format!("*{m}*")));
    if !description.is_empty() {
        embed = embed.description(description.join("\n"));
    }

    if let Some(cost) = stage.ap_cost {
        embed = embed.field("Sanity", cost.to_string(), true);
    }
    let Some(detail) = detail else {
        return embed.footer(CreateEmbedFooter::new(format!(
            "{} · enemies and drops unavailable",
            stage.stage_id
        )));
    };
    if let Some(danger) = detail.stage.danger_level.as_deref().map(str::trim)
        && !danger.is_empty()
        && danger != "-"
    {
        embed = embed.field("Danger", danger, true);
    }
    // A variant's extra rule, under the variant's own name ("Challenge Mode", "Extreme"...).
    if let Some(mode) = stage.mode()
        && let Some(condition) = detail
            .stage
            .description
            .as_deref()
            .and_then(stage_condition)
    {
        embed = embed.field(mode, field(&condition), false);
    }

    let mut seen = HashSet::new();
    let enemies: Vec<&str> = detail
        .level_data
        .iter()
        .flat_map(|l| l.enemy_db_refs.iter().flatten())
        .filter(|r| seen.insert(r.id.as_str()))
        .filter_map(|r| detail.enemies.get(&r.id).map(|e| e.name.as_str()))
        .collect();
    if !enemies.is_empty() {
        embed = embed.field(
            format!("Enemies ({})", enemies.len()),
            field(&enemies.join(", ")),
            false,
        );
    }

    let drops = stage_drops(detail);
    if !drops.is_empty() {
        embed = embed.field("Drops", field(&drops), false);
    }
    embed.footer(CreateEmbedFooter::new(stage.stage_id.clone()))
}

/// The extra rule a stage variant adds, pulled out of its briefing.
///
/// Every Challenge Mode, Extreme and Adverse Environment stage in the index (854 of 854)
/// carries it the same way: a header line whose text starts with "Condition" ("Condition:",
/// "Conditions:", "Environmental Conditions:"), then the rule on the following lines. The
/// header's markup varies (`<@lv.fs>`, `<@lv.mhfs>`, an empty `<@lv.fs></>` before it) and
/// some put the rule on the header line itself, so this works on the markup-stripped lines.
/// The rule ends at a time-limit line, or at a map-object line once a rule line has been read
/// (a rule may itself start with an object name, like "<Turrets> damage increased").
fn stage_condition(raw: &str) -> Option<String> {
    let mut found = false;
    let mut rule: Vec<String> = Vec::new();
    for line in raw.lines() {
        let text = strip_rich_text(line);
        let text = text.trim();
        if !found {
            let Some(at) = text.find("Condition") else {
                continue;
            };
            if at != 0 && text[..at].trim() != "Environmental" {
                continue;
            }
            found = true;
            let rest = text[at + "Condition".len()..].trim_start_matches('s');
            let rest = rest.trim_start().trim_start_matches(':').trim();
            if !rest.is_empty() {
                rule.push(rest.to_string());
            }
            continue;
        }
        let markup = line.trim_start();
        if markup.starts_with("<@act.timeLimit>") {
            break;
        }
        if !rule.is_empty()
            && (markup.starts_with("<@lv.item>") || markup.starts_with("<@lv.mhitem>"))
        {
            break;
        }
        if !text.is_empty() {
            rule.push(text.to_string());
        }
    }
    (!rule.is_empty()).then(|| rule.join("\n"))
}

/// "Regular: Orirock Cube\nExtra: Orirock, Ester", one line per drop kind that has a named
/// item. First-clear Orundum (`COMPLETE`) and anything missing from `materials` are left out.
fn stage_drops(detail: &StageDetail) -> String {
    let Some(info) = detail.stage.stage_drop_info.as_ref() else {
        return String::new();
    };
    let mut lines = Vec::new();
    for (kind, label) in [
        ("NORMAL", "Regular"),
        ("SPECIAL", "Special"),
        ("ADDITIONAL", "Extra"),
        ("ONCE", "First clear"),
    ] {
        let names: Vec<&str> = info
            .display_detail_rewards
            .iter()
            .filter(|r| r.drop_type == kind)
            .filter_map(|r| detail.materials.get(&r.id).map(|m| m.name.as_str()))
            .collect();
        if !names.is_empty() {
            lines.push(format!("{label}: {}", names.join(", ")));
        }
    }
    lines.join("\n")
}

// ---------------------------------------------------------------------------
// Stories
// ---------------------------------------------------------------------------

fn story_names(group: &StoryGroup) -> Vec<&str> {
    let mut names = vec![group.name.as_str()];
    names.extend(group.operator.as_deref());
    names
}

async fn autocomplete_story(ctx: Context<'_>, partial: &str) -> Vec<serenity::AutocompleteChoice> {
    // Never waits on a fetch: an uncached list answers empty and loads in the background.
    let Some(stories) = ctx.data().gamedata.cached_stories().await else {
        return Vec::new();
    };
    choices(
        &stories,
        partial,
        story_names,
        |_| 0,
        |g| (format!("{} ({})", g.name, story_kind(g)), g.id.clone()),
    )
}

/// "Main story", "Side story", "Vignette", or "Operator record: Ifrit".
fn story_kind(group: &StoryGroup) -> String {
    let category = match group.category.as_str() {
        "main" => "Main story",
        "side" => "Side story",
        "vignette" => "Vignette",
        "record" => "Operator record",
        other => other,
    };
    match group.operator.as_deref() {
        Some(op) => format!("{category}: {op}"),
        None => category.to_string(),
    }
}

fn story_embed(group: &StoryGroup, gamedata: &GameData, frontend: &str) -> CreateEmbed {
    let mut description = story_kind(group);
    if let Some(chapter) = group.chapter.as_deref() {
        let _ = write!(description, " · {chapter}");
    }
    let mut embed = CreateEmbed::new()
        .title(ellipsize(&group.name, TITLE_MAX))
        .description(description)
        .colour(COLOR_INFO)
        .field("Stories", group.stories.len().to_string(), true);
    if let Some(words) = group.word_count {
        let words = i32::try_from(words).map_or_else(|_| words.to_string(), commafy);
        embed = embed.field("Words", words, true);
    }
    // The reader has no page per group, so the link opens the group's first story.
    if !frontend.is_empty()
        && let Some(first) = group.stories.first()
    {
        embed = embed.url(format!(
            "{frontend}/stories/{}",
            crate::api::gamedata::encode_path_segment(&first.id)
        ));
    }
    if let Some(art) = group.banner_url.as_deref().or(group.cover_url.as_deref()) {
        embed = embed.image(gamedata.asset_url(art));
    }
    embed.footer(CreateEmbedFooter::new(group.id.clone()))
}

// ---------------------------------------------------------------------------
// Formatting
// ---------------------------------------------------------------------------

/// An embed field value: never empty, never over Discord's 1024 characters.
fn field(value: &str) -> String {
    if value.trim().is_empty() {
        return "-".to_string();
    }
    ellipsize(value, FIELD_MAX)
}

/// A stat as the handbook prints it: whole numbers without a decimal point.
fn number(n: f64) -> String {
    if n.fract() == 0.0 && n.abs() < 1e15 {
        format!("{n:.0}")
    } else {
        format!("{n:.1}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_stage_conditions() {
        // Real briefings from /api/stages/{id}/detail, `stage.description`.
        let cases = [
            (
                "<@lv.fs>Condition:</>\nDeployment Points recover at half the normal rate.",
                Some("Deployment Points recover at half the normal rate."),
            ),
            (
                "<@lv.fs></>Condition: \n<Originiutant Puppets> will spawn more \
                 <Originiutant Excrescences> when defeated.",
                Some(
                    "<Originiutant Puppets> will spawn more <Originiutant Excrescences> when \
                     defeated.",
                ),
            ),
            (
                "<@lv.mhtx>Only heroes who reach the apex will be able to pass this test.</>\n\
                 <@lv.mhfs>Condition:</><@lv.mhtx>8 tiles are no longer deployable, and more \
                 enemies will appear.</>\n",
                Some("8 tiles are no longer deployable, and more enemies will appear."),
            ),
            (
                "<@lv.fs>Condition: </>\n<@lv.item><Turrets></> damage increased.",
                Some("<Turrets> damage increased."),
            ),
            (
                "You realize that under such sight, you never had anywhere to hide. \n\
                 <@lv.item><Londinium Secondary Defense Artillery></> Once fully charged, deals \
                 area True damage.\n<@lv.fs>Environmental Conditions: </>\nSarkaz Requisitioned \
                 Engineering Drones charge the Artillery at increased speed.",
                Some(
                    "Sarkaz Requisitioned Engineering Drones charge the Artillery at increased speed.",
                ),
            ),
            (
                "<@lv.fs>Condition:</>\nBig Bob's HP is greatly increased.\n\
                 <@act.timeLimit>Stage available from 7/29 10:00 A.M. to 8/12 3:59 A.M.</>",
                Some("Big Bob's HP is greatly increased."),
            ),
            (
                "<@lv.mhfs>Condition:</>\n<@lv.mhtx>Enemies have lowered ASPD.</>\n\
                 <@lv.mhitem><Heat Pump Passage></> <@lv.mhtx>Periodically deals damage.</>",
                Some("Enemies have lowered ASPD."),
            ),
            (
                "Please combine your Operators' firepower with the Stun Generator.",
                None,
            ),
        ];
        for (raw, want) in cases {
            assert_eq!(stage_condition(raw).as_deref(), want, "{raw:?}");
        }
    }

    #[test]
    fn formats_numbers_and_labels() {
        assert_eq!(number(550.0), "550");
        assert_eq!(number(45.0), "45");
        assert_eq!(number(2.5), "2.5");
        assert_eq!(apply_way_label("ALL"), "Melee and ranged");
        assert_eq!(field(""), "-");
    }
}
