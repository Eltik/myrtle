//! The operator view's pages, built from the backend's data into [`Block`]s.
//!
//! Everything here is synchronous and pure: [`Sources`] carries every response a page reads,
//! so the whole view can be rendered offline.

use std::collections::HashMap;
use std::fmt::Write as _;

use crate::api::gamedata::{GameData, Operator, encode_path_segment};
use crate::api::operator_detail::{
    Attributes, BoardEntry, BoardEntryPascal, ItemCost, Module, ModuleItemCost, ModulePhase,
    OperatorDetail, ParadoxEntry, ParadoxStage, Range, SkillLevel, Skin, SkinData, TableSkill,
    Talent, TalentCandidate, Token, Trait, VoiceData, VoiceLine, phase_number,
};
use crate::cmds::operator::layout::{Block, Choice, capped_choices, range_grid};
use crate::cmds::operator::state::Page;
use crate::gametext::{blackboard, plain_text, render, strip_rich_text};
use crate::utils::{group_thousands, non_blank};

/// Everything a page can read. The first four are fetched for every page, the voice lines
/// for the Voice page and an operator's first view, the rest only for the page that needs
/// them (see [`Needs`]).
pub struct Sources<'a> {
    pub op: &'a Operator,
    pub detail: &'a OperatorDetail,
    pub ranges: &'a HashMap<String, Range>,
    /// This operator's Paradox Simulation, if it has one.
    pub paradox: Option<&'a ParadoxEntry>,
    /// Item names by id (Costs, the Paradox reward). Without them, items show by id.
    pub items: Option<&'a HashMap<String, String>>,
    /// The skill table (Summons).
    pub skill_table: Option<&'a HashMap<String, TableSkill>>,
    pub skins: Option<&'a SkinData>,
    pub voices: Option<&'a VoiceData>,
    /// The Paradox stage's enemies and waves, and its preview asset path.
    pub paradox_stage: Option<&'a ParadoxStage>,
    pub paradox_preview: Option<&'a str>,
    /// Builds the backend's asset URLs.
    pub urls: &'a GameData,
    /// The public frontend, without a trailing slash; empty when unconfigured.
    pub frontend: &'a str,
}

/// The responses a page needs beyond the ones every view reads.
///
/// Every view reads the record, the ranges and the Paradox index, and fetches voice lines
/// itself. One flag per response is the plain shape here; a page needs at most two of them.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Needs {
    pub items: bool,
    pub skill_table: bool,
    pub skins: bool,
    pub paradox_stage: bool,
}

#[must_use]
pub const fn needs(page: Page) -> Needs {
    let mut needs = Needs {
        items: false,
        skill_table: false,
        skins: false,
        paradox_stage: false,
    };
    match page {
        Page::Costs => needs.items = true,
        Page::Summons => needs.skill_table = true,
        Page::Outfits => needs.skins = true,
        Page::Paradox => {
            needs.paradox_stage = true;
            needs.items = true;
        }
        _ => {}
    }
    needs
}

/// A built page: its blocks (the first is the header), the page's own select, and the label
/// of what that select currently shows (for the footer).
pub struct Built {
    pub blocks: Vec<Block>,
    pub slim: Block,
    pub choices: Vec<Choice>,
    pub placeholder: &'static str,
    pub detail: Option<String>,
}

/// The pages that apply to this operator, in select order. Outfits are listed for everyone:
/// all 441 EN operators have them (2026-10-08).
#[must_use]
pub fn applicable(detail: &OperatorDetail, has_paradox: bool, has_voices: bool) -> Vec<Page> {
    Page::ALL
        .into_iter()
        .filter(|page| match page {
            Page::Overview | Page::Outfits => true,
            Page::Voice => has_voices,
            Page::Skills => detail
                .skills
                .iter()
                .any(|s| s.static_.as_ref().is_some_and(|st| !st.levels.is_empty())),
            Page::Summons => !detail.drones.is_empty(),
            Page::Modules => detail.advanced_modules().next().is_some(),
            Page::Base => detail.base_skills.as_ref().is_some_and(|b| !b.is_empty()),
            Page::Costs => !cost_kinds(detail).is_empty(),
            Page::Lore => detail
                .handbook
                .as_ref()
                .is_some_and(|h| h.story_text_audio.iter().any(|s| !s.stories.is_empty())),
            Page::Paradox => has_paradox,
        })
        .collect()
}

/// Build `page` with sub-selection `sel` (`None` for the page's default).
pub fn build(src: &Sources<'_>, page: Page, sel: Option<usize>) -> Result<Built, String> {
    match page {
        Page::Overview => Ok(overview(src, sel)),
        Page::Skills => Ok(skills(src, sel)),
        Page::Summons => summons(src, sel),
        Page::Modules => Ok(modules(src, sel)),
        Page::Base => Ok(base(src)),
        Page::Costs => Ok(costs(src, sel)),
        Page::Outfits => outfits(src, sel),
        Page::Lore => Ok(lore(src, sel)),
        Page::Voice => voice(src, sel),
        Page::Paradox => paradox(src),
    }
}

// ---------------------------------------------------------------------------
// Header
// ---------------------------------------------------------------------------

/// "★★★★★★ · Supporter · Summoner · Ranged".
fn identity_line(op: &Operator) -> String {
    let position = title_case(&op.position);
    let mut parts: Vec<&str> = [op.class(), op.branch()].into_iter().flatten().collect();
    if !position.is_empty() {
        parts.push(&position);
    }
    let stars = "★".repeat(usize::from(op.rarity));
    if parts.is_empty() {
        stars
    } else {
        format!("{stars} · {}", parts.join(" · "))
    }
}

/// The header every page opens with: name, link to the site, avatar, identity line.
fn header(src: &Sources<'_>) -> Block {
    let op = src.op;
    let title = match op.appellation() {
        Some(a) => format!("{} ({a})", op.name),
        None => op.name.clone(),
    };
    let mut block = Block::titled(title);
    if !src.frontend.is_empty() {
        block.url = Some(format!("{}/operators/{}", src.frontend, op.id));
    }
    block.thumbnail = Some(route(src, "avatar", &op.id));
    block.description = Some(identity_line(op));
    block
}

fn finish(
    src: &Sources<'_>,
    blocks: Vec<Block>,
    choices: Vec<Choice>,
    placeholder: &'static str,
    detail: Option<String>,
) -> Built {
    Built {
        blocks,
        slim: header(src),
        choices,
        placeholder,
        detail,
    }
}

// ---------------------------------------------------------------------------
// Overview
// ---------------------------------------------------------------------------

fn overview(src: &Sources<'_>, sel: Option<usize>) -> Built {
    let detail = src.detail;
    let phases = &detail.phases;
    let elite = sel
        .unwrap_or(usize::MAX)
        .min(phases.len().saturating_sub(1));
    let mut head = header(src);
    if let Some(text) = trait_text(detail.trait_.as_ref(), detail.description.as_deref()) {
        head.paragraph(text);
    }
    head.image = Some(route(src, "charart", &src.op.id));

    let phase = phases.get(elite);
    if let Some((phase, top)) = phase.and_then(|p| p.attributes_key_frames.last().map(|k| (p, k))) {
        let trust = detail
            .favor_key_frames
            .as_ref()
            .and_then(|f| f.last())
            .map(|k| k.data);
        head.paragraph(format!(
            "**Elite {elite} Lv.{}**{}",
            phase.max_level,
            if trust.is_some_and(|t| has_trust_bonus(&t)) {
                ", max trust bonus in brackets"
            } else {
                ""
            }
        ));
        stat_fields(&mut head, &top.data, trust.as_ref());
        if let Some(grid) = range_of(src.ranges, phase.range_id.as_deref()) {
            head.field("Range", grid, true);
        }
    }
    for talent in detail.talents.iter().flatten() {
        if let Some((name, value)) = talent_at(talent, elite, src.ranges) {
            head.field(name, value, false);
        }
    }
    let potentials: Vec<String> = detail
        .potential_ranks
        .iter()
        .enumerate()
        .filter_map(|(i, p)| {
            non_blank(p.description.as_deref())
                .map(|d| format!("Pot {}: {}", i + 2, strip_rich_text(d)))
        })
        .collect();
    head.field("Potentials", potentials.join("\n"), false);
    let factions = src.op.factions();
    head.field("Faction", factions.join(" · "), true);
    let tags = names_list(&src.op.tag_list);
    head.field("Tags", tags.join(", "), true);

    let choices = phases
        .iter()
        .enumerate()
        .map(|(i, p)| {
            Choice::new(
                &format!("Elite {i}"),
                i.to_string(),
                Some(&format!("Max level {}", p.max_level)),
            )
            .selected(i == elite)
        })
        .collect();
    finish(
        src,
        vec![head],
        choices,
        "Promotion",
        Some(format!("Elite {elite}")),
    )
}

const fn has_trust_bonus(t: &Attributes) -> bool {
    t.max_hp != 0.0 || t.atk != 0.0 || t.def != 0.0 || t.magic_resistance != 0.0
}

/// HP, ATK, DEF, RES (each with its trust bonus when there is one), DP cost, block, redeploy
/// and attack interval, as inline fields.
fn stat_fields(block: &mut Block, a: &Attributes, trust: Option<&Attributes>) {
    let with_trust = |base: f64, bonus: f64| {
        if bonus == 0.0 {
            number(base)
        } else {
            format!("{} (+{})", number(base + bonus), number(bonus))
        }
    };
    let t = trust.copied();
    let bonus = |f: fn(&Attributes) -> f64| t.as_ref().map_or(0.0, f);
    block.field("HP", with_trust(a.max_hp, bonus(|t| t.max_hp)), true);
    block.field("ATK", with_trust(a.atk, bonus(|t| t.atk)), true);
    block.field("DEF", with_trust(a.def, bonus(|t| t.def)), true);
    block.field(
        "RES",
        with_trust(a.magic_resistance, bonus(|t| t.magic_resistance)),
        true,
    );
    block.field("DP Cost", number(a.cost), true);
    block.field("Block", number(a.block_cnt), true);
    block.field("Redeploy", format!("{}s", number(a.respawn_time)), true);
    block.field(
        "Attack Interval",
        format!("{}s", decimal(interval(a))),
        true,
    );
}

/// The real attack interval: it scales with ASPD (100 is normal speed).
fn interval(a: &Attributes) -> f64 {
    if a.attack_speed > 0.0 {
        a.base_attack_time * 100.0 / a.attack_speed
    } else {
        a.base_attack_time
    }
}

/// The trait as the game shows it: the last candidate's override, or the class trait, filled
/// from the last candidate's blackboard.
fn trait_text(trait_: Option<&Trait>, description: Option<&str>) -> Option<String> {
    let candidate = trait_.and_then(|t| t.candidates.last());
    let template = candidate
        .and_then(|c| non_blank(c.override_description.as_deref()))
        .or_else(|| non_blank(description))?;
    let board = pascal_board(candidate.and_then(|c| c.blackboard.as_deref()));
    Some(fill(template, &board))
}

/// A talent as it stands at `elite`: the last visible candidate unlocked by then (the highest
/// potential among them). A talent not yet unlocked shows its first form and when it unlocks.
fn talent_at(
    talent: &Talent,
    elite: usize,
    ranges: &HashMap<String, Range>,
) -> Option<(String, String)> {
    let candidates: Vec<&TalentCandidate> = talent
        .candidates
        .iter()
        .flatten()
        .filter(|c| !c.is_hide_talent)
        .collect();
    let name = candidates
        .iter()
        .rev()
        .find_map(|c| non_blank(c.name.as_deref()))?;
    let unlocked = candidates
        .iter()
        .rfind(|c| c.unlock_condition.elite() <= elite);
    let (shown, note) = if let Some(c) = unlocked {
        let note = (c.required_potential_rank > 0)
            .then(|| format!(" (Pot {})", c.required_potential_rank + 1));
        (*c, note)
    } else {
        let first = candidates.first()?;
        let note = format!(
            " (unlocks at E{} Lv.{})",
            first.unlock_condition.elite(),
            first.unlock_condition.level
        );
        (*first, Some(note))
    };
    let description = non_blank(shown.description.as_deref())?;
    let mut value = fill(description, &pascal_board(shown.blackboard.as_deref()));
    if let Some(grid) = range_of(ranges, shown.range_id.as_deref()) {
        value.push('\n');
        value.push_str(&grid);
    }
    Some((format!("{name}{}", note.unwrap_or_default()), value))
}

// ---------------------------------------------------------------------------
// Skills
// ---------------------------------------------------------------------------

/// "Level 1" to "Level 7", then "Mastery 1" to "Mastery 3".
fn level_label(index: usize) -> String {
    if index < 7 {
        format!("Level {}", index + 1)
    } else {
        format!("Mastery {}", index - 6)
    }
}

fn skills(src: &Sources<'_>, sel: Option<usize>) -> Built {
    let detail = src.detail;
    let top = detail.top_skill_level();
    let level = sel.unwrap_or(top).min(top);
    let mut head = header(src);
    head.paragraph(format!("**Skills at {}**", level_label(level)));
    let mut blocks = vec![head];
    for (i, skill) in detail.skills.iter().enumerate() {
        let Some(st) = skill.static_.as_ref() else {
            continue;
        };
        let Some(own) = st.levels.get(level.min(st.levels.len().saturating_sub(1))) else {
            continue;
        };
        let clamped = level >= st.levels.len();
        let mut block = skill_block(&format!("S{}", i + 1), own, src.ranges);
        if clamped {
            block.paragraph(format!(
                "*No {}: shown at {}.*",
                level_label(level),
                level_label(st.levels.len() - 1)
            ));
        }
        block.thumbnail = Some(route(src, "skill-icon", st.icon()));
        blocks.push(block);
    }
    let choices = (0..=top)
        .rev()
        .map(|i| Choice::new(&level_label(i), i.to_string(), None).selected(i == level))
        .collect();
    finish(
        src,
        blocks,
        choices,
        "Skill level",
        Some(level_label(level)),
    )
}

/// One skill at one level: SP and activation, initial SP, cost, duration, the description,
/// and the skill's own range when it has one.
fn skill_block(slot: &str, level: &SkillLevel, ranges: &HashMap<String, Range>) -> Block {
    let mut block = Block::titled(format!("{slot} · {}", level.name.trim()));
    block.description = Some(skill_stats(level));
    let mut board = blackboard(level.blackboard.iter().map(|b| (b.key.as_str(), b.value)));
    // `{duration}` (Heavyrain's S2) reads the level's own duration, not a blackboard entry.
    board
        .entry("duration".to_string())
        .or_insert(level.duration);
    if let Some(description) = non_blank(level.description.as_deref()) {
        block.paragraph(fill(description, &board));
    }
    if let Some(grid) = range_of(ranges, level.range_id.as_deref()) {
        block.field("Range", grid, false);
    }
    block
}

/// "*Offensive recovery · Manual*\nInitial SP 0 · SP cost 30 · Duration 25s".
fn skill_stats(level: &SkillLevel) -> String {
    let recovery = match level.sp_data.sp_type.as_str() {
        "INCREASE_WITH_TIME" => Some("Auto recovery"),
        "INCREASE_WHEN_ATTACK" => Some("Offensive recovery"),
        "INCREASE_WHEN_TAKEN_DAMAGE" => Some("Defensive recovery"),
        _ => None,
    };
    let trigger = match level.skill_type.as_str() {
        "MANUAL" => "Manual trigger",
        "AUTO" => "Auto trigger",
        "PASSIVE" => "Passive",
        other => other,
    };
    let kind = recovery.map_or_else(|| trigger.to_string(), |r| format!("{r} · {trigger}"));
    let mut numbers = Vec::new();
    if recovery.is_some() {
        numbers.push(format!("Initial SP {}", level.sp_data.init_sp));
        numbers.push(format!("SP cost {}", level.sp_data.sp_cost));
    }
    if level.duration > 0.0 {
        numbers.push(format!("Duration {}s", decimal(level.duration)));
    } else if level.duration_type.as_deref() == Some("AMMO") {
        numbers.push("Ammo".to_string());
    }
    if numbers.is_empty() {
        format!("*{kind}*")
    } else {
        format!("*{kind}*\n{}", numbers.join(" · "))
    }
}

// ---------------------------------------------------------------------------
// Summons
// ---------------------------------------------------------------------------

/// The summons page's choices: each summon with each skill that brings it (Ling's three
/// dragons, one per skill), or with every skill when no skill names it (Phantom's clone
/// changes with whichever skill is equipped). `None` for an operator with no skills.
fn summon_pairs(detail: &OperatorDetail) -> Vec<(usize, Option<usize>)> {
    let mut pairs = Vec::new();
    for (t, token) in detail.drones.iter().enumerate() {
        let linked: Vec<usize> = detail
            .skills
            .iter()
            .enumerate()
            .filter(|(_, s)| s.override_token_key.as_deref() == Some(token.id.as_str()))
            .map(|(i, _)| i)
            .collect();
        if !linked.is_empty() {
            pairs.extend(linked.into_iter().map(|i| (t, Some(i))));
        } else if detail.skills.is_empty() {
            pairs.push((t, None));
        } else {
            pairs.extend((0..detail.skills.len()).map(|i| (t, Some(i))));
        }
    }
    pairs
}

fn summons(src: &Sources<'_>, sel: Option<usize>) -> Result<Built, String> {
    let detail = src.detail;
    let pairs = summon_pairs(detail);
    let pick = sel.unwrap_or(0).min(pairs.len().saturating_sub(1));
    let &(t, skill) = pairs.get(pick).ok_or("This operator has no summons.")?;
    let token = &detail.drones[t];
    let top = detail.top_skill_level();
    let mut blocks = vec![header(src)];

    if let Some(i) = skill
        && let Some(st) = detail.skills.get(i).and_then(|s| s.static_.as_ref())
        && let Some(level) = st.levels.get(top.min(st.levels.len().saturating_sub(1)))
    {
        let mut block = skill_block(&format!("S{}", i + 1), level, src.ranges);
        block.thumbnail = Some(route(src, "skill-icon", st.icon()));
        blocks.push(block);
    }
    blocks.push(token_block(src, token));

    let table = src
        .skill_table
        .ok_or("The skill table is unavailable right now.")?;
    let slot = skill.unwrap_or(0);
    if let Some(id) = token.skills.get(slot).and_then(|s| s.skill_id.as_deref()) {
        let entry = table
            .get(id)
            .ok_or_else(|| format!("The skill table has no `{id}`."))?;
        if let Some(level) = entry
            .levels
            .get(top.min(entry.levels.len().saturating_sub(1)))
        {
            let mut block = skill_block("Summon skill", level, src.ranges);
            // 90 of 747 summon skills have no icon (`image` null), and the icon route 404s
            // for them; every operator skill has one, `image` or not.
            if entry.image.is_some() {
                block.thumbnail = Some(route(src, "skill-icon", entry.icon()));
            }
            blocks.push(block);
        }
    }

    let choices = capped_choices(pairs.len(), pick, "summons", "Summon", |i| {
        let (t, skill) = pairs[i];
        let name = detail.drones[t].name.trim().to_string();
        let with = skill.and_then(|s| {
            detail.skills.get(s)?.static_.as_ref().and_then(|st| {
                st.levels
                    .first()
                    .map(|l| format!("With S{}: {}", s + 1, l.name.trim()))
            })
        });
        (name, with)
    });
    let shown = token.name.trim().to_string();
    Ok(finish(src, blocks, choices, "Summon", Some(shown)))
}

/// A summon's card: trait, stats at its top level, range, and named talents.
fn token_block(src: &Sources<'_>, token: &Token) -> Block {
    let mut block = Block::titled(token.name.trim());
    block.thumbnail = Some(route(src, "avatar", &token.id));
    if let Some(text) = trait_text(token.trait_.as_ref(), token.description.as_deref()) {
        block.paragraph(text);
    }
    if let Some(phase) = token.phases.last()
        && let Some(top) = phase.attributes_key_frames.last()
    {
        block.paragraph(format!(
            "**Elite {} Lv.{}**",
            token.phases.len() - 1,
            top.level
        ));
        stat_fields(&mut block, &top.data, None);
        if let Some(grid) = range_of(src.ranges, phase.range_id.as_deref()) {
            block.field("Range", grid, true);
        }
    }
    let elite = token.phases.len().saturating_sub(1);
    for talent in &token.talents {
        if let Some((name, value)) = talent_at(talent, elite, src.ranges) {
            block.field(name, value, false);
        }
    }
    block
}

// ---------------------------------------------------------------------------
// Modules
// ---------------------------------------------------------------------------

/// "SUM-Y · Fond Dream with Poetry". The game shows type `D` as Δ.
fn module_name(module: &Module) -> String {
    let variant = module
        .type_name2
        .as_deref()
        .map(|t| if t == "D" { "Δ" } else { t });
    let kind = match (module.type_name1.as_deref(), variant) {
        (Some(base), Some(v)) => format!("{base}-{v}"),
        (Some(base), None) => base.to_string(),
        (None, Some(v)) => v.to_string(),
        (None, None) => String::new(),
    };
    if kind.is_empty() {
        module.uni_equip_name.clone()
    } else {
        format!("{kind} · {}", module.uni_equip_name)
    }
}

fn module_stages(detail: &OperatorDetail) -> usize {
    detail
        .advanced_modules()
        .filter_map(|m| m.data.as_ref().map(|d| d.phases.len()))
        .max()
        .unwrap_or(0)
}

fn modules(src: &Sources<'_>, sel: Option<usize>) -> Built {
    let detail = src.detail;
    let stages = module_stages(detail);
    let last = stages.saturating_sub(1);
    let stage = sel.unwrap_or(last).min(last);
    let mut head = header(src);
    head.paragraph(format!("**Modules at stage {}**", stage + 1));
    let mut blocks = vec![head];
    for module in detail.advanced_modules() {
        let mut block = Block::titled(module_name(module));
        block.thumbnail = Some(route(src, "module-icon", &module.uni_equip_id));
        block.description = Some(format!(
            "*Unlocks at E{} Lv.{}*",
            phase_number(&module.unlock_evolve_phase),
            module.unlock_level
        ));
        let phase = module
            .data
            .as_ref()
            .and_then(|d| d.phases.get(stage.min(d.phases.len().saturating_sub(1))));
        match phase {
            Some(phase) => module_fields(src, &mut block, phase),
            None => block.paragraph("No stage data."),
        }
        blocks.push(block);
    }
    let choices = (0..stages)
        .map(|i| Choice::new(&format!("Stage {}", i + 1), i.to_string(), None).selected(i == stage))
        .collect();
    finish(
        src,
        blocks,
        choices,
        "Module stage",
        Some(format!("Stage {}", stage + 1)),
    )
}

fn module_fields(src: &Sources<'_>, block: &mut Block, phase: &ModulePhase) {
    block.field("Stats", stat_bonus(&phase.attribute_blackboard), false);
    for part in &phase.parts {
        if let Some(c) = part
            .override_trait_data_bundle
            .as_ref()
            .and_then(|b| b.candidates.as_ref())
            .and_then(|c| c.last())
        {
            let board = camel_board(&c.blackboard);
            if let Some(added) = non_blank(c.additional_description.as_deref()) {
                block.field("Trait", format!("+ {}", fill(added, &board)), false);
            } else if let Some(new) = non_blank(c.override_description.as_deref()) {
                block.field("New trait", fill(new, &board), false);
            }
            if let Some(grid) = range_of(src.ranges, c.range_id.as_deref()) {
                block.field("Range", grid, false);
            }
        }
        if let Some(c) = part
            .add_or_override_talent_data_bundle
            .as_ref()
            .and_then(|b| b.candidates.as_ref())
            .and_then(|c| c.last())
        {
            let board = camel_board(&c.blackboard);
            if let Some(upgrade) = non_blank(c.upgrade_description.as_deref()) {
                let label = non_blank(c.name.as_deref()).unwrap_or("Talent");
                block.field(label, fill(upgrade, &board), false);
            }
            if let Some(grid) = range_of(src.ranges, c.range_id.as_deref()) {
                block.field("Range", grid, false);
            }
        }
    }
    let summons: Vec<String> = phase
        .token_attribute_blackboard
        .iter()
        .filter(|t| !t.value.is_empty())
        .map(|t| {
            let name = src
                .detail
                .drones
                .iter()
                .find(|d| d.id == t.key)
                .map_or(t.key.as_str(), |d| d.name.trim());
            format!("{name}: {}", stat_bonus(&t.value).replace('\n', ", "))
        })
        .collect();
    block.field("Summon stats", summons.join("\n"), false);
}

/// "ATK +50\nHP +200", one bonus per line, labelled as the frontend labels them.
fn stat_bonus(board: &[BoardEntry]) -> String {
    board
        .iter()
        .map(|b| {
            let name = match b.key.as_str() {
                "atk" => "ATK".to_string(),
                "max_hp" => "HP".to_string(),
                "def" => "DEF".to_string(),
                "attack_speed" => "ASPD".to_string(),
                "magic_resistance" => "RES".to_string(),
                "cost" => "DP Cost".to_string(),
                "respawn_time" => "Redeploy".to_string(),
                "block_cnt" => "Block".to_string(),
                other => title_words(other),
            };
            let sign = if b.value >= 0.0 { "+" } else { "" };
            format!("{name} {sign}{}", decimal(b.value))
        })
        .collect::<Vec<_>>()
        .join("\n")
}

// ---------------------------------------------------------------------------
// Base skills
// ---------------------------------------------------------------------------

fn base(src: &Sources<'_>) -> Built {
    let mut head = header(src);
    head.paragraph("**Base skills**");
    let mut blocks = vec![head];
    for skill in src.detail.base_skills.iter().flatten() {
        let mut block = Block::titled(skill.buff_name.trim());
        if let Some(icon) = non_blank(skill.skill_icon.as_deref()) {
            block.thumbnail = Some(src.urls.asset_url(&format!(
                "textures/spritepack/building_ui_buff_skills_h1_0/{icon}.png"
            )));
        }
        let mut unlock = format!("E{}", skill.unlock_elite);
        if skill.unlock_level > 1 {
            let _ = write!(unlock, " Lv.{}", skill.unlock_level);
        }
        block.description = Some(format!(
            "*{} · unlocks at {unlock}*",
            room_label(&skill.room_type)
        ));
        // No base skill description carries a `{...}` template (0 of 884, 2026-10-08), so
        // there is nothing to interpolate; the markup still goes.
        if let Some(text) = non_blank(skill.description.as_deref()) {
            block.paragraph(strip_rich_text(text));
        }
        blocks.push(block);
    }
    finish(src, blocks, Vec::new(), "", None)
}

/// The in-game room names.
fn room_label(room: &str) -> &str {
    match room {
        "MANUFACTURE" => "Factory",
        "TRADING" => "Trading Post",
        "CONTROL" => "Control Center",
        "DORMITORY" => "Dormitory",
        "MEETING" => "Reception Room",
        "HIRE" => "Office",
        "TRAINING" => "Training Room",
        "WORKSHOP" => "Workshop",
        "POWER" => "Power Plant",
        other => other,
    }
}

// ---------------------------------------------------------------------------
// Costs
// ---------------------------------------------------------------------------

/// The cost views, by the number the select carries. The number is fixed per kind, so a
/// control stays valid whichever kinds an operator has.
const COST_KINDS: [&str; 4] = ["Promotions", "Skill levels", "Masteries", "Modules"];

/// The cost kinds this operator has anything for.
fn cost_kinds(detail: &OperatorDetail) -> Vec<usize> {
    let promotions = detail
        .phases
        .iter()
        .any(|p| p.evolve_cost.as_ref().is_some_and(|c| !c.is_empty()));
    let levels = detail
        .all_skill_level_up
        .iter()
        .flatten()
        .any(|l| l.lvl_up_cost.as_ref().is_some_and(|c| !c.is_empty()));
    let masteries = detail.skills.iter().any(|s| {
        s.level_up_cost_cond
            .iter()
            .flatten()
            .any(|m| m.level_up_cost.as_ref().is_some_and(|c| !c.is_empty()))
    });
    let modules = detail
        .advanced_modules()
        .any(|m| m.item_cost.as_ref().is_some_and(|c| !c.is_empty()));
    [promotions, levels, masteries, modules]
        .into_iter()
        .enumerate()
        .filter_map(|(i, has)| has.then_some(i))
        .collect()
}

fn costs(src: &Sources<'_>, sel: Option<usize>) -> Built {
    let detail = src.detail;
    let kinds = cost_kinds(detail);
    let kind = sel
        .filter(|k| kinds.contains(k))
        .or_else(|| kinds.first().copied())
        .unwrap_or(0);
    let names = src.items;
    let mut head = header(src);
    head.paragraph(format!(
        "**Upgrade costs: {}**",
        COST_KINDS[kind].to_lowercase()
    ));
    if names.is_none() {
        head.paragraph("*Item names are unavailable right now; items show by id.*");
    }
    let mut blocks = vec![head];
    match kind {
        0 => blocks.push(promotion_costs(detail, names)),
        1 => blocks.push(skill_level_costs(detail, names)),
        2 => blocks.extend(mastery_costs(src, names)),
        _ => blocks.extend(module_costs(src, names)),
    }
    let choices = kinds
        .iter()
        .map(|&k| Choice::new(COST_KINDS[k], k.to_string(), None).selected(k == kind))
        .collect();
    finish(
        src,
        blocks,
        choices,
        "Cost type",
        Some(COST_KINDS[kind].to_string()),
    )
}

/// Promotion costs per elite, then the LMD and EXP to level through each phase.
fn promotion_costs(detail: &OperatorDetail, names: Option<&HashMap<String, String>>) -> Block {
    let mut block = Block::titled("Promotions");
    for (i, phase) in detail.phases.iter().enumerate().skip(1) {
        let items = item_lines(phase.evolve_cost.iter().flatten().map(game_cost), names);
        block.field(format!("Elite {i}"), items, true);
    }
    let levelling: Vec<String> = detail
        .phases
        .iter()
        .enumerate()
        .filter_map(|(i, phase)| {
            let line = item_inline(phase.level_up_cost.iter().flatten().map(game_cost), names);
            (!line.is_empty()).then(|| format!("E{i} Lv.1 to {}: {line}", phase.max_level))
        })
        .collect();
    block.field("Levelling", levelling.join("\n"), false);
    block
}

/// The shared skill levels 2 to 7.
fn skill_level_costs(detail: &OperatorDetail, names: Option<&HashMap<String, String>>) -> Block {
    let mut block = Block::titled("Skill levels");
    for (i, level) in detail.all_skill_level_up.iter().flatten().enumerate() {
        let items = item_lines(level.lvl_up_cost.iter().flatten().map(game_cost), names);
        block.field(format!("Level {}", i + 2), items, true);
    }
    block
}

/// One block per skill with masteries.
fn mastery_costs(src: &Sources<'_>, names: Option<&HashMap<String, String>>) -> Vec<Block> {
    let mut blocks = Vec::new();
    for (i, skill) in src.detail.skills.iter().enumerate() {
        let Some(conds) = skill.level_up_cost_cond.as_ref().filter(|c| !c.is_empty()) else {
            continue;
        };
        let name = skill
            .static_
            .as_ref()
            .and_then(|st| st.levels.first())
            .map_or(skill.skill_id.as_str(), |l| l.name.trim());
        let mut block = Block::titled(format!("S{} · {name}", i + 1));
        if let Some(st) = skill.static_.as_ref() {
            block.thumbnail = Some(route(src, "skill-icon", st.icon()));
        }
        for (m, cond) in conds.iter().enumerate() {
            let items = item_lines(cond.level_up_cost.iter().flatten().map(game_cost), names);
            block.field(format!("Mastery {}", m + 1), items, true);
        }
        blocks.push(block);
    }
    blocks
}

/// One block per advanced module with costs, a field per stage.
fn module_costs(src: &Sources<'_>, names: Option<&HashMap<String, String>>) -> Vec<Block> {
    let mut blocks = Vec::new();
    for module in src.detail.advanced_modules() {
        let Some(stages) = module.item_cost.as_ref().filter(|c| !c.is_empty()) else {
            continue;
        };
        let mut block = Block::titled(module_name(module));
        block.thumbnail = Some(route(src, "module-icon", &module.uni_equip_id));
        for (stage, cost) in stages {
            let items = item_lines(cost.iter().map(module_cost), names);
            block.field(format!("Stage {stage}"), items, true);
        }
        blocks.push(block);
    }
    blocks
}

const fn game_cost(c: &ItemCost) -> (&str, u64) {
    (c.id.as_str(), c.count)
}

const fn module_cost(c: &ModuleItemCost) -> (&str, u64) {
    (c.id.as_str(), c.count)
}

/// Repeats summed, first-seen order kept, names looked up (the id when there is none).
fn totals<'a>(
    costs: impl IntoIterator<Item = (&'a str, u64)>,
    names: Option<&'a HashMap<String, String>>,
) -> Vec<(&'a str, u64)> {
    let mut totals: Vec<(&str, u64)> = Vec::new();
    for (id, count) in costs {
        match totals.iter_mut().find(|(i, _)| *i == id) {
            Some(entry) => entry.1 += count,
            None => totals.push((id, count)),
        }
    }
    totals
        .into_iter()
        .map(|(id, count)| {
            let name = names.and_then(|n| n.get(id)).map_or(id, String::as_str);
            (name, count)
        })
        .collect()
}

/// "LMD x30,000\nCaster Chip x5", one item per line.
fn item_lines<'a>(
    costs: impl IntoIterator<Item = (&'a str, u64)>,
    names: Option<&'a HashMap<String, String>>,
) -> String {
    totals(costs, names)
        .into_iter()
        .map(|(name, count)| format!("{name} x{}", group_thousands(count)))
        .collect::<Vec<_>>()
        .join("\n")
}

/// "LMD x30,000 · EXP x24,400", on one line.
fn item_inline<'a>(
    costs: impl IntoIterator<Item = (&'a str, u64)>,
    names: Option<&'a HashMap<String, String>>,
) -> String {
    item_lines(costs, names).replace('\n', " · ")
}

// ---------------------------------------------------------------------------
// Outfits
// ---------------------------------------------------------------------------

/// The operator's outfits in release order, the default arts (released at 0) first.
#[must_use]
pub fn sorted_skins(data: &SkinData) -> Vec<&Skin> {
    let mut skins: Vec<&Skin> = data.char_skins.values().collect();
    skins.sort_by(|a, b| {
        let time = |s: &Skin| s.display_skin.as_ref().map_or(0, |d| d.get_time);
        time(a)
            .cmp(&time(b))
            .then_with(|| a.skin_id.cmp(&b.skin_id))
    });
    skins
}

/// "Default Outfit, Elite 2" for the promotion arts, "0011 Craft/VIII: It Does Wash the
/// Strings" for the rest.
fn skin_label(skin: &Skin) -> String {
    let display = skin.display_skin.as_ref();
    let group = display
        .and_then(|d| non_blank(d.skin_group_name.as_deref()))
        .unwrap_or("Outfit");
    match display.and_then(|d| non_blank(d.skin_name.as_deref())) {
        Some(name) => format!("{group}: {name}"),
        None => match skin_elite(&skin.skin_id) {
            Some(e) => format!("{group}, Elite {e}"),
            None => group.to_string(),
        },
    }
}

/// The promotion a default art belongs to: `char_x#1` is the E0 art, `char_x#2` the E2 art.
fn skin_elite(skin_id: &str) -> Option<u32> {
    if skin_id.contains('@') {
        return None;
    }
    match skin_id.rsplit_once('#')?.1.trim_end_matches('+') {
        "1" => Some(0),
        "2" => Some(2),
        _ => None,
    }
}

/// The full art's asset path, the way the frontend's Skins tab finds it: outfits (`@` ids)
/// under `textures/skinpack/<operator>/`, promotion arts under `textures/chararts/<operator>/`.
/// The folder is the operator being shown, not the skin's `charId`: Amiya's Guard and Medic
/// forms list their outfits under `char_002_amiya` but keep the art in their own folders.
fn skin_art_path(op_id: &str, skin: &Skin) -> String {
    let id = &skin.skin_id;
    if id.contains('@') {
        format!("textures/skinpack/{op_id}/{}.png", id.replace('@', "_"))
    } else {
        format!("textures/chararts/{op_id}/{}.png", id.replacen('#', "_", 1))
    }
}

fn outfits(src: &Sources<'_>, sel: Option<usize>) -> Result<Built, String> {
    let data = src
        .skins
        .ok_or("The outfit list is unavailable right now.")?;
    let skins = sorted_skins(data);
    if skins.is_empty() {
        return Err("The backend lists no outfits for this operator.".to_string());
    }
    let pick = sel.unwrap_or(0).min(skins.len() - 1);
    let skin = skins[pick];
    let mut block = Block::titled(skin_label(skin));
    if let Some(avatar) = non_blank(skin.avatar_id.as_deref()) {
        block.thumbnail = Some(route(src, "avatar", avatar));
    }
    block.image = Some(src.urls.asset_url(&skin_art_path(&src.op.id, skin)));
    if let Some(display) = skin.display_skin.as_ref() {
        if let Some(content) = non_blank(display.content.as_deref()) {
            block.paragraph(plain_text(content));
        }
        let artists = names_list(display.drawer_list.as_deref().unwrap_or_default());
        let label = if artists.len() == 1 {
            "Artist"
        } else {
            "Artists"
        };
        block.field(label, artists.join(", "), true);
        let designers = names_list(display.designer_list.as_deref().unwrap_or_default());
        let label = if designers.len() == 1 {
            "Designer"
        } else {
            "Designers"
        };
        block.field(label, designers.join(", "), true);
        if let Some(obtain) = non_blank(display.obtain_approach.as_deref()) {
            block.field("Obtained", obtain, true);
        }
        if display.get_time > 0 {
            block.field("Released", format!("<t:{}:D>", display.get_time), true);
        }
    }
    let choices = capped_choices(skins.len(), pick, "outfits", "Outfit", |i| {
        (skin_label(skins[i]), None)
    });
    let mut head = header(src);
    head.paragraph(format!("**Outfit {} of {}**", pick + 1, skins.len()));
    Ok(finish(
        src,
        vec![head, block],
        choices,
        "Outfit",
        Some(format!("{} of {}", pick + 1, skins.len())),
    ))
}

// ---------------------------------------------------------------------------
// Lore
// ---------------------------------------------------------------------------

fn lore(src: &Sources<'_>, sel: Option<usize>) -> Built {
    let sections: Vec<_> = src
        .detail
        .handbook
        .iter()
        .flat_map(|h| &h.story_text_audio)
        .filter(|s| !s.stories.is_empty())
        .collect();
    let pick = sel.unwrap_or(0).min(sections.len().saturating_sub(1));
    let mut head = header(src);
    if pick == 0 {
        if let Some(usage) = non_blank(src.detail.item_usage.as_deref()) {
            head.paragraph(strip_rich_text(usage));
        }
        if let Some(desc) = non_blank(src.detail.item_desc.as_deref()) {
            head.paragraph(format!("*{}*", strip_rich_text(desc)));
        }
        if let Some(obtain) = non_blank(src.detail.item_obtain_approach.as_deref()) {
            head.field("Obtained", obtain, true);
        }
    }
    let mut blocks = vec![head];
    if let Some(section) = sections.get(pick) {
        let mut block = Block::titled(section.story_title.trim());
        for story in &section.stories {
            if let Some(unlock) = story_unlock(&story.unlock_type, &story.unlock_param) {
                block.paragraph(format!("*{unlock}*"));
            }
            block.paragraph(archive_lines(&plain_text(&story.story_text)));
        }
        blocks.push(block);
    }
    let choices = capped_choices(sections.len(), pick, "files", "File", |i| {
        (sections[i].story_title.trim().to_string(), None)
    });
    let shown = sections.get(pick).map(|s| s.story_title.trim().to_string());
    finish(src, blocks, choices, "File", shown)
}

/// When an archive entry unlocks: `FAVOR` is trust in percent, `AWAKE` is `"<elite>;<level>"`.
fn story_unlock(kind: &str, param: &str) -> Option<String> {
    match kind {
        "FAVOR" => Some(format!("Unlocks at {}% trust", param.trim())),
        "AWAKE" => {
            let elite = param.split(';').next()?.trim();
            Some(format!("Unlocks at Elite {elite}"))
        }
        _ => None,
    }
}

/// The archive's `[Gender] Female` rows as `**Gender:** Female`; other lines as they are.
fn archive_lines(text: &str) -> String {
    text.lines()
        .map(|line| {
            let trimmed = line.trim_start();
            if let Some(rest) = trimmed.strip_prefix('[')
                && let Some((key, value)) = rest.split_once(']')
                && !key.is_empty()
                && key.len() <= 40
            {
                let value = value.trim();
                if value.is_empty() {
                    format!("**{key}**")
                } else {
                    format!("**{key}:** {value}")
                }
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

// ---------------------------------------------------------------------------
// Voice lines
// ---------------------------------------------------------------------------

/// The frontend's voice-line buckets (`VOICE_CATEGORY_ORDER`), in display order.
const VOICE_CATEGORIES: [&str; 9] = [
    "Greetings",
    "Conversations",
    "Trust",
    "Promotions",
    "Battle",
    "Idle",
    "Dorm",
    "Special",
    "Other",
];

/// Which bucket a line's `placeType` files under, as the frontend's `VOICE_CATEGORY_MAP` has it.
fn voice_category(place: &str) -> usize {
    let name = match place {
        "HOME_PLACE" | "HOME_SHOW" | "HOME_WAIT" | "GREETING" => "Greetings",
        "BIRTHDAY" | "NEW_YEAR" | "ANNIVERSARY" | "GACHA" => "Special",
        "SQUAD" | "SQUAD_FIRST" => "Conversations",
        "LEVEL_UP" | "EVOLVE_ONE" | "EVOLVE_TWO" => "Promotions",
        "BATTLE_START" | "BATTLE_FACE_ENEMY" | "BATTLE_SELECT" | "BATTLE_PLACE"
        | "BATTLE_SKILL_1" | "BATTLE_SKILL_2" | "BATTLE_SKILL_3" | "BATTLE_SKILL_4"
        | "FOUR_STAR" | "THREE_STAR" | "TWO_STAR" | "LOSE" => "Battle",
        "BUILDING_PLACE" | "BUILDING_TOUCHING" | "BUILDING_FAVOR_BUBBLE" => "Dorm",
        _ => "Other",
    };
    VOICE_CATEGORIES
        .iter()
        .position(|c| *c == name)
        .unwrap_or(VOICE_CATEGORIES.len() - 1)
}

/// The operator's own voice set: lines keyed by the operator id. When none are (an alternate
/// form sharing its original's set), the largest set, the first key breaking ties.
#[must_use]
pub fn voice_set<'a>(data: &'a VoiceData, op_id: &str) -> Vec<&'a VoiceLine> {
    let mut lines: Vec<&VoiceLine> = data
        .char_words
        .values()
        .filter(|l| l.word_key == op_id)
        .collect();
    if lines.is_empty() {
        let mut counts: HashMap<&str, usize> = HashMap::new();
        for line in data.char_words.values() {
            *counts.entry(line.word_key.as_str()).or_default() += 1;
        }
        let best = counts
            .into_iter()
            .max_by(|a, b| a.1.cmp(&b.1).then_with(|| b.0.cmp(a.0)))
            .map(|(k, _)| k);
        lines = data
            .char_words
            .values()
            .filter(|l| Some(l.word_key.as_str()) == best)
            .collect();
    }
    // The endpoint's key order changes between responses; the game's index is stable.
    lines.sort_by(|a, b| {
        a.voice_index
            .cmp(&b.voice_index)
            .then_with(|| a.voice_title.cmp(&b.voice_title))
    });
    lines
}

fn voice(src: &Sources<'_>, sel: Option<usize>) -> Result<Built, String> {
    let data = src
        .voices
        .ok_or("The voice lines are unavailable right now.")?;
    let lines = voice_set(data, &src.op.id);
    if lines.is_empty() {
        return Err("The backend has no voice lines for this operator.".to_string());
    }
    let present: Vec<usize> = (0..VOICE_CATEGORIES.len())
        .filter(|c| lines.iter().any(|l| voice_category(&l.place_type) == *c))
        .collect();
    let category = sel
        .filter(|c| present.contains(c))
        .or_else(|| present.first().copied())
        .unwrap_or(0);
    let mut head = header(src);
    let actors = names_list(&src.op.voice_actors);
    if !actors.is_empty() {
        head.paragraph(format!("**Voice actors:** {}", actors.join(", ")));
    }
    let mut block = Block::titled(VOICE_CATEGORIES[category]);
    for line in lines
        .iter()
        .filter(|l| voice_category(&l.place_type) == category)
    {
        let text = line
            .voice_text
            .as_deref()
            .map(plain_text)
            .unwrap_or_default();
        block.field(line.voice_title.trim(), text, false);
    }
    let choices = present
        .iter()
        .map(|&c| {
            let count = lines
                .iter()
                .filter(|l| voice_category(&l.place_type) == c)
                .count();
            Choice::new(
                VOICE_CATEGORIES[c],
                c.to_string(),
                Some(&format!(
                    "{count} line{}",
                    if count == 1 { "" } else { "s" }
                )),
            )
            .selected(c == category)
        })
        .collect();
    Ok(finish(
        src,
        vec![head, block],
        choices,
        "Voice line group",
        Some(VOICE_CATEGORIES[category].to_string()),
    ))
}

// ---------------------------------------------------------------------------
// Paradox Simulation
// ---------------------------------------------------------------------------

fn paradox(src: &Sources<'_>) -> Result<Built, String> {
    let entry = src
        .paradox
        .ok_or("This operator has no Paradox Simulation.")?;
    let stage = src
        .paradox_stage
        .ok_or("The Paradox Simulation stage is unavailable right now.")?;
    let mut block = Block::titled(entry.name.trim());
    if let Some(unlock) = entry
        .unlock_param
        .iter()
        .find(|u| u.unlock_type == "AWAKE")
        .and_then(|u| u.unlock_param1.as_deref())
    {
        block.paragraph(format!("*Unlocks at Elite {}*", unlock.trim()));
    }
    if let Some(description) = non_blank(entry.description.as_deref()) {
        block.paragraph(plain_text(description));
    }
    if let Some(preview) = src.paradox_preview {
        block.image = Some(src.urls.asset_url(preview));
    }

    let groups = paradox_enemies(stage);
    for (label, lines) in ["Normal", "Elite", "Boss"].into_iter().zip(&groups) {
        block.field(label, lines.join("\n"), false);
    }
    let rewards = item_inline(entry.reward_item.iter().map(module_cost), src.items);
    block.field("Reward", rewards, true);
    Ok(finish(src, vec![header(src), block], Vec::new(), "", None))
}

/// The stage's enemies as lines ("B1 Originium Slug (Lv.2) **x4**"), grouped Normal, Elite,
/// Boss. The count is how many the waves spawn.
fn paradox_enemies(stage: &ParadoxStage) -> [Vec<String>; 3] {
    let mut spawned: HashMap<&str, u32> = HashMap::new();
    let level = stage.level_data.as_ref();
    for action in level
        .and_then(|l| l.waves.as_ref())
        .into_iter()
        .flatten()
        .flat_map(|w| &w.fragments)
        .flat_map(|f| &f.actions)
        .filter(|a| a.is_spawn())
    {
        *spawned.entry(action.key.as_str()).or_default() += action.count;
    }
    let mut groups: [Vec<String>; 3] = Default::default();
    for r in level
        .and_then(|l| l.enemy_db_refs.as_ref())
        .into_iter()
        .flatten()
    {
        let over = r.overwritten_data.as_ref();
        let prefab = over
            .and_then(|o| o.prefab_key.as_ref()?.text())
            .and_then(|k| stage.enemies.get(k));
        let enemy = stage.enemies.get(&r.id).or(prefab);
        let name = over
            .and_then(|o| o.name.as_ref()?.text())
            .or_else(|| enemy.map(|e| e.name.as_str()));
        let mut line = match (
            name,
            enemy.and_then(|e| non_blank(e.enemy_index.as_deref())),
        ) {
            (Some(name), Some(index)) => format!("{index} {}", name.trim()),
            (Some(name), None) => name.trim().to_string(),
            // Neither the stage's enemy list nor the level file names it.
            (None, _) => format!("`{}`", r.id),
        };
        if enemy
            .and_then(|e| e.stats.as_ref())
            .is_some_and(|s| s.levels.len() > 1)
        {
            let _ = write!(line, " (Lv.{})", r.level + 1);
        }
        // Spawned by something other than a wave (a summoner, a map object): no count.
        if let Some(count) = spawned.get(r.id.as_str()) {
            let _ = write!(line, " **x{count}**");
        }
        let class = over
            .and_then(|o| o.level_type.as_ref()?.text())
            .or_else(|| enemy.and_then(|e| e.enemy_level.as_deref()));
        let group = match class {
            Some("BOSS") => 2,
            Some("ELITE") => 1,
            _ => 0,
        };
        groups[group].push(line);
    }
    groups
}

// ---------------------------------------------------------------------------
// Formatting
// ---------------------------------------------------------------------------

/// Fill a template and strip its markup. The few templates no blackboard fills (none across
/// the 441 EN records, 2026-10-08) stay visible as written.
fn fill<S: std::hash::BuildHasher>(template: &str, board: &HashMap<String, f64, S>) -> String {
    render(template, board).0.trim().to_string()
}

/// A backend image route for `id`, percent-encoded: skill ids carry `[` and `]`
/// (`skcom_atk_up[2]`), outfit avatars `#`.
fn route(src: &Sources<'_>, route: &str, id: &str) -> String {
    src.urls
        .api_url(&format!("/{route}/{}", encode_path_segment(id)))
}

fn range_of(ranges: &HashMap<String, Range>, id: Option<&str>) -> Option<String> {
    ranges.get(id?).map(|r| range_grid(&r.grids))
}

fn pascal_board(entries: Option<&[BoardEntryPascal]>) -> HashMap<String, f64> {
    blackboard(
        entries
            .unwrap_or_default()
            .iter()
            .map(|b| (b.key.as_str(), b.value)),
    )
}

fn camel_board(entries: &[BoardEntry]) -> HashMap<String, f64> {
    blackboard(entries.iter().map(|b| (b.key.as_str(), b.value)))
}

/// The non-blank entries of a profile list, trimmed: the data has stray spaces ("Ruan ").
fn names_list(items: &[String]) -> Vec<&str> {
    items
        .iter()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect()
}

/// "MELEE" -> "Melee".
fn title_case(s: &str) -> String {
    let lower = s.to_lowercase();
    let mut chars = lower.chars();
    chars.next().map_or_else(String::new, |first| {
        first.to_uppercase().chain(chars).collect()
    })
}

/// `max_deck_stack_cnt` -> "Max Deck Stack Cnt", as the frontend titles an unmapped key.
fn title_words(key: &str) -> String {
    key.split('_')
        .filter(|w| !w.is_empty())
        .map(title_case)
        .collect::<Vec<_>>()
        .join(" ")
}

/// A whole stat, rounded, with thousands separators.
fn number(n: f64) -> String {
    let rounded = n.round();
    if rounded < 0.0 {
        return format!("{rounded:.0}");
    }
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    group_thousands(rounded as u64)
}

/// A stat that may be fractional (intervals, durations, bonuses): float noise trimmed.
fn decimal(n: f64) -> String {
    let text = format!("{n:.2}");
    text.trim_end_matches('0').trim_end_matches('.').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A record with nothing but what every operator has, plus `extra` merged in.
    fn record(extra: &serde_json::Value) -> OperatorDetail {
        let mut base = serde_json::json!({
            "id": "char_x", "description": null, "itemUsage": null, "itemDesc": null,
            "itemObtainApproach": null, "trait": null,
            "phases": [{"MaxLevel": 30, "RangeId": "1-1", "AttributesKeyFrames": [],
                        "EvolveCost": null, "LevelUpCost": null}],
            "skills": [], "talents": null, "potentialRanks": [], "favorKeyFrames": null,
            "allSkillLevelUp": [], "modules": [], "drones": [],
            "handbook": {"storyTextAudio": []}, "baseSkills": []
        });
        for (k, v) in extra.as_object().unwrap() {
            base[k] = v.clone();
        }
        serde_json::from_value(base).unwrap()
    }

    #[test]
    fn pages_apply_only_where_there_is_data() {
        // A reserve operator: no skills, summons, modules, base skills, costs or archive.
        let bare = record(&serde_json::json!({}));
        assert_eq!(
            applicable(&bare, false, false),
            vec![Page::Overview, Page::Outfits]
        );
        assert_eq!(
            applicable(&bare, true, true),
            vec![Page::Overview, Page::Outfits, Page::Voice, Page::Paradox]
        );

        let level = serde_json::json!({"name": "S", "description": "d", "rangeId": null,
            "skillType": "MANUAL", "durationType": "NONE",
            "spData": {"spType": "INCREASE_WITH_TIME", "spCost": 10, "initSp": 0},
            "duration": 10.0, "blackboard": []});
        let cost = serde_json::json!([{"Id": "4001", "Count": 100}]);
        let full = record(&serde_json::json!({
            "phases": [
                {"MaxLevel": 50, "RangeId": "1-1", "AttributesKeyFrames": [],
                 "EvolveCost": null, "LevelUpCost": null},
                {"MaxLevel": 80, "RangeId": "1-1", "AttributesKeyFrames": [],
                 "EvolveCost": cost, "LevelUpCost": null}],
            "skills": [{"skillId": "s", "overrideTokenKey": "token_x", "levelUpCostCond": [],
                        "static": {"SkillId": "s", "IconId": null, "Levels": [level]}}],
            "drones": [{"id": "token_x", "name": "T", "description": null, "trait": null,
                        "phases": [], "skills": [], "talents": []}],
            "modules": [{"uniEquipId": "uniequip_002_x", "uniEquipName": "M",
                         "typeName1": "SUM", "typeName2": "Y", "type": "ADVANCED",
                         "unlockEvolvePhase": "PHASE_2", "unlockLevel": 60,
                         "unlockFavorPoint": 0, "itemCost": null, "data": null}],
            "baseSkills": [{"buffName": "B", "description": "d", "roomType": "CONTROL",
                            "skillIcon": null, "unlockElite": 0, "unlockLevel": 1}],
            "handbook": {"storyTextAudio": [{"storyTitle": "Basic Info", "stories": [
                {"storyText": "[Gender] Female", "unlockType": "DIRECT", "unLockParam": ""}]}]}
        }));
        assert_eq!(applicable(&full, true, true), Page::ALL.to_vec());
        // Costs come from promotions alone here.
        assert_eq!(cost_kinds(&full), vec![0]);
        // One summon, brought by S1 only.
        assert_eq!(summon_pairs(&full), vec![(0, Some(0))]);

        // An advanced module without costs still gets the Modules page; an INITIAL one doesn't.
        let mut initial = full.clone();
        initial.modules[0].kind = "INITIAL".to_string();
        assert!(!applicable(&initial, false, false).contains(&Page::Modules));
    }

    #[test]
    fn formats_numbers_and_labels() {
        assert_eq!(group_thousands(1_234_567), "1,234,567");
        assert_eq!(group_thousands(999), "999");
        assert_eq!(number(2500.0), "2,500");
        assert_eq!(number(-3.0), "-3");
        assert_eq!(decimal(1.600_000_023_841_858), "1.6");
        assert_eq!(decimal(70.0), "70");
        assert_eq!(room_label("HIRE"), "Office");
        assert_eq!(level_label(6), "Level 7");
        assert_eq!(level_label(9), "Mastery 3");
        assert_eq!(title_words("max_deck_stack_cnt"), "Max Deck Stack Cnt");
        assert_eq!(skin_elite("char_002_amiya#1+"), Some(0));
        assert_eq!(skin_elite("char_2023_ling#2"), Some(2));
        assert_eq!(skin_elite("char_2023_ling@nian#9"), None);
        assert_eq!(
            story_unlock("FAVOR", "50").as_deref(),
            Some("Unlocks at 50% trust")
        );
        assert_eq!(
            story_unlock("AWAKE", "2;1").as_deref(),
            Some("Unlocks at Elite 2")
        );
        assert_eq!(story_unlock("DIRECT", ""), None);
        assert_eq!(
            archive_lines("[Gender] Female\n[Classified Log]\nPlain line"),
            "**Gender:** Female\n**Classified Log**\nPlain line"
        );
        assert_eq!(voice_category("BATTLE_SKILL_4"), 4);
        assert_eq!(voice_category("LOADING_PANEL"), 8);
    }
}
