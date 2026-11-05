use std::{
    collections::{HashMap, HashSet},
    sync::LazyLock,
};

use myrtle_gamedata_raw::tables::{
    enemy::EnemyHandbook,
    gacha::GachaData,
    handbook_team::HandbookTeam,
    operator::{Operator, OperatorProfession},
    stage::Stage,
};
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::index::squad::{
    policy::{PROFESSION_ORDER, THREAT_GRADE_DEF, THREAT_GRADE_MOVE_SPEED, THREAT_GRADE_RES},
    targets::{Target, Vocab, highest_only, parse_cond, parse_faction, parse_need, parse_target},
    text::{any_of, has_tag, lower_plain, plain},
    types::SynergyCond,
};

pub mod policy;
pub mod targets;
pub mod text;
pub mod types;

#[derive(Debug, Clone, Copy)]
pub struct SquadIndexData<'a> {
    pub operators: &'a HashMap<String, Operator>,
    pub gacha: &'a GachaData,
    pub enemies: &'a EnemyHandbook,
    pub stages: &'a HashMap<String, Stage>,
    pub factions: &'a HashMap<String, HandbookTeam>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub enum TextKey {
    /// `Operator.description`
    Trait { char_id: String },
    /// `Trait_.Candidates[cand].OverrideDescripton`
    TraitCandidate { char_id: String, cand: usize },
    /// `Talents[talent].Candidates[cand].Description`
    Talent {
        char_id: String,
        talent: usize,
        cand: usize,
    },
    /// Module trait part: its override (`additional` false) or its
    /// additional line (`additional` true)
    ModuleTrait {
        module_id: String,
        phase: usize,
        part: usize,
        cand: usize,
        additional: bool,
    },
    /// A module talent part's description (`upgrade_description`, else
    /// `description`)
    ModuleTalent {
        module_id: String,
        phase: usize,
        part: usize,
        cand: usize,
    },
    /// `skill_table` `Levels[level].Description`
    Skill { skill_id: String, level: usize },
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct TextFacts {
    pub arts_damage: bool,
    pub physical_damage: bool,
    pub true_damage: bool,
    pub aerial_first: bool,
    pub all_targets: bool,
    pub all_enemies: bool,
    pub no_attack: bool,
    pub refund: bool,
    pub counters_invisible: bool,
    pub names_enemy: bool,
    pub ally_word: bool,
    pub heal_area_words: bool,
    pub dp_gain: Vec<String>,
    pub first_redeploy: bool,
    pub dp_cost_cut: Vec<(String, bool)>,
    pub faction: Option<Target>,
    pub talent: Option<TalentText>,
}

impl TextFacts {
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
            out.dp_gain.extend(p.dp_gain.iter().cloned());
            out.first_redeploy |= p.first_redeploy;
            out.dp_cost_cut.extend(p.dp_cost_cut.iter().cloned());
        }
        out
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TalentText {
    pub target: Option<Target>,
    pub cond: SynergyCond,
    /// `(count, counts_giver)`.
    pub need: Option<(i32, bool)>,
    pub highest_only: bool,
    /// Opens with a stat ("DP Cost -1", "Redeployment Time -30 seconds")
    pub opens_with_own_stat: bool,
    pub says_dp_cost: bool,
    pub speaks_of_dp_gain: bool,
    pub in_squad: bool,
    pub next_operator: bool,
    pub starting_sp: bool,
    pub on_defeat: bool,
    /// "if ", "when ", "first time" or "after "
    pub conditional: bool,
    /// Thanks Closure
    pub lowers_dp_floor: bool,
    pub own_redeploy_numbers: Vec<(f64, bool)>,
    /// Every number the rendered text prints, with whether a `%` follows it
    /// (`(+2%)` potential notes included)
    pub numbers: Vec<(f64, bool)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatScale {
    Ratio,
    Flat,
}

impl TalentText {
    #[must_use]
    pub fn scale_of(&self, v: f64, complement: bool) -> Option<StatScale> {
        scale_in(&self.numbers, v, complement)
    }

    #[must_use]
    pub fn own_redeploy_scale(&self, v: f64) -> Option<StatScale> {
        scale_in(&self.own_redeploy_numbers, v, true)
    }
}

fn scale_in(numbers: &[(f64, bool)], v: f64, complement: bool) -> Option<StatScale> {
    const EPS: f64 = 1e-3;
    let a = v.abs();
    let pct = a * 100.0;
    let ratio = numbers.iter().any(|&(n, p)| {
        p && ((n - pct).abs() < EPS || (complement && (1.0 - a).mul_add(-100.0, n).abs() < EPS))
    });
    let flat = numbers.iter().any(|&(n, p)| !p && (n - a).abs() < EPS);
    match (ratio, flat) {
        (true, false) => Some(StatScale::Ratio),
        (false, true) => Some(StatScale::Flat),
        _ => None,
    }
}

fn numbers_in(p: &str) -> Vec<(f64, bool)> {
    NUMBER
        .captures_iter(p)
        .filter_map(|c| Some((c[1].parse().ok()?, !c[2].is_empty())))
        .collect()
}

static SENTENCE_END: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[.;](?:\s|$)").expect("sentence regex"));

fn own_redeploy_numbers(p: &str, holder: &str) -> Vec<(f64, bool)> {
    SENTENCE_END
        .split(p)
        .map(str::trim)
        .filter(|sentence| {
            let lower = sentence.to_lowercase();
            let opens_with_holder = (!holder.is_empty() && sentence.starts_with(holder))
                || lower.starts_with("this unit");
            opens_with_holder && lower.contains("redeployment time")
        })
        .flat_map(numbers_in)
        .collect()
}

static NUMBER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(\d+(?:\.\d+)?)\s*(%?)").expect("number regex"));

static ALLY_WORD: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b(all|ally|allies|allied|operators?|friendly unit)\b").expect("ally regex")
});

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

fn counters_invisible(text: &str) -> bool {
    if !has_tag(text, "ba.invisible") {
        return false;
    }
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

static PLACEHOLDER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\{\+?([^{}:]+)(?::[^{}]*)?\}").expect("placeholder regex"));

fn dp_gain_keys(raw: &str) -> Vec<String> {
    let mut keys = Vec::new();
    let mut prev: Option<(usize, String)> = None;
    for c in PLACEHOLDER.captures_iter(raw) {
        let whole = c.get(0).expect("group 0");
        let key = c[1].trim().to_string();
        let lower_bound = prev
            .as_ref()
            .filter(|(end, _)| plain(&raw[*end..whole.start()]).trim() == "-")
            .map(|(_, k)| k.clone());
        prev = Some((whole.end(), key.clone()));
        if lower_bound.is_none() && raw[..whole.start()].ends_with('-') {
            continue;
        }
        let after = raw[whole.end()..]
            .trim_start_matches("</>")
            .trim_start()
            .to_lowercase();
        let Some(dp) = after
            .strip_prefix("dp")
            .or_else(|| after.strip_prefix("deployment point"))
        else {
            continue;
        };
        if dp.trim_start().starts_with("cost") {
            continue;
        }
        let before = raw[..whole.start()].to_lowercase();
        let lead = &before[before.char_indices().rev().nth(39).map_or(0, |(i, _)| i)..];
        if any_of(lead, &["consum", "spend", "costs", "up to", "additional"]) {
            continue;
        }
        if before
            .rfind("<@ba.rem>")
            .is_some_and(|i| !before[i..].contains("</>"))
        {
            continue;
        }
        keys.push(lower_bound.unwrap_or(key));
    }
    keys
}

fn dp_cost_cut_keys(raw: &str) -> Vec<(String, bool)> {
    let mut keys = Vec::new();
    for c in PLACEHOLDER.captures_iter(raw) {
        let whole = c.get(0).expect("group 0");
        let key = c[1].trim().to_string();
        let after = raw[whole.end()..]
            .trim_start_matches("</>")
            .trim_start()
            .to_lowercase();
        if raw[..whole.start()].ends_with('-') && after.starts_with("dp cost") {
            keys.push((key, true));
            continue;
        }
        let clause_start = raw[..whole.start()].rfind([';', ',']).map_or(0, |i| i + 1);
        let clause = lower_plain(&raw[clause_start..whole.start()]);
        let clause_end = after.find([';', ',']).unwrap_or(after.len());
        if clause.contains("dp cost of") && !PLACEHOLDER.is_match(&after[..clause_end]) {
            keys.push((key, false));
        }
    }
    keys
}

fn read(raw: &str, vocab: &Vocab, talent: bool, holder: &str) -> TextFacts {
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
            lowers_dp_floor: has_tag(raw, "ba.costlowerbound"),
            own_redeploy_numbers: own_redeploy_numbers(p, holder),
            numbers: numbers_in(p),
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
                "allied units around",
                "allies around",
            ],
        ),
        dp_gain: dp_gain_keys(raw),
        first_redeploy: lower.contains("first redeployment"),
        dp_cost_cut: dp_cost_cut_keys(raw),
        faction: parse_faction(raw, vocab),
        talent,
    }
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct TextIndex {
    #[serde(serialize_with = "entries")]
    facts: HashMap<TextKey, TextFacts>,
}

fn entries<S: serde::Serializer>(
    map: &HashMap<TextKey, TextFacts>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.collect_seq(map.iter())
}

impl TextIndex {
    #[must_use]
    pub fn get(&self, key: &TextKey) -> Option<&TextFacts> {
        self.facts.get(key)
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.facts.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.facts.is_empty()
    }

    fn build(data: &SquadIndexData<'_>) -> Self {
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
    let name = op.name.as_str();
    out.insert(
        TextKey::Trait { char_id: id() },
        read(&op.description, vocab, false, name),
    );
    if let Some(t) = &op.trait_data {
        for (cand, c) in t.candidates.iter().enumerate() {
            if let Some(d) = c.override_description.as_ref().filter(|d| !d.is_empty()) {
                out.insert(
                    TextKey::TraitCandidate {
                        char_id: id(),
                        cand,
                    },
                    read(d, vocab, false, name),
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
                read(d, vocab, true, name),
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
                        read(d, vocab, true, name),
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
                        out.insert(key(false), read(d, vocab, false, name));
                    }
                    if !c.additional_description.is_empty() {
                        out.insert(
                            key(true),
                            read(&c.additional_description, vocab, false, name),
                        );
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
            .or_insert_with(|| read(&l.description, vocab, false, name));
        }
    }
}

const fn obtainable_playable(op: &Operator) -> bool {
    playable(op) && !op.is_not_obtainable
}

#[must_use]
pub const fn playable(op: &Operator) -> bool {
    !matches!(
        op.profession,
        OperatorProfession::Token | OperatorProfession::Trap
    )
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct TagIndex {
    ids: HashMap<String, i32>,
    names: HashMap<i32, String>,
}

impl TagIndex {
    fn build(data: &SquadIndexData<'_>) -> Self {
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

    #[must_use]
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

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ThreatThresholds {
    pub high_res: Option<f64>,
    pub high_def: Option<f64>,
    pub fast_move_speed: Option<f64>,
    pub res_grade: String,
    pub def_grade: String,
    pub move_speed_grade: String,
}

impl ThreatThresholds {
    fn build(data: &SquadIndexData<'_>) -> Self {
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

#[derive(Debug, Clone, Default, Serialize)]
pub struct SquadIndex {
    pub text: TextIndex,
    pub tags: TagIndex,
    pub threats: ThreatThresholds,
    pub professions: Vec<OperatorProfession>,
    pub normal_of: HashMap<String, String>,
    pub token_branches: HashSet<String>,
    pub top_rarity: i16,
}

impl SquadIndex {
    #[must_use]
    pub fn build(data: &SquadIndexData<'_>) -> Self {
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
