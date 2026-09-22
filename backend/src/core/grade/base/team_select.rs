//! Balanced multi-team selection for the shift rotation.
//!
//! The rotation staffs each production GROUP (rooms making the same product: the
//! Gold factories, the EXP factories, the Trading Posts) with `ceil(3N/2)` teams
//! tiled as 24-hour blocks over the group's `N rooms × 3 shifts` cells - the
//! classic login rhythm where exactly one team per group swaps out every 12h.
//!
//! Teams are chosen JOINTLY across all groups to maximize the summed output of
//! every team, not best-team-first. Best-first "frontloads" the strong operators
//! into the first team and leaves the last team hollow (107%/106%/75%); the sum
//! objective yields the balanced spread (≈100/95/95/90) and routes cross-formula
//! generics to whichever product values them most. Whole candidate TEAMS are the
//! packing unit, so superadditive pairs (Texas + Lappland) stay intact.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

use crate::core::gamedata::types::building::BuildingDataFile;

use super::assignment::{
    CandidateTeam, CcCondition, POST_TIE_BAND, assignment_value, build_op_index, candidate_pool,
    compute_team_efficiency, enumerate_candidate_teams, has_automation_buff, op_is_nullifier,
    padding_cost, room_search_score,
};
use super::buff_registry::BuffResolutionStrategy;
use super::shift_rotation::SHIFT_COUNT;
use super::sustain_sim::sustains_24h_block;
use super::types::{OperatorBaseProfile, RoomAssignment, UserRoom};
use super::util::max_stationed_at_level;

/// Widen the candidate pool beyond the single-team cut when several disjoint teams
/// must come out of one enumeration.
const BASE_POOL: usize = 16;
const POOL_PER_EXTRA_TEAM: usize = 8;
/// Keep this many top candidates per group for the packing search.
const CANDIDATES_PER_GROUP: usize = 400;
/// Beam width for the disjoint-selection search.
const BEAM_WIDTH: usize = 300;
/// Disjoint extensions considered per team slot per beam state.
const EXTENSIONS_PER_SLOT: usize = 24;
/// Local-search polish passes after the beam.
const POLISH_PASSES: usize = 4;

/// A set of operators, as a fixed-width bitset.
///
/// The beam search tests team disjointness millions of times, so the set has to
/// be a couple of machine words rather than a `HashSet`. 256 bits covers the
/// worst-case universe: three production groups, each enumerating from a pool of
/// `BASE_POOL + POOL_PER_EXTRA_TEAM × (teams − 1)` operators (~72 at 5 rooms),
/// so ~216 distinct operators at the extreme.
///
/// This replaced a bare `u128`, whose 128-bit budget the union across all three
/// groups could exceed - every candidate team containing an operator past the
/// budget was then silently discarded, shrinking the pool the rotation could
/// draw on without any signal that it had happened.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct OpMask([u64; 4]);

impl OpMask {
    pub(crate) const EMPTY: Self = Self([0; 4]);
    /// Distinct operators representable. Indices at or past this are rejected.
    pub(crate) const CAPACITY: usize = 256;

    const fn set(&mut self, bit: usize) {
        self.0[bit / 64] |= 1u64 << (bit % 64);
    }

    fn intersects(&self, other: &Self) -> bool {
        self.0.iter().zip(&other.0).any(|(a, b)| a & b != 0)
    }

    fn union(&self, other: &Self) -> Self {
        let mut out = *self;
        for (o, b) in out.0.iter_mut().zip(&other.0) {
            *o |= *b;
        }
        out
    }

    fn difference(&self, other: &Self) -> Self {
        let mut out = *self;
        for (o, b) in out.0.iter_mut().zip(&other.0) {
            *o &= !*b;
        }
        out
    }
}

/// One production group to staff: rooms making the same product.
pub struct GroupSpec {
    pub(crate) room_type: String,
    pub(crate) formula_type: Option<String>,
    /// `(slot_id, level)` per room, in emission order.
    pub(crate) rooms: Vec<(String, i32)>,
}

/// A staffed production group: the selected teams (ordinal order matches the
/// tiling - ordinal 0 holds the shifts-1+2 block) and the room×shift tiling.
pub struct PlannedGroup {
    pub(crate) room_type: String,
    pub(crate) formula_type: Option<String>,
    pub(crate) rooms: Vec<(String, i32)>,
    pub(crate) teams: Vec<CandidateTeam>,
    /// `cells[room_idx][shift]` = index into `teams`.
    pub(crate) cells: Vec<[usize; SHIFT_COUNT]>,
}

/// Tile a group of `n_rooms` into 24h team blocks: chain the `3N` room-shift cells
/// room-major and hand each team two consecutive cells (the trailing cell of an odd
/// chain goes to the last team alone - it works a single 12h block).
///
/// For 2 rooms this is the login rhythm the player runs by hand:
/// `room0 = [T0, T0, T1]`, `room1 = [T1, T2, T2]` - every team works 24h straight
/// (T1's block wraps shift 3 -> shift 1), rests 12h, and exactly one team per group
/// swaps out at each login.
pub fn tile_group(n_rooms: usize) -> Vec<[usize; SHIFT_COUNT]> {
    let mut cells = vec![[0usize; SHIFT_COUNT]; n_rooms];
    for pos in 0..(n_rooms * SHIFT_COUNT) {
        cells[pos / SHIFT_COUNT][pos % SHIFT_COUNT] = pos / 2;
    }
    cells
}

/// Number of teams `tile_group(n_rooms)` needs: one per pair of cells.
pub const fn teams_for_rooms(n_rooms: usize) -> usize {
    (n_rooms * SHIFT_COUNT).div_ceil(2)
}

/// How many cells each team ordinal occupies in the tiling (2, except a trailing 1).
fn ordinal_weights(n_rooms: usize) -> Vec<usize> {
    let teams = teams_for_rooms(n_rooms);
    let mut w = vec![0usize; teams];
    for room in tile_group(n_rooms) {
        for t in room {
            w[t] += 1;
        }
    }
    w
}

/// Everything `enumerate_candidate_teams` reads that varies within one
/// core-planner run: the room shape, the enumeration flags, the
/// Control-Center conditions in force and the FILTERED candidate ids.
/// Registry, building data, facility counts and morale drains are fixed for
/// the run, which is why an [`EnumerationMemo`] must not outlive it.
#[derive(Hash, PartialEq, Eq)]
struct MemoKey {
    room_type: String,
    formula: Option<String>,
    level: i32,
    capacity: i32,
    pool: usize,
    include_automation: bool,
    require_24h: bool,
    cc: Vec<String>,
    candidates: Vec<String>,
}

/// Memo for `enumerate_candidate_teams` over one core-planner run.
///
/// The Fiammetta 24/7-seat oracle re-plans the whole base once per trading
/// candidate, and the rotation runs twice (two passes): on a full base the
/// same four-factory enumeration came back from identical inputs twenty
/// times in one planner request (0.4 s each in a debug build). The candidate
/// a trial excludes is a trader with no factory skill, so the factory pools -
/// and therefore the enumerations - are the same. The key holds the filtered
/// ids, so a hit means the enumeration would have read exactly the same
/// inputs; the result is the same by construction.
#[derive(Default)]
pub struct EnumerationMemo(RefCell<HashMap<MemoKey, Vec<CandidateTeam>>>);

impl EnumerationMemo {
    #[allow(clippy::too_many_arguments)]
    fn key(
        room_type: &str,
        level: i32,
        formula: Option<&str>,
        operators: &[OperatorBaseProfile],
        assigned: &HashSet<String>,
        registry: &HashMap<String, BuffResolutionStrategy>,
        building_data: &BuildingDataFile,
        morale_drains: &HashMap<String, f64>,
        capacity: i32,
        pool: usize,
        include_automation: bool,
        require_24h: bool,
        cc_conditions: &[CcCondition],
    ) -> MemoKey {
        MemoKey {
            room_type: room_type.to_string(),
            formula: formula.map(str::to_string),
            level,
            capacity,
            pool,
            include_automation,
            require_24h,
            cc: cc_conditions.iter().map(CcCondition::fingerprint).collect(),
            candidates: candidate_pool(
                room_type,
                formula,
                operators,
                assigned,
                registry,
                building_data,
                morale_drains,
                include_automation,
                require_24h,
            )
            .iter()
            .map(|op| op.char_id.clone())
            .collect(),
        }
    }

    fn get_or_enumerate(
        &self,
        key: MemoKey,
        enumerate: impl FnOnce() -> Vec<CandidateTeam>,
    ) -> Vec<CandidateTeam> {
        let hit = self.0.borrow().get(&key).cloned();
        if let Some(teams) = hit {
            return teams;
        }
        let teams = enumerate();
        self.0.borrow_mut().insert(key, teams.clone());
        teams
    }
}

/// Select the balanced team sets for every production group and pick the best
/// gold/EXP split, returning the planned groups (teams padded to capacity, ordinal
/// order strongest-first with the weakest team on any trailing 1-cell block).
#[allow(clippy::too_many_arguments)]
pub fn plan_production_groups(
    production_rooms: &[&UserRoom],
    operators: &[OperatorBaseProfile],
    assigned: &HashSet<String>,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
    facility_counts: &HashMap<String, usize>,
    total_dorm_levels: i32,
    global_bonuses: &HashMap<String, f64>,
    cc_conditions: &[CcCondition],
    morale_drains: &HashMap<String, f64>,
    reserved_seats: &HashMap<String, usize>,
    kept: &HashMap<String, Vec<String>>,
    memo: &EnumerationMemo,
) -> Vec<PlannedGroup> {
    // Seats held out of each room: `reserved_seats` (a seat the sustain
    // oracle keeps EMPTY to price a plan without its candidate) plus the
    // pinned crews, which the trading posts fold into their teams.
    let mut reserved: HashMap<String, usize> = reserved_seats.clone();
    for (slot, ops) in kept {
        *reserved.entry(slot.clone()).or_insert(0) += ops.len();
    }
    let mut factory_rooms: Vec<&&UserRoom> = production_rooms
        .iter()
        .filter(|r| r.room_type == "MANUFACTURE")
        .collect();
    // Lay the gold/EXP split onto the slots the player ALREADY runs that way (the
    // split optimizes the COUNT; the physical layout stays theirs).
    factory_rooms.sort_by_key(|r| match r.current_formula.as_deref() {
        Some("F_GOLD") => 0,
        Some("F_EXP") => 1,
        _ => 2,
    });
    let trading_rooms: Vec<&&UserRoom> = production_rooms
        .iter()
        .filter(|r| r.room_type == "TRADING")
        .collect();

    let num_factories = factory_rooms.len();
    // Gold factories must cover the trading posts' gold demand; only the splits at
    // and just above that are competitive (excess gold is unsold).
    let min_gold = trading_rooms.len().min(num_factories);
    let max_gold = (min_gold + 1).min(num_factories);

    let mut best: Option<(f64, Vec<PlannedGroup>)> = None;
    for num_gold in min_gold..=max_gold {
        let mut specs: Vec<GroupSpec> = Vec::new();
        let gold: Vec<(String, i32)> = factory_rooms[..num_gold]
            .iter()
            .map(|r| (r.slot_id.clone(), r.level))
            .collect();
        if !gold.is_empty() {
            specs.push(GroupSpec {
                room_type: "MANUFACTURE".into(),
                formula_type: Some("F_GOLD".into()),
                rooms: gold,
            });
        }
        let exp: Vec<(String, i32)> = factory_rooms[num_gold..]
            .iter()
            .map(|r| (r.slot_id.clone(), r.level))
            .collect();
        if !exp.is_empty() {
            specs.push(GroupSpec {
                room_type: "MANUFACTURE".into(),
                formula_type: Some("F_EXP".into()),
                rooms: exp,
            });
        }
        // One trading group PER POST LEVEL: a post's order rarity, base
        // limit and order value follow its level, so a level-3 and a
        // level-2 post planned as one group (at the lower level) priced
        // Shamare's tailoring at nothing and Proviso's bonus bars at the
        // wrong post. Same-level posts still share a group and a tiling.
        let mut levels: Vec<i32> = trading_rooms.iter().map(|r| r.level).collect();
        levels.sort_unstable();
        levels.dedup();
        levels.reverse();
        for level in levels {
            specs.push(GroupSpec {
                room_type: "TRADING".into(),
                formula_type: None,
                rooms: trading_rooms
                    .iter()
                    .filter(|r| r.level == level)
                    .map(|r| (r.slot_id.clone(), r.level))
                    .collect(),
            });
        }
        if specs.is_empty() {
            continue;
        }

        let mut planned = select_balanced_teams(
            &specs,
            operators,
            assigned,
            registry,
            building_data,
            facility_counts,
            total_dorm_levels,
            global_bonuses,
            cc_conditions,
            morale_drains,
            kept,
            memo,
        );
        pad_teams(
            &mut planned,
            operators,
            assigned,
            registry,
            building_data,
            facility_counts,
            total_dorm_levels,
            cc_conditions,
            morale_drains,
            &reserved,
        );
        let objective = tiled_objective(&planned, global_bonuses);
        if best.as_ref().is_none_or(|(b, _)| objective > *b) {
            best = Some((objective, planned));
        }
    }
    best.map(|(_, planned)| planned).unwrap_or_default()
}

/// The rotation objective for a planned split: realized yield summed over the three
/// shifts, staffing each room with the team its tiling cell names. This couples the
/// gold factories to the trading posts per shift (LMD = min(made, sold) × 500), the
/// same yield model the peak optimizer scores splits with.
pub fn tiled_objective(groups: &[PlannedGroup], global_bonuses: &HashMap<String, f64>) -> f64 {
    (0..SHIFT_COUNT)
        .map(|shift| {
            let rooms: Vec<RoomAssignment> = groups
                .iter()
                .flat_map(|g| {
                    g.rooms
                        .iter()
                        .enumerate()
                        .filter_map(move |(ri, (slot, level))| {
                            let team = g.teams.get(g.cells[ri][shift])?;
                            let global = *global_bonuses.get(&g.room_type).unwrap_or(&0.0);
                            Some(RoomAssignment {
                                slot_id: slot.clone(),
                                room_type: g.room_type.clone(),
                                level: *level,
                                formula_type: g.formula_type.clone(),
                                operators: team.ops.clone(),
                                total_efficiency: team.speed + global,
                                order_value: team.value,
                                order_gold: team.gold,
                                order_limit: team.order_limit,
                                ..Default::default()
                            })
                        })
                })
                .collect();
            assignment_value(&rooms)
        })
        .sum()
}

/// The trading posts' own LMD summed over the tiling, uncoupled from the
/// gold supply: the tie-breaker inside `POST_TIE_BAND`.
fn trading_uncoupled_lmd(groups: &[PlannedGroup], global_bonuses: &HashMap<String, f64>) -> f64 {
    let global = *global_bonuses.get("TRADING").unwrap_or(&0.0);
    groups
        .iter()
        .filter(|g| g.room_type == "TRADING")
        .flat_map(|g| {
            g.rooms
                .iter()
                .enumerate()
                .flat_map(move |(ri, (_, level))| {
                    (0..SHIFT_COUNT).filter_map(move |shift| {
                        let team = g.teams.get(g.cells[ri][shift])?;
                        Some(
                            super::yield_model::room_yield(
                                "TRADING",
                                None,
                                *level,
                                team.speed + global,
                                team.value,
                                team.ops.len(),
                                team.order_limit,
                            )
                            .lmd_per_day,
                        )
                    })
                })
        })
        .sum()
}

/// Beam-search the disjoint team selection maximizing `Σ score × cells_worked`
/// across every group's team slots, then polish with local moves. Returns each
/// group planned with its teams in ordinal order (strongest on the widest block).
#[allow(clippy::too_many_arguments)]
fn select_balanced_teams(
    specs: &[GroupSpec],
    operators: &[OperatorBaseProfile],
    assigned: &HashSet<String>,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
    facility_counts: &HashMap<String, usize>,
    total_dorm_levels: i32,
    global_bonuses: &HashMap<String, f64>,
    cc_conditions: &[CcCondition],
    morale_drains: &HashMap<String, f64>,
    kept: &HashMap<String, Vec<String>>,
    memo: &EnumerationMemo,
) -> Vec<PlannedGroup> {
    let op_index = build_op_index(operators);
    // Phase A: enumerate candidate teams per group (once per group - this replaces
    // the old per-room searches). Team size is bounded by the group's SMALLEST room
    // so any team fits any room its block spans.
    let per_group: Vec<Vec<CandidateTeam>> = specs
        .iter()
        .map(|spec| {
            let capacity = group_capacity(spec, building_data);
            // A pinned 24/7 crew (the sustained Shamare, Proviso) is part
            // of every team its post fields, so the post's candidates carry
            // it and are scored WITH it: enumerated without her and merged
            // afterwards, a level-3 post picked a Proviso team and the merge
            // put Proviso beside Shamare, who nullifies her (00980819).
            // Only a group whose rooms all hold the same pinned crew can be
            // folded this way; others keep the seat-count reservation.
            let group_kept: Option<&Vec<String>> = {
                let sets: Vec<&Vec<String>> = spec
                    .rooms
                    .iter()
                    .filter_map(|(slot, _)| kept.get(slot))
                    .filter(|ops| !ops.is_empty())
                    .collect();
                (sets.len() == spec.rooms.len() && sets.iter().all(|s| *s == sets[0]))
                    .then(|| sets[0])
            };
            let teams_needed = teams_for_rooms(spec.rooms.len());
            let min_level = spec.rooms.iter().map(|(_, l)| *l).min().unwrap_or(1);
            let pool = BASE_POOL + POOL_PER_EXTRA_TEAM * teams_needed.saturating_sub(1);
            // The pinned crew is allowed back into ITS post's enumeration:
            // the enumerator knows how to build a nullifier's squad (Shamare
            // with the order-value partners that survive her), and a team
            // built without her and folded afterwards never is one.
            let assigned_for_group: HashSet<String> = group_kept.map_or_else(
                || assigned.clone(),
                |pinned| {
                    assigned
                        .iter()
                        .filter(|id| !pinned.contains(id))
                        .cloned()
                        .collect()
                },
            );
            let assigned = &assigned_for_group;
            let key = EnumerationMemo::key(
                &spec.room_type,
                min_level,
                spec.formula_type.as_deref(),
                operators,
                assigned,
                registry,
                building_data,
                morale_drains,
                capacity as i32,
                pool,
                spec.room_type == "MANUFACTURE",
                true,
                cc_conditions,
            );
            let mut teams = memo.get_or_enumerate(key, || {
                enumerate_candidate_teams(
                    &spec.room_type,
                    min_level,
                    spec.formula_type.as_deref(),
                    operators,
                    assigned,
                    registry,
                    building_data,
                    facility_counts,
                    total_dorm_levels,
                    capacity as i32,
                    cc_conditions,
                    morale_drains,
                    false,
                    pool,
                    // Factories can run automation teams (Weedy + facility-count scalers
                    // like Purestream); enumerate them as ordinary candidates.
                    spec.room_type == "MANUFACTURE",
                    // Rotation teams work 24h blocks - heavy-drainers can't finish one.
                    true,
                )
            });
            teams.truncate(CANDIDATES_PER_GROUP);
            if let Some(pinned) = group_kept
                && spec.room_type == "TRADING"
            {
                let level = spec.rooms.iter().map(|(_, l)| *l).min().unwrap_or(1);
                let free = capacity.saturating_sub(pinned.len());
                let mut seen: HashSet<Vec<String>> = HashSet::new();
                let mut folded: Vec<CandidateTeam> = Vec::new();
                for t in &teams {
                    let mut ops: Vec<String> = pinned.clone();
                    ops.extend(
                        t.ops
                            .iter()
                            .filter(|o| !pinned.contains(o))
                            .take(free)
                            .cloned(),
                    );
                    let mut key = ops.clone();
                    key.sort();
                    if !seen.insert(key) {
                        continue;
                    }
                    // A team the enumerator built around the pin keeps its
                    // own figures; only the folded ones are re-scored.
                    if pinned.iter().all(|p| t.ops.contains(p)) && t.ops.len() == ops.len() {
                        folded.push(t.clone());
                        continue;
                    }
                    let totals = super::assignment::compute_team_totals(
                        &ops,
                        &spec.room_type,
                        None,
                        Some(level),
                        &op_index,
                        registry,
                        building_data,
                        facility_counts,
                        total_dorm_levels,
                        morale_drains,
                        cc_conditions,
                    );
                    folded.push(CandidateTeam {
                        ops,
                        speed: totals.speed_pct,
                        value: totals.order_value_pct,
                        gold: totals.order_gold_pct,
                        order_limit: totals.order_limit,
                        score: room_search_score(
                            &spec.room_type,
                            totals.speed_pct,
                            totals.order_value_pct,
                        ),
                    });
                }
                folded.sort_by(|a, b| {
                    b.score
                        .partial_cmp(&a.score)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                teams = folded;
            }
            teams
        })
        .collect();

    // Operator universe -> bit indices for the disjointness masks. The universe is
    // the UNION across every group's candidate pool, which on a full base
    // (gold + EXP + trading, each up to `BASE_POOL + 8×extra` operators) runs to
    // ~200 - so [`OpMask`] is sized to hold it rather than dropping the overflow.
    let mut bit_of: HashMap<String, usize> = HashMap::new();
    for teams in &per_group {
        for t in teams {
            for op in &t.ops {
                let next = bit_of.len();
                if next < OpMask::CAPACITY {
                    bit_of.entry(op.clone()).or_insert(next);
                }
            }
        }
    }
    let mask_of = |ops: &[String]| -> Option<OpMask> {
        let mut m = OpMask::EMPTY;
        for op in ops {
            m.set(*bit_of.get(op.as_str())?);
        }
        Some(m)
    };
    // (candidates, masks) per group. A team can only fail to mask if the universe
    // overflowed `OpMask::CAPACITY`, which the sizing above makes unreachable in
    // practice; dropping it is still safer than mis-scoring a collision.
    let masked: Vec<Vec<(CandidateTeam, OpMask)>> = per_group
        .into_iter()
        .map(|teams| {
            teams
                .into_iter()
                .filter_map(|t| mask_of(&t.ops).map(|m| (t, m)))
                .collect()
        })
        .collect();

    // Team slots, groups interleaved (g0t0, g1t0, g2t0, g0t1, …) so no group
    // exhausts the shared generics before another gets a turn. Slot weight = how
    // many tiling cells that ordinal works (2, or the trailing 1).
    let weights: Vec<Vec<usize>> = specs
        .iter()
        .map(|s| ordinal_weights(s.rooms.len()))
        .collect();
    let max_teams = weights.iter().map(Vec::len).max().unwrap_or(0);
    let mut slots: Vec<(usize, usize)> = Vec::new(); // (group, ordinal)
    for t in 0..max_teams {
        for (g, w) in weights.iter().enumerate() {
            if t < w.len() {
                slots.push((g, t));
            }
        }
    }

    // Beam search over slots. State: (used ops mask, weighted total, pick per slot).
    #[derive(Clone)]
    struct State {
        mask: OpMask,
        total: f64,
        picks: Vec<Option<usize>>,
    }
    let mut beam = vec![State {
        mask: OpMask::EMPTY,
        total: 0.0,
        picks: vec![None; slots.len()],
    }];
    for (si, &(g, ordinal)) in slots.iter().enumerate() {
        let weight = weights[g][ordinal] as f64;
        let mut next: HashMap<OpMask, State> = HashMap::new();
        let mut push = |st: State| {
            next.entry(st.mask)
                .and_modify(|cur| {
                    if st.total > cur.total {
                        *cur = st.clone();
                    }
                })
                .or_insert(st);
        };
        for st in &beam {
            // An empty slot keeps small rosters feasible: the room rests dark on that block.
            push(st.clone());
            let mut taken = 0usize;
            for (ci, (cand, mask)) in masked[g].iter().enumerate() {
                if mask.intersects(&st.mask) {
                    continue;
                }
                let mut new = st.clone();
                new.mask = new.mask.union(mask);
                new.total += cand.score * weight;
                new.picks[si] = Some(ci);
                push(new);
                taken += 1;
                if taken >= EXTENSIONS_PER_SLOT {
                    break;
                }
            }
        }
        let mut states: Vec<State> = next.into_values().collect();
        states.sort_by(|a, b| {
            b.total
                .partial_cmp(&a.total)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.mask.cmp(&b.mask))
        });
        states.truncate(BEAM_WIDTH);
        beam = states;
    }
    let mut best = beam.into_iter().next().unwrap_or(State {
        mask: OpMask::EMPTY,
        total: 0.0,
        picks: Vec::new(),
    });

    // Local-search polish: (a) replace any slot's team with a better disjoint
    // candidate from its group; (b) swap two slots' teams across groups when each
    // team also exists as a candidate in the other group (a generic gold team may
    // be worth more as an EXP team - the cross-formula fix at team granularity).
    let ops_key = |ops: &[String]| -> Vec<String> {
        let mut k = ops.to_vec();
        k.sort();
        k
    };
    let by_ops: Vec<HashMap<Vec<String>, usize>> = masked
        .iter()
        .map(|teams| {
            let mut m = HashMap::new();
            for (ci, (cand, _)) in teams.iter().enumerate() {
                m.entry(ops_key(&cand.ops)).or_insert(ci);
            }
            m
        })
        .collect();
    for _ in 0..POLISH_PASSES {
        let mut improved = false;
        // (a) single-slot replacement (also fills empty slots freed by better packing).
        for (si, &(g, ordinal)) in slots.iter().enumerate() {
            let weight = weights[g][ordinal] as f64;
            let cur_pick = best.picks[si];
            let cur_mask = cur_pick.map_or(OpMask::EMPTY, |ci| masked[g][ci].1);
            let cur_score = cur_pick.map_or(0.0, |ci| masked[g][ci].0.score);
            let rest = best.mask.difference(&cur_mask);
            for (ci, (cand, mask)) in masked[g].iter().enumerate() {
                if mask.intersects(&rest) {
                    continue;
                }
                if cand.score > cur_score + 1e-9 {
                    best.mask = rest.union(mask);
                    best.total += (cand.score - cur_score) * weight;
                    best.picks[si] = Some(ci);
                    improved = true;
                    break;
                }
            }
        }
        // (b) cross-group swap of two picked teams.
        for i in 0..slots.len() {
            for j in (i + 1)..slots.len() {
                let (gi, oi) = slots[i];
                let (gj, oj) = slots[j];
                if gi == gj {
                    continue;
                }
                let (Some(pi), Some(pj)) = (best.picks[i], best.picks[j]) else {
                    continue;
                };
                let (wi, wj) = (weights[gi][oi] as f64, weights[gj][oj] as f64);
                let (ti, tj) = (&masked[gi][pi].0, &masked[gj][pj].0);
                let (Some(&i_in_j), Some(&j_in_i)) = (
                    by_ops[gj].get(&ops_key(&ti.ops)),
                    by_ops[gi].get(&ops_key(&tj.ops)),
                ) else {
                    continue;
                };
                let new_total = best.total - ti.score * wi - tj.score * wj
                    + masked[gi][j_in_i].0.score * wi
                    + masked[gj][i_in_j].0.score * wj;
                if new_total > best.total + 1e-9 {
                    best.total = new_total;
                    best.picks[i] = Some(j_in_i);
                    best.picks[j] = Some(i_in_j);
                    improved = true;
                }
            }
        }
        if !improved {
            break;
        }
    }

    // Materialize: per group, selected teams sorted strongest-first onto the
    // ordinals sorted widest-block-first, so any trailing 1-cell block gets the
    // weakest team. Missing picks leave an empty team, which `backfill_empty_teams`
    // then fills - an unstaffed production room is never the right answer.
    let materialize = |picks: &[Option<usize>]| -> Vec<PlannedGroup> {
        specs
            .iter()
            .enumerate()
            .map(|(g, spec)| {
                let n = spec.rooms.len();
                let teams_needed = teams_for_rooms(n);
                let mut selected: Vec<CandidateTeam> = Vec::new();
                for (si, &(sg, _)) in slots.iter().enumerate() {
                    if sg == g
                        && let Some(ci) = picks[si]
                    {
                        selected.push(masked[g][ci].0.clone());
                    }
                }
                selected.sort_by(|a, b| {
                    b.score
                        .partial_cmp(&a.score)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                // Ordinals by descending cell count, strongest team first.
                let w = ordinal_weights(n);
                let mut ordinals: Vec<usize> = (0..teams_needed).collect();
                ordinals.sort_by_key(|&o| std::cmp::Reverse(w[o]));
                let mut teams: Vec<CandidateTeam> = (0..teams_needed)
                    .map(|_| CandidateTeam {
                        ops: Vec::new(),
                        speed: 0.0,
                        value: 0.0,
                        gold: 0.0,
                        order_limit: None,
                        score: 0.0,
                    })
                    .collect();
                for (rank, &o) in ordinals.iter().enumerate() {
                    if let Some(t) = selected.get(rank) {
                        teams[o] = t.clone();
                    }
                }
                PlannedGroup {
                    room_type: spec.room_type.clone(),
                    formula_type: spec.formula_type.clone(),
                    rooms: spec.rooms.clone(),
                    teams,
                    cells: tile_group(n),
                }
            })
            .collect()
    };

    // Coupled polish for the TRADING slots: the beam ranks a post's teams by
    // raw score (speed x order value), which cannot see that a gold-starved
    // base sells no more than its factories make, nor which post level a
    // value shape pays at. Each trading slot tries every disjoint candidate
    // and keeps the one that raises the tiled objective (the peak search's
    // coupled yield, per shift) - the rotation's twin of the peak search's
    // yield-picked posts. Texas/Lappland at 122 beat the Shamare squad at 97
    // on score while the squad realized more (00980819, 2026-09-21).
    let trading_slots: Vec<usize> = slots
        .iter()
        .enumerate()
        .filter(|(_, (g, _))| specs[*g].room_type == "TRADING")
        .map(|(si, _)| si)
        .collect();
    // Within `POST_TIE_BAND` of the coupled value the posts' own (uncoupled)
    // LMD breaks the tie, as the peak search's `best_post_combination`
    // does: on a starved base the coupled value cannot tell Proviso at the
    // level-2 post from a plain speed pair there, and her value shape pays
    // most at the lower post. A move inside the band must raise the tie-
    // breaker, a move outside it must clear the band: monotone both ways,
    // so the polish cannot cycle.
    if !trading_slots.is_empty() {
        let measure = |picks: &[Option<usize>]| {
            let groups = materialize(picks);
            (
                tiled_objective(&groups, global_bonuses),
                trading_uncoupled_lmd(&groups, global_bonuses),
            )
        };
        // Pair moves draw from each slot's best teams by raw score AND by
        // the post's own LMD at its level: at a level-3 post every top
        // score is a Proviso team, while the Shamare squad (lower score,
        // more LMD per order) is what the coupled objective wants there.
        let pair_pool: Vec<Vec<usize>> = specs
            .iter()
            .enumerate()
            .map(|(g, spec)| {
                if spec.room_type != "TRADING" {
                    return Vec::new();
                }
                let level = spec.rooms.iter().map(|(_, l)| *l).min().unwrap_or(1);
                let global = *global_bonuses.get("TRADING").unwrap_or(&0.0);
                let mut by_yield: Vec<usize> = (0..masked[g].len()).collect();
                by_yield.sort_by(|&a, &b| {
                    let lmd = |ci: usize| {
                        let t = &masked[g][ci].0;
                        super::yield_model::room_yield(
                            "TRADING",
                            None,
                            level,
                            t.speed + global,
                            t.value,
                            t.ops.len(),
                            t.order_limit,
                        )
                        .lmd_per_day
                    };
                    lmd(b)
                        .partial_cmp(&lmd(a))
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                let mut pool: Vec<usize> = (0..EXTENSIONS_PER_SLOT.min(masked[g].len())).collect();
                for ci in by_yield.into_iter().take(EXTENSIONS_PER_SLOT) {
                    if !pool.contains(&ci) {
                        pool.push(ci);
                    }
                }
                pool
            })
            .collect();
        let (mut current, mut current_tie) = measure(&best.picks);
        for _ in 0..POLISH_PASSES {
            let mut improved = false;
            for &si in &trading_slots {
                let g = slots[si].0;
                let cur_mask = best.picks[si].map_or(OpMask::EMPTY, |ci| masked[g][ci].1);
                let rest = best.mask.difference(&cur_mask);
                let mut best_alt: Option<(usize, f64, f64)> = None;
                for (ci, (_, mask)) in masked[g].iter().enumerate() {
                    if Some(ci) == best.picks[si] || mask.intersects(&rest) {
                        continue;
                    }
                    let mut picks = best.picks.clone();
                    picks[si] = Some(ci);
                    let (value, tie) = measure(&picks);
                    let band = current.abs() * POST_TIE_BAND;
                    let better = value > current + band
                        || (value >= current - band && tie > current_tie + 1e-6);
                    let beats_alt = best_alt.is_none_or(|(_, v, t)| {
                        let b = v.abs() * POST_TIE_BAND;
                        value > v + b || (value >= v - b && tie > t + 1e-6)
                    });
                    if better && beats_alt {
                        best_alt = Some((ci, value, tie));
                    }
                }
                if let Some((ci, value, tie)) = best_alt {
                    best.mask = rest.union(&masked[g][ci].1);
                    best.picks[si] = Some(ci);
                    current = value;
                    current_tie = tie;
                    improved = true;
                }
            }
            // Pair moves: two trading slots re-picked together over their
            // top candidates, so an operator held by one slot can move to
            // the other (Proviso parked in the level-3 post's single-shift
            // slot could never reach the level-2 post's two-shift slot by
            // single moves - she was "taken" from that slot's view).
            for (pi, &si) in trading_slots.iter().enumerate() {
                for &sj in &trading_slots[pi + 1..] {
                    let (gi, gj) = (slots[si].0, slots[sj].0);
                    let mask_i = best.picks[si].map_or(OpMask::EMPTY, |ci| masked[gi][ci].1);
                    let mask_j = best.picks[sj].map_or(OpMask::EMPTY, |cj| masked[gj][cj].1);
                    let rest = best.mask.difference(&mask_i).difference(&mask_j);
                    let mut best_pair: Option<(usize, usize, f64, f64)> = None;
                    for &ci in &pair_pool[gi] {
                        let mi = &masked[gi][ci].1;
                        if mi.intersects(&rest) {
                            continue;
                        }
                        for &cj in &pair_pool[gj] {
                            let mj = &masked[gj][cj].1;
                            if mj.intersects(&rest)
                                || mj.intersects(mi)
                                || (Some(ci) == best.picks[si] && Some(cj) == best.picks[sj])
                            {
                                continue;
                            }
                            let mut picks = best.picks.clone();
                            picks[si] = Some(ci);
                            picks[sj] = Some(cj);
                            let (value, tie) = measure(&picks);
                            let band = current.abs() * POST_TIE_BAND;
                            let better = value > current + band
                                || (value >= current - band && tie > current_tie + 1e-6);
                            let beats_alt = best_pair.is_none_or(|(_, _, v, t)| {
                                let b = v.abs() * POST_TIE_BAND;
                                value > v + b || (value >= v - b && tie > t + 1e-6)
                            });
                            if better && beats_alt {
                                best_pair = Some((ci, cj, value, tie));
                            }
                        }
                    }
                    if let Some((ci, cj, value, tie)) = best_pair {
                        best.mask = rest.union(&masked[gi][ci].1).union(&masked[gj][cj].1);
                        best.picks[si] = Some(ci);
                        best.picks[sj] = Some(cj);
                        current = value;
                        current_tie = tie;
                        improved = true;
                    }
                }
            }
            if !improved {
                break;
            }
        }
    }

    let mut groups: Vec<PlannedGroup> = materialize(&best.picks);

    backfill_empty_teams(
        &mut groups,
        operators,
        assigned,
        registry,
        building_data,
        facility_counts,
        total_dorm_levels,
        cc_conditions,
        morale_drains,
        kept,
        memo,
    );
    groups
}

/// Seat anyone still free in a team the beam left empty.
///
/// Rotation teams work 24-hour blocks, so their candidate pool excludes every
/// operator whose morale drain outpaces the bar (`require_24h_sustain`). On a
/// wide base that pool runs dry - a roster can own plenty of Trading Post
/// operators and still leave the third team of a two-post group empty, which
/// tiles into a production room standing dark for two of three shifts.
///
/// That trade is backwards. An unstaffed production room generates nothing at
/// all: a certain, total loss for every hour it is dark. A heavy drainer working
/// that block produces at full rate until their bar empties, and morale is
/// recoverable in a dormitory. So when the sustainable pool is exhausted, relax
/// the 24h requirement rather than resting the room - "someone imperfect" beats
/// "nobody" every time.
///
/// This only fires where the beam already failed to fill a block, so bases whose
/// sustainable pool is deep enough are completely unaffected.
#[allow(clippy::too_many_arguments)]
fn backfill_empty_teams(
    groups: &mut [PlannedGroup],
    operators: &[OperatorBaseProfile],
    assigned: &HashSet<String>,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
    facility_counts: &HashMap<String, usize>,
    total_dorm_levels: i32,
    cc_conditions: &[CcCondition],
    morale_drains: &HashMap<String, f64>,
    kept: &HashMap<String, Vec<String>>,
    memo: &EnumerationMemo,
) {
    if !groups
        .iter()
        .any(|g| g.teams.iter().any(|t| t.ops.is_empty()))
    {
        return;
    }
    let op_index = build_op_index(operators);

    // Teams must stay genuinely disjoint: an operator seated by the beam, or
    // already committed elsewhere in the base, is not available here.
    let mut used: HashSet<String> = assigned.clone();
    for g in groups.iter() {
        for t in &g.teams {
            used.extend(t.ops.iter().cloned());
        }
    }

    for g in groups.iter_mut() {
        let capacity = g
            .rooms
            .iter()
            .map(|(_, level)| max_stationed_at_level(building_data, &g.room_type, *level).max(0))
            .min()
            .unwrap_or(0);
        if capacity <= 0 {
            continue;
        }
        let min_level = g.rooms.iter().map(|(_, l)| *l).min().unwrap_or(1);
        // A pinned trading crew is part of the backfilled team too, scored
        // with it: filled without the pin, a post's spare block took a
        // speed pair whose raw score outranked the pinned squad, and the
        // pair then sat under the nullifier while the squad worked one
        // shift (00980819).
        let pinned: Option<&Vec<String>> = (g.room_type == "TRADING")
            .then(|| {
                let sets: Vec<&Vec<String>> = g
                    .rooms
                    .iter()
                    .filter_map(|(slot, _)| kept.get(slot))
                    .filter(|ops| !ops.is_empty())
                    .collect();
                (sets.len() == g.rooms.len() && sets.iter().all(|s| *s == sets[0])).then(|| sets[0])
            })
            .flatten();

        for ordinal in 0..g.teams.len() {
            if !g.teams[ordinal].ops.is_empty() {
                continue;
            }
            let pool_used: HashSet<String> = pinned.map_or_else(
                || used.clone(),
                |p| used.iter().filter(|id| !p.contains(id)).cloned().collect(),
            );
            let key = EnumerationMemo::key(
                &g.room_type,
                min_level,
                g.formula_type.as_deref(),
                operators,
                &pool_used,
                registry,
                building_data,
                morale_drains,
                capacity,
                BASE_POOL,
                g.room_type == "MANUFACTURE",
                false,
                cc_conditions,
            );
            let relaxed = memo.get_or_enumerate(key, || {
                enumerate_candidate_teams(
                    &g.room_type,
                    min_level,
                    g.formula_type.as_deref(),
                    operators,
                    &pool_used,
                    registry,
                    building_data,
                    facility_counts,
                    total_dorm_levels,
                    capacity,
                    cc_conditions,
                    morale_drains,
                    false,
                    BASE_POOL,
                    g.room_type == "MANUFACTURE",
                    // The whole point of the pass: consider the operators the strict
                    // rotation pool threw away.
                    false,
                )
            });
            let Some(team) = relaxed.into_iter().find(|t| !t.ops.is_empty()) else {
                // Genuinely nobody left with an applicable skill - resting the
                // room really is all that's on offer.
                break;
            };
            let team = match pinned {
                Some(p) if !p.iter().all(|id| team.ops.contains(id)) => {
                    #[allow(clippy::cast_sign_loss)]
                    let free = (capacity as usize).saturating_sub(p.len());
                    let mut ops = p.clone();
                    ops.extend(
                        team.ops
                            .iter()
                            .filter(|o| !p.contains(o))
                            .take(free)
                            .cloned(),
                    );
                    let totals = super::assignment::compute_team_totals(
                        &ops,
                        &g.room_type,
                        None,
                        Some(min_level),
                        &op_index,
                        registry,
                        building_data,
                        facility_counts,
                        total_dorm_levels,
                        morale_drains,
                        cc_conditions,
                    );
                    CandidateTeam {
                        ops,
                        speed: totals.speed_pct,
                        value: totals.order_value_pct,
                        gold: totals.order_gold_pct,
                        order_limit: totals.order_limit,
                        score: room_search_score(
                            &g.room_type,
                            totals.speed_pct,
                            totals.order_value_pct,
                        ),
                    }
                }
                _ => team,
            };
            used.extend(team.ops.iter().cloned());
            g.teams[ordinal] = team;
        }
    }
}

/// Team size for a group: bounded by its smallest room so a team fits any room its
/// 24h block spans.
fn group_capacity(spec: &GroupSpec, building_data: &BuildingDataFile) -> usize {
    spec.rooms
        .iter()
        .map(|(_, level)| {
            max_stationed_at_level(building_data, &spec.room_type, *level).max(0) as usize
        })
        .min()
        .unwrap_or(0)
}

/// Top every team up to its group's capacity from the shared leftover pool, weakest
/// team first (fillers with small relevant buffs land where they help most). Mirrors
/// the peak `pad_production_rooms` rules: never pad an automation/nullifier operator
/// into a normal team, never accept a filler that lowers the team's output, and pad
/// automation teams with the lowest-opportunity-cost benchwarmers.
#[allow(clippy::too_many_arguments)]
fn pad_teams(
    groups: &mut [PlannedGroup],
    operators: &[OperatorBaseProfile],
    assigned: &HashSet<String>,
    registry: &HashMap<String, BuffResolutionStrategy>,
    building_data: &BuildingDataFile,
    facility_counts: &HashMap<String, usize>,
    total_dorm_levels: i32,
    cc_conditions: &[CcCondition],
    morale_drains: &HashMap<String, f64>,
    reserved: &HashMap<String, usize>,
) {
    let op_index = build_op_index(operators);
    let mut used: HashSet<String> = assigned.clone();
    for g in groups.iter() {
        for t in &g.teams {
            used.extend(t.ops.iter().cloned());
        }
    }

    let mut order: Vec<(usize, usize)> = groups
        .iter()
        .enumerate()
        .flat_map(|(gi, g)| (0..g.teams.len()).map(move |ti| (gi, ti)))
        .collect();
    order.sort_by(|a, b| {
        let sa = groups[a.0].teams[a.1].score;
        let sb = groups[b.0].teams[b.1].score;
        sa.partial_cmp(&sb).unwrap_or(std::cmp::Ordering::Equal)
    });

    for (gi, ti) in order {
        let (room_type, formula, capacity) = {
            let g = &groups[gi];
            // A team's usable size is bounded by the LARGEST room its tiling cells
            // touch, net of any slot a pinned 24/7 operator permanently occupies there.
            // A team spanning both a pinned and a free room keeps its full size (the
            // scored merge benches the least-useful member in the pinned cell); a team
            // that only ever works the pinned room shrinks - padding it further would
            // bench the filler invisibly every shift.
            let capacity = g
                .rooms
                .iter()
                .enumerate()
                .filter(|(ri, _)| g.cells[*ri].contains(&ti))
                .map(|(_, (slot, level))| {
                    let cap =
                        max_stationed_at_level(building_data, &g.room_type, *level).max(0) as usize;
                    cap.saturating_sub(reserved.get(slot).copied().unwrap_or(0))
                })
                .max()
                .unwrap_or(0);
            (g.room_type.clone(), g.formula_type.clone(), capacity)
        };
        // An empty team stays empty - the block rests dark rather than burning
        // benchwarmers' morale on a zero-value cell.
        if groups[gi].teams[ti].ops.is_empty() {
            continue;
        }
        let is_auto = groups[gi].teams[ti]
            .ops
            .iter()
            .filter_map(|id| op_index.get(id.as_str()).copied())
            .any(|op| has_automation_buff(op, registry));
        let team_nullifies = groups[gi].teams[ti]
            .ops
            .iter()
            .filter_map(|id| op_index.get(id.as_str()).copied())
            .any(|op| op_is_nullifier(op, &room_type, formula.as_deref(), registry, building_data));

        while groups[gi].teams[ti].ops.len() < capacity {
            let team = &groups[gi].teams[ti];
            let current_score = room_search_score(&room_type, team.speed, team.value);
            let mut best_pick: Option<(String, f64, f64, f64, usize)> = None;
            for op in operators {
                if used.contains(&op.char_id) {
                    continue;
                }
                if is_auto {
                    // Extra members are nullified - take the lowest-opportunity-cost
                    // benchwarmer (no scoring needed; a same-room specialist whose
                    // contribution automation zeroes is penalized as wasted).
                    let cost = padding_cost(
                        op,
                        &room_type,
                        formula.as_deref(),
                        registry,
                        building_data,
                        false,
                        true,
                    );
                    if best_pick.as_ref().is_none_or(|(_, _, _, _, c)| cost < *c) {
                        let t = &groups[gi].teams[ti];
                        best_pick =
                            Some((op.char_id.clone(), t.speed, t.value, current_score, cost));
                    }
                    continue;
                }
                if has_automation_buff(op, registry)
                    || op_is_nullifier(op, &room_type, formula.as_deref(), registry, building_data)
                    // Padding seats the op into a 24h-block team - a heavy-drainer
                    // would run dry mid-block.
                    || !sustains_24h_block(op, morale_drains)
                {
                    continue;
                }
                let mut trial = team.ops.clone();
                trial.push(op.char_id.clone());
                let (speed, value) = compute_team_efficiency(
                    &trial,
                    &room_type,
                    formula.as_deref(),
                    None,
                    &op_index,
                    registry,
                    building_data,
                    facility_counts,
                    total_dorm_levels,
                    morale_drains,
                    cc_conditions,
                );
                let score = room_search_score(&room_type, speed, value);
                let cost = padding_cost(
                    op,
                    &room_type,
                    formula.as_deref(),
                    registry,
                    building_data,
                    team_nullifies,
                    false,
                );
                let better = match &best_pick {
                    None => true,
                    Some((_, _, _, s, c)) => {
                        score > *s + 1e-9 || ((score - *s).abs() <= 1e-9 && cost < *c)
                    }
                };
                if better {
                    best_pick = Some((op.char_id.clone(), speed, value, score, cost));
                }
            }
            match best_pick {
                Some((id, speed, value, score, _)) if is_auto || score >= current_score - 1e-9 => {
                    used.insert(id.clone());
                    let team = &mut groups[gi].teams[ti];
                    team.ops.push(id);
                    if !is_auto {
                        team.speed = speed;
                        team.value = value;
                        team.score = score;
                    }
                }
                _ => break,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{SHIFT_COUNT, teams_for_rooms, tile_group};

    #[test]
    fn tiling_invariants_hold_for_small_groups() {
        for n_rooms in 1..=4 {
            let cells = tile_group(n_rooms);
            let teams = teams_for_rooms(n_rooms);
            // Every cell is covered by a valid team index.
            let mut count = vec![0usize; teams];
            for room in &cells {
                for &t in room {
                    assert!(t < teams, "cell team index in range (n={n_rooms})");
                    count[t] += 1;
                }
            }
            // Every team works 1-2 cells; only the trailing team of an odd chain works 1.
            for (t, &c) in count.iter().enumerate() {
                let expect = if (n_rooms * SHIFT_COUNT) % 2 == 1 && t == teams - 1 {
                    1
                } else {
                    2
                };
                assert_eq!(c, expect, "team {t} cell count (n={n_rooms})");
            }
            // A team's cells are consecutive in the room-major chain, so its work
            // block is 24h of consecutive shifts (possibly wrapping via the chain).
            for t in 0..teams {
                let positions: Vec<usize> = (0..n_rooms * SHIFT_COUNT)
                    .filter(|p| cells[p / SHIFT_COUNT][p % SHIFT_COUNT] == t)
                    .collect();
                if positions.len() == 2 {
                    assert_eq!(positions[1] - positions[0], 1, "team {t} block consecutive");
                }
            }
        }
    }

    #[test]
    fn two_room_tiling_matches_the_login_rhythm() {
        // room0 = [T0, T0, T1], room1 = [T1, T2, T2]: each team works 24h then rests
        // 12h, and exactly one of the group's rooms changes team at each boundary.
        let cells = tile_group(2);
        assert_eq!(cells, vec![[0, 0, 1], [1, 2, 2]]);
        for s in 1..SHIFT_COUNT {
            let changes = cells.iter().filter(|room| room[s] != room[s - 1]).count();
            assert_eq!(changes, 1, "one room swaps its team at boundary {s}");
        }
    }
}
