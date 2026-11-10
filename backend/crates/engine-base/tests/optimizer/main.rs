//! End-to-end checks for the infrastructure (base) optimizer against real game
//! data, using generic constructed bases and rosters (no captured user). Focus:
//! named-teammate conditional synergies, Control Center stacking and faction
//! buffs, the staggered rotation, and the high-level grade.

// Test-local `const CHAR: &str = "..."` next to the assertions and small numeric casts on
// counts are deliberate: pedantic/nursery lints not worth reshaping test code over.
#![allow(
    clippy::items_after_statements,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::too_many_lines,
    clippy::ignore_without_reason,
    clippy::float_cmp,
    clippy::iter_on_single_items,
    clippy::wildcard_imports
)]
// The ignored probes read a repro dump named by an env var and one times itself.
#![allow(clippy::disallowed_methods)]

mod common;
mod control;
mod factory;
mod ledger;
mod morale;
mod optimizer;
mod pools;
mod registry;
mod rotation;
mod support;
mod trading;

use common::{load_game_data, max_stationed};
use myrtle_engine_base::BaseData;
use myrtle_engine_base::assignment::{
    compute_optimal_assignment, compute_optimal_assignment_with_pins, compute_sustained_assignment,
    team_value,
};
use myrtle_engine_base::registry::{
    BuffResolutionStrategy, OrderEffect, build_name_to_char, build_registry,
};
use myrtle_engine_base::score::grade_base;
use myrtle_engine_base::types::{OperatorBaseProfile, UserBuilding, UserRoom};
use myrtle_types::roster::RosterEntry;

fn room(slot: &str, room_type: &str, level: i32) -> UserRoom {
    UserRoom {
        slot_id: slot.into(),
        room_type: room_type.into(),
        level,
        ..Default::default()
    }
}

/// A realistic base layout (Control Center + 2 trading posts + 4 factories + 4
/// dormitories), used instead of any captured user.
fn generic_base() -> UserBuilding {
    let mut rooms = vec![room("cc", "CONTROL", 5)];
    rooms.extend((0..2).map(|i| room(&format!("tp{i}"), "TRADING", 3)));
    rooms.extend((0..4).map(|i| room(&format!("mf{i}"), "MANUFACTURE", 3)));
    rooms.extend((0..4).map(|i| room(&format!("d{i}"), "DORMITORY", 5)));
    UserBuilding {
        drone_capacity: None,
        rooms,
    }
}

/// Every operator with base skills, at E2-max: a deep, user-independent roster
/// (mirrors the "best possible" baseline the grader builds).
fn full_roster(gd: &BaseData) -> Vec<OperatorBaseProfile> {
    gd.building()
        .chars
        .keys()
        .filter(|id| id.starts_with("char_"))
        .map(|id| profile(gd, id))
        .collect()
}

/// Faction tags plus the game's glossary tags (`cc.tag.knight` lists Gravel,
/// Viviana, the Nearls...), the way `BaseContext::build` derives them.
fn faction_tags_with_glossary(
    gd: &BaseData,
    char_id: &str,
    op: &myrtle_gamedata_raw::tables::operator::Operator,
) -> Vec<String> {
    let glossary = myrtle_engine_base::registry::glossary_tags(
        gd.consts(),
        &build_name_to_char(gd.operators()),
    );
    myrtle_engine_base::registry::faction_tags_for(char_id, op, &glossary)
}

/// A minimal E2-max `RosterEntry` for an operator (for the high-level `grade_base`).
fn roster_entry(operator_id: &str) -> RosterEntry {
    RosterEntry {
        user_id: uuid::Uuid::nil(),
        operator_id: operator_id.into(),
        elite: 2,
        level: 90,
        exp: 0,
        potential: 0,
        skill_level: 7,
        favor_point: 0,
        skin_id: None,
        default_skill: None,
        voice_lan: None,
        current_equip: None,
        current_tmpl: None,
        obtained_at: None,
        masteries: serde_json::json!([]),
        modules: serde_json::json!([]),
    }
}

/// Build a fully-elited (E2-max) base profile for an operator by taking the
/// highest-unlock buff in every skill slot.
fn profile(gd: &BaseData, char_id: &str) -> OperatorBaseProfile {
    let bc = gd
        .building()
        .chars
        .get(char_id)
        .unwrap_or_else(|| panic!("{char_id} missing from building data"));
    let available_buffs: Vec<String> = bc
        .buff_char
        .iter()
        .filter_map(|slot| slot.buff_data.last().map(|e| e.buff_id.clone()))
        .collect();
    let faction_tags = gd
        .operators()
        .get(char_id)
        .map(|op| faction_tags_with_glossary(gd, char_id, op))
        .unwrap_or_default();
    let match_tags = myrtle_engine_base::types::compute_match_tags(
        &faction_tags,
        &available_buffs,
        gd.building(),
    );
    OperatorBaseProfile {
        char_id: char_id.to_string(),
        available_buffs,
        faction_tags,
        match_tags,
        rarity: gd
            .operators()
            .get(char_id)
            .map_or(0, |o| o.rarity.to_star_int()),
        elite: 2,
    }
}

/// [`profile`] limited to the base skills unlocked at `elite` (the highest
/// tier at or below it in every slot), for pricing an E0 operator.
fn profile_at(gd: &BaseData, char_id: &str, elite: i32) -> OperatorBaseProfile {
    let bc = gd
        .building()
        .chars
        .get(char_id)
        .unwrap_or_else(|| panic!("{char_id} missing from building data"));
    let available_buffs: Vec<String> = bc
        .buff_char
        .iter()
        .filter_map(|slot| {
            slot.buff_data
                .iter()
                .rfind(|e| e.cond.elite() <= elite)
                .map(|e| e.buff_id.clone())
        })
        .collect();
    let faction_tags = gd
        .operators()
        .get(char_id)
        .map(|op| faction_tags_with_glossary(gd, char_id, op))
        .unwrap_or_default();
    let match_tags = myrtle_engine_base::types::compute_match_tags(
        &faction_tags,
        &available_buffs,
        gd.building(),
    );
    OperatorBaseProfile {
        char_id: char_id.to_string(),
        available_buffs,
        faction_tags,
        match_tags,
        rarity: gd
            .operators()
            .get(char_id)
            .map_or(0, |o| o.rarity.to_star_int()),
        elite: elite as i16,
    }
}

fn trading_post(level: i32) -> UserBuilding {
    single_room("TRADING", level)
}

fn single_room(room_type: &str, level: i32) -> UserBuilding {
    UserBuilding {
        drone_capacity: None,
        rooms: vec![UserRoom {
            slot_id: "slot_1".into(),
            room_type: room_type.into(),
            level,
            ..Default::default()
        }],
    }
}

fn trading_efficiency(gd: &BaseData, roster: &[OperatorBaseProfile]) -> f64 {
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);
    let asn =
        compute_optimal_assignment(roster, &trading_post(3), gd.building(), &registry, &drains);
    asn.rooms
        .iter()
        .find(|r| r.room_type == "TRADING")
        .map_or(0.0, |r| r.total_efficiency)
}

/// The trading post's potential LMD/day: reflects BOTH order speed and order
/// value (so Proviso, who adds value not speed, shows up here).
fn trading_lmd(gd: &BaseData, roster: &[OperatorBaseProfile]) -> f64 {
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);
    let asn =
        compute_optimal_assignment(roster, &trading_post(3), gd.building(), &registry, &drains);
    asn.rooms
        .iter()
        .find(|r| r.room_type == "TRADING")
        .map_or(0.0, |r| {
            myrtle_engine_base::scoring::yield_model::room_yield(
                &r.room_type,
                r.formula_type.as_deref(),
                r.level,
                r.total_efficiency,
                r.order_value,
                r.operators.len(),
                r.order_limit,
            )
            .lmd_per_day
        })
}

fn factory_efficiency(gd: &BaseData, roster: &[OperatorBaseProfile]) -> f64 {
    let name_to_char = build_name_to_char(gd.operators());
    let (registry, drains) = build_registry(&gd.building().buffs, &name_to_char);
    let asn = compute_optimal_assignment(
        roster,
        &single_room("MANUFACTURE", 3),
        gd.building(),
        &registry,
        &drains,
    );
    asn.rooms
        .iter()
        .find(|r| r.room_type == "MANUFACTURE")
        .map_or(0.0, |r| r.total_efficiency)
}

const TEXAS: &str = "char_102_texas";
const EXUSIAI: &str = "char_103_angel";
const LAPPLAND: &str = "char_140_whitew";

// ─── BanG Dream "Passion" combo: joint, slot-budgeted Control-Center valuation ──────────────

const SAKIKO: &str = "char_4182_oblvns"; // Plentiful Work Experience: global FACTORY consumer
const MORTIS: &str = "char_4183_mortis"; // Monster of Acting: generates +20 AND global TRADING consumer
const AMORIS: &str = "char_4185_amoris"; // Diligent Worker: generates +10
const DOLRIS: &str = "char_4184_dolris"; // Idol's Aura: generates +1 per dormitory operator
const TMORIS: &str = "char_4186_tmoris"; // Reliable Companion: generates +10

/// A real 243 layout (Control Center + 2 trading + 4 factories + 3 power + dorms) so the
/// resource economy is in play: the whole Passion combo lives in the Control Center.
fn bd_base_243() -> UserBuilding {
    let mut rooms = vec![room("cc", "CONTROL", 5)];
    rooms.extend((0..2).map(|i| room(&format!("tp{i}"), "TRADING", 3)));
    rooms.extend((0..4).map(|i| room(&format!("mf{i}"), "MANUFACTURE", 3)));
    rooms.extend((0..3).map(|i| room(&format!("pp{i}"), "POWER", 3)));
    rooms.extend((0..4).map(|i| room(&format!("d{i}"), "DORMITORY", 5)));
    UserBuilding {
        drone_capacity: None,
        rooms,
    }
}

/// Owns at least one Control Center buff.
fn is_control_op(gd: &BaseData, op: &OperatorBaseProfile) -> bool {
    op.available_buffs.iter().any(|b| {
        gd.building()
            .buffs
            .get(b)
            .is_some_and(|buff| buff.room_type == "CONTROL")
    })
}

// ─── Part C: Reception Room ambience model (rarity + elite + skill, exchange & solo) ─────────

const CAPER: &str = "char_4100_caper"; // +30% clue search, ONLY during Clue Exchange
const VIGIL: &str = "char_427_vigil"; // +25% clue search
const RED: &str = "char_144_red"; // +25% clue search

fn meeting_room(level: i32) -> UserBuilding {
    single_room("MEETING", level)
}

/// Score one trading crew at a post level through the ledger alone, as the
/// community sheets price it: no Control Center, no facility context.
fn ledger_totals(
    gd: &BaseData,
    registry: &std::collections::HashMap<String, BuffResolutionStrategy>,
    crew: &[OperatorBaseProfile],
    level: i32,
) -> myrtle_engine_base::scoring::ledger::RoomTotals {
    use myrtle_engine_base::scoring::ledger::{RoomEval, score_room};
    let op_index: std::collections::HashMap<&str, &OperatorBaseProfile> =
        crew.iter().map(|o| (o.char_id.as_str(), o)).collect();
    let ids: Vec<String> = crew.iter().map(|o| o.char_id.clone()).collect();
    let counts = std::collections::HashMap::new();
    score_room(&RoomEval {
        member_ids: &ids,
        room_type: "TRADING",
        formula_type: None,
        op_index: &op_index,
        registry,
        building_data: gd.building(),
        facility_counts: &counts,
        total_dorm_levels: 0,
        cc_conditions: &[],
        room_level: Some(level),
        deployed_work_area: None,
    })
}
