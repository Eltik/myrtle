//! Recommended 3-shift base rotation, to compare against the player's presets.
//!
//! It follows the login rhythm players run:
//!
//! - **Production groups** (gold factories, EXP factories, posts) get
//!   `ceil(3N/2)` teams (3 for a pair of rooms) tiled as 24h blocks: two
//!   consecutive shifts on (wrapping), one off, and ONE team per group swaps at
//!   each 12h login:
//!
//!   ```text
//!     room 1:  Team A   Team A   Team B
//!     room 2:  Team B   Team C   Team C
//!   ```
//!
//! - **Power, Office, Reception, Control Center** alternate two squads: Squad 1
//!   on shifts 1 & 3, Squad 2 on shift 2. When a CC operator has a cross-room
//!   synergy (Viviana needs her Knights in the factories), the CC flips to a 24h
//!   BLOCK (Squad 1 on 1+2, Squad 2 on 3) and the dependent team is phased onto
//!   the same block so provider and dependents work together.
//!
//! Production teams are picked jointly (`team_select`), not best-first, so the
//! strong operators don't stack into one 107% team and leave the last hollow.
//!
//! An operator held 24/7 by a morale-swap manager (Fiammetta) is pinned to one
//! room every shift and badged; owning a manager also proposes sustaining the
//! highest-gain trading operator.

use std::collections::{HashMap, HashSet};

use crate::core::gamedata::types::building::BuildingDataFile;

use super::types::UserRoom;
use super::{
    assignment::{
        assign_auxiliary_rooms, base_wide_relevant, build_op_index, compute_team_efficiency,
        compute_team_totals, effective_facility_counts, fill_remaining_slots, morale_recovery,
        num_morale_swap_managers, op_uptime, resolve_base_wide, resolve_room_presence,
        room_presence_relevant, room_search_score, rotation_cc_plan,
    },
    buff_registry::BuffResolutionStrategy,
    team_select::{PlannedGroup, plan_production_groups, tiled_objective},
    types::{OperatorBaseProfile, UserBuilding},
    util::{is_production_room, max_stationed_at_level},
};

/// 12h each, two logins a day.
pub const SHIFT_COUNT: usize = 3;

/// `SHIFT_COUNT` shifts, each with every room's recommended crew beside the
/// player's preset.
pub struct ShiftRotation {
    pub shifts: Vec<Shift>,
    /// Held 24/7 by a morale-swap manager (Fiammetta): pinned to one room every shift.
    /// Badged "24/7 - Fiammetta" in the frontend.
    pub sustained: Vec<String>,
    /// Operators on SPARE CC seats after every value pick (bonus greedy, morale
    /// reservation, economy pins): chosen for low opportunity cost, not skills.
    /// Badged so a gated skill on a benchwarmer doesn't read as the reasoning.
    pub bench: Vec<String>,
    /// Zero-morale TOKENS (the "dead Lancet"): a named plant-count gate counts them
    /// at any morale and a robot-exclusion gate stops seeing them once dead, so both
    /// apply. Same room every shift, never rested, own skills forfeited; the morale
    /// sim leaves them at zero by design.
    pub parked: Vec<String>,
}

pub struct Shift {
    /// 1-indexed shift number.
    pub index: usize,
    pub rooms: Vec<ShiftRoom>,
}

#[derive(Clone)]
pub struct ShiftRoom {
    pub slot_id: String,
    pub room_type: String,
    /// Production formula for factories (`F_GOLD`/`F_EXP`/...), else `None`.
    pub formula_type: Option<String>,
    pub recommended: Vec<String>,
    /// The player's preset for this room and shift, if any.
    pub current: Vec<String>,
    /// False when deliberately UNSTAFFED (no spare team; resting dark beats burning
    /// benchwarmers' morale).
    pub active: bool,
    /// Crew efficiency % (speed incl. Squad-1 globals) for production and power
    /// cells, to show how output is distributed.
    pub efficiency: Option<f64>,
    /// Same id across the two shift columns a production team's 24h block covers,
    /// so the frontend can tie them together.
    pub team_id: Option<String>,
    /// Display label: "Team A/B/C" for production blocks, "Squad 1/2" elsewhere.
    pub team_label: Option<String>,
}

#[derive(Clone, Copy, PartialEq)]
enum SquadPattern {
    /// Squad 1 works shifts 1 & 3, Squad 2 shift 2: the per-login swap.
    Alternating,
    /// Squad 1 works shifts 1+2 (24h block aligned with its dependent production
    /// teams), Squad 2 shift 3.
    Block,
}

impl SquadPattern {
    /// Which squad (0/1) staffs shift `k`.
    fn squad_at(self, k: usize) -> usize {
        match self {
            Self::Alternating => k % 2,
            Self::Block => usize::from(k >= 2),
        }
    }
}

/// Seat a 24/7 pin into every cell of `slot` in a plan that reserved the seat:
/// each cell becomes `team + pinned`, re-scored by the ledger (a Shamare pin
/// zeroes the bodies beside her; a Tequila pin adds his rider to the cell's
/// Tailoring), so the tiled objective prices the pin as it will really run.
#[allow(clippy::too_many_arguments)]
fn seat_pinned_operator(
    groups: &mut [PlannedGroup],
    slot: &str,
    pinned: &str,
    op_index: &HashMap<&str, &OperatorBaseProfile>,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
    facility_counts: &HashMap<String, usize>,
    total_dorm_levels: i32,
    morale_drains: &HashMap<String, f64>,
    cc_conditions: &[super::assignment::CcCondition],
) {
    for g in groups.iter_mut() {
        let Some(ri) = g.rooms.iter().position(|(s, _)| s == slot) else {
            continue;
        };
        for shift in 0..SHIFT_COUNT {
            let Some(team) = g.teams.get(g.cells[ri][shift]) else {
                continue;
            };
            let mut ops = team.ops.clone();
            if !ops.iter().any(|o| o == pinned) {
                ops.push(pinned.to_string());
            }
            let totals = compute_team_totals(
                &ops,
                &g.room_type,
                g.formula_type.as_deref(),
                Some(g.rooms[ri].1),
                op_index,
                registry,
                building_data,
                facility_counts,
                total_dorm_levels,
                morale_drains,
                cc_conditions,
            );
            let score = room_search_score(&g.room_type, totals.speed_pct, totals.order_value_pct);
            g.teams.push(super::assignment::CandidateTeam {
                ops,
                speed: totals.speed_pct,
                value: totals.order_value_pct,
                gold: totals.order_gold_pct,
                order_limit: totals.order_limit,
                score,
            });
            g.cells[ri][shift] = g.teams.len() - 1;
        }
    }
}

/// Operators in EVERY saved preset of a production room (never rotated out) that
/// a morale-swap manager (Fiammetta) can hold at full morale. Capped at managers
/// owned, lowest natural uptime first: a low-drain operator sustains itself and
/// shouldn't spend a manager. Empty with no manager or nobody in every preset.
fn preset_sustained_operators(
    building: &UserBuilding,
    operators: &[OperatorBaseProfile],
    morale_drains: &HashMap<String, f64>,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
) -> HashSet<String> {
    let managers = num_morale_swap_managers(operators, building_data);
    // No manager, or no dorm that can host one (her seat + the swap seat): she has
    // to STAY in a dorm to work.
    if managers == 0 || !super::dorms::building_hosts_manager(building, building_data) {
        return HashSet::new();
    }
    // Her swap needs HER at full morale (Fiammetta recharges +2/h exclusive, one
    // swap per 12h login), so she can only hold drains that don't outrun that.
    let swap_rate = super::assignment::morale_swap_enabler(operators, building_data)
        .and_then(|id| operators.iter().find(|o| o.char_id == id))
        .map_or(0.0, |m| {
            super::dorms::manager_swap_rate(m, registry, building_data)
        });
    // Present in every non-empty preset of a production room, with two distinct
    // presets required: a single preset is just the current team, not a deliberate
    // round-the-clock hold.
    let mut candidates: HashSet<String> = HashSet::new();
    for room in building
        .rooms
        .iter()
        .filter(|r| is_production_room(&r.room_type))
    {
        let presets: Vec<&Vec<String>> = room
            .preset_shifts
            .iter()
            .filter(|p| !p.is_empty())
            .collect();
        if presets.len() < 2 {
            continue;
        }
        for op in presets[0] {
            if presets.iter().all(|p| p.contains(op)) {
                let feasible = operators.iter().find(|o| &o.char_id == op).is_none_or(|o| {
                    super::dorms::manager_can_sustain(
                        swap_rate,
                        super::sustain_sim::game_morale_drain(o, morale_drains),
                    )
                });
                if feasible {
                    candidates.insert(op.clone());
                }
            }
        }
    }
    // Lowest natural uptime (highest drain) first.
    let recovery = morale_recovery(building);
    let op_index: HashMap<&str, &OperatorBaseProfile> =
        operators.iter().map(|o| (o.char_id.as_str(), o)).collect();
    let mut ranked: Vec<String> = candidates.into_iter().collect();
    ranked.sort_by(|a, b| {
        let ua = op_index
            .get(a.as_str())
            .map_or(1.0, |o| op_uptime(o, morale_drains, recovery));
        let ub = op_index
            .get(b.as_str())
            .map_or(1.0, |o| op_uptime(o, morale_drains, recovery));
        ua.partial_cmp(&ub)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.cmp(b))
    });
    ranked.truncate(managers);
    ranked.into_iter().collect()
}

/// Tiny inputs: a handful of teams/rooms at most.
fn permutations(n: usize) -> Vec<Vec<usize>> {
    let mut out = Vec::new();
    let mut current: Vec<usize> = Vec::with_capacity(n);
    let mut used = vec![false; n];
    fn rec(n: usize, current: &mut Vec<usize>, used: &mut [bool], out: &mut Vec<Vec<usize>>) {
        if current.len() == n {
            out.push(current.clone());
            return;
        }
        for i in 0..n {
            if !used[i] {
                used[i] = true;
                current.push(i);
                rec(n, current, used, out);
                current.pop();
                used[i] = false;
            }
        }
    }
    rec(n, &mut current, &mut used, &mut out);
    out
}

/// Exactly-`size` combinations of `pool` (tiny inputs: a room holds ≤ 5).
fn small_subsets(pool: &[String], size: usize) -> Vec<Vec<String>> {
    if size == 0 {
        return vec![Vec::new()];
    }
    if pool.len() < size {
        return Vec::new();
    }
    let mut out = Vec::new();
    let mut idx: Vec<usize> = (0..size).collect();
    loop {
        out.push(idx.iter().map(|&i| pool[i].clone()).collect());
        let mut i = size;
        loop {
            if i == 0 {
                return out;
            }
            i -= 1;
            if idx[i] != i + pool.len() - size {
                break;
            }
        }
        idx[i] += 1;
        for j in (i + 1)..size {
            idx[j] = idx[j - 1] + 1;
        }
    }
}

/// A cell's crew with the room's 24/7 pins seated first; remaining seats go to
/// the team SUBSET that scores best beside them. Naive truncation could seat a
/// second order-value operator that doesn't stack with the pin (Bibeak next to a
/// pinned Proviso) while a speed operator rests. Nothing pinned = the team.
#[allow(clippy::too_many_arguments)]
fn merge_kept(
    kept: &[String],
    team: &[String],
    capacity: usize,
    room_type: &str,
    formula_type: Option<&str>,
    op_index: &HashMap<&str, &OperatorBaseProfile>,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
    facility_counts: &HashMap<String, usize>,
    total_dorm_levels: i32,
    morale_drains: &HashMap<String, f64>,
) -> Vec<String> {
    let mut crew: Vec<String> = kept.to_vec();
    crew.dedup();
    let members: Vec<String> = team
        .iter()
        .filter(|op| !crew.contains(op))
        .cloned()
        .collect();
    let free = capacity.saturating_sub(crew.len());
    if crew.is_empty() || members.len() <= free {
        for op in members.into_iter().take(free) {
            crew.push(op);
        }
        return crew;
    }
    let mut best: Option<(f64, Vec<String>)> = None;
    for subset in small_subsets(&members, free) {
        let mut trial = crew.clone();
        trial.extend(subset);
        let (speed, value) = compute_team_efficiency(
            &trial,
            room_type,
            formula_type,
            None,
            op_index,
            registry,
            building_data,
            facility_counts,
            total_dorm_levels,
            morale_drains,
            &[],
        );
        let score = room_search_score(room_type, speed, value);
        if best.as_ref().is_none_or(|(b, _)| score > *b) {
            best = Some((score, trial));
        }
    }
    best.map_or(crew, |(_, trial)| trial)
}

/// Build the recommended rotation, pairing each shift's crews with the saved
/// presets.
///
/// Same base-wide-conditional two-pass as the peak optimizer: pass 1 runs with
/// Hoederer-style "when X works any Work Area" bonuses off to learn who is
/// deployed, pass 2 re-plans with the unlocked bonuses.
pub fn recommend_shift_rotation(
    operators: &[OperatorBaseProfile],
    building: &UserBuilding,
    building_data: &BuildingDataFile,
    registry: &HashMap<String, BuffResolutionStrategy>,
    morale_drains: &HashMap<String, f64>,
    pins: &[(String, String)],
) -> ShiftRotation {
    let pinned_seats: HashMap<String, String> = pins.iter().cloned().collect();
    if !base_wide_relevant(operators, registry) && !room_presence_relevant(operators, registry) {
        return rotation_core(
            operators,
            building,
            building_data,
            registry,
            morale_drains,
            pins,
            &pinned_seats,
        );
    }
    let layout_registry =
        super::assignment::resolve_layout_branches(registry, building, building_data);
    let registry = &layout_registry;
    let pass1_registry = resolve_room_presence(
        &resolve_base_wide(registry, &HashSet::new(), &HashSet::new()),
        &HashMap::new(),
        operators,
    );
    let pass1 = rotation_core(
        operators,
        building,
        building_data,
        &pass1_registry,
        morale_drains,
        pins,
        &pinned_seats,
    );
    let deployed: HashSet<String> = pass1
        .shifts
        .iter()
        .flat_map(|s| s.rooms.iter().filter(|r| r.active))
        .flat_map(|r| r.recommended.iter().cloned())
        .collect();
    // Room-presence gates resolve against the first shift's stationed room types
    // (room type per slot is constant across shifts).
    let deployed_rooms: HashMap<String, String> = pass1
        .shifts
        .iter()
        .flat_map(|s| s.rooms.iter().filter(|r| r.active))
        .flat_map(|r| {
            r.recommended
                .iter()
                .map(|op| (op.clone(), r.room_type.clone()))
        })
        .collect();
    let pass2_registry = resolve_room_presence(
        &resolve_base_wide(
            registry,
            &deployed,
            &deployed
                .iter()
                .cloned()
                .chain(super::assignment::dorm_residents(building))
                .collect(),
        ),
        &deployed_rooms,
        operators,
    );
    let mut pass2_seats = deployed_rooms;
    pass2_seats.extend(pins.iter().cloned());
    let counts_unchanged =
        effective_facility_counts(building, operators, registry, building_data, &pass2_seats)
            == effective_facility_counts(
                building,
                operators,
                registry,
                building_data,
                &pinned_seats,
            );
    if pass2_registry == pass1_registry && counts_unchanged {
        return pass1;
    }
    rotation_core(
        operators,
        building,
        building_data,
        &pass2_registry,
        morale_drains,
        pins,
        &pass2_seats,
    )
}

fn rotation_core(
    operators: &[OperatorBaseProfile],
    building: &UserBuilding,
    building_data: &BuildingDataFile,
    registry: &HashMap<String, BuffResolutionStrategy>,
    morale_drains: &HashMap<String, f64>,
    pins: &[(String, String)],
    seats: &HashMap<String, String>,
) -> ShiftRotation {
    let facility_counts =
        effective_facility_counts(building, operators, registry, building_data, seats);
    let parked = super::assignment::parked_tokens(seats, operators, registry);
    let total_dorm_levels = building.total_dorm_levels();
    let production_rooms: Vec<&UserRoom> = building
        .rooms
        .iter()
        .filter(|r| is_production_room(&r.room_type))
        .collect();

    // Economy pins (pool plans and accepted bundles: Ling/Dusk in the CC, Senshi in
    // a dorm, robots in plants) are reserved from EVERY rotation pool so they can't
    // be burned as fillers. CC pins also join BOTH squads since consumers need them
    // whenever they work; the morale sim reports if that 24/7 presence fails.
    let roster_ids: HashSet<&str> = operators.iter().map(|o| o.char_id.as_str()).collect();
    let mut pinned_ids: HashSet<String> = pins
        .iter()
        .filter(|(id, _)| roster_ids.contains(id.as_str()))
        .map(|(id, _)| id.clone())
        .collect();
    let cc_pinned: Vec<String> = pins
        .iter()
        .filter(|(id, rt)| rt == "CONTROL" && pinned_ids.contains(id))
        .map(|(id, _)| id.clone())
        .collect();

    // The rotation reserves the morale-swap manager itself, not the caller's pins:
    // its 24/7 sustains assume her swap, which only works while she stays in a
    // dorm. If the building can host her (her seat + a free swap seat), she's out
    // of every working pool and the dorm pass seats her every shift.
    let manager_resident: Option<String> =
        if super::dorms::building_hosts_manager(building, building_data) {
            super::assignment::morale_swap_enabler(operators, building_data)
        } else {
            None
        };
    if let Some(m) = &manager_resident {
        pinned_ids.insert(m.clone());
    }
    // Parked tokens hold their seat every shift.
    pinned_ids.extend(parked.iter().cloned());

    // CC Squad 1 (with its globals / faction conditions) and the balanced teams,
    // re-selecting the CC when a member is dead weight against the chosen teams.
    // One memo per run: registry, facility counts and drains are fixed here.
    let memo = super::team_select::EnumerationMemo::default();
    let (cc_plan, mut groups) = rotation_cc_plan(
        operators,
        building,
        building_data,
        registry,
        &cc_pinned,
        |assigned, global_bonuses, cc_conditions| {
            let mut assigned = assigned.clone();
            assigned.extend(pinned_ids.iter().cloned());
            plan_production_groups(
                &production_rooms,
                operators,
                &assigned,
                registry,
                building_data,
                &facility_counts,
                total_dorm_levels,
                global_bonuses,
                cc_conditions,
                morale_drains,
                &HashMap::new(),
                &HashMap::new(),
                &memo,
            )
        },
    );

    // Fiammetta 24/7 sustain. Preset evidence first (the player's choice), then a
    // proactive top-up: the trading operator whose 24/7 seat adds the most REALIZED
    // yield. Each candidate gets a with/without oracle on the rotation objective:
    // teams re-planned around the seat, every shift scored seated vs unseated
    // against the same teams, so only the candidate differs, not planner noise.
    // That couples the pick to gold supply (base expert, 2026-09-08): a starved post
    // gains nothing from speed, so Tequila's per-bar rider outranks Shamare's speed
    // there, while a gold-rich post keeps its speed anchor.
    let mut sustained =
        preset_sustained_operators(building, operators, morale_drains, registry, building_data);
    let managers = num_morale_swap_managers(operators, building_data);
    let op_index = build_op_index(operators);
    // Post the oracle priced each candidate's seat at.
    let mut oracle_slot: HashMap<String, String> = HashMap::new();
    if sustained.len() < managers && super::dorms::building_hosts_manager(building, building_data) {
        let recovery = morale_recovery(building);
        // She can only hold a drain that doesn't outrun her recharge (Fiammetta: one
        // full-bar swap per 12h at +2/h exclusive self-recovery).
        let swap_rate = super::assignment::morale_swap_enabler(operators, building_data)
            .and_then(|id| operators.iter().find(|o| o.char_id == id))
            .map_or(0.0, |m| {
                super::dorms::manager_swap_rate(m, registry, building_data)
            });
        let mut exclude_base: HashSet<String> = cc_plan.squad1.iter().cloned().collect();
        exclude_base.extend(pinned_ids.iter().cloned());
        let mut cands: Vec<(String, f64)> = Vec::new();
        let trading_slots: Vec<String> = groups
            .iter()
            .filter(|g| g.room_type == "TRADING")
            .flat_map(|g| g.rooms.iter().map(|(s, _)| s.clone()))
            .collect();
        for g in groups.iter().filter(|g| g.room_type == "TRADING") {
            let group_rooms: Vec<String> = g.rooms.iter().map(|(s, _)| s.clone()).collect();
            for team in g.teams.iter().filter(|t| !t.ops.is_empty()) {
                for id in &team.ops {
                    if sustained.contains(id) || cands.iter().any(|(c, _)| c == id) {
                        continue;
                    }
                    let feasible = op_index.get(id.as_str()).is_none_or(|o| {
                        super::dorms::manager_can_sustain(
                            swap_rate,
                            super::sustain_sim::game_morale_drain(o, morale_drains),
                        )
                    });
                    if !feasible {
                        continue;
                    }
                    // Try the seat at EVERY post, keep the best: value pays by post
                    // level (Proviso at L2, the Shamare squad at L3), and "first room
                    // of her group" pinned Proviso to the L3 post, where the plan built
                    // Shamare's squad around her (31010962). A saved preset holding
                    // the operator in a post still wins.
                    let slots: Vec<String> = match preset_room_of(building, id)
                        .filter(|slot| trading_slots.contains(slot))
                    {
                        Some(slot) => vec![slot],
                        None => trading_slots.clone(),
                    };
                    let _ = &group_rooms;
                    let uptime = op_index
                        .get(id.as_str())
                        .map_or(1.0, |op| op_uptime(op, morale_drains, recovery));
                    // Without the manager: ~2 of 3 shifts at morale-limited uptime; with: all 3 at
                    // full.
                    let extra_uptime = 1.0 - uptime * (2.0 / 3.0);
                    let mut best: Option<(String, f64)> = None;
                    for slot in slots {
                        let mut exclude = exclude_base.clone();
                        exclude.insert(id.clone());
                        let reserved: HashMap<String, usize> = HashMap::from([(slot.clone(), 1)]);
                        let mut pinned_plan = plan_production_groups(
                            &production_rooms,
                            operators,
                            &exclude,
                            registry,
                            building_data,
                            &facility_counts,
                            total_dorm_levels,
                            &cc_plan.global_bonuses,
                            &cc_plan.conditions,
                            morale_drains,
                            &reserved,
                            &HashMap::new(),
                            &memo,
                        );
                        let unseated = tiled_objective(&pinned_plan, &cc_plan.global_bonuses);
                        seat_pinned_operator(
                            &mut pinned_plan,
                            &slot,
                            id,
                            &op_index,
                            registry,
                            building_data,
                            &facility_counts,
                            total_dorm_levels,
                            morale_drains,
                            &cc_plan.conditions,
                        );
                        let seated = tiled_objective(&pinned_plan, &cc_plan.global_bonuses);
                        let gain = (seated - unseated).max(0.0) * extra_uptime;
                        if std::env::var_os("BASE_PIN_TRACE").is_some() {
                            eprintln!(
                                "[pin] {id} @ {slot}: unseated {unseated:.1} seated {seated:.1} uptime {uptime:.2} -> gain {gain:.1}"
                            );
                        }
                        if best.as_ref().is_none_or(|(_, g)| gain > *g + 1e-9) {
                            best = Some((slot, gain));
                        }
                    }
                    if let Some((slot, gain)) = best {
                        oracle_slot.insert(id.clone(), slot);
                        cands.push((id.clone(), gain));
                    }
                }
            }
        }
        cands.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.0.cmp(&b.0))
        });
        for (id, gain) in cands {
            if sustained.len() >= managers {
                break;
            }
            if gain > 1e-9 && !sustained.contains(&id) {
                sustained.insert(id);
            }
        }
    }
    // Pin each fielded sustained operator to ONE room (their preset room, else the
    // group's first) for all shifts, then RE-PLAN with pins out of the pool: the pin
    // holds a slot, so teams must not spend members that don't stack with it (a
    // second order-value operator beside a pinned Proviso is wasted).
    let mut kept_by_room: HashMap<String, Vec<String>> = HashMap::new();
    for g in &groups {
        let group_rooms: Vec<String> = g.rooms.iter().map(|(s, _)| s.clone()).collect();
        for team in &g.teams {
            for op in team.ops.iter().filter(|op| sustained.contains(*op)) {
                // Preset post, else the oracle's post, else the group's first room. A trading
                // preset may name a post in another level's group (posts group per level); the
                // player's choice still wins.
                let same_kind = |slot: &String| {
                    group_rooms.contains(slot)
                        || (g.room_type == "TRADING"
                            && groups
                                .iter()
                                .filter(|h| h.room_type == "TRADING")
                                .any(|h| h.rooms.iter().any(|(s, _)| s == slot)))
                };
                let room = preset_room_of(building, op)
                    .filter(same_kind)
                    .or_else(|| oracle_slot.get(op).cloned())
                    .or_else(|| group_rooms.first().cloned());
                if let Some(slot) = room {
                    kept_by_room.entry(slot).or_default().push(op.clone());
                }
            }
        }
    }
    let mut sustained_label: Vec<String> = kept_by_room.values().flatten().cloned().collect();
    sustained_label.sort();
    sustained_label.dedup();
    // Every production team works a 24h block ([A,A,B] / [B,C,C]), so a member
    // whose drain outruns a full bar (Aroma's "+0.25 Morale per hour" rider) runs
    // dry mid-block. They leave the pool unless the manager sustains them or a plan
    // pins them; the morale sim reports on what remains.
    let heavy: Vec<String> = operators
        .iter()
        .filter(|op| !super::sustain_sim::sustains_24h_block(op, morale_drains))
        .filter(|op| !sustained_label.contains(&op.char_id) && !pinned_ids.contains(&op.char_id))
        .map(|op| op.char_id.clone())
        .collect();
    if !sustained_label.is_empty() || !heavy.is_empty() {
        let mut exclude: HashSet<String> = cc_plan.squad1.iter().cloned().collect();
        exclude.extend(sustained_label.iter().cloned());
        exclude.extend(pinned_ids.iter().cloned());
        exclude.extend(heavy);
        // Re-planned around the pin, without heavy drainers: the pin's permanent slot
        // shrinks teams that only work its room.
        groups = plan_production_groups(
            &production_rooms,
            operators,
            &exclude,
            registry,
            building_data,
            &facility_counts,
            total_dorm_levels,
            &cc_plan.global_bonuses,
            &cc_plan.conditions,
            morale_drains,
            &HashMap::new(),
            &kept_by_room,
            &memo,
        );
    }

    let mut used: HashSet<String> = cc_plan.squad1.iter().cloned().collect();
    for g in &groups {
        for t in &g.teams {
            used.extend(t.ops.iter().cloned());
        }
    }
    used.extend(sustained_label.iter().cloned());
    used.extend(pinned_ids.iter().cloned());

    // Office / Reception: Squad 1 from the best leftovers, Squad 2 from the rest.
    // Squad 1 works a 24h block, so it skips heavy drainers; they stay available for
    // single-shift Squad 2.
    let aux_squads = |assigned: &mut HashSet<String>,
                      only_24h_sustainable: bool|
     -> HashMap<String, (String, Vec<String>)> {
        let mut rooms = Vec::new();
        assign_auxiliary_rooms(
            &mut rooms,
            building,
            building_data,
            operators,
            registry,
            morale_drains,
            assigned,
            only_24h_sustainable,
        );
        rooms
            .into_iter()
            .map(|r| (r.slot_id.clone(), (r.room_type, r.operators)))
            .collect()
    };
    let mut assigned = used.clone();
    let mut aux1 = aux_squads(&mut assigned, true);
    let mut aux2 = aux_squads(&mut assigned, false);

    // An economy pin in an Office or Reception (Mulberry's per-slot Worldly Plight,
    // Whisperain's Memory Fragments) works that room EVERY shift, like a CC pin in
    // both squads: its consumers (Mr. Nothing, Rosmontis) need it whenever they
    // work. Reserved from the pools above, such a pin never reached a squad: the
    // optimal view had Whisperain in the Office while every shift showed
    // Haruka/Penance (55699327), and a Worldly Plight Office went to Provence over
    // Mulberry.
    for (id, room_type) in pins.iter().filter(|(id, _)| pinned_ids.contains(id)) {
        if !matches!(room_type.as_str(), "HIRE" | "MEETING") {
            continue;
        }
        let Some(room) = building.rooms.iter().find(|r| &r.room_type == room_type) else {
            continue;
        };
        #[allow(clippy::cast_sign_loss)]
        let capacity = max_stationed_at_level(building_data, room_type, room.level).max(1) as usize;
        for squads in [&mut aux1, &mut aux2] {
            let (_, ops) = squads
                .entry(room.slot_id.clone())
                .or_insert_with(|| (room_type.clone(), Vec::new()));
            if ops.iter().any(|o| o == id) {
                continue;
            }
            if ops.len() >= capacity {
                ops.truncate(capacity - 1);
            }
            ops.insert(0, id.clone());
        }
    }

    // CC Squad 2: best global-bonus fill from leftovers, so the CC keeps granting
    // while Squad 1 rests. It sees every fielded team, so a conditional operator
    // whose gate no team meets is evicted as in Squad 1's dead-weight loop.
    let team_rooms: Vec<super::types::RoomAssignment> = groups
        .iter()
        .flat_map(|g| {
            g.teams
                .iter()
                .filter(|t| !t.ops.is_empty())
                .map(|t| super::types::RoomAssignment {
                    room_type: g.room_type.clone(),
                    formula_type: g.formula_type.clone(),
                    operators: t.ops.clone(),
                    ..Default::default()
                })
        })
        .collect();
    let cc_squad2 = cc_plan.squad2(operators, building_data, registry, &assigned, &team_rooms);
    assigned.extend(cc_squad2.iter().cloned());

    let power_plan = build_power_plan(
        operators,
        building,
        building_data,
        registry,
        &facility_counts,
        &assigned,
        morale_drains,
        &parked,
    );
    for plant in &power_plan {
        assigned.extend(plant.main.iter().cloned());
        assigned.extend(plant.backup.iter().cloned());
    }

    // Top both CC squads up with the lowest-opportunity-cost leftovers, like the
    // peak plan: a half-empty CC reads as a mistake and spare seats are free for
    // benchwarmers. Runs LAST so nothing useful is taken from production, aux or
    // power. An entirely empty Squad 2 stays dark.
    let mut cc1 = cc_plan.squad1.clone();
    let mut cc2 = cc_squad2;
    // Economy pins sit in Squad 1: two of three shifts under either pattern, the
    // uptime the perception economy priced. Both squads = 36h unbroken, which no
    // bar survives; generators rest when Squad 2 covers.
    #[allow(clippy::cast_sign_loss)]
    let cc_cap = cc_plan.control_slots.max(0) as usize;
    for id in cc_pinned.iter().rev() {
        if !cc1.contains(id) {
            cc1.insert(0, id.clone());
        }
    }
    if cc_cap > 0 {
        cc1.truncate(cc_cap);
    }
    // A pin Squad 2 also picked on merit would work all three shifts; drop it from
    // the 12h seat so the rest shift survives.
    cc2.retain(|id| !cc_pinned.contains(id));
    // Spare-seat picks, tracked for the UI badge.
    let mut bench: HashSet<String> = HashSet::new();
    if !cc1.is_empty() {
        bench.extend(fill_remaining_slots(
            &mut cc1,
            cc_plan.control_slots,
            "CONTROL",
            operators,
            building_data,
            registry,
            &mut assigned,
        ));
    }
    if !cc2.is_empty() {
        bench.extend(fill_remaining_slots(
            &mut cc2,
            cc_plan.control_slots,
            "CONTROL",
            operators,
            building_data,
            registry,
            &mut assigned,
        ));
    }

    // Synergy phase alignment: a CC Squad-1 faction condition (Viviana + her
    // Knights) flips the CC to a 24h block on shifts 1+2, and each linked group
    // moves its linked team onto that block (ordinal 0).
    let mut cc_pattern = SquadPattern::Alternating;
    let mut linked_groups: HashSet<usize> = HashSet::new();
    for (gi, g) in groups.iter_mut().enumerate() {
        let mut linked: Option<usize> = None;
        let mut best_weight = 0.0;
        for (ti, team) in g.teams.iter().enumerate() {
            if team.ops.is_empty() {
                continue;
            }
            // What the CC's conditions ADD to this team as scored, not on paper. A
            // nullifier kills a per-operator grant on roommates (Umiri's Siracusa +5% on
            // Texas and Lappland under Shamare); a paper-linked team got swapped onto the
            // two-shift block over the squad that really earns there (00980819, 2026-09-21).
            let scored = |conditions: &[super::assignment::CcCondition]| {
                compute_team_efficiency(
                    &team.ops,
                    &g.room_type,
                    g.formula_type.as_deref(),
                    None,
                    &op_index,
                    registry,
                    building_data,
                    &facility_counts,
                    total_dorm_levels,
                    morale_drains,
                    conditions,
                )
                .0
            };
            let weight = scored(&cc_plan.conditions) - scored(&[]);
            if weight > best_weight + 1e-9 {
                best_weight = weight;
                linked = Some(ti);
            }
        }
        if let Some(ti) = linked
            && best_weight > 0.0
        {
            cc_pattern = SquadPattern::Block;
            linked_groups.insert(gi);
            // Ordinal 0 holds the shifts-1+2 block.
            if ti != 0 {
                g.teams.swap(0, ti);
            }
        }
    }

    // Block makes Squad 1 a 24h seat, so the 24h gate applies: a member who can't
    // finish a 24h bar trades places with a sustaining Squad-2 member. Economy pins
    // stay: their presence is the plan, and the morale sim reports if it fails.
    if cc_pattern == SquadPattern::Block {
        let is_pin = |id: &str| cc_pinned.iter().any(|p| p == id);
        let sustains = |id: &str| {
            op_index
                .get(id)
                .is_none_or(|op| super::sustain_sim::sustains_24h_block(op, morale_drains))
        };
        let mut heavy: Vec<String> = Vec::new();
        cc1.retain(|id| {
            if is_pin(id) || sustains(id) {
                true
            } else {
                heavy.push(id.clone());
                false
            }
        });
        for id in heavy {
            // Promote the sustaining Squad-2 member whose move wastes least: what they add
            // to Squad 1 minus what Squad 2 loses. A "+7% trading" beside Squad 1's
            // same-family +7% scores negative, so a plain filler (0 - 0) wins.
            let best = cc2
                .iter()
                .enumerate()
                .filter(|(_, c)| !is_pin(c) && sustains(c) && !cc1.contains(*c))
                .map(|(j, c)| {
                    let gain = super::assignment::cc_marginal_over(
                        &cc1,
                        c,
                        &op_index,
                        registry,
                        building_data,
                    );
                    let loss = super::assignment::cc_marginal_over(
                        &cc2,
                        c,
                        &op_index,
                        registry,
                        building_data,
                    );
                    (j, gain - loss)
                })
                .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(j, _)| j);
            if let Some(j) = best {
                cc1.push(cc2[j].clone());
                cc2[j] = id;
            } else if !cc2.contains(&id) && (cc2.len() as i32) < cc_plan.control_slots {
                cc2.push(id);
            }
            // else: benched; no CC seat holds this drain 24h.
        }
    }

    let presets: HashMap<&str, Vec<&Vec<String>>> = building
        .rooms
        .iter()
        .map(|r| {
            let non_empty: Vec<&Vec<String>> =
                r.preset_shifts.iter().filter(|p| !p.is_empty()).collect();
            (r.slot_id.as_str(), non_empty)
        })
        .collect();
    // Saved presets are equal alternating halves (two = the A/B login swap, three =
    // one per shift), so cycle them; preset 1 is not a "main".
    let preset_for = |slot: &str, k: usize| -> Vec<String> {
        presets
            .get(slot)
            .filter(|p| !p.is_empty())
            .map(|p| p[k % p.len()].clone())
            .unwrap_or_default()
    };

    // Preset-phase alignment: which unit is "Team A" / "Squad 1" is arbitrary
    // (every unit works the same share), so pick the arrangement matching the saved
    // presets. The plan then confirms operators already in a phase instead of
    // asking for swaps.
    let sym_diff = |a: &[String], b: &[String]| -> usize {
        if b.is_empty() {
            return 0; // no preset saved
        }
        let sa: HashSet<&str> = a.iter().map(String::as_str).collect();
        let sb: HashSet<&str> = b.iter().map(String::as_str).collect();
        sa.symmetric_difference(&sb).count()
    };
    for (gi, g) in groups.iter_mut().enumerate() {
        let n = g.rooms.len();
        if n == 0 || n > 3 || g.teams.is_empty() {
            continue;
        }
        let linked = linked_groups.contains(&gi);
        let mut ordinal_cells = vec![0usize; g.teams.len()];
        for row in &g.cells {
            for &t in row {
                ordinal_cells[t] += 1;
            }
        }
        // Every room order x team permutation (teams only move between ordinals with
        // the same cell count; a synergy-linked team stays on shifts 1+2). Ranked by
        // OUTPUT first (a 24/7 pin makes phases matter: a Shamare team on a pinned
        // Proviso's post wastes her), then preset churn among ties.
        let mut best: Option<(i64, usize, Vec<usize>, Vec<usize>)> = None;
        for room_order in permutations(n) {
            for team_perm in permutations(g.teams.len()) {
                if linked && team_perm[0] != 0 {
                    continue;
                }
                if team_perm
                    .iter()
                    .enumerate()
                    .any(|(o, &t)| ordinal_cells[o] != ordinal_cells[t])
                {
                    continue;
                }
                let mut cost = 0usize;
                let mut output = 0.0f64;
                for (ri, &old_ri) in room_order.iter().enumerate() {
                    let (slot, level) = &g.rooms[old_ri];
                    let kept = kept_by_room.get(slot).map_or(&[][..], Vec::as_slice);
                    let capacity =
                        max_stationed_at_level(building_data, &g.room_type, *level).max(0) as usize;
                    for k in 0..SHIFT_COUNT {
                        let team = &g.teams[team_perm[g.cells[ri][k]]];
                        let crew = merge_kept(
                            kept,
                            &team.ops,
                            capacity,
                            &g.room_type,
                            g.formula_type.as_deref(),
                            &op_index,
                            registry,
                            building_data,
                            &facility_counts,
                            total_dorm_levels,
                            morale_drains,
                        );
                        let (speed, value) = compute_team_efficiency(
                            &crew,
                            &g.room_type,
                            g.formula_type.as_deref(),
                            Some(*level),
                            &op_index,
                            registry,
                            building_data,
                            &facility_counts,
                            total_dorm_levels,
                            morale_drains,
                            &cc_plan.conditions,
                        );
                        output += room_search_score(&g.room_type, speed, value);
                        cost += sym_diff(&crew, &preset_for(slot, k));
                    }
                }
                // Quantized so float noise doesn't defeat the churn tie-break.
                let output_key = (output * 100.0).round() as i64;
                let better = match &best {
                    None => true,
                    Some((bo, bc, _, _)) => output_key > *bo || (output_key == *bo && cost < *bc),
                };
                if better {
                    best = Some((output_key, cost, room_order.clone(), team_perm));
                }
            }
        }
        if let Some((_, _, room_order, team_perm)) = best {
            g.rooms = room_order.iter().map(|&ri| g.rooms[ri].clone()).collect();
            g.teams = team_perm.iter().map(|&t| g.teams[t].clone()).collect();
        }
    }
    // Two-squad rooms: flipping is free under Alternating (an even A/B swap), so
    // flip when the presets run the squads in the opposite phase.
    let flip_costs = |slot: &str, s1: &[String], s2: &[String]| -> (usize, usize) {
        let (mut keep, mut flip) = (0usize, 0usize);
        for k in 0..SHIFT_COUNT {
            let preset = preset_for(slot, k);
            let (a, b) = if SquadPattern::Alternating.squad_at(k) == 0 {
                (s1, s2)
            } else {
                (s2, s1)
            };
            keep += sym_diff(a, &preset);
            flip += sym_diff(b, &preset);
        }
        (keep, flip)
    };
    let flip_better = |slot: &str, s1: &[String], s2: &[String]| -> bool {
        if s2.is_empty() {
            return false;
        }
        let (keep, flip) = flip_costs(slot, s1, s2);
        flip < keep
    };
    // Power squads were split EVENLY across plants, so flip all plants or none
    // (flipping one reshuffles them lopsided). The STRONGER squad stays first
    // (Squad 1 covers two of three shifts), so a flip can't demote it.
    let mut power_plan = power_plan;
    if power_plan.iter().all(|p| !p.backup.is_empty()) {
        let squad_total = |pick: fn(&PowerPlant) -> &Vec<String>| -> f64 {
            power_plan
                .iter()
                .flat_map(|p| pick(p).iter())
                .filter_map(|id| op_index.get(id.as_str()).copied())
                .map(|op| power_value(op, building_data, registry, &facility_counts))
                .sum()
        };
        let (main_total, backup_total) = (squad_total(|p| &p.main), squad_total(|p| &p.backup));
        let (keep, flip) = power_plan.iter().fold((0, 0), |(k, f), p| {
            let (pk, pf) = flip_costs(&p.slot_id, &p.main, &p.backup);
            (k + pk, f + pf)
        });
        // A flip promotes old Squad 2 onto Squad 1's 24h block; every promoted bar
        // must survive one.
        let bar_ok_ids = |ids: &[String]| -> bool {
            ids.iter().all(|id| {
                op_index
                    .get(id.as_str())
                    .is_none_or(|op| super::sustain_sim::sustains_24h_block(op, morale_drains))
            })
        };
        if flip < keep
            && backup_total + 1e-9 >= main_total
            && power_plan.iter().all(|p| bar_ok_ids(&p.backup))
        {
            for plant in &mut power_plan {
                std::mem::swap(&mut plant.main, &mut plant.backup);
            }
        }
    }
    let bar_ok_ids = |ids: &[String]| -> bool {
        ids.iter().all(|id| {
            op_index
                .get(id.as_str())
                .is_none_or(|op| super::sustain_sim::sustains_24h_block(op, morale_drains))
        })
    };
    for (slot, (_, squad1)) in &mut aux1 {
        if let Some((_, squad2)) = aux2.get_mut(slot)
            && flip_better(slot, squad1, squad2)
            && bar_ok_ids(squad2)
        {
            std::mem::swap(squad1, squad2);
        }
    }
    if cc_pattern == SquadPattern::Alternating
        && let Some(cc_slot) = &cc_plan.slot_id
        && flip_better(cc_slot, &cc1, &cc2)
        && bar_ok_ids(&cc2)
    {
        std::mem::swap(&mut cc1, &mut cc2);
    }

    // Squad-2 dead-weight check, per shift. Squad 2 works one shift and the tiling
    // is settled, so judge its conditional operators against THAT shift's teams.
    // The earlier all-teams check in `squad2` is only a lower bound: Jessica the
    // Liberator's Blacksteel gate fired on the shifts-1+2 factory team while she
    // sat shift 3, collecting nothing. Squad 1 needs no pass: phase alignment puts
    // linked teams on its block. Evicted seats refill from the bench; evictees stay
    // in `assigned` so they can't be re-seated.
    if let Some(k2) = (0..SHIFT_COUNT).find(|&k| cc_pattern.squad_at(k) == 1)
        && !cc2.is_empty()
    {
        let room_type_of: HashMap<&str, &str> = building
            .rooms
            .iter()
            .map(|r| (r.slot_id.as_str(), r.room_type.as_str()))
            .collect();
        let mut shift_rooms: Vec<super::types::RoomAssignment> = groups
            .iter()
            .flat_map(|g| {
                g.rooms.iter().enumerate().filter_map(|(ri, (slot, _))| {
                    let team = &g.teams[g.cells[ri][k2]];
                    let mut ops = kept_by_room.get(slot).cloned().unwrap_or_default();
                    ops.extend(team.ops.iter().cloned());
                    (!ops.is_empty()).then(|| super::types::RoomAssignment {
                        room_type: g.room_type.clone(),
                        formula_type: g.formula_type.clone(),
                        operators: ops,
                        ..Default::default()
                    })
                })
            })
            .collect();
        // 24/7-sustained rooms outside the groups also work this shift and can meet a
        // gate.
        for (slot, ops) in &kept_by_room {
            if !groups
                .iter()
                .any(|g| g.rooms.iter().any(|(s, _)| s == slot))
                && let Some(rt) = room_type_of.get(slot.as_str())
            {
                shift_rooms.push(super::types::RoomAssignment {
                    room_type: (*rt).to_string(),
                    formula_type: None,
                    operators: ops.clone(),
                    ..Default::default()
                });
            }
        }
        cc2.retain(|id| {
            op_index.get(id.as_str()).is_none_or(|op| {
                !super::assignment::cc_op_is_dead(
                    op,
                    &shift_rooms,
                    &op_index,
                    registry,
                    building_data,
                )
            })
        });
        if !cc2.is_empty() {
            bench.extend(fill_remaining_slots(
                &mut cc2,
                cc_plan.control_slots,
                "CONTROL",
                operators,
                building_data,
                registry,
                &mut assigned,
            ));
        }
    }

    let team_letter = |ordinal: usize| -> String {
        char::from(b'A' + u8::try_from(ordinal % 26).unwrap_or(0)).to_string()
    };
    // Crew efficiency (speed incl. Squad-1 globals) for production/power cells.
    let crew_efficiency = |crew: &[String], room_type: &str, formula: Option<&str>| -> f64 {
        let (speed, _) = compute_team_efficiency(
            crew,
            room_type,
            formula,
            None,
            &op_index,
            registry,
            building_data,
            &facility_counts,
            total_dorm_levels,
            morale_drains,
            &cc_plan.conditions,
        );
        speed
            + cc_plan
                .global_bonuses
                .get(room_type)
                .copied()
                .unwrap_or(0.0)
    };

    let mut shifts = Vec::with_capacity(SHIFT_COUNT);
    for k in 0..SHIFT_COUNT {
        let mut rooms = Vec::new();

        // Each room runs its tiling cell's team, pinned 24/7 operators merged first.
        // An empty team rests the room dark.
        for g in &groups {
            for (ri, (slot, level)) in g.rooms.iter().enumerate() {
                let ordinal = g.cells[ri][k];
                let team = &g.teams[ordinal];
                let capacity =
                    max_stationed_at_level(building_data, &g.room_type, *level).max(0) as usize;
                let kept = kept_by_room.get(slot).cloned().unwrap_or_default();
                let crew = merge_kept(
                    &kept,
                    &team.ops,
                    capacity,
                    &g.room_type,
                    g.formula_type.as_deref(),
                    &op_index,
                    registry,
                    building_data,
                    &facility_counts,
                    total_dorm_levels,
                    morale_drains,
                );
                let group_tag = g
                    .formula_type
                    .as_deref()
                    .map_or_else(|| g.room_type.clone(), |f| format!("{}:{f}", g.room_type));
                let efficiency = crew_efficiency(&crew, &g.room_type, g.formula_type.as_deref());
                rooms.push(ShiftRoom {
                    slot_id: slot.clone(),
                    room_type: g.room_type.clone(),
                    formula_type: g.formula_type.clone(),
                    active: !crew.is_empty(),
                    recommended: crew,
                    current: preset_for(slot, k),
                    efficiency: Some(efficiency),
                    team_id: Some(format!("{group_tag}:{ordinal}")),
                    team_label: Some(format!("Team {}", team_letter(ordinal))),
                });
            }
        }

        // Power: Squad 1 / Squad 2 alternating. A plant generates regardless of crew,
        // so an empty squad just rests it dark.
        for plant in &power_plan {
            let squad = SquadPattern::Alternating.squad_at(k);
            let crew = if squad == 0 || plant.backup.is_empty() {
                &plant.main
            } else {
                &plant.backup
            };
            let active = (squad == 0 || !plant.backup.is_empty()) && !crew.is_empty();
            let efficiency = crew_efficiency(
                &super::assignment::working_crew(crew, &parked),
                "POWER",
                None,
            );
            rooms.push(ShiftRoom {
                slot_id: plant.slot_id.clone(),
                room_type: "POWER".to_string(),
                formula_type: None,
                recommended: crew.clone(),
                current: preset_for(&plant.slot_id, k),
                active,
                efficiency: Some(efficiency),
                team_id: Some(format!("POWER:{}:{squad}", plant.slot_id)),
                team_label: Some(format!("Squad {}", squad + 1)),
            });
        }

        // Office / Reception alternate; no second squad = dark on Squad 2's shift.
        for (slot, (room_type, squad1)) in &aux1 {
            let squad = SquadPattern::Alternating.squad_at(k);
            let squad2 = aux2.get(slot).map(|(_, ops)| ops);
            let (crew, active) = match (squad, squad2) {
                (1, Some(b)) if !b.is_empty() => (b.clone(), true),
                (1, _) => (squad1.clone(), false),
                _ => (squad1.clone(), true),
            };
            rooms.push(ShiftRoom {
                slot_id: slot.clone(),
                room_type: room_type.clone(),
                formula_type: None,
                recommended: crew,
                current: preset_for(slot, k),
                active,
                efficiency: None,
                team_id: Some(format!("{room_type}:{slot}:{squad}")),
                team_label: Some(format!("Squad {}", squad + 1)),
            });
        }

        // CC: Alternating by default, Block (shifts 1+2) when a Squad-1 synergy needs
        // its dependent teams' 24h window.
        if let Some(cc_slot) = &cc_plan.slot_id {
            let squad = cc_pattern.squad_at(k);
            let (crew, active) = match (squad, cc2.is_empty()) {
                (1, false) => (cc2.clone(), true),
                (1, true) => (cc1.clone(), false),
                _ => (cc1.clone(), true),
            };
            rooms.push(ShiftRoom {
                slot_id: cc_slot.clone(),
                room_type: "CONTROL".to_string(),
                formula_type: None,
                recommended: crew,
                current: preset_for(cc_slot, k),
                active,
                efficiency: None,
                team_id: Some(format!("CONTROL:{squad}")),
                team_label: Some(format!("Squad {}", squad + 1)),
            });
        }

        // A facility-count enabler counts only in shifts it works: Greyy the
        // Lightningbearer's "+1 Power Plant" is gone from his rest shift, and with it
        // Weedy's and Eunectes' per-plant productivity. Teams were picked against the
        // optimal seats' counts; the DISPLAYED figure is re-priced on this shift's seats
        // (44947595: Weedy/Eunectes read 136 in both shifts, one with three plain
        // plants).
        let shift_seats: HashMap<String, String> = rooms
            .iter()
            .flat_map(|r| {
                r.recommended
                    .iter()
                    .map(move |op| (op.clone(), r.room_type.clone()))
            })
            .collect();
        let shift_counts =
            effective_facility_counts(building, operators, registry, building_data, &shift_seats);
        if shift_counts != facility_counts {
            for r in &mut rooms {
                if r.efficiency.is_none()
                    || !(is_production_room(&r.room_type) || r.room_type == "POWER")
                {
                    continue;
                }
                let (speed, _) = compute_team_efficiency(
                    &super::assignment::working_crew(&r.recommended, &parked),
                    &r.room_type,
                    r.formula_type.as_deref(),
                    None,
                    &op_index,
                    registry,
                    building_data,
                    &shift_counts,
                    total_dorm_levels,
                    morale_drains,
                    &cc_plan.conditions,
                );
                r.efficiency = Some(
                    speed
                        + cc_plan
                            .global_bonuses
                            .get(&r.room_type)
                            .copied()
                            .unwrap_or(0.0),
                );
            }
        }

        shifts.push(Shift {
            index: k + 1,
            rooms,
        });
    }

    // Dormitories. Rest is a seat: each shift's off-duty workers go INTO specific
    // dorms, heaviest drainers to the best (a 2/5/2's one good dorm is why levels
    // matter), and unseated dorm-skill holders take permanent dorm seats. A pinned
    // manager (Fiammetta) keeps her dorm seat every shift. The sustainability sim
    // reads these same cells, so verdict and display agree.
    let dorms = super::dorms::dorm_list(building, building_data);
    if !dorms.is_empty() {
        let mut seated: HashSet<String> = HashSet::new();
        let mut working: Vec<HashSet<String>> = vec![HashSet::new(); SHIFT_COUNT];
        for (k, shift) in shifts.iter().enumerate() {
            for room in shift.rooms.iter().filter(|r| r.active) {
                for id in &room.recommended {
                    seated.insert(id.clone());
                    working[k].insert(id.clone());
                }
            }
        }
        let sustained_set: HashSet<&str> = sustained_label.iter().map(String::as_str).collect();
        // Heaviest drain first: they need the best dorm most.
        let resters_by_shift: Vec<Vec<String>> = (0..SHIFT_COUNT)
            .map(|k| {
                let mut r: Vec<String> = seated
                    .iter()
                    .filter(|id| {
                        !working[k].contains(id.as_str()) && !sustained_set.contains(id.as_str())
                    })
                    .cloned()
                    .collect();
                r.sort_by(|a, b| {
                    let da = op_index.get(a.as_str()).map_or(0.0, |op| {
                        super::sustain_sim::game_morale_drain(op, morale_drains)
                    });
                    let db = op_index.get(b.as_str()).map_or(0.0, |op| {
                        super::sustain_sim::game_morale_drain(op, morale_drains)
                    });
                    db.partial_cmp(&da)
                        .unwrap_or(std::cmp::Ordering::Equal)
                        .then_with(|| a.cmp(b))
                });
                r
            })
            .collect();
        // The manager works FROM a dorm, permanently. She takes the worst dorm, leaving
        // the good ones to operators whose recovery depends on the rate. Joins any
        // caller-pinned dorm residents.
        let mut dorm_pinned: Vec<String> = pins
            .iter()
            .filter(|(id, rt)| rt == "DORMITORY" && !seated.contains(id))
            .map(|(id, _)| id.clone())
            .collect();
        if let Some(m) = &manager_resident
            && !dorm_pinned.contains(m)
            && !seated.contains(m)
        {
            dorm_pinned.push(m.clone());
        }
        let peak_rest = resters_by_shift.iter().map(Vec::len).max().unwrap_or(0);
        let capacity: usize = dorms.iter().map(|d| d.capacity).sum();
        // A pinned manager needs TWO seats of headroom: hers and a free one beside her.
        // The swap only fires when the drained operator is assigned INTO her dorm
        // ("swaps Morale with the previous Operator assigned to that Dormitory"), so a
        // dorm packed solid around her breaks it.
        let headroom = capacity
            .saturating_sub(peak_rest)
            .saturating_sub(dorm_pinned.len() * 2);
        let leftovers: Vec<&OperatorBaseProfile> = operators
            .iter()
            .filter(|op| !seated.contains(&op.char_id))
            .filter(|op| !dorm_pinned.iter().any(|id| id == &op.char_id))
            .filter(|op| !sustained_set.contains(op.char_id.as_str()))
            .collect();
        let mut staffing =
            super::dorms::plan_dorm_staffing(&dorms, &leftovers, headroom, registry, building_data);
        // Worst dorm with a spare permanent seat.
        for id in &dorm_pinned {
            if let Some((_, crew)) = staffing.iter_mut().rev().find(|(slot, crew)| {
                let cap = dorms
                    .iter()
                    .find(|d| &d.slot_id == slot)
                    .map_or(0, |d| d.capacity);
                crew.len() < cap
            }) {
                crew.push(id.clone());
            }
        }
        for (k, shift) in shifts.iter_mut().enumerate() {
            let mut queue = resters_by_shift[k].iter();
            for (di, dorm) in dorms.iter().enumerate() {
                let mut crew = staffing[di].1.clone();
                let free = dorm.capacity.saturating_sub(crew.len());
                crew.extend(queue.by_ref().take(free).cloned());
                shift.rooms.push(ShiftRoom {
                    slot_id: dorm.slot_id.clone(),
                    room_type: "DORMITORY".to_string(),
                    formula_type: None,
                    active: !crew.is_empty(),
                    recommended: crew,
                    current: preset_for(&dorm.slot_id, k),
                    efficiency: None,
                    team_id: Some(format!("DORMITORY:{}", dorm.slot_id)),
                    team_label: Some("Resting".to_string()),
                });
            }
        }
    }

    ShiftRotation {
        shifts,
        sustained: sustained_label,
        bench: bench.into_iter().collect(),
        parked: parked.into_iter().collect(),
    }
}

/// The production room whose saved presets ALL contain `op`: where the player
/// parks a 24/7 operator.
fn preset_room_of(building: &UserBuilding, op: &str) -> Option<String> {
    building
        .rooms
        .iter()
        .filter(|r| is_production_room(&r.room_type))
        .find(|r| {
            let presets: Vec<&Vec<String>> =
                r.preset_shifts.iter().filter(|p| !p.is_empty()).collect();
            !presets.is_empty() && presets.iter().all(|p| p.iter().any(|id| id == op))
        })
        .map(|r| r.slot_id.clone())
}

/// Plant output is fixed by level; only Power Plant skills (drone recovery,
/// shared drain) matter there. Anyone else is dead weight, so plants get
/// specialists only.
fn has_power_skill(op: &OperatorBaseProfile, building_data: &BuildingDataFile) -> bool {
    op.available_buffs.iter().any(|b| {
        building_data
            .buffs
            .get(b)
            .is_some_and(|buff| buff.room_type == "POWER")
    })
}

/// Largest value any of the operator's power buffs resolves to, for ranking.
/// Promoted skills resolve higher, so an E2 specialist outranks an E0.
fn power_value(
    op: &OperatorBaseProfile,
    building_data: &BuildingDataFile,
    registry: &HashMap<String, BuffResolutionStrategy>,
    facility_counts: &HashMap<String, usize>,
) -> f64 {
    super::ledger::op_power_rank_value(op, building_data, registry, facility_counts)
}

struct PowerPlant {
    slot_id: String,
    /// Squad 1's shifts.
    main: Vec<String>,
    /// Squad 2's shift; empty if no spare power operator (plant rests dark).
    backup: Vec<String>,
}

/// Plants generate regardless of crew, so the optimizer doesn't pick them. Staff
/// them with the best unplaced power specialists in two EVEN squads (each joins
/// the lower running total) so drone recovery stays level across the
/// alternation. Non-specialists are left out; here they'd only drain morale.
#[allow(clippy::too_many_arguments)]
fn build_power_plan(
    operators: &[OperatorBaseProfile],
    building: &UserBuilding,
    building_data: &BuildingDataFile,
    registry: &HashMap<String, BuffResolutionStrategy>,
    facility_counts: &HashMap<String, usize>,
    used: &HashSet<String>,
    morale_drains: &HashMap<String, f64>,
    parked: &HashSet<String>,
) -> Vec<PowerPlant> {
    let power_rooms: Vec<&UserRoom> = building
        .rooms
        .iter()
        .filter(|r| r.room_type == "POWER")
        .collect();
    if power_rooms.is_empty() {
        return Vec::new();
    }

    let mut ranked: Vec<(String, f64)> = operators
        .iter()
        .filter(|op| !used.contains(op.char_id.as_str()))
        .filter(|op| has_power_skill(op, building_data))
        .map(|op| {
            (
                op.char_id.clone(),
                power_value(op, building_data, registry, facility_counts),
            )
        })
        .collect();
    ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    let slots: Vec<usize> = power_rooms
        .iter()
        .map(|r| max_stationed_at_level(building_data, "POWER", r.level).max(1) as usize)
        .collect();
    // Parked tokens hold a seat in BOTH squads, plant by plant, first free seat.
    // Filling a plant's only seat sends the specialists (the exclusion gate's holder
    // among them) to the other plants, where that gate wants them.
    let mut reserved: Vec<Vec<String>> = vec![Vec::new(); power_rooms.len()];
    let mut tokens: Vec<&String> = parked.iter().collect();
    tokens.sort();
    for token in tokens {
        if let Some(i) = (0..power_rooms.len()).find(|&i| reserved[i].len() < slots[i]) {
            reserved[i].push(token.clone());
        }
    }
    let per_squad: usize =
        slots.iter().sum::<usize>() - reserved.iter().map(Vec::len).sum::<usize>();

    // Strongest first, each into the lower-total squad until its plant slots fill.
    // With 25/20/20/20/20/15 this gives 60/60 vs best-first's 65/55.
    let bar_ok = |id: &str| -> bool {
        operators
            .iter()
            .find(|op| op.char_id == id)
            .is_none_or(|op| super::sustain_sim::sustains_24h_block(op, morale_drains))
    };
    let mut squads: [(Vec<String>, f64); 2] = [(Vec::new(), 0.0), (Vec::new(), 0.0)];
    for (id, value) in ranked.into_iter().take(per_squad * 2) {
        let pick = (0..2)
            .filter(|&i| squads[i].0.len() < per_squad)
            // Squad 1 wraps into a 24h block; heavy drainers only fit Squad 2.
            .filter(|&i| i == 1 || bar_ok(&id))
            .min_by(|&a, &b| {
                squads[a]
                    .1
                    .partial_cmp(&squads[b].1)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        if let Some(i) = pick {
            squads[i].0.push(id);
            squads[i].1 += value;
        }
    }
    // Not always exactly even: the STRONGER half leads as Squad 1 (shifts 1 & 3),
    // unless that would put a heavy drainer on the 24h block.
    if squads[1].1 > squads[0].1 && squads[1].0.iter().all(|id| bar_ok(id)) {
        squads.swap(0, 1);
    }
    let [(squad1, _), (squad2, _)] = squads;
    let mut squad1 = squad1.into_iter();
    let mut squad2 = squad2.into_iter();

    power_rooms
        .iter()
        .zip(&slots)
        .zip(reserved)
        .map(|((r, n), held)| {
            let free = n - held.len();
            PowerPlant {
                slot_id: r.slot_id.clone(),
                main: held
                    .iter()
                    .cloned()
                    .chain((0..free).filter_map(|_| squad1.next()))
                    .collect(),
                backup: {
                    let picks: Vec<String> = (0..free).filter_map(|_| squad2.next()).collect();
                    // A plant held only by its token has no second squad; it keeps the token.
                    if picks.is_empty() && held.is_empty() {
                        Vec::new()
                    } else {
                        held.iter().cloned().chain(picks).collect()
                    }
                },
            }
        })
        .collect()
}
