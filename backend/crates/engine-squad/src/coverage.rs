use std::collections::HashMap;

use myrtle_gamedata::index::squad::TextFacts;
use myrtle_gamedata_raw::tables::operator::OperatorProfession;

use crate::{
    Ctx,
    build::{Board, Seat, bb},
    economy::{skill_dp, talent_gains_dp},
    policy::{BLOCK_ROW_MIN, CC_STATUS_KEYS, FAST_REDEPLOY_SECS, TAG_FALLBACK},
    types::{
        CoverageGroup, CoverageGroupKey, CoverageOp, CoverageRow, CoverageRowKey, Economy,
        SourceKind, SourceRef,
    },
};

/// Branches whose trait heals one ally at a time with an empty trait blackboard
const HEAL_SINGLE_CURATED: &[&str] = &["physician", "watchman", "guardian"];
/// `ringhealer`, whose trait restores the HP of 3 allies
const HEAL_AREA_CURATED: &[&str] = &["ringhealer"];
/// Archetypes that hit several targets by trait with no blackboard key for it
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
/// Branches whose trait moves enemies with an empty trait blackboard
const SHIFT_BRANCHES: &[&str] = &["hookmaster", "pusher"];
/// Hexers, whose trait debuffs with an empty trait blackboard
const DEBUFF_BRANCHES: &[&str] = &["underminer"];

fn heal_single_board(tb: &Board) -> bool {
    board_any_gt(tb, &["heal_scale", "ep_heal_ratio", "scale"], 0.0)
}

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

fn true_elemental(facts: Option<&TextFacts>, board: &Board) -> bool {
    facts.is_some_and(|f| f.true_damage)
        || board.iter().any(|(k, _)| {
            (k.starts_with("ep_damage") || k.starts_with("element")) && !k.contains("resist")
        })
}

fn enemy_debuff(facts: Option<&TextFacts>, board: &Board) -> bool {
    ["def", "magic_resistance", "atk"]
        .iter()
        .any(|k| bb(board, k).is_some_and(|v| v < 0.0))
        && facts.is_some_and(|f| f.names_enemy)
}

fn block_bonus_reaches(board: &Board, base_block: i32) -> bool {
    bb(board, "block_cnt")
        .is_some_and(|d| d > 0.0 && f64::from(base_block) + d >= f64::from(BLOCK_ROW_MIN))
}

fn board_shifts(board: &Board) -> bool {
    ["force", "attack@force"]
        .iter()
        .any(|k| bb(board, k).is_some_and(|v| v != 0.0))
}

fn board_heals_elemental(board: &Board) -> bool {
    board.iter().any(|(k, _)| k.starts_with("ep_heal"))
}

#[allow(clippy::too_many_lines)]
fn seat_hits(ctx: &Ctx<'_>, s: &Seat<'_>, economy: &Economy) -> Vec<(CoverageRowKey, SourceRef)> {
    use CoverageRowKey as R;
    let mut hits: Vec<(R, SourceRef)> = Vec::new();
    let sub = s.op.sub_profession_id.as_str();
    let tb = &s.trait_board;
    let tf = ctx.trait_facts(s);
    let tf = tf.as_ref();
    let says = |f: fn(&TextFacts) -> bool| tf.is_some_and(f);

    let mut tr = |row: R| hits.push((row, SourceRef::bare(SourceKind::Trait)));

    // DP per kill
    if bb(tb, "cost").is_some_and(|c| c > 0.0) && bb(tb, "interval").is_none() {
        tr(R::DpGen);
    }
    let trait_heals = heal_single_board(&s.branch_board) || HEAL_SINGLE_CURATED.contains(&sub);
    if trait_heals {
        tr(R::HealSingle);
    }
    if heal_area_board(&s.branch_board, &s.op.profession) || HEAL_AREA_CURATED.contains(&sub) {
        tr(R::HealArea);
    }
    if bb(tb, "ep_heal_ratio").is_some_and(|v| v > 0.0) {
        tr(R::HealElemental);
    }
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
    if s.op
        .display_token_dict
        .as_ref()
        .is_some_and(|d| !d.is_empty())
        && ctx.data.squad.token_branches.contains(sub)
    {
        tr(R::Summons);
    }

    let base_block = s.key_frame().map_or(0, |k| k.block_cnt);

    if let Some(sk) = &s.skill {
        let r = sk.source();
        let b = &sk.board;
        let f = ctx.facts(&sk.key);
        let says = |g: fn(&TextFacts) -> bool| f.is_some_and(g);
        let mut add = |row: R| hits.push((row, r.clone()));
        if skill_dp(b, f).is_some() {
            add(R::DpGen);
        }
        if block_bonus_reaches(b, base_block) {
            add(R::Block3);
        }
        let heal = board_any_gt(
            b,
            &[
                "heal_scale",
                "attack@heal_scale",
                "hp_recovery_per_sec_ratio",
            ],
            0.0,
        );
        if heal && says(|g| g.ally_word) {
            add(R::HealSingle);
        }
        let attacks_heal = trait_heals && bb(&s.branch_board, "scale").is_none();
        let heal_targets = board_any_gt(
            b,
            &["attack@max_target", "max_target", "chain.max_target"],
            1.0,
        );
        if (heal && says(|g| g.heal_area_words))
            || b.iter()
                .any(|(k, v)| k.contains("max_target_heal") && *v > 0.0)
            || ((heal || attacks_heal) && heal_targets)
        {
            add(R::HealArea);
        }
        if board_heals_elemental(b) {
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
        if board_shifts(b) {
            add(R::Shift);
        }
        if enemy_debuff(f, b) {
            add(R::Debuff);
        }
        if says(|g| g.counters_invisible) {
            add(R::Invisible);
        }
        let skill_token =
            s.op.skills
                .get(sk.index)
                .and_then(|e| e.override_token_key.as_ref())
                .is_some_and(|k| !k.is_empty());
        if skill_token {
            add(R::Summons);
        }
    }

    for t in &s.talents {
        let r = t.source();
        let b = &t.board;
        let f = ctx.facts(&t.key);
        let mut add = |row: R| hits.push((row, r.clone()));
        if f.is_some_and(|f| talent_gains_dp(f, b)) {
            add(R::DpGen);
        }
        if block_bonus_reaches(b, base_block) {
            add(R::Block3);
        }
        if board_heals_elemental(b) {
            add(R::HealElemental);
        }
        if true_elemental(f, b) {
            add(R::TrueElemental);
        }
        if board_has_cc(b) {
            add(R::Cc);
        }
        if board_shifts(b) {
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

    let eco = economy.seats.iter().find(|e| e.seat == s.seat);
    if let Some(e) = eco {
        if e.cost <= economy.start_dp - economy.dp_lower_limit {
            hits.push((R::Openers, SourceRef::bare(SourceKind::Stats)));
        }
        if e.respawn <= FAST_REDEPLOY_SECS {
            hits.push((R::FastRedeploy, SourceRef::bare(SourceKind::Stats)));
        }
    }
    if s.key_frame().is_some_and(|k| k.block_cnt >= BLOCK_ROW_MIN) {
        hits.push((R::Block3, SourceRef::bare(SourceKind::Stats)));
    }

    for (tag, row) in TAG_FALLBACK {
        if s.tag_ids.contains(tag) {
            hits.push((*row, SourceRef::bare(SourceKind::Tag)));
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
