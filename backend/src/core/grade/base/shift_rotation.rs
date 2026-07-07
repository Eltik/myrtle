//! Recommended 3-shift base rotation, for comparing the player's in-game presets
//! against an optimized rotation.
//!
//! The rotation follows the login rhythm players actually run:
//!
//! - **Production groups** (the Gold factories, the EXP factories, the Trading
//!   Posts) are staffed by `ceil(3N/2)` teams (3 teams for a pair of rooms) tiled
//!   as 24-hour blocks: every team works two consecutive shifts (wrapping past the
//!   cycle boundary) then rests one, and exactly ONE team per group swaps out at
//!   each 12h login:
//!
//!   ```text
//!     room 1:  Team A   Team A   Team B
//!     room 2:  Team B   Team C   Team C
//!   ```
//!
//! - **Power plants, Office, Reception and the Control Center** alternate two
//!   squads: Squad 1 works shifts 1 & 3, Squad 2 covers shift 2 - the swap you
//!   make at every login. When a Control-Center operator carries a cross-room
//!   synergy (Viviana's buff needs her Knights working in the factories), the CC
//!   flips to a 24h BLOCK (Squad 1 on shifts 1+2, Squad 2 on shift 3) and the
//!   dependent production team is phased onto the same shifts-1+2 block, so the
//!   provider and its dependents are always active together.
//!
//! Production teams are chosen JOINTLY (balanced, sum-of-teams objective - see
//! `team_select`) instead of best-team-first, so the strong operators spread
//! across the rotation instead of stacking into one 107% team that leaves the
//! last team hollow.
//!
//! An operator the player sustains 24/7 with a morale-swap manager (Fiammetta) is
//! pinned to one room across every shift and badged; owning a manager also
//! proactively recommends sustaining the highest-gain trading operator.

use std::collections::{HashMap, HashSet};

use crate::core::gamedata::types::building::BuildingDataFile;

use super::types::UserRoom;
use super::{
    assignment::{
        assign_auxiliary_rooms, base_wide_relevant, base_wide_unlocked, build_op_index,
        compute_team_efficiency, effective_facility_counts, fill_remaining_slots, morale_recovery,
        num_morale_swap_managers, op_uptime, resolve_base_wide, room_search_score,
        rotation_cc_plan,
    },
    buff_registry::BuffResolutionStrategy,
    evaluate::evaluate_buff,
    team_select::plan_production_groups,
    types::{EvalContext, OperatorBaseProfile, UserBuilding},
    util::{is_production_room, max_stationed_at_level},
};

/// The number of shifts in a recommended rotation (12h each; two logins per day).
pub const SHIFT_COUNT: usize = 3;

/// A full recommended rotation: `SHIFT_COUNT` shifts, each listing every room's
/// recommended crew alongside the player's preset for that room and shift.
pub struct ShiftRotation {
    pub shifts: Vec<Shift>,
    /// Operators the player runs 24/7 with a morale-swap manager (Fiammetta) - pinned to one
    /// room every shift instead of rotating. The frontend badges these as "24/7 - Fiammetta".
    pub sustained: Vec<String>,
}

pub struct Shift {
    /// 1-indexed shift number.
    pub index: usize,
    pub rooms: Vec<ShiftRoom>,
}

pub struct ShiftRoom {
    pub slot_id: String,
    pub room_type: String,
    /// Production formula for factories (`F_GOLD`/`F_EXP`/...), else `None`.
    pub formula_type: Option<String>,
    /// Recommended operators (`char_id`s) for this room in this shift.
    pub recommended: Vec<String>,
    /// The player's preset operators for this room in this shift, if they have one.
    pub current: Vec<String>,
    /// False when the room is deliberately UNSTAFFED this shift (no spare team -
    /// the room rests dark rather than burning benchwarmers' morale).
    pub active: bool,
    /// The displayed crew's room efficiency % (speed incl. Squad-1 global bonuses) for
    /// production and power cells, so players can see how output is distributed.
    pub efficiency: Option<f64>,
    /// Stable identity of the team/squad staffing this cell - the same id spans the
    /// two consecutive shift columns a production team's 24h block covers, so the
    /// frontend can visually tie them together.
    pub team_id: Option<String>,
    /// Display label: "Team A/B/C" for production blocks, "Squad 1/2" elsewhere.
    pub team_label: Option<String>,
}

/// How a two-squad room cycles across the three shifts.
#[derive(Clone, Copy, PartialEq)]
enum SquadPattern {
    /// Squad 1 works shifts 1 & 3, Squad 2 covers shift 2 - the per-login swap.
    Alternating,
    /// Squad 1 works shifts 1+2 (a 24h block, aligned with its dependent
    /// production teams), Squad 2 covers shift 3.
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

/// Operators the player runs 24/7 - present in EVERY saved preset of a production room, so they're
/// kept in across both presets and never rotated out - that a morale-swap manager (Fiammetta) can
/// hold at full morale around the clock. Capped at the number of managers owned, preferring the
/// operators with the LOWEST natural uptime (highest morale drain): those are the ones that actually
/// need a manager to sustain 24/7, whereas a low-drain operator sustains itself and shouldn't spend
/// a manager. Empty when the roster owns no manager or runs nobody across every preset.
fn preset_sustained_operators(
    building: &UserBuilding,
    operators: &[OperatorBaseProfile],
    morale_drains: &HashMap<String, f64>,
    building_data: &BuildingDataFile,
) -> HashSet<String> {
    let managers = num_morale_swap_managers(operators, building_data);
    if managers == 0 {
        return HashSet::new();
    }
    // 24/7 candidates: operators present in every (non-empty) saved preset of a production room. Two
    // distinct presets are required - a single saved preset is just "the current team", not a swap
    // the player deliberately holds around the clock.
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
                candidates.insert(op.clone());
            }
        }
    }
    // A manager holds the operators that benefit most - lowest natural uptime (highest drain) first.
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

/// All permutations of `0..n` (tiny inputs: at most a handful of teams/rooms).
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

/// All index-combinations of `pool` of exactly `size` (tiny inputs: a room holds ≤ 5).
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
        // Advance the combination indices.
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

/// A cell's crew with the room's pinned 24/7 operators worked in ahead of the rotating
/// team: the remaining seats go to the SUBSET of the team that scores best alongside
/// the pinned operators - not a naive truncation, which could seat a second order-value
/// operator whose bonus doesn't stack with the pinned one (Bibeak next to a pinned
/// Proviso) while a speed operator rests. With nothing pinned this is just the team.
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

/// Build the recommended rotation for the player's base, pairing each shift's
/// recommended crews with their saved presets.
///
/// Wrapped in the same base-wide-conditional two-pass as the peak optimizer: pass 1
/// runs with Hoederer-style "when X works any Work Area" bonuses off to learn who is
/// actually deployed, pass 2 re-plans with the unlocked bonuses baked in.
pub fn recommend_shift_rotation(
    operators: &[OperatorBaseProfile],
    building: &UserBuilding,
    building_data: &BuildingDataFile,
    registry: &HashMap<String, BuffResolutionStrategy>,
    morale_drains: &HashMap<String, f64>,
) -> ShiftRotation {
    if !base_wide_relevant(operators, registry) {
        return rotation_core(operators, building, building_data, registry, morale_drains);
    }
    let pass1_registry = resolve_base_wide(registry, &HashSet::new());
    let pass1 = rotation_core(
        operators,
        building,
        building_data,
        &pass1_registry,
        morale_drains,
    );
    let deployed: HashSet<String> = pass1
        .shifts
        .iter()
        .flat_map(|s| s.rooms.iter().filter(|r| r.active))
        .flat_map(|r| r.recommended.iter().cloned())
        .collect();
    if !base_wide_unlocked(operators, registry, &deployed) {
        return pass1;
    }
    let pass2_registry = resolve_base_wide(registry, &deployed);
    rotation_core(
        operators,
        building,
        building_data,
        &pass2_registry,
        morale_drains,
    )
}

fn rotation_core(
    operators: &[OperatorBaseProfile],
    building: &UserBuilding,
    building_data: &BuildingDataFile,
    registry: &HashMap<String, BuffResolutionStrategy>,
    morale_drains: &HashMap<String, f64>,
) -> ShiftRotation {
    let facility_counts = effective_facility_counts(building, operators, registry);
    let total_dorm_levels = building.total_dorm_levels();
    let production_rooms: Vec<&UserRoom> = building
        .rooms
        .iter()
        .filter(|r| is_production_room(&r.room_type))
        .collect();

    // Control Center Squad 1 (with its global bonuses / faction conditions) and the
    // balanced production teams, re-selecting the CC when one of its operators turns
    // out dead weight against the teams actually chosen.
    let (cc_plan, mut groups) = rotation_cc_plan(
        operators,
        building,
        building_data,
        registry,
        |assigned, global_bonuses, cc_conditions| {
            plan_production_groups(
                &production_rooms,
                operators,
                assigned,
                registry,
                building_data,
                &facility_counts,
                total_dorm_levels,
                global_bonuses,
                cc_conditions,
                morale_drains,
                &HashMap::new(),
            )
        },
    );

    // ── Fiammetta 24/7 sustain ────────────────────────────────────────────────────
    // Preset evidence first (the player's explicit choice), then a proactive top-up:
    // owning a manager recommends sustaining the trading operator whose 24/7 presence
    // preserves the most output. That's the operator with the largest DROP-ONE delta
    // (a Shamare-type nullifier whose team collapses without her, or a Proviso-type
    // whose order-value multiplier vanishes), scaled by the extra uptime the pin buys
    // (a fast-draining operator gains more from never resting).
    let mut sustained =
        preset_sustained_operators(building, operators, morale_drains, building_data);
    let managers = num_morale_swap_managers(operators, building_data);
    let op_index = build_op_index(operators);
    if sustained.len() < managers {
        let recovery = morale_recovery(building);
        let mut cands: Vec<(String, f64)> = Vec::new();
        for g in groups.iter().filter(|g| g.room_type == "TRADING") {
            for team in g.teams.iter().filter(|t| !t.ops.is_empty()) {
                let team_score = room_search_score(&g.room_type, team.speed, team.value);
                for id in &team.ops {
                    if sustained.contains(id) {
                        continue;
                    }
                    let without: Vec<String> =
                        team.ops.iter().filter(|o| o != &id).cloned().collect();
                    let (speed, value) = compute_team_efficiency(
                        &without,
                        &g.room_type,
                        g.formula_type.as_deref(),
                        &op_index,
                        registry,
                        building_data,
                        &facility_counts,
                        total_dorm_levels,
                        morale_drains,
                        &cc_plan.conditions,
                    );
                    let delta =
                        (team_score - room_search_score(&g.room_type, speed, value)).max(0.0);
                    let uptime = op_index
                        .get(id.as_str())
                        .map_or(1.0, |op| op_uptime(op, morale_drains, recovery));
                    // Without the manager the operator works ~2 of 3 shifts at their
                    // morale-limited uptime; with it, all three at full.
                    let extra_uptime = 1.0 - uptime * (2.0 / 3.0);
                    cands.push((id.clone(), delta * extra_uptime));
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
    // Pin each sustained operator that a team actually fields to ONE room (the room
    // their presets hold them in, else the first room of their group) for all three
    // shifts, then RE-PLAN the teams with the pinned operators excluded from the pool:
    // the pin permanently occupies a slot, so teams planned around it must not spend
    // members that don't stack with it (a second order-value operator next to a pinned
    // Proviso is wasted - the packer needs to know she's already there).
    let mut kept_by_room: HashMap<String, Vec<String>> = HashMap::new();
    for g in &groups {
        let group_rooms: Vec<String> = g.rooms.iter().map(|(s, _)| s.clone()).collect();
        for team in &g.teams {
            for op in team.ops.iter().filter(|op| sustained.contains(*op)) {
                let room = preset_room_of(building, op)
                    .filter(|slot| group_rooms.contains(slot))
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
    if !sustained_label.is_empty() {
        let mut exclude: HashSet<String> = cc_plan.squad1.iter().cloned().collect();
        exclude.extend(sustained_label.iter().cloned());
        // Teams re-planned around the pin: the pinned operator is out of the pool and
        // their permanent slot shrinks the teams that only ever work their room.
        let reserved: HashMap<String, usize> = kept_by_room
            .iter()
            .map(|(slot, ops)| (slot.clone(), ops.len()))
            .collect();
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
            &reserved,
        );
    }

    // ── Second squads for the two-squad rooms ────────────────────────────────────
    let mut used: HashSet<String> = cc_plan.squad1.iter().cloned().collect();
    for g in &groups {
        for t in &g.teams {
            used.extend(t.ops.iter().cloned());
        }
    }
    used.extend(sustained_label.iter().cloned());

    // Office / Reception: Squad 1 from the best leftovers, Squad 2 from what remains.
    let aux_squads = |assigned: &mut HashSet<String>| -> HashMap<String, (String, Vec<String>)> {
        let mut rooms = Vec::new();
        assign_auxiliary_rooms(
            &mut rooms,
            building,
            building_data,
            operators,
            registry,
            morale_drains,
            assigned,
        );
        rooms
            .into_iter()
            .map(|r| (r.slot_id.clone(), (r.room_type, r.operators)))
            .collect()
    };
    let mut assigned = used.clone();
    let aux1 = aux_squads(&mut assigned);
    let aux2 = aux_squads(&mut assigned);

    // Control Center Squad 2: the best global-bonus fill from the leftovers, so the
    // CC keeps granting bonuses while Squad 1 rests.
    let cc_squad2 = cc_plan.squad2(operators, building_data, registry, &assigned);
    assigned.extend(cc_squad2.iter().cloned());

    // Power plants: two squads of the best leftover power specialists per plant.
    let power_plan = build_power_plan(
        operators,
        building,
        building_data,
        registry,
        &facility_counts,
        &assigned,
    );
    for plant in &power_plan {
        assigned.extend(plant.main.iter().cloned());
        assigned.extend(plant.backup.iter().cloned());
    }

    // Top both CC squads up to a full crew with the lowest-opportunity-cost leftovers,
    // exactly like the peak plan does - a half-empty Control Center (Squad 2 = two
    // bonus operators and three vacant seats) reads as a mistake, and spare seats are
    // free real estate for benchwarmers. Runs LAST so no useful operator is stolen
    // from production, aux, or power. An entirely empty Squad 2 stays empty (dark).
    let mut cc1 = cc_plan.squad1.clone();
    let mut cc2 = cc_squad2;
    if !cc1.is_empty() {
        fill_remaining_slots(
            &mut cc1,
            cc_plan.control_slots,
            "CONTROL",
            operators,
            building_data,
            &mut assigned,
        );
    }
    if !cc2.is_empty() {
        fill_remaining_slots(
            &mut cc2,
            cc_plan.control_slots,
            "CONTROL",
            operators,
            building_data,
            &mut assigned,
        );
    }

    // ── Synergy phase alignment ──────────────────────────────────────────────────
    // A CC Squad-1 faction condition (Viviana + her Knights) links the CC to the
    // production teams that satisfy it: the CC flips to a 24h block on shifts 1+2 and
    // each linked group phases its linked team onto the same block (ordinal 0).
    let mut cc_pattern = SquadPattern::Alternating;
    let mut linked_groups: HashSet<usize> = HashSet::new();
    for (gi, g) in groups.iter_mut().enumerate() {
        let mut linked: Option<usize> = None;
        let mut best_weight = 0.0;
        for (ti, team) in g.teams.iter().enumerate() {
            let profiles: Vec<&OperatorBaseProfile> = team
                .ops
                .iter()
                .filter_map(|id| op_index.get(id.as_str()).copied())
                .collect();
            let weight: f64 = cc_plan
                .conditions
                .iter()
                .map(|c| c.contribution(&g.room_type, &profiles))
                .sum();
            if weight > best_weight {
                best_weight = weight;
                linked = Some(ti);
            }
        }
        if let Some(ti) = linked
            && best_weight > 0.0
        {
            cc_pattern = SquadPattern::Block;
            linked_groups.insert(gi);
            // Ordinal 0 holds the shifts-1+2 block; move the linked team there.
            if ti != 0 {
                g.teams.swap(0, ti);
            }
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
    // The player's saved presets are equal alternating halves (two presets = the A/B
    // swap they make at every login; three = one per shift), so cycle through them -
    // preset 1 is NOT a "main" that outranks the others.
    let preset_for = |slot: &str, k: usize| -> Vec<String> {
        presets
            .get(slot)
            .filter(|p| !p.is_empty())
            .map(|p| p[k % p.len()].clone())
            .unwrap_or_default()
    };

    // ── Preset-phase alignment ───────────────────────────────────────────────────
    // Which team is "Team A" / which squad is "Squad 1" is arbitrary - over the real
    // login alternation every unit works the same share - so pick the arrangement that
    // matches the player's saved presets. The plan then CONFIRMS operators they
    // already run in a phase instead of telling them to swap between phases.
    let sym_diff = |a: &[String], b: &[String]| -> usize {
        if b.is_empty() {
            return 0; // no preset saved - nothing to match against
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
        // Try every room ordering x team permutation (teams only move between
        // ordinals working the same number of cells, and a synergy-linked team
        // stays on the shifts-1+2 block). Arrangements are ranked OUTPUT FIRST -
        // rooms with a pinned 24/7 operator make phases output-sensitive (parking
        // a Shamare team on a pinned Proviso's post wastes her value) - and only
        // then by preset churn among output ties.
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
                // Quantize the output so float noise doesn't defeat the churn tie-break.
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
    // Two-squad rooms: flipping Squad 1/2 is free for the Alternating pattern (the
    // real cadence is an even A/B swap), so flip whenever the player's presets run
    // the squads in the opposite phase.
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
    // The power squads were split EVENLY across the plants, so flipping must be a
    // group decision (all plants or none) - flipping plants individually would
    // reshuffle the balanced squads back into a lopsided pair. And the STRONGER
    // squad stays first (Squad 1 covers two of the three displayed shifts), so a
    // preset-phase flip is only taken when it doesn't demote the stronger squad.
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
        if flip < keep && backup_total + 1e-9 >= main_total {
            for plant in &mut power_plan {
                std::mem::swap(&mut plant.main, &mut plant.backup);
            }
        }
    }
    let mut aux1 = aux1;
    let mut aux2 = aux2;
    for (slot, (_, squad1)) in &mut aux1 {
        if let Some((_, squad2)) = aux2.get_mut(slot)
            && flip_better(slot, squad1, squad2)
        {
            std::mem::swap(squad1, squad2);
        }
    }
    if cc_pattern == SquadPattern::Alternating
        && let Some(cc_slot) = &cc_plan.slot_id
        && flip_better(cc_slot, &cc1, &cc2)
    {
        std::mem::swap(&mut cc1, &mut cc2);
    }

    // ── Emit the three shifts ────────────────────────────────────────────────────
    let team_letter = |ordinal: usize| -> String {
        char::from(b'A' + u8::try_from(ordinal % 26).unwrap_or(0)).to_string()
    };
    // The displayed crew's room efficiency (speed incl. Squad-1 globals), for the
    // production/power cells where output distribution matters.
    let crew_efficiency = |crew: &[String], room_type: &str, formula: Option<&str>| -> f64 {
        let (speed, _) = compute_team_efficiency(
            crew,
            room_type,
            formula,
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

        // Production groups: each room runs the team its tiling cell names, with any
        // pinned 24/7 operator merged in ahead. An empty team rests the room dark.
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

        // Power plants: Squad 1 / Squad 2 alternating (a plant generates electricity
        // regardless of who staffs it, so an empty squad just rests the plant dark).
        for plant in &power_plan {
            let squad = SquadPattern::Alternating.squad_at(k);
            let crew = if squad == 0 || plant.backup.is_empty() {
                &plant.main
            } else {
                &plant.backup
            };
            let active = (squad == 0 || !plant.backup.is_empty()) && !crew.is_empty();
            let efficiency = crew_efficiency(crew, "POWER", None);
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

        // Office / Reception: Squad 1 / Squad 2 alternating; with no second squad the
        // room rests dark on Squad 2's shift.
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

        // Control Center: Alternating by default; a Block (shifts 1+2) when a Squad-1
        // synergy needs its dependent production teams' 24h window.
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

        shifts.push(Shift {
            index: k + 1,
            rooms,
        });
    }

    ShiftRotation {
        shifts,
        sustained: sustained_label,
    }
}

/// The production room whose saved presets ALL contain `op` - where the player
/// physically parks a 24/7 operator.
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

/// A power plant's electricity output is fixed by its level - operators only matter
/// if they carry a Power Plant base skill (drone recovery, shared morale drain, etc.).
/// An operator with no power skill (e.g. a pure Dormitory/Training operator) is dead
/// weight there, so only genuine power specialists are recommended for the plants.
fn has_power_skill(op: &OperatorBaseProfile, building_data: &BuildingDataFile) -> bool {
    op.available_buffs.iter().any(|b| {
        building_data
            .buffs
            .get(b)
            .is_some_and(|buff| buff.room_type == "POWER")
    })
}

/// Rough strength of an operator's POWER base skill, for ranking who staffs the plants:
/// the largest value any of its power buffs resolves to (drone recovery, power output,
/// etc.). Promoted skills resolve higher, so an E2 power specialist outranks an E0 one.
fn power_value(
    op: &OperatorBaseProfile,
    building_data: &BuildingDataFile,
    registry: &HashMap<String, BuffResolutionStrategy>,
    facility_counts: &HashMap<String, usize>,
) -> f64 {
    let ctx = EvalContext {
        facility_counts,
        total_dorm_levels: 0,
        room_teammates: Vec::new(),
        self_order_limit: 0,
    };
    op.available_buffs
        .iter()
        .filter(|b| {
            building_data
                .buffs
                .get(*b)
                .is_some_and(|buff| buff.room_type == "POWER")
        })
        .filter_map(|b| registry.get(b))
        .map(|s| match s {
            // A facility-count enabler (Greyy the Lightningbearer E2) earns no drone output, but
            // it must be STATIONED in a power plant for its "+1 Power Plant" to fire and power the
            // automation factories - so rank it highly here, above ordinary drone operators.
            BuffResolutionStrategy::FacilityCountModifier {
                target_room,
                amount,
            } if target_room == "POWER" => f64::from(*amount) * 30.0,
            other => evaluate_buff(other, &ctx),
        })
        .fold(0.0, f64::max)
}

/// A power plant's Squad 1 plus the Squad 2 that covers its alternating rest shift.
struct PowerPlant {
    slot_id: String,
    /// Best specialist(s), stationed on Squad 1's shifts.
    main: Vec<String>,
    /// Next-best specialist(s), covering Squad 2's shift (empty if the roster has no
    /// spare power operator, in which case the plant rests dark that shift).
    backup: Vec<String>,
}

/// Power plants generate electricity regardless of who staffs them, so the optimizer
/// doesn't pick their crews. Staff the plants with the BEST leftover power specialists
/// (operators carrying a power base skill that aren't already placed anywhere else in
/// the rotation), split into two EVEN squads: each specialist joins whichever squad has
/// the lower running total, so drone recovery stays level across the alternation
/// instead of one squad hoarding the strongest operators. Operators with no power skill
/// are left out entirely - parking them here would do nothing but drain morale.
fn build_power_plan(
    operators: &[OperatorBaseProfile],
    building: &UserBuilding,
    building_data: &BuildingDataFile,
    registry: &HashMap<String, BuffResolutionStrategy>,
    facility_counts: &HashMap<String, usize>,
    used: &HashSet<String>,
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
    let per_squad: usize = slots.iter().sum();

    // Even split: strongest first, each into the squad with the lower running total
    // (a squad stops accepting once its plant slots are full). With 25/20/20/20/20/15
    // specialists this yields 60/60 rather than best-first's 65/55.
    let mut squads: [(Vec<String>, f64); 2] = [(Vec::new(), 0.0), (Vec::new(), 0.0)];
    for (id, value) in ranked.into_iter().take(per_squad * 2) {
        let pick = (0..2)
            .filter(|&i| squads[i].0.len() < per_squad)
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
    // The split can't always be exactly even; the STRONGER half leads as Squad 1
    // (it covers shifts 1 & 3 in the displayed cycle), the weaker covers shift 2.
    if squads[1].1 > squads[0].1 {
        squads.swap(0, 1);
    }
    let [(squad1, _), (squad2, _)] = squads;
    let mut squad1 = squad1.into_iter();
    let mut squad2 = squad2.into_iter();

    power_rooms
        .iter()
        .zip(&slots)
        .map(|(r, n)| PowerPlant {
            slot_id: r.slot_id.clone(),
            main: (0..*n).filter_map(|_| squad1.next()).collect(),
            backup: (0..*n).filter_map(|_| squad2.next()).collect(),
        })
        .collect()
}
