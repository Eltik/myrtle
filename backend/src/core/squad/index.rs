//! The Squad Builder's load-time tables, built ONCE per server when its game
//! data loads (`gamedata::init_game_data_with_art`) and kept on `GameData`:
//!
//! * `text`: every English text rule's reading of every trait, talent, module
//!   part and skill rank, keyed by stable ids (`TextKey`). Only EN's table is
//!   ever read: the request path takes it from the EN server's data whatever
//!   server the reader picked (`services::squad`), because the rules are
//!   English. Each id is the same on every server: on 2026-10-11 the talent
//!   candidates of all 1,140 operators EN shares with CN, JP and KR carry
//!   identical unlock, potential and blackboard shapes, so EN's reading of
//!   `(char_id, talent, candidate)` is the reading of the same candidate on
//!   any server.
//! * `tags`: this server's recruitment tag names to `GachaTags` ids.
//! * `threats`: the enemy grade bands (`enemy_handbook_table.LevelInfoList`)
//!   the stage threats read.
//! * `professions`: which professions playable operators have.
//! * `normal_of`: a challenge stage's normal stage, the inverse of
//!   `stage_table` `HardStagedId`.
//! * `token_branches`: the sub-professions every playable operator of which
//!   carries a `DisplayTokenDict` token, so the branch trait grants the unit.

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};
use ts_rs::TS;
use utoipa::ToSchema;

use crate::core::gamedata::types::GameData;
use crate::core::gamedata::types::operator::{Operator, OperatorProfession};

use super::policy::{
    PROFESSION_ORDER, THREAT_GRADE_DEF, THREAT_GRADE_MOVE_SPEED, THREAT_GRADE_RES,
};
use super::targets::{Target, Vocab, highest_only, parse_cond, parse_need, parse_target};
use super::text::{any_of, has_tag, lower_plain, plain};
use super::types::SynergyCond;

/// Where a piece of text sits in the tables, by ids that do not change between
/// servers.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TextKey {
    /// `Operator.description`, the base trait text.
    Trait { char_id: String },
    /// `Trait_.Candidates[cand].OverrideDescripton`.
    TraitCandidate { char_id: String, cand: usize },
    /// `Talents[talent].Candidates[cand].Description`.
    Talent {
        char_id: String,
        talent: usize,
        cand: usize,
    },
    /// A module trait part: its override (`additional` false) or its
    /// additional line (`additional` true).
    ModuleTrait {
        module_id: String,
        phase: usize,
        part: usize,
        cand: usize,
        additional: bool,
    },
    /// A module talent part's description (`upgrade_description`, else
    /// `description`).
    ModuleTalent {
        module_id: String,
        phase: usize,
        part: usize,
        cand: usize,
    },
    /// `skill_table` `Levels[level].Description`.
    Skill { skill_id: String, level: usize },
}

/// What the English text rules read in one piece of text. Every flag is a
/// word or tag rule on the plain text; the blackboard numbers stay with the
/// reader's server.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TextFacts {
    /// "arts damage".
    pub arts_damage: bool,
    /// "physical damage".
    pub physical_damage: bool,
    /// "true damage" or an elemental damage tag (`<$ba.dt.burning2>`).
    pub true_damage: bool,
    /// "aerial enemies first".
    pub aerial_first: bool,
    /// "all blocked enemies", "all targets" or "all enemies" (a trait's
    /// multi-target words).
    pub all_targets: bool,
    /// "all enemies" or "all blocked enemies" (a skill's).
    pub all_enemies: bool,
    /// "does not attack" or "restores the hp": the unit deals no damage.
    pub no_attack: bool,
    /// "refund".
    pub refund: bool,
    /// Reveals or strips enemy Invisibility (`counters_invisible`).
    pub counters_invisible: bool,
    /// Speaks of enemies or "the target", so a stat cut on its blackboard is
    /// an enemy debuff, not a self penalty.
    pub names_enemy: bool,
    /// A word for an ally (`ALLY_WORD`), so a heal skill heals others.
    pub ally_word: bool,
    /// A heal-many phrase ("all allies", "allies within", ...).
    pub heal_area_words: bool,
    /// Talent texts only.
    pub talent: Option<TalentText>,
}

impl TextFacts {
    /// Facts of several texts read as one (a trait override plus a module's
    /// additional line): a word anywhere is a word in the whole.
    pub fn union<'a>(parts: impl IntoIterator<Item = &'a Self>) -> Self {
        let mut out = Self::default();
        for p in parts {
            out.arts_damage |= p.arts_damage;
            out.physical_damage |= p.physical_damage;
            out.true_damage |= p.true_damage;
            out.aerial_first |= p.aerial_first;
            out.all_targets |= p.all_targets;
            out.all_enemies |= p.all_enemies;
            out.no_attack |= p.no_attack;
            out.refund |= p.refund;
            out.counters_invisible |= p.counters_invisible;
            out.names_enemy |= p.names_enemy;
            out.ally_word |= p.ally_word;
            out.heal_area_words |= p.heal_area_words;
        }
        out
    }
}

/// What the English rules read in a talent: its target, when it works, and
/// the wording the economy rules need.
#[derive(Debug, Clone, PartialEq)]
pub struct TalentText {
    pub target: Option<Target>,
    pub cond: SynergyCond,
    /// `(count, counts_giver)`.
    pub need: Option<(i32, bool)>,
    pub highest_only: bool,
    /// Opens with the stat ("DP Cost -1", "Redeployment Time -30 seconds"):
    /// the cost or redeploy change is the holder's own.
    pub opens_with_own_stat: bool,
    /// Says "dp cost": a `cost` key is a DP Cost change or bound.
    pub says_dp_cost: bool,
    /// Speaks of DP (" dp") without "dp cost": a positive `cost` is DP gained.
    pub speaks_of_dp_gain: bool,
    /// "in the squad" / "in squad".
    pub in_squad: bool,
    /// "next operator".
    pub next_operator: bool,
    /// "starting sp" / "initial sp".
    pub starting_sp: bool,
    /// "defeat": DP per kill.
    pub on_defeat: bool,
    /// "if ", "when ", "first time" or "after ".
    pub conditional: bool,
    /// Every number the rendered text prints, with whether a `%` follows it
    /// (`(+2%)` potential notes included). Digits and `%` read the same in
    /// every language, so this is not a word rule.
    pub numbers: Vec<(f64, bool)>,
}

/// How a talent's text shows a blackboard value: as a percentage of it
/// (`atk` 0.08 printed "+8%") or as the value itself (`respawn_time` -30
/// printed "-30 seconds").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatScale {
    Ratio,
    Flat,
}

impl TalentText {
    /// The scale the text prints `v` at, `None` when it prints neither form or
    /// both. With `complement`, `(1 - |v|) x 100%` also reads as a ratio: a
    /// redeploy multiplier (Chiave `respawn_time` 0.25 printed "-75%").
    ///
    /// DATA RULE replacing the old magnitude guess (|v| < 1 a ratio, else
    /// flat). EN 2026-10-11, every talent candidate with a non-zero `atk` or
    /// `respawn_time` (365, `talent_scale_reads_the_printed_number`): it
    /// classifies 359. 4 disagree with the old guess, each a ratio of 100% or
    /// more the guess read as +1 flat ATK: Vigna "Fierce Stabbing" 1.0 and 1.1 ("by 100%", "110%"),
    /// Tomimi "Arts of the Wild" 1.0 and 1.2 ("ATK +100%", "+120%"). 6
    /// unclassified, where `atk` is not the ATK the text prints (Necrass
    /// "Rekindle" x4, a summon upgrade; Suzuran "Binding Circle" x2, whose
    /// text prints the Fragile %), now left out of the hand-off instead of
    /// read as a ratio. Every `respawn_time` read agrees with the guess.
    pub fn scale_of(&self, v: f64, complement: bool) -> Option<StatScale> {
        const EPS: f64 = 1e-3;
        let a = v.abs();
        let pct = a * 100.0;
        let ratio = self.numbers.iter().any(|&(n, p)| {
            p && ((n - pct).abs() < EPS || (complement && (n - (1.0 - a) * 100.0).abs() < EPS))
        });
        let flat = self.numbers.iter().any(|&(n, p)| !p && (n - a).abs() < EPS);
        match (ratio, flat) {
            (true, false) => Some(StatScale::Ratio),
            (false, true) => Some(StatScale::Flat),
            _ => None,
        }
    }
}

/// A number and an optional `%` right after it.
static NUMBER: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
    regex::Regex::new(r"(\d+(?:\.\d+)?)\s*(%?)").expect("number regex")
});

/// A word for an ally or allies in a heal skill's text. TEXT HEURISTIC that
/// keeps self-heals (Matoimaru S1 `heal_scale`) out.
static ALLY_WORD: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
    regex::Regex::new(r"\b(all|ally|allies|allied|operators?|friendly unit)\b").expect("ally regex")
});

/// An elemental damage type tag (`<$ba.dt.apoptosis2>`, `burning2`,
/// `neural2`). `ba.dt.element` is "Elemental Injury", the heal side, so it is
/// left out.
fn elemental_damage_tag(text: &str) -> bool {
    for opener in ["<$ba.dt.", "<@ba.dt."] {
        let mut rest = text;
        while let Some(i) = rest.find(opener) {
            let after = &rest[i + opener.len()..];
            if !after.starts_with("element>") {
                return true;
            }
            rest = after;
        }
    }
    false
}

/// Reveals or strips enemy Invisibility: one sentence (split on `.` and `;`)
/// holding the `ba.invisible` tag, a verb that works against it, and "enem"
/// or "target". So "gains Invisibility" does not count, nor does Manticore's
/// own ("Stays Invisible normally. The first attack ... removes Invisibility").
/// TEXT HEURISTIC on the wording; the tag itself is the same on every server.
fn counters_invisible(text: &str) -> bool {
    if !has_tag(text, "ba.invisible") {
        return false;
    }
    // Every tag name holds a `.` (`<@ba.vup>` as much as the one we want), so
    // mark the Invisibility tag, then drop all other tags before splitting
    // into sentences. Splitting first cut Elysium's "Locked enemies have ...
    // and lose Invisibility" at the `<@ba.vup>` tags, away from "enemies".
    let marked = plain(
        &text
            .replace("<$ba.invisible>", "\u{1}")
            .replace("<@ba.invisible>", "\u{1}"),
    );
    marked.split(['.', ';']).any(|clause| {
        let lower = clause.to_lowercase();
        clause.contains('\u{1}')
            && any_of(
                &lower,
                &["reveal", "remove", "lose", "nullif", "can attack"],
            )
            && (lower.contains("enem") || lower.contains("target"))
    })
}

/// Read one piece of text. `talent` adds the talent rules.
fn read(raw: &str, vocab: &Vocab, talent: bool) -> TextFacts {
    let lower = lower_plain(raw);
    let talent = talent.then(|| {
        let p = plain(raw);
        let p = p.trim_start();
        TalentText {
            target: parse_target(raw, vocab),
            cond: parse_cond(raw),
            need: parse_need(raw),
            highest_only: highest_only(raw),
            opens_with_own_stat: p.starts_with("DP Cost") || p.starts_with("Redeployment Time"),
            says_dp_cost: lower.contains("dp cost"),
            speaks_of_dp_gain: lower.contains(" dp") && !lower.contains("dp cost"),
            in_squad: lower.contains("in the squad") || lower.contains("in squad"),
            next_operator: lower.contains("next operator"),
            starting_sp: lower.contains("starting sp") || lower.contains("initial sp"),
            on_defeat: lower.contains("defeat"),
            conditional: ["if ", "when ", "first time", "after "]
                .iter()
                .any(|w| lower.contains(w)),
            numbers: NUMBER
                .captures_iter(p)
                .filter_map(|c| Some((c[1].parse().ok()?, !c[2].is_empty())))
                .collect(),
        }
    });
    TextFacts {
        arts_damage: lower.contains("arts damage"),
        physical_damage: lower.contains("physical damage"),
        true_damage: lower.contains("true damage") || elemental_damage_tag(raw),
        aerial_first: lower.contains("aerial enemies first"),
        all_targets: any_of(
            &lower,
            &["all blocked enemies", "all targets", "all enemies"],
        ),
        all_enemies: any_of(&lower, &["all enemies", "all blocked enemies"]),
        no_attack: lower.contains("does not attack") || lower.contains("restores the hp"),
        refund: lower.contains("refund"),
        counters_invisible: counters_invisible(raw),
        names_enemy: lower.contains("enem") || lower.contains("the target"),
        ally_word: ALLY_WORD.is_match(&lower),
        heal_area_words: any_of(
            &lower,
            &[
                "all allies",
                "allies within",
                "all friendly",
                "allied units within",
            ],
        ),
        talent,
    }
}

/// Every text rule's reading of every playable operator's texts.
#[derive(Debug, Clone, Default)]
pub struct TextIndex {
    facts: HashMap<TextKey, TextFacts>,
}

impl TextIndex {
    pub fn get(&self, key: &TextKey) -> Option<&TextFacts> {
        self.facts.get(key)
    }

    pub fn len(&self) -> usize {
        self.facts.len()
    }

    pub fn is_empty(&self) -> bool {
        self.facts.is_empty()
    }

    fn build(data: &GameData) -> Self {
        let vocab = Vocab::from_data(data);
        let mut facts = HashMap::new();
        for (char_id, op) in data.operators.iter().filter(|(_, op)| playable(op)) {
            read_operator(char_id, op, &vocab, &mut facts);
        }
        Self { facts }
    }
}

fn read_operator(
    char_id: &str,
    op: &Operator,
    vocab: &Vocab,
    out: &mut HashMap<TextKey, TextFacts>,
) {
    let id = || char_id.to_string();
    out.insert(
        TextKey::Trait { char_id: id() },
        read(&op.description, vocab, false),
    );
    if let Some(t) = &op.trait_data {
        for (cand, c) in t.candidates.iter().enumerate() {
            if let Some(d) = c.override_description.as_ref().filter(|d| !d.is_empty()) {
                out.insert(
                    TextKey::TraitCandidate {
                        char_id: id(),
                        cand,
                    },
                    read(d, vocab, false),
                );
            }
        }
    }
    for (talent, t) in op.talents.iter().enumerate() {
        for (cand, c) in t.candidates.iter().enumerate() {
            let d = c.description.as_deref().unwrap_or_default();
            out.insert(
                TextKey::Talent {
                    char_id: id(),
                    talent,
                    cand,
                },
                read(d, vocab, true),
            );
        }
    }
    for m in &op.modules {
        let module_id = &m.module.uni_equip_id;
        for (phase, ph) in m.data.phases.iter().enumerate() {
            for (part, pt) in ph.parts.iter().enumerate() {
                for (cand, c) in pt
                    .add_or_override_talent_data_bundle
                    .candidates
                    .iter()
                    .flatten()
                    .enumerate()
                {
                    let d = if c.upgrade_description.is_empty() {
                        c.description.as_deref().unwrap_or_default()
                    } else {
                        &c.upgrade_description
                    };
                    out.insert(
                        TextKey::ModuleTalent {
                            module_id: module_id.clone(),
                            phase,
                            part,
                            cand,
                        },
                        read(d, vocab, true),
                    );
                }
                for (cand, c) in pt
                    .override_trait_data_bundle
                    .candidates
                    .iter()
                    .flatten()
                    .enumerate()
                {
                    let key = |additional| TextKey::ModuleTrait {
                        module_id: module_id.clone(),
                        phase,
                        part,
                        cand,
                        additional,
                    };
                    if let Some(d) = c.override_description.as_ref().filter(|d| !d.is_empty()) {
                        out.insert(key(false), read(d, vocab, false));
                    }
                    if !c.additional_description.is_empty() {
                        out.insert(key(true), read(&c.additional_description, vocab, false));
                    }
                }
            }
        }
    }
    for s in &op.skills {
        let Some(st) = &s.static_data else { continue };
        for (level, l) in st.levels.iter().enumerate() {
            out.entry(TextKey::Skill {
                skill_id: s.skill_id.clone(),
                level,
            })
            .or_insert_with(|| read(&l.description, vocab, false));
        }
    }
}

/// A playable operator a player can own: what the per-server tallies count.
fn obtainable_playable(op: &Operator) -> bool {
    playable(op) && !op.is_not_obtainable
}

/// An operator a squad can field: not a token or a trap.
pub fn playable(op: &Operator) -> bool {
    !matches!(
        op.profession,
        OperatorProfession::Token | OperatorProfession::Trap
    )
}

/// This server's recruitment tag names to `GachaTags` ids, and back.
#[derive(Debug, Clone, Default)]
pub struct TagIndex {
    ids: HashMap<String, i32>,
    names: HashMap<i32, String>,
}

impl TagIndex {
    fn build(data: &GameData) -> Self {
        let mut ids = HashMap::new();
        let mut names = HashMap::new();
        for t in data
            .gacha
            .gacha_tags
            .iter()
            .filter(|t| !t.tag_name.is_empty())
        {
            ids.insert(t.tag_name.clone(), t.tag_id);
            names.insert(t.tag_id, t.tag_name.clone());
        }
        Self { ids, names }
    }

    /// The ids of an operator's `tag_list` names.
    pub fn ids_of(&self, op: &Operator) -> Vec<i32> {
        op.tag_list
            .iter()
            .filter_map(|n| self.ids.get(n).copied())
            .collect()
    }

    pub fn name(&self, id: i32) -> Option<&str> {
        self.names.get(&id).map(String::as_str)
    }
}

/// Where an enemy stat starts to count as a stage threat: the lower bound of
/// the grade `policy` names, read from `enemy_handbook_table.LevelInfoList`.
/// `None` when the table lacks the grade, and the page then flags nothing for
/// that stat.
#[derive(TS, ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ThreatThresholds {
    /// RES at or above this asks for physical or true/elemental damage.
    pub high_res: Option<f64>,
    /// DEF at or above this asks for arts or true/elemental damage.
    pub high_def: Option<f64>,
    /// Move speed at or above this asks for crowd control or a shift.
    pub fast_move_speed: Option<f64>,
    /// The grades the three come from, for the page's tooltip.
    pub res_grade: String,
    pub def_grade: String,
    pub move_speed_grade: String,
}

impl ThreatThresholds {
    fn build(data: &GameData) -> Self {
        let grade = |name: &str| {
            data.enemies
                .level_info_list
                .iter()
                .find(|g| g.class_level == name)
        };
        Self {
            high_res: grade(THREAT_GRADE_RES).map(|g| g.magic_res.min),
            high_def: grade(THREAT_GRADE_DEF).map(|g| g.def.min),
            fast_move_speed: grade(THREAT_GRADE_MOVE_SPEED).map(|g| g.move_speed.min),
            res_grade: THREAT_GRADE_RES.to_string(),
            def_grade: THREAT_GRADE_DEF.to_string(),
            move_speed_grade: THREAT_GRADE_MOVE_SPEED.to_string(),
        }
    }
}

/// The Squad Builder's tables for one server. See the module docs.
#[derive(Debug, Clone, Default)]
pub struct SquadIndex {
    pub text: TextIndex,
    pub tags: TagIndex,
    pub threats: ThreatThresholds,
    /// The professions playable operators have, in `policy::PROFESSION_ORDER`.
    pub professions: Vec<OperatorProfession>,
    /// Challenge stage id -> its normal stage id (`HardStagedId` inverted).
    pub normal_of: HashMap<String, String>,
    /// Sub-professions whose every playable obtainable operator carries a
    /// non-empty `character_table` `DisplayTokenDict`: the branch trait is
    /// what grants the unit (EN 2026-10-11: summoner 7/7, soulcaster 2/2,
    /// tactician 5/5, craftsman 6/6, traper 6/6; every other branch with a
    /// token holder is partial, e.g. physician 2/13).
    pub token_branches: HashSet<String>,
    /// The highest star count of any playable obtainable operator.
    pub top_rarity: i16,
}

impl SquadIndex {
    pub fn build(data: &GameData) -> Self {
        let present: Vec<&OperatorProfession> = data
            .operators
            .values()
            .filter(|op| obtainable_playable(op))
            .map(|op| &op.profession)
            .collect();
        let mut branch_tokens: HashMap<&str, (usize, usize)> = HashMap::new();
        for op in data.operators.values().filter(|op| obtainable_playable(op)) {
            let e = branch_tokens
                .entry(op.sub_profession_id.as_str())
                .or_default();
            e.0 += usize::from(
                op.display_token_dict
                    .as_ref()
                    .is_some_and(|d| !d.is_empty()),
            );
            e.1 += 1;
        }
        let top_rarity = data
            .operators
            .values()
            .filter(|op| obtainable_playable(op))
            .map(|op| op.rarity.to_star_int())
            .max()
            .unwrap_or_default();
        Self {
            top_rarity,
            token_branches: branch_tokens
                .into_iter()
                .filter(|(_, (with, all))| with == all)
                .map(|(sub, _)| sub.to_owned())
                .collect(),
            text: TextIndex::build(data),
            tags: TagIndex::build(data),
            threats: ThreatThresholds::build(data),
            professions: PROFESSION_ORDER
                .iter()
                .filter(|p| present.contains(p))
                .cloned()
                .collect(),
            normal_of: data
                .stages
                .iter()
                .filter_map(|(id, s)| {
                    let hard = s.hard_staged_id.as_ref().filter(|h| !h.is_empty())?;
                    Some((hard.clone(), id.clone()))
                })
                .collect(),
        }
    }
}
