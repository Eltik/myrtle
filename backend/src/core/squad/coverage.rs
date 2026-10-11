//! The Coverage checklist: which seats answer each need, and from what.
//!
//! Every row reads a game-data field first (trait text or blackboard, the
//! chosen skill's blackboard or text, live talents, promotion stats) and falls
//! back to the operator's recruitment tag. Lists of sub-professions are CURATED
//! where the trait encodes the capability in neither text nor blackboard; each
//! says so where it is declared.

use std::collections::HashMap;

use crate::core::gamedata::types::operator::OperatorProfession;

use super::Ctx;
use super::build::{Board, Seat, bb};
use super::economy::{skill_dp, talent_gains_dp};
use super::index::TextFacts;
use super::policy::{BLOCK_ROW_MIN, CC_STATUS_KEYS, FAST_REDEPLOY_SECS, TAG_FALLBACK};
use super::types::{
    CoverageGroup, CoverageGroupKey, CoverageOp, CoverageRow, CoverageRowKey, Economy, SourceKind,
    SourceRef,
};

/// CURATED: branches whose trait heals one ally at a time with an EMPTY trait
/// blackboard, so no field says so: `physician` (17 Medics, one with only a
/// `char_id` key), `watchman` (Taraxacum) and `guardian` (Guardian Defenders,
/// who heal through skills). The other single-heal branches are read off
/// their trait blackboard (`heal_single_board`).
const HEAL_SINGLE_CURATED: &[&str] = &["physician", "watchman", "guardian"];
/// CURATED: `ringhealer`, whose trait ("Restores the HP of 3 allies") has an
/// empty blackboard. Chain and Bard healers are read off their blackboard.
const HEAL_AREA_CURATED: &[&str] = &["ringhealer"];
/// CURATED: archetypes that hit several targets by trait with no blackboard
/// key that says so (checked on 2026-10-11: empty or unrelated trait
/// blackboards). Ifrit's "AOE Arts damage in a long line" has no
/// `max_target`. `hammer` is read off `attack@ability_range_radius`.
const AREA_BRANCHES: &[&str] = &[
    "aoesniper",
    "splashcaster",
    "blastcaster",
    "reaperrange",
    "bombarder",
    "centurion",
    "crusher",
    "reaper",
    "phalanx",
    "fortress",
    "stalker",
    "pusher",
];
/// CURATED: branches whose trait moves enemies with an empty trait
/// blackboard (no `force`).
const SHIFT_BRANCHES: &[&str] = &["hookmaster", "pusher"];
/// CURATED: Hexers, whose trait debuffs with an empty trait blackboard.
const DEBUFF_BRANCHES: &[&str] = &["underminer"];

/// A branch trait blackboard (`Seat::branch_board`, before any module) that
/// heals one ally: `heal_scale` (Therapists, Abjurers), `ep_heal_ratio`
/// (Wandering Medics) or `scale` (Incantation Medics, the only branch whose
/// trait carries it). Read before the module, because the Crusher modules
/// add a self-heal `heal_scale` to a trait that heals nobody else.
fn heal_single_board(tb: &Board) -> bool {
    board_any_gt(tb, &["heal_scale", "ep_heal_ratio", "scale"], 0.0)
}

/// A branch trait blackboard that heals several allies: a Medic's chained
/// `max_target` (Chain Healers' `attack@chain.max_target`; Chain Casters carry
/// the same key and are not Medics) or an `atk_to_hp_recovery_ratio` (Bards).
fn heal_area_board(tb: &Board, prof: &OperatorProfession) -> bool {
    (*prof == OperatorProfession::Medic
        && tb
            .iter()
            .any(|(k, v)| k.ends_with("max_target") && *v > 1.0))
        || tb
            .iter()
            .any(|(k, v)| k.ends_with("atk_to_hp_recovery_ratio") && *v > 0.0)
}

const GROUPS: &[(CoverageGroupKey, &[CoverageRowKey])] = &[
    (
        CoverageGroupKey::GettingOnTheField,
        &[
            CoverageRowKey::DpGen,
            CoverageRowKey::Openers,
            CoverageRowKey::FastRedeploy,
        ],
    ),
    (
        CoverageGroupKey::HoldingTheLine,
        &[
            CoverageRowKey::Block3,
            CoverageRowKey::HealSingle,
            CoverageRowKey::HealArea,
            CoverageRowKey::HealElemental,
        ],
    ),
    (
        CoverageGroupKey::Damage,
        &[
            CoverageRowKey::Physical,
            CoverageRowKey::Arts,
            CoverageRowKey::TrueElemental,
            CoverageRowKey::AntiAir,
            CoverageRowKey::Multi,
        ],
    ),
    (
        CoverageGroupKey::Control,
        &[
            CoverageRowKey::Cc,
            CoverageRowKey::Shift,
            CoverageRowKey::Debuff,
        ],
    ),
    (
        CoverageGroupKey::Utility,
        &[CoverageRowKey::Invisible, CoverageRowKey::Summons],
    ),
];

const fn rank(k: SourceKind) -> u8 {
    match k {
        SourceKind::Trait => 0,
        SourceKind::Skill => 1,
        SourceKind::Talent => 2,
        SourceKind::Module => 3,
        SourceKind::Stats => 4,
        SourceKind::Tag => 5,
    }
}

/// Whether a blackboard key names the status `tok` as a whole segment.
fn key_has_token(key: &str, tok: &str) -> bool {
    key.split(['@', '.', '[', ']'])
        .any(|seg| seg == tok || seg.strip_prefix(tok).is_some_and(|r| r.starts_with('_')))
}

fn board_has_cc(board: &Board) -> bool {
    board
        .iter()
        .any(|(k, _)| CC_STATUS_KEYS.iter().any(|t| key_has_token(k, t)))
}

fn board_any_gt(board: &Board, keys: &[&str], min: f64) -> bool {
    keys.iter().any(|k| bb(board, k).is_some_and(|v| v > min))
}

/// Elemental or True damage: the text says so (`facts.true_damage`), or the
/// blackboard has a key starting `ep_damage`/`element` that is not a
/// resistance (Mulberry S2 `ep_damage_resistance` protects allies).
fn true_elemental(facts: Option<&TextFacts>, board: &Board) -> bool {
    facts.is_some_and(|f| f.true_damage)
        || board.iter().any(|(k, _)| {
            (k.starts_with("ep_damage") || k.starts_with("element")) && !k.contains("resist")
        })
}

/// An enemy stat cut on a blackboard (`def`, `magic_resistance` or `atk` < 0)
/// whose text speaks of enemies or "the target", so a self penalty (`SilverAsh`
/// S3 `def` -0.7) does not count.
fn enemy_debuff(facts: Option<&TextFacts>, board: &Board) -> bool {
    ["def", "magic_resistance", "atk"]
        .iter()
        .any(|k| bb(board, k).is_some_and(|v| v < 0.0))
        && facts.is_some_and(|f| f.names_enemy)
}

const fn src(kind: SourceKind) -> SourceRef {
    SourceRef {
        kind,
        name: None,
        skill_index: None,
    }
}

/// Every (row, source) one seat earns; the caller keeps the first source per
/// row in `trait, skill, talent, module, stats, tag` order. Word rules read
/// the load-time English readings (`ctx`); a seat with none gets only the
/// rules that read blackboards, stats, ids and tags.
#[allow(clippy::too_many_lines)] // one rule per row, in display order
fn seat_hits(ctx: &Ctx<'_>, s: &Seat<'_>, economy: &Economy) -> Vec<(CoverageRowKey, SourceRef)> {
    use CoverageRowKey as R;
    let mut hits: Vec<(R, SourceRef)> = Vec::new();
    let sub = s.op.sub_profession_id.as_str();
    let tb = &s.trait_board;
    let tf = ctx.trait_facts(s);
    let tf = tf.as_ref();
    let says = |f: fn(&TextFacts) -> bool| tf.is_some_and(f);

    // ---- trait
    let mut tr = |row: R| hits.push((row, src(SourceKind::Trait)));
    // DP per kill: a positive trait `cost` with no `interval` (Chargers, the
    // only branch with that shape; Merchants pay `cost` < 0 per `interval`).
    if bb(tb, "cost").is_some_and(|c| c > 0.0) && bb(tb, "interval").is_none() {
        tr(R::DpGen);
    }
    if heal_single_board(&s.branch_board) || HEAL_SINGLE_CURATED.contains(&sub) {
        tr(R::HealSingle);
    }
    if heal_area_board(&s.branch_board, &s.op.profession) || HEAL_AREA_CURATED.contains(&sub) {
        tr(R::HealArea);
    }
    if bb(tb, "ep_heal_ratio").is_some_and(|v| v > 0.0) {
        tr(R::HealElemental);
    }
    // Base damage type: Casters and any trait that says "Arts damage" deal
    // Arts; Medics (unless Arts) and traits that say the unit does not attack
    // or restores HP deal none; everyone else deals Physical. A seat with no
    // English reading gets Arts from the profession and no Physical guess.
    let arts = s.op.profession == OperatorProfession::Caster || says(|f| f.arts_damage);
    let no_damage =
        (s.op.profession == OperatorProfession::Medic && !arts) || says(|f| f.no_attack);
    if arts {
        tr(R::Arts);
    } else if !no_damage && tf.is_some() {
        tr(R::Physical);
    }
    if true_elemental(tf, tb) {
        tr(R::TrueElemental);
    }
    if says(|f| f.aerial_first) {
        tr(R::AntiAir);
    }
    if board_any_gt(
        tb,
        &["attack@max_target", "max_target", "attack@chain.max_target"],
        1.0,
    ) || board_any_gt(&s.branch_board, &["attack@ability_range_radius"], 0.0)
        || says(|f| f.all_targets)
        || AREA_BRANCHES.contains(&sub)
    {
        tr(R::Multi);
    }
    if board_has_cc(tb) {
        tr(R::Cc);
    }
    if SHIFT_BRANCHES.contains(&sub) {
        tr(R::Shift);
    }
    if DEBUFF_BRANCHES.contains(&sub) {
        tr(R::Debuff);
    }
    if says(|f| f.counters_invisible) {
        tr(R::Invisible);
    }
    // A branch whose every operator carries a token (`token_branches`) gets
    // it from the trait; the seat must carry one too.
    if s.op
        .display_token_dict
        .as_ref()
        .is_some_and(|d| !d.is_empty())
        && ctx.data.squad.token_branches.contains(sub)
    {
        tr(R::Summons);
    }

    // ---- skill
    if let Some(sk) = &s.skill {
        let r = SourceRef {
            kind: SourceKind::Skill,
            name: Some(sk.level.name.clone()),
            skill_index: i32::try_from(sk.index).ok(),
        };
        let b = &sk.board;
        let f = ctx.facts(&sk.key);
        let says = |g: fn(&TextFacts) -> bool| f.is_some_and(g);
        let mut add = |row: R| hits.push((row, r.clone()));
        if skill_dp(b).is_some() {
            add(R::DpGen);
        }
        let base_block = s.key_frame().map_or(0, |k| k.block_cnt);
        if bb(b, "block_cnt")
            .is_some_and(|d| d > 0.0 && f64::from(base_block) + d >= f64::from(BLOCK_ROW_MIN))
        {
            add(R::Block3);
        }
        let heal = board_any_gt(b, &["heal_scale", "attack@heal_scale"], 0.0);
        if heal && says(|g| g.ally_word) {
            add(R::HealSingle);
        }
        if heal && says(|g| g.heal_area_words) {
            add(R::HealArea);
        }
        if b.iter().any(|(k, _)| k.starts_with("ep_heal")) {
            add(R::HealElemental);
        }
        if says(|g| g.arts_damage) {
            add(R::Arts);
        }
        if says(|g| g.physical_damage) {
            add(R::Physical);
        }
        if true_elemental(f, b) {
            add(R::TrueElemental);
        }
        if board_any_gt(
            b,
            &["max_target", "attack@max_target", "max_target_cnt"],
            1.0,
        ) || says(|g| g.all_enemies)
        {
            add(R::Multi);
        }
        if board_has_cc(b) {
            add(R::Cc);
        }
        if ["force", "attack@force"]
            .iter()
            .any(|k| bb(b, k).is_some_and(|v| v != 0.0))
        {
            add(R::Shift);
        }
        if enemy_debuff(f, b) {
            add(R::Debuff);
        }
        if says(|g| g.counters_invisible) {
            add(R::Invisible);
        }
        // A skill that places its own unit carries `OverrideTokenKey`.
        let skill_token =
            s.op.skills
                .get(sk.index)
                .and_then(|e| e.override_token_key.as_ref())
                .is_some_and(|k| !k.is_empty());
        if skill_token {
            add(R::Summons);
        }
    }

    // ---- talents
    for t in &s.talents {
        let r = SourceRef {
            kind: SourceKind::Talent,
            name: Some(t.name.clone()),
            skill_index: None,
        };
        let b = &t.board;
        let f = ctx.facts(&t.key);
        let mut add = |row: R| hits.push((row, r.clone()));
        if f.is_some_and(|f| talent_gains_dp(f, b)) {
            add(R::DpGen);
        }
        let base_block = s.key_frame().map_or(0, |k| k.block_cnt);
        if bb(b, "block_cnt")
            .is_some_and(|d| d > 0.0 && f64::from(base_block) + d >= f64::from(BLOCK_ROW_MIN))
        {
            add(R::Block3);
        }
        if b.iter().any(|(k, _)| k.starts_with("ep_heal")) {
            add(R::HealElemental);
        }
        if true_elemental(f, b) {
            add(R::TrueElemental);
        }
        if board_has_cc(b) {
            add(R::Cc);
        }
        if ["force", "attack@force"]
            .iter()
            .any(|k| bb(b, k).is_some_and(|v| v != 0.0))
        {
            add(R::Shift);
        }
        if enemy_debuff(f, b) {
            add(R::Debuff);
        }
        if f.is_some_and(|f| f.counters_invisible) {
            add(R::Invisible);
        }
        if t.token_key.is_some() {
            add(R::Summons);
        }
    }

    // ---- stats
    let eco = economy.seats.iter().find(|e| e.seat == s.seat);
    if let Some(e) = eco {
        if e.cost <= economy.start_dp {
            hits.push((R::Openers, src(SourceKind::Stats)));
        }
        if e.respawn <= FAST_REDEPLOY_SECS {
            hits.push((R::FastRedeploy, src(SourceKind::Stats)));
        }
    }
    if s.key_frame().is_some_and(|k| k.block_cnt >= BLOCK_ROW_MIN) {
        hits.push((R::Block3, src(SourceKind::Stats)));
    }

    // ---- recruitment tags, the fallback, by tag id on the seat's server
    for (tag, row) in TAG_FALLBACK {
        if s.tag_ids.contains(tag) {
            hits.push((*row, src(SourceKind::Tag)));
        }
    }
    hits
}

pub fn compute(ctx: &Ctx<'_>, seats: &[Seat<'_>], economy: &Economy) -> Vec<CoverageGroup> {
    let mut by_row: HashMap<CoverageRowKey, Vec<CoverageOp>> = HashMap::new();
    for s in seats {
        let mut best: HashMap<CoverageRowKey, SourceRef> = HashMap::new();
        for (row, why) in seat_hits(ctx, s, economy) {
            let better = best.get(&row).is_none_or(|b| rank(why.kind) < rank(b.kind));
            if better {
                best.insert(row, why);
            }
        }
        for (row, why) in best {
            by_row.entry(row).or_default().push(CoverageOp {
                seat: s.seat,
                char_id: s.char_id.clone(),
                support: s.support,
                why,
            });
        }
    }
    GROUPS
        .iter()
        .map(|(key, rows)| CoverageGroup {
            key: *key,
            rows: rows
                .iter()
                .map(|r| {
                    let mut ops = by_row.remove(r).unwrap_or_default();
                    ops.sort_by_key(|o| o.seat);
                    CoverageRow { key: *r, ops }
                })
                .collect(),
        })
        .collect()
}
