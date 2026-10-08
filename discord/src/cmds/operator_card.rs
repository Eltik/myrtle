//! The full operator card: `/collection operator compact:false`.
//!
//! Builds every section the payload of `GET /api/operators/{id}` supports (attributes, trait,
//! talents, skills, modules, base skills, costs, profile) as embeds, then packs them into as
//! few messages as Discord allows. A section with no data is left out rather than shown empty.

use std::collections::HashMap;
use std::fmt::Write as _;
use std::hash::BuildHasher;

use ::serenity::builder::CreateEmbed;

use crate::api::operator_detail::{
    Attributes, BoardEntry, BoardEntryPascal, ItemCost, Module, ModuleItemCost, OperatorDetail,
    Skill, SkillLevel, Talent, TalentCandidate,
};
use crate::gametext::{blackboard, render, strip_rich_text};
use crate::utils::ellipsize;

/// Discord's per-embed field cap.
const FIELDS_PER_EMBED: usize = 25;
/// Discord's field-value cap.
const FIELD_MAX: usize = 1024;
/// Discord's cap on the characters of all embeds in one message, which also bounds one embed.
const CHARS_PER_MESSAGE: usize = 6000;
/// Discord's cap on embeds per message.
const EMBEDS_PER_MESSAGE: usize = 10;
/// One embed of a section stops taking fields past this, leaving room for the title and the
/// "(cont.)" suffix inside [`CHARS_PER_MESSAGE`].
const SECTION_BUDGET: usize = 5600;

/// The packed card: messages of embeds, and the template keys nothing resolved (for logging).
pub struct Card {
    pub messages: Vec<Vec<CreateEmbed>>,
    pub unresolved: Vec<String>,
}

/// Build the full card. `overview` is the compact embed, which leads the card unchanged
/// except for the fields added after it.
#[must_use]
pub fn full_card<S: BuildHasher>(
    overview: CreateEmbed,
    detail: &OperatorDetail,
    items: &HashMap<String, String, S>,
    colour: u32,
) -> Card {
    let mut text = Renderer::default();
    let mut sections: Vec<Section> = Vec::new();

    let overview = overview_fields(overview, detail);
    let attributes = attributes_section(detail);
    let trait_ = trait_section(detail, &mut text);
    let talents = talents_section(detail, &mut text);
    let skills = skills_section(detail, &mut text);
    let modules = modules_section(detail, &mut text);
    let base = base_section(detail);
    let costs = costs_section(detail, items);
    let profile = profile_section(detail);
    sections.extend(
        [
            attributes, trait_, talents, skills, modules, base, costs, profile,
        ]
        .into_iter()
        .flatten(),
    );

    let mut embeds = vec![overview];
    for section in sections {
        embeds.extend(section.into_embeds(colour));
    }
    let mut unresolved = text.missing;
    unresolved.sort();
    unresolved.dedup();
    Card {
        messages: pack(embeds),
        unresolved,
    }
}

/// Renders game text and remembers the template keys it couldn't fill.
#[derive(Default)]
struct Renderer {
    missing: Vec<String>,
}

impl Renderer {
    fn render(&mut self, template: &str, board: &HashMap<String, f64>) -> String {
        let (text, missing) = render(template, board);
        self.missing.extend(missing);
        text.trim().to_string()
    }
}

/// A titled group of fields that becomes one or more embeds.
struct Section {
    title: String,
    description: Option<String>,
    fields: Vec<(String, String, bool)>,
}

impl Section {
    fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            description: None,
            fields: Vec::new(),
        }
    }

    /// Add a field, split across several when `value` is over Discord's 1024 characters. Splits
    /// fall between lines, so a description is never cut inside a sentence's numbers; a single
    /// line longer than a whole field is shortened with an ellipsis.
    fn field(&mut self, name: &str, value: &str, inline: bool) {
        let value = value.trim();
        if value.is_empty() {
            return;
        }
        for (i, chunk) in split_lines(value, FIELD_MAX).into_iter().enumerate() {
            let name = if i == 0 {
                name.to_string()
            } else {
                format!("{name} (cont.)")
            };
            self.fields.push((ellipsize(&name, 256), chunk, inline));
        }
    }

    fn into_embeds(self, colour: u32) -> Vec<CreateEmbed> {
        let mut embeds = Vec::new();
        let mut current: Vec<(String, String, bool)> = Vec::new();
        let mut used = self.title.chars().count()
            + self.description.as_deref().map_or(0, |d| d.chars().count());
        let mut first = true;
        let flush = |fields: Vec<(String, String, bool)>, first: bool| {
            let title = if first {
                self.title.clone()
            } else {
                format!("{} (cont.)", self.title)
            };
            let mut embed = CreateEmbed::new()
                .title(ellipsize(&title, 256))
                .colour(colour);
            if first && let Some(d) = self.description.as_deref() {
                embed = embed.description(d);
            }
            embed.fields(fields)
        };
        for field in self.fields.clone() {
            let size = field.0.chars().count() + field.1.chars().count();
            if !current.is_empty()
                && (current.len() == FIELDS_PER_EMBED || used + size > SECTION_BUDGET)
            {
                embeds.push(flush(std::mem::take(&mut current), first));
                first = false;
                used = self.title.chars().count() + " (cont.)".len();
            }
            used += size;
            current.push(field);
        }
        if !current.is_empty() || first {
            embeds.push(flush(current, first));
        }
        embeds
    }
}

/// Split `text` into chunks of at most `max` characters, breaking between lines.
fn split_lines(text: &str, max: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut current = String::new();
    for line in text.lines() {
        let line = if line.chars().count() > max {
            ellipsize(line, max)
        } else {
            line.to_string()
        };
        let extra = usize::from(!current.is_empty());
        if current.chars().count() + extra + line.chars().count() > max {
            chunks.push(std::mem::take(&mut current));
        }
        if !current.is_empty() {
            current.push('\n');
        }
        current.push_str(&line);
    }
    if !current.is_empty() {
        chunks.push(current);
    }
    chunks
}

/// The characters Discord counts toward an embed's limits: title, description, field names
/// and values, footer text and author name.
#[must_use]
pub fn embed_chars(embed: &CreateEmbed) -> usize {
    let value = serde_json::to_value(embed).unwrap_or_default();
    let text = |v: &serde_json::Value| v.as_str().map_or(0, |s| s.chars().count());
    let mut n = text(&value["title"]) + text(&value["description"]);
    n += text(&value["footer"]["text"]) + text(&value["author"]["name"]);
    if let Some(fields) = value["fields"].as_array() {
        n += fields
            .iter()
            .map(|f| text(&f["name"]) + text(&f["value"]))
            .sum::<usize>();
    }
    n
}

/// Pack embeds, in order, into as few messages as the per-message limits allow.
fn pack(embeds: Vec<CreateEmbed>) -> Vec<Vec<CreateEmbed>> {
    let mut messages: Vec<Vec<CreateEmbed>> = Vec::new();
    let mut current: Vec<CreateEmbed> = Vec::new();
    let mut used = 0;
    for embed in embeds {
        let size = embed_chars(&embed);
        if !current.is_empty()
            && (current.len() == EMBEDS_PER_MESSAGE || used + size > CHARS_PER_MESSAGE)
        {
            messages.push(std::mem::take(&mut current));
            used = 0;
        }
        used += size;
        current.push(embed);
    }
    if !current.is_empty() {
        messages.push(current);
    }
    messages
}

// ---------------------------------------------------------------------------
// Sections
// ---------------------------------------------------------------------------

fn overview_fields(mut overview: CreateEmbed, detail: &OperatorDetail) -> CreateEmbed {
    let mut about = Vec::new();
    if let Some(usage) = non_blank(detail.item_usage.as_deref()) {
        about.push(strip_rich_text(usage));
    }
    if let Some(desc) = non_blank(detail.item_desc.as_deref()) {
        about.push(format!("*{}*", strip_rich_text(desc)));
    }
    if !about.is_empty() {
        overview = overview.field("About", ellipsize(&about.join("\n"), FIELD_MAX), false);
    }
    if let Some(obtain) = non_blank(detail.item_obtain_approach.as_deref()) {
        overview = overview.field("Obtain", ellipsize(obtain, FIELD_MAX), true);
    }
    overview
}

fn attributes_section(detail: &OperatorDetail) -> Option<Section> {
    let mut section = Section::new("Attributes");
    for (i, phase) in detail.phases.iter().enumerate() {
        let Some(top) = phase.attributes_key_frames.last() else {
            continue;
        };
        section.field(
            &format!("E{i} Lv.{}", phase.max_level),
            &attribute_lines(&top.data),
            true,
        );
    }
    if let Some(trust) = detail
        .favor_key_frames
        .as_ref()
        .and_then(|f| f.last())
        .map(|k| trust_line(&k.data))
        .filter(|t| !t.is_empty())
    {
        section.field("Trust bonus (max)", &trust, false);
    }
    (!section.fields.is_empty()).then_some(section)
}

fn attribute_lines(a: &Attributes) -> String {
    // The real interval scales with ASPD (100 is normal speed).
    let interval = if a.attack_speed > 0.0 {
        a.base_attack_time * 100.0 / a.attack_speed
    } else {
        a.base_attack_time
    };
    [
        format!("HP {}", number(a.max_hp)),
        format!("ATK {}", number(a.atk)),
        format!("DEF {}", number(a.def)),
        format!("RES {}", number(a.magic_resistance)),
        format!("Interval {}s", decimal(interval)),
        format!("Block {}", number(a.block_cnt)),
        format!("DP cost {}", number(a.cost)),
        format!("Redeploy {}s", number(a.respawn_time)),
    ]
    .join("\n")
}

fn trust_line(a: &Attributes) -> String {
    [
        ("HP", a.max_hp),
        ("ATK", a.atk),
        ("DEF", a.def),
        ("RES", a.magic_resistance),
    ]
    .into_iter()
    .filter(|(_, v)| *v != 0.0)
    .map(|(name, v)| format!("{name} +{}", number(v)))
    .collect::<Vec<_>>()
    .join(", ")
}

fn trait_section(detail: &OperatorDetail, text: &mut Renderer) -> Option<Section> {
    let candidate = detail.trait_.as_ref().and_then(|t| t.candidates.last());
    let template = candidate
        .and_then(|c| non_blank(c.override_description.as_deref()))
        .or_else(|| non_blank(detail.description.as_deref()))?;
    let board = pascal_board(candidate.and_then(|c| c.blackboard.as_deref()));
    let mut section = Section::new("Trait");
    section.description = Some(ellipsize(&text.render(template, &board), 4096));
    Some(section)
}

fn talents_section(detail: &OperatorDetail, text: &mut Renderer) -> Option<Section> {
    let mut section = Section::new("Talents and potentials");
    for talent in detail.talents.iter().flatten() {
        if let Some((name, value)) = talent_field(talent, text) {
            section.field(&name, &value, false);
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
    if !potentials.is_empty() {
        section.field("Potentials", &potentials.join("\n"), false);
    }
    (!section.fields.is_empty()).then_some(section)
}

/// One talent at its best (the last candidate: highest promotion and potential), titled with
/// its name and when it first unlocks.
fn talent_field(talent: &Talent, text: &mut Renderer) -> Option<(String, String)> {
    let candidates: Vec<&TalentCandidate> = talent
        .candidates
        .iter()
        .flatten()
        .filter(|c| !c.is_hide_talent)
        .collect();
    let best = candidates.last()?;
    let first = candidates.first()?;
    let name = candidates
        .iter()
        .rev()
        .find_map(|c| non_blank(c.name.as_deref()))?;
    let description = non_blank(best.description.as_deref())?;
    let board = pascal_board(best.blackboard.as_deref());
    let mut unlock = format!(
        "{} Lv.{}",
        phase_label(&first.unlock_condition.phase),
        first.unlock_condition.level
    );
    if first.required_potential_rank > 0 {
        let _ = write!(unlock, ", Pot {}", first.required_potential_rank + 1);
    }
    Some((
        format!("{name} (unlocks at {unlock})"),
        text.render(description, &board),
    ))
}

fn skills_section(detail: &OperatorDetail, text: &mut Renderer) -> Option<Section> {
    let mut section = Section::new("Skills");
    for (i, skill) in detail.skills.iter().enumerate() {
        if let Some((name, value)) = skill_field(i + 1, skill, text) {
            section.field(&name, &value, false);
        }
    }
    (!section.fields.is_empty()).then_some(section)
}

/// One skill at its top level: M3 when it has masteries (10 levels), else its last level.
fn skill_field(slot: usize, skill: &Skill, text: &mut Renderer) -> Option<(String, String)> {
    let levels = &skill.static_.as_ref()?.levels;
    let level = levels.last()?;
    let rank = if levels.len() == 10 {
        "M3".to_string()
    } else {
        format!("Lv.{}", levels.len())
    };
    let mut board = blackboard(level.blackboard.iter().map(|b| (b.key.as_str(), b.value)));
    // `{duration}` (Heavyrain's S2) reads the level's own duration, not a blackboard entry.
    board
        .entry("duration".to_string())
        .or_insert(level.duration);
    let mut value = skill_stats(level);
    if let Some(description) = non_blank(level.description.as_deref()) {
        value.push('\n');
        value.push_str(&text.render(description, &board));
    }
    Some((format!("S{slot}: {} ({rank})", level.name.trim()), value))
}

/// "Manual · Offensive recovery · SP 4 (initial 0) · 15s".
fn skill_stats(level: &SkillLevel) -> String {
    let mut parts: Vec<String> = Vec::new();
    parts.push(
        match level.skill_type.as_str() {
            "MANUAL" => "Manual",
            "AUTO" => "Auto",
            "PASSIVE" => "Passive",
            other => other,
        }
        .to_string(),
    );
    let recovery = match level.sp_data.sp_type.as_str() {
        "INCREASE_WITH_TIME" => Some("Auto recovery"),
        "INCREASE_WHEN_ATTACK" => Some("Offensive recovery"),
        "INCREASE_WHEN_TAKEN_DAMAGE" => Some("Defensive recovery"),
        _ => None,
    };
    if let Some(recovery) = recovery {
        parts.push(recovery.to_string());
        parts.push(format!(
            "SP {} (initial {})",
            level.sp_data.sp_cost, level.sp_data.init_sp
        ));
    }
    if level.duration > 0.0 {
        parts.push(format!("{}s", decimal(level.duration)));
    } else if level.duration_type.as_deref() == Some("AMMO") {
        parts.push("Ammo".to_string());
    }
    format!("*{}*", parts.join(" · "))
}

fn modules_section(detail: &OperatorDetail, text: &mut Renderer) -> Option<Section> {
    let mut section = Section::new("Modules");
    for module in detail.modules.iter().filter(|m| m.kind == "ADVANCED") {
        // A module with no stage-3 data still gets its name in the list.
        let value = module_stage3(module, text).unwrap_or_else(|| "-".to_string());
        section.field(&module_name(module), &value, false);
    }
    (!section.fields.is_empty()).then_some(section)
}

/// "MAR-X · Exusiai's Surprise". The game shows type `D` as Δ.
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

/// Stage 3's stat bonus, trait change and talent upgrade, one line each.
fn module_stage3(module: &Module, text: &mut Renderer) -> Option<String> {
    let stage = module
        .data
        .as_ref()?
        .phases
        .iter()
        .max_by_key(|p| p.equip_level)?;
    let mut lines = Vec::new();
    let stats = stat_bonus(&stage.attribute_blackboard);
    if !stats.is_empty() {
        lines.push(format!("Stage {}: {stats}", stage.equip_level));
    }
    for part in &stage.parts {
        if let Some(c) = part
            .override_trait_data_bundle
            .as_ref()
            .and_then(|b| b.candidates.as_ref())
            .and_then(|c| c.last())
        {
            let board = camel_board(&c.blackboard);
            if let Some(added) = non_blank(c.additional_description.as_deref()) {
                lines.push(format!("Trait: {}", text.render(added, &board)));
            } else if let Some(new) = non_blank(c.override_description.as_deref()) {
                lines.push(format!("New trait: {}", text.render(new, &board)));
            }
        }
        if let Some(c) = part
            .add_or_override_talent_data_bundle
            .as_ref()
            .and_then(|b| b.candidates.as_ref())
            .and_then(|c| c.last())
            && let Some(upgrade) = non_blank(c.upgrade_description.as_deref())
        {
            let board = camel_board(&c.blackboard);
            let label = non_blank(c.name.as_deref()).unwrap_or("Talent");
            lines.push(format!("{label}: {}", text.render(upgrade, &board)));
        }
    }
    (!lines.is_empty()).then(|| lines.join("\n"))
}

/// "HP +190, ATK +41" from a module stage's attribute blackboard.
fn stat_bonus(board: &[BoardEntry]) -> String {
    board
        .iter()
        .map(|b| {
            let name = match b.key.as_str() {
                "max_hp" => "HP",
                "atk" => "ATK",
                "def" => "DEF",
                "magic_resistance" => "RES",
                "attack_speed" => "ASPD",
                "cost" => "DP cost",
                "respawn_time" => "Redeploy",
                "block_cnt" => "Block",
                other => other,
            };
            let sign = if b.value >= 0.0 { "+" } else { "" };
            format!("{name} {sign}{}", decimal(b.value))
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn base_section(detail: &OperatorDetail) -> Option<Section> {
    let mut section = Section::new("Base skills");
    for skill in detail.base_skills.iter().flatten() {
        let mut unlock = format!("E{}", skill.unlock_elite);
        if skill.unlock_level > 1 {
            let _ = write!(unlock, " Lv.{}", skill.unlock_level);
        }
        let name = format!(
            "{} ({}, {unlock})",
            skill.buff_name.trim(),
            room_label(&skill.room_type)
        );
        let description = skill
            .description
            .as_deref()
            .map(strip_rich_text)
            .unwrap_or_default();
        section.field(&name, &description, false);
    }
    (!section.fields.is_empty()).then_some(section)
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

fn costs_section<S: BuildHasher>(
    detail: &OperatorDetail,
    items: &HashMap<String, String, S>,
) -> Option<Section> {
    let mut section = Section::new("Costs");
    for (i, phase) in detail.phases.iter().enumerate().skip(1) {
        let costs = item_list(phase.evolve_cost.iter().flatten().map(game_cost), items);
        section.field(&format!("Elite {i}"), &costs, false);
    }
    let skill_levels = item_list(
        detail
            .all_skill_level_up
            .iter()
            .flatten()
            .flat_map(|l| l.lvl_up_cost.iter().flatten())
            .map(game_cost),
        items,
    );
    section.field("Skill Lv.2-7 (total)", &skill_levels, false);
    for (i, skill) in detail.skills.iter().enumerate() {
        let mastery = item_list(
            skill
                .level_up_cost_cond
                .iter()
                .flatten()
                .flat_map(|m| m.level_up_cost.iter().flatten())
                .map(game_cost),
            items,
        );
        section.field(&format!("S{} M1-M3 (total)", i + 1), &mastery, false);
    }
    for module in detail.modules.iter().filter(|m| m.kind == "ADVANCED") {
        let total = item_list(
            module
                .item_cost
                .iter()
                .flat_map(|c| c.values())
                .flatten()
                .map(module_cost),
            items,
        );
        section.field(
            &format!("{} (stages 1-3)", module_name(module)),
            &total,
            false,
        );
    }
    (!section.fields.is_empty()).then_some(section)
}

const fn game_cost(c: &ItemCost) -> (&str, u64) {
    (c.id.as_str(), c.count)
}

const fn module_cost(c: &ModuleItemCost) -> (&str, u64) {
    (c.id.as_str(), c.count)
}

/// "LMD x30,000 · Caster Chip x5", summing repeats and keeping first-seen order.
fn item_list<'a, S: BuildHasher>(
    costs: impl IntoIterator<Item = (&'a str, u64)>,
    items: &HashMap<String, String, S>,
) -> String {
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
            let name = items.get(id).map_or(id, String::as_str);
            format!("{name} x{}", group_thousands(count))
        })
        .collect::<Vec<_>>()
        .join(" · ")
}

fn profile_section(detail: &OperatorDetail) -> Option<Section> {
    let info = detail.profile.as_ref()?.basic_info.as_ref()?;
    let lines: Vec<String> = [
        ("gender", "Gender"),
        ("combatExperience", "Combat experience"),
        ("placeOfBirth", "Place of birth"),
        ("dateOfBirth", "Date of birth"),
        ("race", "Race"),
        ("height", "Height"),
    ]
    .into_iter()
    .filter_map(|(key, label)| {
        non_blank(info.get(key)?.as_deref()).map(|v| format!("**{label}:** {v}"))
    })
    .collect();
    if lines.is_empty() {
        return None;
    }
    let mut section = Section::new("Profile");
    section.description = Some(lines.join("\n"));
    Some(section)
}

// ---------------------------------------------------------------------------
// Formatting
// ---------------------------------------------------------------------------

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

fn non_blank(s: Option<&str>) -> Option<&str> {
    s.map(str::trim).filter(|s| !s.is_empty())
}

/// `PHASE_2` -> `E2`.
fn phase_label(phase: &str) -> String {
    phase
        .strip_prefix("PHASE_")
        .map_or_else(|| phase.to_string(), |n| format!("E{n}"))
}

/// A whole stat: rounded, no decimal point.
fn number(n: f64) -> String {
    format!("{n:.0}")
}

/// A stat that may be fractional (intervals, durations): float noise trimmed.
fn decimal(n: f64) -> String {
    let text = format!("{n:.2}");
    text.trim_end_matches('0').trim_end_matches('.').to_string()
}

fn group_thousands(n: u64) -> String {
    let digits = n.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_long_fields_between_lines() {
        let line = "x".repeat(600);
        let text = format!("{line}\n{line}\n{line}");
        let chunks = split_lines(&text, FIELD_MAX);
        assert_eq!(chunks.len(), 3);
        assert!(chunks.iter().all(|c| c.chars().count() <= FIELD_MAX));
        let long = "y".repeat(1500);
        let chunks = split_lines(&long, FIELD_MAX);
        assert_eq!(chunks.len(), 1);
        assert!(chunks[0].ends_with('…') && chunks[0].chars().count() == FIELD_MAX);
    }

    #[test]
    fn packs_within_message_limits() {
        let big = || CreateEmbed::new().description("z".repeat(2500));
        let messages = pack((0..5).map(|_| big()).collect());
        assert_eq!(messages.len(), 3);
        let small = || CreateEmbed::new().title("t");
        let messages = pack((0..23).map(|_| small()).collect());
        assert_eq!(
            messages.iter().map(Vec::len).collect::<Vec<_>>(),
            vec![10, 10, 3]
        );
    }

    #[test]
    fn formats_numbers_and_names() {
        assert_eq!(group_thousands(1_234_567), "1,234,567");
        assert_eq!(group_thousands(999), "999");
        assert_eq!(decimal(1.600_000_023_841_858), "1.6");
        assert_eq!(decimal(70.0), "70");
        assert_eq!(phase_label("PHASE_2"), "E2");
        assert_eq!(room_label("HIRE"), "Office");
    }
}
