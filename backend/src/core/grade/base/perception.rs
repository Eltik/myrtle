//! Base-wide resource economies (the Rosmontis / Ebenholz "人间烟火" system and any
//! future system shaped like it).
//!
//! A handful of operators turn the base's *resting* roster into productivity through a
//! shared resource pool:
//!   - Generators (Rosmontis, Ebenholz, Virtuosa, ...) add to the pool, mostly
//!     "+1 per operator resting in the dormitories" (so they scale with dorm capacity).
//!   - Converters move points along a chain (Perception Information -> Chain of Thought
//!     for factories, -> Soundless Resonance for trading posts, ...).
//!   - Consumers (Rosmontis, Ebenholz, Jieyun, ...) gain "+X% per N points" of a resource.
//!
//! The pool is base-wide (fed by operators resting across the dormitories, not by a
//! room's own team), so it can't be a per-room buff - it's modeled here and applied to
//! the consumers wherever they're stationed. Pools sum across every deployed generator
//! (the in-game counters accumulate, which the community's "(20+20+20+...)/2" math
//! confirms).
//!
//! SCALABILITY: parsing is anchored to the game's own resource tags. Every resource
//! counter is written `<$cc.bd_XXX><@cc.rem>Display Name</>` in the buff text, and every
//! number lives in a `<@cc.vup>`/`<@cc.kw>` tag, so we read those structural markers
//! rather than guessing from prose or a hardcoded vocabulary. A newly released operator
//! that generates/converts/consumes a `bd_` resource is picked up automatically, and a
//! brand-new resource id needs no code change. Resources with no generator (Monster Meal,
//! Felvine, Ursus Beverage - the item-stockpile consumables) simply have an empty pool,
//! so their consumers correctly contribute 0 without any special-casing.
//!
//! Scope: this is a deliberate LOWER BOUND - the reliably-derivable contribution is the
//! "per resting operator" generation (dorm capacity is known); the deeper cascade
//! (Dreamland / Measure / Memory Fragment intermediates) has no upstream source in the
//! static building data, so those converters contribute 0 here rather than a guess.
//! Callers gate this to 243 layouts; 252 is intentionally left unmodeled for now.

use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

use regex::Regex;

use crate::core::gamedata::types::building::BuildingDataFile;

use super::assignment::{best_ordinary_cc_fill, morale_recovery, op_uptime};
use super::buff_registry::BuffResolutionStrategy;
use super::types::{OperatorBaseProfile, UserBuilding};
use super::util::max_stationed_at_level;

/// A `<$cc.bd_XXX><@cc.rem>Name</>` resource reference: its stable id and display name.
static RE_RESOURCE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<\$cc\.(bd_[A-Za-z0-9]+)><@cc\.rem>([^<]+)</>").unwrap());
/// A numeric value tag: `<@cc.vup>+5%</>`, `<@cc.kw>1</>`, etc.
static RE_NUMBER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<@cc\.(?:vup|kw)>\+?(\d+)(%?)</>").unwrap());
/// A morale gate on a conditional generator: "... morale is above/below ...". Matched
/// against the lowercased description, so byte offsets line up with the original (ASCII).
static RE_MORALE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"morale is (above|below)").unwrap());

/// Total operators that can rest at once - the count the per-resting generators key on
/// (every dormitory kept full between rotations, which is the normal steady state).
pub fn resting_capacity(building: &UserBuilding, building_data: &BuildingDataFile) -> i32 {
    building
        .rooms
        .iter()
        .filter(|r| r.room_type == "DORMITORY")
        .map(|r| max_stationed_at_level(building_data, "DORMITORY", r.level))
        .sum()
}

/// What a generator's "for every X" scales on. Each maps to a count in `BaseContext`,
/// so the same parser handles dorm-fed, HR-fed and CC-fed generators without naming any
/// operator.
#[derive(Clone, Copy)]
enum Unit {
    /// "for every operator in the Dormitories" - every resting operator base-wide.
    Resting,
    /// "for every operator in that Dormitory" - one dormitory's occupants.
    SingleDorm,
    /// "for every Recruit slot" - non-default HR recruitment slots.
    RecruitSlot,
    /// "for each Sui Operator …" - deployed Sui-faction operators.
    Sui,
    /// A FLAT amount produced just by being stationed ("Passion +20") - no per-unit scaling.
    Flat,
}

/// One operator adds `per_unit` points of `resource` for each `unit` counted (optionally
/// capped at `cap` total) when stationed in `room_type` - the room it must occupy to do so.
struct Generator {
    resource: String,
    per_unit: f64,
    unit: Unit,
    cap: Option<f64>,
    room_type: String,
}

impl Generator {
    /// Total points this generator adds in base `ctx`.
    fn contribution(&self, ctx: BaseContext) -> f64 {
        let raw = self.per_unit * ctx.count(self.unit);
        self.cap.map_or(raw, |c| raw.min(c))
    }
}

/// A morale-gated generation branch (Ling/Dusk): a flat `amount` of `resource` produced
/// when the operator's morale is on the `above`/`below`-threshold side. An operator's morale
/// sits on ONE side at a time, so only the branches on the better-paying side fire - the
/// solver groups them per operator and picks that side.
struct Conditional {
    resource: String,
    amount: f64,
    above: bool,
    room_type: String,
}

/// Base-wide counts the generators scale on. Derived from the building where possible;
/// the HR/Sui counts are best-case assumptions for the ceiling estimate (documented in
/// `docs/base-resource-economy.md`).
#[derive(Clone, Copy)]
struct BaseContext {
    resting: i32,
    single_dorm: i32,
    recruit_slots: i32,
    sui_count: i32,
}

impl BaseContext {
    fn count(&self, unit: Unit) -> f64 {
        f64::from(match unit {
            Unit::Resting => self.resting,
            Unit::SingleDorm => self.single_dorm,
            Unit::RecruitSlot => self.recruit_slots,
            Unit::Sui => self.sui_count,
            Unit::Flat => 1,
        })
    }
}

/// Build the base-wide counts for a building. HR recruit slots and Sui count aren't in the
/// building data, so they use best-case constants (the system is a max-ceiling build).
fn base_context(building: &UserBuilding, building_data: &BuildingDataFile) -> BaseContext {
    BaseContext {
        resting: resting_capacity(building, building_data),
        single_dorm: max_stationed_at_level(building_data, "DORMITORY", 5).max(1),
        recruit_slots: 4,
        sui_count: 5,
    }
}

/// One operator turns `from` points into `to` points at `ratio` (non-consuming: every
/// consumer still reads the full upstream pool, matching the observed in-game totals).
struct Converter {
    from: String,
    to: String,
    ratio: f64,
}

/// One operator gains `pct_per_point`% productivity per point of `resource`, in `room`.
/// `global` is true for a Control-Center operator who boosts a whole production ROOM TYPE
/// (Sakiko: "all Factories' Precious-Metal productivity +1% per 20 Passion") rather than its
/// own room - then `room_type` is the production room it targets, not the Control Center.
struct Consumer {
    resource: String,
    pct_per_point: f64,
    room_type: String,
    global: bool,
}

/// A resource reference inside a description: its `bd_` id and byte span.
struct Ref<'a> {
    id: &'a str,
    start: usize,
    end: usize,
}

fn resource_refs(desc: &str) -> Vec<Ref<'_>> {
    RE_RESOURCE
        .captures_iter(desc)
        .map(|c| {
            let m = c.get(0).unwrap();
            Ref {
                id: c.get(1).unwrap().as_str(),
                start: m.start(),
                end: m.end(),
            }
        })
        .collect()
}

/// Nearest numeric value strictly before `pos` (e.g. the "for every N" gating a resource).
fn number_before(desc: &str, pos: usize) -> Option<f64> {
    RE_NUMBER
        .captures_iter(&desc[..pos.min(desc.len())])
        .last()
        .and_then(|c| c[1].parse().ok())
}

/// Nearest numeric value at/after `pos`; `want_pct` filters to (non-)percentage tags.
fn number_after(desc: &str, pos: usize, want_pct: bool) -> Option<f64> {
    RE_NUMBER
        .captures_iter(&desc[pos.min(desc.len())..])
        .find(|c| (c.get(2).is_some_and(|m| !m.as_str().is_empty())) == want_pct)
        .and_then(|c| c[1].parse().ok())
}

/// A GENERATED amount written `<@cc...>+N</>` between `pos` and `window_end` - i.e. a `+`-signed,
/// non-% value. Distinguishes "Passion +20" (generation) from a bare threshold like "when Passion
/// is 40 or higher" (a condition, whose `40` carries no `+` and must not be read as +40 generated).
fn generated_amount_after(desc: &str, pos: usize, window_end: usize) -> Option<f64> {
    static RE_PLUS: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"<@cc\.(?:vup|kw)>\+(\d+)</>").unwrap());
    let end = window_end.min(desc.len());
    let start = pos.min(end);
    RE_PLUS
        .captures_iter(&desc[start..end])
        .next()
        .and_then(|c| c[1].parse().ok())
}

type Classified = (
    Vec<Generator>,
    Vec<Converter>,
    Vec<Consumer>,
    Vec<Conditional>,
);

fn classify(desc: &str, room_type: &str) -> Classified {
    let (mut gens, mut convs, mut cons, mut conds) =
        (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    let refs = resource_refs(desc);
    let lower = desc.to_lowercase();

    // Morale-conditional generator (Ling/Dusk): each "... morale is above/below T, <RES> +N"
    // clause is one branch. The first resource after the morale gate, with a non-% "+N", is
    // the produced one. A non-conditional generator falls through to the scope parser below.
    let conditional = RE_MORALE.is_match(&lower);
    if conditional {
        for m in RE_MORALE.find_iter(&lower) {
            let above = &lower[m.start()..m.end()] == "morale is above";
            if let Some(r) = refs.iter().find(|r| r.start >= m.end())
                && let Some(amount) = number_after(desc, r.end, false)
            {
                conds.push(Conditional {
                    resource: r.id.to_string(),
                    amount,
                    above,
                    room_type: room_type.to_string(),
                });
            }
        }
    } else {
        // Detect the scope phrase and where it sits. Order matters: check the specific
        // "that Dormitory" before the general "the Dormitories".
        let scope = if let Some(p) = lower.find("recruit slot") {
            Some((Unit::RecruitSlot, p))
        } else if let Some(p) = lower.find("sui operator") {
            Some((Unit::Sui, p))
        } else if let Some(p) = lower.find("that dormitor") {
            Some((Unit::SingleDorm, p.saturating_sub(20)))
        } else if lower.contains("ormitor") {
            lower
                .find("perator")
                .map(|p| (Unit::Resting, p.saturating_sub(20)))
        } else {
            None
        };
        if let Some((unit, scope_pos)) = scope {
            // "for every K <scope>" - K (default 1) divides the per-unit amount.
            let k = number_before(desc, scope_pos + 12)
                .filter(|n| *n > 0.0)
                .unwrap_or(1.0);
            if let Some(r) = refs.iter().find(|r| r.start >= scope_pos)
                && let Some(amount) = number_after(desc, r.end, false)
            {
                // Optional "(max N)" cap on the contribution.
                let cap = lower.find("max").and_then(|m| number_after(desc, m, false));
                gens.push(Generator {
                    resource: r.id.to_string(),
                    per_unit: amount / k,
                    unit,
                    cap,
                    room_type: room_type.to_string(),
                });
            }
        } else {
            // No per-unit scope: FLAT generation ("Passion +20"). A resource sits immediately
            // before a non-% "+N", and is NOT gated by a "for every/each/per" clause (that would
            // make it a CONSUMED resource - Mutsumi's "for every 8 Passion" - not a generated one).
            for r in &refs {
                let before = &lower[r.start.saturating_sub(24)..r.start];
                if before.contains("for every")
                    || before.contains("for each")
                    || before.contains("per ")
                {
                    continue;
                }
                // Require a `+`-signed amount: "Passion +20" generates, but "when Passion is 40 or
                // higher" is a threshold whose bare 40 must not be read as +40 generated.
                if let Some(amount) =
                    generated_amount_after(desc, r.end, r.end + 30).filter(|n| *n > 0.0)
                {
                    gens.push(Generator {
                        resource: r.id.to_string(),
                        per_unit: amount,
                        unit: Unit::Flat,
                        cap: None,
                        room_type: room_type.to_string(),
                    });
                }
            }
        }
    }

    // Converter: "every N [points of] <FROM> ... converted [in]to M [points of] <TO>".
    if let Some(cidx) = lower.find("converted") {
        let from = refs.iter().rfind(|r| r.end <= cidx);
        let to = refs.iter().find(|r| r.start >= cidx);
        if let (Some(from), Some(to)) = (from, to) {
            let n = number_before(desc, from.start).unwrap_or(1.0);
            let m = number_before(desc, to.start)
                .filter(|_| true)
                .or_else(|| number_after(desc, cidx, false))
                .unwrap_or(1.0);
            let ratio = if n > 0.0 { m / n } else { 1.0 };
            convs.push(Converter {
                from: from.id.to_string(),
                to: to.id.to_string(),
                ratio,
            });
        }
    }

    // Consumer: "for every N [points of] <RES>, productivity/efficiency +Y%". The
    // resource is whichever one sits next to the productivity payoff. (Match any
    // "efficiency" - "order efficiency", "order acquisition efficiency", etc.)
    if let Some(eidx) = lower
        .find("productivity")
        .or_else(|| lower.find("efficiency"))
    {
        let res = refs
            .iter()
            .min_by_key(|r| r.start.abs_diff(eidx).min(r.end.abs_diff(eidx)));
        // The "+Y%" payoff can sit on either side of "productivity".
        if let Some(res) = res
            && let Some(pct) = number_after(desc, eidx.saturating_sub(48), true)
        {
            let count = number_before(desc, res.start)
                .filter(|n| *n > 0.0)
                .unwrap_or(1.0);
            // A Control-Center consumer boosts a whole production room type globally (Sakiko's
            // factory Precious-Metal productivity per Passion); its effect room is the one it
            // names, and the bonus is granted as a global Control-Center buff, not a per-room one.
            let (effect_room, global) = if room_type == "CONTROL" {
                let target = if lower.contains("trading") {
                    "TRADING"
                } else {
                    "MANUFACTURE"
                };
                (target.to_string(), true)
            } else {
                (room_type.to_string(), false)
            };
            cons.push(Consumer {
                resource: res.id.to_string(),
                pct_per_point: pct / count,
                room_type: effect_room,
                global,
            });
        }
    }

    // Fold a same-skill generate->convert chain so the generator yields the FINAL resource
    // directly. Rosmontis (and Ebenholz) make Perception Information AND convert it - Rosmontis
    // to Chain of Thought, Ebenholz to Soundless Resonance - in ONE skill. Without folding,
    // both would feed a single shared Perception-Information pool and read each other's points
    // (Rosmontis +40%). In game they're independent, each capping at ~dorm capacity (the
    // guide's "20 max"): Rosmontis -> ~+20%, Ebenholz -> ~+10%. The converter is left in
    // `convs` so OTHER sources of the intermediate still flow through it - e.g. Dusk's
    // Perception Information is converted to Rosmontis's Chain of Thought, feeding the A/B side.
    for g in &mut gens {
        if let Some(c) = convs.iter().find(|c| c.from == g.resource) {
            g.per_unit *= c.ratio;
            g.resource = c.to.clone();
        }
    }

    (gens, convs, cons, conds)
}

/// The parsed economy of a roster: who generates / converts / consumes what.
struct Economy {
    /// `(char_id, generator)`.
    generators: Vec<(String, Generator)>,
    converters: Vec<Converter>,
    /// `(char_id, buff_id, consumer)`.
    consumers: Vec<(String, String, Consumer)>,
    /// `(char_id, conditional branch)` - morale-gated generation (Ling/Dusk).
    conditionals: Vec<(String, Conditional)>,
}

fn parse_economy(operators: &[OperatorBaseProfile], building_data: &BuildingDataFile) -> Economy {
    let (mut generators, mut converters, mut consumers, mut conditionals) =
        (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    for op in operators {
        for buff_id in &op.available_buffs {
            let Some(buff) = building_data.buffs.get(buff_id) else {
                continue;
            };
            let (gens, convs, cons, conds) = classify(&buff.description, &buff.room_type);
            generators.extend(gens.into_iter().map(|g| (op.char_id.clone(), g)));
            converters.extend(convs);
            consumers.extend(
                cons.into_iter()
                    .map(|c| (op.char_id.clone(), buff_id.clone(), c)),
            );
            conditionals.extend(conds.into_iter().map(|c| (op.char_id.clone(), c)));
        }
    }
    Economy {
        generators,
        converters,
        consumers,
        conditionals,
    }
}

/// Productivity % a single point of each resource is worth, propagating consumer rates back
/// through converters: a resource is worth its direct consumers PLUS, for each conversion it
/// feeds, the ratio times the downstream resource's effective rate. So Worldly Plight is
/// worth Mr. Nothing + Shu + (1/5 -> Witchcraft Crystal) x Jieyun, and a generator that
/// feeds it is valued by that full chain. This is what lets a support generator earn its
/// support-room seat.
fn effective_rates(
    consumers: &[(String, String, Consumer)],
    converters: &[Converter],
) -> HashMap<String, f64> {
    let mut direct: HashMap<String, f64> = HashMap::new();
    for (_c, _b, cons) in consumers {
        *direct.entry(cons.resource.clone()).or_default() += cons.pct_per_point;
    }
    // Iterate to a fixpoint over the acyclic conversion graph.
    let mut rate = direct.clone();
    for _ in 0..=converters.len() {
        let mut next = direct.clone();
        for c in converters {
            let to = rate.get(&c.to).copied().unwrap_or(0.0);
            *next.entry(c.from.clone()).or_default() += c.ratio * to;
        }
        rate = next;
    }
    rate
}

/// Total stationing capacity for `room_type` across the building (sum over rooms of that
/// type of their level's max stationed).
fn room_capacity(
    building: &UserBuilding,
    building_data: &BuildingDataFile,
    room_type: &str,
) -> i32 {
    building
        .rooms
        .iter()
        .filter(|r| r.room_type == room_type)
        .map(|r| max_stationed_at_level(building_data, room_type, r.level))
        .sum()
}

/// The economy solved for a base: the realized resource pool, the support generators to
/// station, and the parsed consumers.
struct Solved {
    /// `resource -> [(contributor char_id, points)]`. Keeping the contributor (rather than a
    /// flat sum) lets the caller scale each contribution by its generator's uptime: a
    /// consumer's OWN generation is co-present with its consumption (full), while a separate
    /// generator only feeds the pool while it too is working (scaled by that generator's
    /// uptime). The peak is the plain sum; the sustained value applies those factors.
    pool: HashMap<String, Vec<(String, f64)>>,
    deployed: Vec<(String, String)>,
    consumers: Vec<(String, String, Consumer)>,
    /// True when the plan stations a morale-conditional operator (Ling/Dusk): such an
    /// operator must be cycled through its morale bands, which is what a Fiammetta-type
    /// morale-swap manager sustains.
    uses_conditional: bool,
}

/// Per operator: `(above-side branches, below-side branches, room)` - the two morale sides of
/// its conditional generation, of which only the better-paying one fires.
type ConditionalSides = HashMap<String, (Vec<(String, f64)>, Vec<(String, f64)>, String)>;

/// A support operator that could be stationed outside production, with the resource points
/// it would add and the downstream production `value` those points are worth.
struct SupportCandidate {
    value: f64,
    char_id: String,
    room_type: String,
    contributions: Vec<(String, f64)>,
}

/// The Control-Center resource combo decided under the room's seat budget. Closes the three gaps
/// of the old per-operator model: over-subscription (generators and consumers contended for the
/// same seats independently), no supply<->demand reconciliation (the credited pool summed every
/// generator whether or not it could be seated alongside the consumers), and no atomic commit
/// (generators have ~0 standalone value, so the ordinary fill never chose them on merit).
struct CcBundle {
    /// Resources owned by the Control-Center global-consumer economy. A CONTROL generator feeding
    /// one of these is managed here, never by the generic support packing.
    global_resources: HashSet<String>,
    /// Pure-generator `char_ids` to PIN into the Control Center (committed supply). Empty when the
    /// combo isn't worth committing.
    generator_pins: Vec<String>,
    /// Realized pool contributions (`resource -> [(contributor, points)]`) from the seated combo:
    /// the global consumers' own co-present generation plus the pinned generators. Empty when not
    /// committed (the consumers then fall back to their base Control-Center value).
    pool: HashMap<String, Vec<(String, f64)>>,
    /// Control-Center seats the committed combo occupies (consumers + pinned generators), so the
    /// generic support packing doesn't oversubscribe the room.
    cc_seats: usize,
}

/// Jointly value a Control-Center resource combo (Passion: generators + global consumers, all
/// competing for the same Control-Center seats) as ONE atomic commitment, reconciling generator
/// supply with consumer demand under the room's seat budget.
///
/// The global consumers are the demand and MUST be seated (their `GlobalEffect` is the payoff); the
/// pure generators are optional supply. Because the Control Center holds at most ~5 operators, an
/// exhaustive subset search over the generators is exact and cheap. For each feasible seating we
/// build the realized pool from only the seated occupants (each consumer's own co-present
/// generation + the chosen generators), value the consumers' resulting whole-base production bonus
/// in LMD-equivalent, and add the value of ordinary global buffs on the leftover seats. The combo
/// commits only when that total beats the best ordinary fill of the whole room - otherwise the
/// Control Center is left to the ordinary bonus-greedy unchanged.
///
/// A dual-role operator (generates AND consumes, e.g. Monster of Acting) is seated once as a
/// consumer with its generation counted co-present, so it never takes two seats. The consumers are
/// NOT pinned - they're seated by the optimizer's bonus-greedy via their realized `GlobalEffect`
/// (the combo only commits when that out-ranks ordinary buffs, so they win their seats); only the
/// pure generators, which carry no Control-Center bonus of their own, need pinning.
#[allow(clippy::too_many_arguments)]
fn solve_cc_bundle(
    econ: &Economy,
    ctx: BaseContext,
    control_slots: i32,
    operators: &[OperatorBaseProfile],
    building: &UserBuilding,
    building_data: &BuildingDataFile,
    registry: &HashMap<String, BuffResolutionStrategy>,
) -> CcBundle {
    // Global Control-Center consumers (Sakiko -> factories, Monster of Acting -> trading) are the
    // demand; the resources they read are the combo's pool.
    let global_cons: Vec<&(String, String, Consumer)> =
        econ.consumers.iter().filter(|(_, _, c)| c.global).collect();
    let global_resources: HashSet<String> = global_cons
        .iter()
        .map(|(_, _, c)| c.resource.clone())
        .collect();
    let mut bundle = CcBundle {
        global_resources,
        generator_pins: Vec::new(),
        pool: HashMap::new(),
        cc_seats: 0,
    };
    if global_cons.is_empty() || control_slots <= 0 {
        return bundle;
    }
    let consumer_chars: HashSet<&str> = global_cons.iter().map(|(c, _, _)| c.as_str()).collect();

    // CONTROL generators feeding a combo resource. A generator whose operator is ALSO a global
    // consumer is co-present self-generation (full, no extra seat); the rest are optional supply,
    // one seat each (summed per operator, deduped to one seat).
    let mut self_gen: Vec<(String, String, f64)> = Vec::new();
    let mut pure_gen: HashMap<String, (String, f64)> = HashMap::new();
    for (char_id, g) in &econ.generators {
        if g.room_type != "CONTROL" || !bundle.global_resources.contains(&g.resource) {
            continue;
        }
        let pts = g.contribution(ctx);
        if pts <= 0.0 {
            continue;
        }
        if consumer_chars.contains(char_id.as_str()) {
            self_gen.push((char_id.clone(), g.resource.clone(), pts));
        } else {
            let slot = pure_gen
                .entry(char_id.clone())
                .or_insert_with(|| (g.resource.clone(), 0.0));
            slot.1 += pts;
        }
    }

    let reserved = consumer_chars.len() as i32;
    if reserved > control_slots {
        return bundle; // can't even seat the demand
    }
    let optional_seats = (control_slots - reserved).max(0) as usize;
    // Strongest generators first, so capping a (hypothetically) large pool keeps the best supply.
    let mut pure: Vec<(String, String, f64)> =
        pure_gen.into_iter().map(|(c, (r, p))| (c, r, p)).collect();
    pure.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

    // Combo-eligible operators are excluded from the leftover-seat (filler) ordinary fill so a seat
    // isn't credited twice; the baseline (no combo) excludes nothing.
    let exclude_combo: HashSet<String> = consumer_chars
        .iter()
        .map(|s| (*s).to_string())
        .chain(pure.iter().map(|(c, _, _)| c.clone()))
        .collect();
    let baseline = best_ordinary_cc_fill(
        operators,
        building,
        building_data,
        registry,
        control_slots,
        &HashSet::new(),
    );
    let room_count = |rt: &str| building.rooms.iter().filter(|r| r.room_type == rt).count();
    // Ordinary value of the leftover seats depends only on HOW MANY the combo leaves free, so
    // precompute it per generator-count instead of inside the subset loop.
    let filler_by_used: Vec<f64> = (0..=optional_seats)
        .map(|used| {
            let leftover = control_slots - reserved - used as i32;
            best_ordinary_cc_fill(
                operators,
                building,
                building_data,
                registry,
                leftover,
                &exclude_combo,
            )
        })
        .collect();

    // Realized consumer bonus (LMD-equivalent) for a seating of `subset` pure generators, plus the
    // direct pool contributions (pre-conversion) to merge if this seating wins.
    let value_of =
        |subset: &[&(String, String, f64)]| -> (f64, HashMap<String, Vec<(String, f64)>>) {
            let mut direct: HashMap<String, Vec<(String, f64)>> = HashMap::new();
            for (c, r, p) in &self_gen {
                direct.entry(r.clone()).or_default().push((c.clone(), *p));
            }
            for (c, r, p) in subset {
                direct.entry(r.clone()).or_default().push((c.clone(), *p));
            }
            // Apply conversions onto a pre-conversion snapshot (matches `solve`), for valuation only.
            let mut converted = direct.clone();
            for cv in &econ.converters {
                if let Some(from) = direct.get(&cv.from) {
                    for (contributor, amount) in from {
                        converted
                            .entry(cv.to.clone())
                            .or_default()
                            .push((contributor.clone(), amount * cv.ratio));
                    }
                }
            }
            let mut consumer_value = 0.0;
            for (_c, _b, cons) in &global_cons {
                let points: f64 = converted
                    .get(&cons.resource)
                    .map_or(0.0, |v| v.iter().map(|(_, a)| a).sum());
                let pct = points * cons.pct_per_point;
                consumer_value += super::yield_model::global_bonus_value(
                    &cons.room_type,
                    room_count(&cons.room_type),
                    pct,
                );
            }
            (consumer_value + filler_by_used[subset.len()], direct)
        };

    // Exhaustive subset search (control_slots is tiny, so the generator pool is too). Bound the
    // bitmask defensively in case a future roster fields an unusually large combo. The best seating
    // keeps its value, the generators to pin, and the realized direct pool to merge.
    type Seating = (f64, Vec<String>, HashMap<String, Vec<(String, f64)>>);
    let n = pure.len().min(16);
    let mut best: Option<Seating> = None;
    for mask in 0u32..(1u32 << n) {
        let subset: Vec<&(String, String, f64)> = (0..n)
            .filter(|i| mask & (1 << i) != 0)
            .map(|i| &pure[i])
            .collect();
        if subset.len() > optional_seats {
            continue;
        }
        let (val, direct) = value_of(&subset);
        if best.as_ref().is_none_or(|(b, _, _)| val > *b) {
            let pins = subset.iter().map(|(c, _, _)| c.clone()).collect();
            best = Some((val, pins, direct));
        }
    }

    let Some((best_val, pins, direct)) = best else {
        return bundle;
    };
    if best_val <= baseline {
        return bundle; // ordinary fill wins - leave the Control Center to the bonus-greedy
    }
    bundle.cc_seats = reserved as usize + pins.len();
    bundle.generator_pins = pins;
    bundle.pool = direct;
    bundle
}

/// Solve the economy for a base. Production self-feeding generators (Rosmontis, Ebenholz,
/// Mr. Nothing - generator *and* consumer in their own room) count once, since the main
/// optimizer stations them for their bonus. Support generators (Mulberry in HR,
/// Virtuosa/Iris/Czerny in dorms, Chongyue/Ling/Dusk in the Control Center) are value-ranked
/// and packed into each support room up to its capacity - a generator earns a support seat
/// only when its resource has consumers downstream. A morale-conditional operator (Ling,
/// Dusk) sits on one morale side at a time, so its branches are grouped per operator and only
/// the better-paying side fires. (The Control-Center opportunity cost vs global-buff
/// operators is a known simplification - see `docs/base-resource-economy.md`.)
fn solve(
    operators: &[OperatorBaseProfile],
    building: &UserBuilding,
    building_data: &BuildingDataFile,
    registry: &HashMap<String, BuffResolutionStrategy>,
) -> Solved {
    let ctx = base_context(building, building_data);
    let econ = parse_economy(operators, building_data);
    let rates = effective_rates(&econ.consumers, &econ.converters);
    let consumer_chars: HashSet<&str> = econ.consumers.iter().map(|(c, _, _)| c.as_str()).collect();
    let rate = |res: &str| rates.get(res).copied().unwrap_or(0.0);

    let mut pool: HashMap<String, Vec<(String, f64)>> = HashMap::new();
    let mut deployed: Vec<(String, String)> = Vec::new();
    let mut support: Vec<SupportCandidate> = Vec::new();

    // A Control-Center resource combo (Passion: generators + global consumers, all contesting the
    // same Control-Center seats) is valued JOINTLY under the room's seat budget and committed only
    // when it beats the best ordinary Control-Center fill - see `solve_cc_bundle`. The combo OWNS
    // its resources: a CONTROL generator feeding one is seated (or dropped) by the bundle, never by
    // the generic support packing below, so the credited pool reflects what the room can actually
    // hold rather than every generator at once.
    let control_slots = room_capacity(building, building_data, "CONTROL");
    let bundle = solve_cc_bundle(
        &econ,
        ctx,
        control_slots,
        operators,
        building,
        building_data,
        registry,
    );
    for id in &bundle.generator_pins {
        deployed.push((id.clone(), "CONTROL".to_string()));
    }
    for (res, contribs) in &bundle.pool {
        pool.entry(res.clone())
            .or_default()
            .extend(contribs.iter().cloned());
    }

    for (char_id, g) in econ.generators {
        // CONTROL generators feeding a combo resource belong to the bundle's seat-budgeted solve
        // (committed as a pin above, or intentionally dropped when the combo doesn't pay).
        if g.room_type == "CONTROL" && bundle.global_resources.contains(&g.resource) {
            continue;
        }
        match g.room_type.as_str() {
            // A production generator is realised only if its operator is also a consumer
            // (so the optimizer actually stations it for the bonus). Tag the contribution with
            // its own char_id: it is co-present with its consumption, so it never gets
            // uptime-scaled against itself.
            "MANUFACTURE" | "TRADING" if consumer_chars.contains(char_id.as_str()) => {
                pool.entry(g.resource.clone())
                    .or_default()
                    .push((char_id.clone(), g.contribution(ctx)));
            }
            "DORMITORY" | "HIRE" | "CONTROL" => {
                let amount = g.contribution(ctx);
                support.push(SupportCandidate {
                    value: amount * rate(&g.resource),
                    char_id,
                    room_type: g.room_type.clone(),
                    contributions: vec![(g.resource.clone(), amount)],
                });
            }
            _ => {}
        }
    }

    // Morale-conditional operators (Ling/Dusk): group branches per operator into the two
    // morale sides, then keep only the side that pays more (the operator can't be on both).
    let mut sides: ConditionalSides = HashMap::new();
    for (char_id, c) in econ.conditionals {
        let entry = sides
            .entry(char_id)
            .or_insert_with(|| (Vec::new(), Vec::new(), c.room_type.clone()));
        if c.above { &mut entry.0 } else { &mut entry.1 }.push((c.resource, c.amount));
    }
    let conditional_chars: std::collections::HashSet<String> = sides.keys().cloned().collect();
    for (char_id, (above, below, room)) in sides {
        // TIME-SIMULATION (the sustainable side, not the transient peak). A Fiammetta-type
        // manager resets morale to FULL, so the only state an operator can be HELD in is its
        // high-morale (above-threshold) side; the low side is reached only by draining toward
        // 0, which isn't sustainable. So we do NOT credit the "both operators on the single
        // best resource" peak - that needs one of them at low morale and, per the guide,
        // "only lasts 1 shift". Each operator instead contributes its high-morale branch
        // (Ling -> Worldly Plight, Dusk -> Perception Information): one resource each, 24/7.
        // (Fall back to the low side only if an operator has no high-morale branch at all.)
        let contributions = if above.is_empty() { below } else { above };
        let value = contributions.iter().map(|(r, a)| a * rate(r)).sum();
        support.push(SupportCandidate {
            value,
            char_id,
            room_type: room,
            contributions,
        });
    }

    // Fill each support room with its highest-value candidates up to capacity, skipping any
    // whose contribution has no downstream consumer (value 0). Sorting globally then checking
    // each candidate's room capacity packs every room with its best candidates first.
    support.sort_by(|a, b| {
        b.value
            .partial_cmp(&a.value)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let mut used: HashMap<String, i32> = HashMap::new();
    // The committed Control-Center combo already holds `cc_seats` of the room (its consumers, seated
    // by the optimizer's bonus-greedy, plus the pinned generators), so other support generators
    // (Ling/Dusk/Chongyue) only contend for what's left - never oversubscribing the Control Center.
    if bundle.cc_seats > 0 {
        used.insert("CONTROL".to_string(), bundle.cc_seats as i32);
    }
    for cand in support {
        if cand.value <= 0.0 {
            continue;
        }
        let cap = room_capacity(building, building_data, &cand.room_type);
        let slot = used.entry(cand.room_type.clone()).or_insert(0);
        if *slot >= cap {
            continue;
        }
        *slot += 1;
        for (res, amount) in cand.contributions {
            // A support generator feeds the pool only while it is itself working, so its
            // contribution is tagged with its char_id and gets uptime-scaled downstream.
            pool.entry(res)
                .or_default()
                .push((cand.char_id.clone(), amount));
        }
        deployed.push((cand.char_id, cand.room_type));
    }

    // Resolve conversions (single-pass additive off a pre-conversion snapshot, so converters
    // sharing a hub don't compound). Correct for today's single-step bd chains (PI -> CoT/SR,
    // WP -> WC); deeper chains route through non-bd intermediates that resolve to 0 anyway.
    // Each converted contribution keeps its original contributor's char_id, so a cross-room
    // source feeding a consumer through a converter (e.g. Dusk's Perception Information ->
    // Rosmontis's Chain of Thought) still scales by that source's uptime.
    let direct = pool.clone();
    for c in &econ.converters {
        if let Some(from) = direct.get(&c.from) {
            for (contributor, amount) in from {
                pool.entry(c.to.clone())
                    .or_default()
                    .push((contributor.clone(), amount * c.ratio));
            }
        }
    }

    let uses_conditional = deployed.iter().any(|(c, _)| conditional_chars.contains(c));
    Solved {
        pool,
        deployed,
        consumers: econ.consumers,
        uses_conditional,
    }
}

/// The roster's morale-swap manager (Fiammetta and any future equivalent): an operator with
/// a base skill that swaps Morale with another. This is what keeps a morale-conditional
/// rotation (Ling/Dusk) running, so it's surfaced as the comp's rotation requirement.
fn morale_swap_enabler(
    operators: &[OperatorBaseProfile],
    building_data: &BuildingDataFile,
) -> Option<String> {
    operators
        .iter()
        .find(|op| {
            op.available_buffs.iter().any(|b| {
                building_data.buffs.get(b).is_some_and(|buff| {
                    buff.description.to_lowercase().contains("swap")
                        && buff.description.to_lowercase().contains("morale with")
                })
            })
        })
        .map(|op| op.char_id.clone())
}

/// A production consumer that benefits from the economy, with the bonus it receives.
#[derive(Debug)]
pub struct PerceptionConsumer {
    pub char_id: String,
    pub room_type: String,
    /// `bd_` resource id the operator consumes (for display/debug).
    pub resource: String,
    /// Peak bonus - the snapshot value while the operator is working.
    pub bonus_pct: f64,
    /// Sustained (24/7) bonus. The operator's bonus is a productivity multiplier it earns
    /// whenever it works, so its magnitude tracks the resource POOL, not the operator's own
    /// rest. The pool is reduced only when a SEPARATE generator feeding it rests: each
    /// cross-source contribution is scaled by that generator's uptime, while the consumer's
    /// own (co-present) generation and the steady dorm occupancy count in full.
    pub sustained_pct: f64,
}

/// Everything the resource economy yields for a base, from a single solve.
pub struct PerceptionResult {
    /// `buff_id -> productivity %` overrides to fold into the optimizer's registry so it
    /// values and places the production consumers. Empty when there's no economy.
    pub overrides: HashMap<String, f64>,
    /// `buff_id -> (target room type, productivity %)` for GLOBAL Control-Center consumers
    /// (Sakiko's Passion -> all Factories). Folded as a global Control-Center buff, so the
    /// optimizer values stationing the operator in the Control Center for the whole-base bonus.
    pub global_overrides: HashMap<String, (String, f64)>,
    /// `(char_id, room_type)` support generators to station outside production (e.g.
    /// "Mulberry -> HR Office") - the actionable cross-room recommendation.
    pub support: Vec<(String, String)>,
    /// Production consumers that gain from the economy, deduped per operator (strongest).
    pub consumers: Vec<PerceptionConsumer>,
    /// The morale-swap manager (Fiammetta) the plan should use to sustain its
    /// morale-conditional rotation, if the roster has one. `None` when the plan uses no
    /// conditional operators, or when one is needed but the roster lacks it (see
    /// `needs_rotation_manager`).
    pub rotation_manager: Option<String>,
    /// True when the plan stations a morale-conditional operator (Ling/Dusk) but the roster
    /// has no morale-swap manager to sustain it - a "you also need a Fiammetta" flag.
    pub needs_rotation_manager: bool,
}

/// Evaluate the resource economy for a base: the registry overrides (integration hook), the
/// support-generator deployment (cross-room recommendation), and the production consumers
/// that benefit - all from one solve. Empty when the roster has no economy. Callers gate to
/// 243 layouts.
pub fn evaluate(
    operators: &[OperatorBaseProfile],
    building: &UserBuilding,
    building_data: &BuildingDataFile,
    morale_drains: &HashMap<String, f64>,
    registry: &HashMap<String, BuffResolutionStrategy>,
) -> PerceptionResult {
    let s = solve(operators, building, building_data, registry);
    let recovery = morale_recovery(building);
    let uptimes: HashMap<&str, f64> = operators
        .iter()
        .map(|op| (op.char_id.as_str(), op_uptime(op, morale_drains, recovery)))
        .collect();
    let mut overrides: HashMap<String, f64> = HashMap::new();
    let mut global_overrides: HashMap<String, (String, f64)> = HashMap::new();
    let mut by_char: HashMap<String, PerceptionConsumer> = HashMap::new();
    for (char_id, buff_id, c) in &s.consumers {
        if c.room_type != "MANUFACTURE" && c.room_type != "TRADING" {
            continue;
        }
        let contribs = s.pool.get(&c.resource);
        // Peak = the full pool (a fresh-operator snapshot). Sustained scales each contribution
        // by whether the consumer earns it 24/7: the consumer's OWN generation is co-present
        // with its consumption (factor 1), while a SEPARATE generator only feeds the pool while
        // it too is working (factor = that generator's uptime). The dorm occupancy each
        // generator reads stays full as individuals rotate, so it is treated as steady.
        let points: f64 = contribs.map_or(0.0, |v| v.iter().map(|(_, a)| a).sum());
        let sustained_points: f64 = contribs.map_or(0.0, |v| {
            v.iter()
                .map(|(contributor, a)| {
                    let factor = if contributor == char_id {
                        1.0
                    } else {
                        uptimes.get(contributor.as_str()).copied().unwrap_or(1.0)
                    };
                    a * factor
                })
                .sum()
        });
        let bonus = points * c.pct_per_point;
        if bonus <= 0.0 {
            continue;
        }
        let sustained = sustained_points * c.pct_per_point;
        // Collapse by buff for the registry (PEAK - the optimal view is a snapshot), and by
        // operator (strongest) for display. A global Control-Center consumer (Sakiko) folds in
        // as a whole-room buff; a per-room consumer (Rosmontis) as that operator's direct one.
        if c.global {
            let ov = global_overrides
                .entry(buff_id.clone())
                .or_insert_with(|| (c.room_type.clone(), 0.0));
            ov.1 = ov.1.max(bonus);
        } else {
            let ov = overrides.entry(buff_id.clone()).or_insert(0.0);
            *ov = ov.max(bonus);
        }
        let slot = by_char
            .entry(char_id.clone())
            .or_insert_with(|| PerceptionConsumer {
                char_id: char_id.clone(),
                room_type: c.room_type.clone(),
                resource: c.resource.clone(),
                bonus_pct: 0.0,
                sustained_pct: 0.0,
            });
        if bonus > slot.bonus_pct {
            slot.room_type = c.room_type.clone();
            slot.resource = c.resource.clone();
            slot.bonus_pct = bonus;
            slot.sustained_pct = sustained;
        }
    }
    // The morale-conditional rotation (Ling/Dusk) needs a morale-swap manager to sustain.
    let enabler = s
        .uses_conditional
        .then(|| morale_swap_enabler(operators, building_data))
        .flatten();
    let needs_rotation_manager = s.uses_conditional && enabler.is_none();
    PerceptionResult {
        overrides,
        global_overrides,
        support: s.deployed,
        consumers: by_char.into_values().collect(),
        rotation_manager: enabler,
        needs_rotation_manager,
    }
}

#[cfg(test)]
mod bd_tests {
    use super::{Unit, classify};

    #[test]
    fn control_center_global_consumer_targets_the_production_room() {
        // Sakiko-style: a Control-Center buff that boosts ALL factories per a resource is a
        // GLOBAL consumer whose effect room is the factory, not the Control Center.
        let desc = "the productivity of Precious Metals of all Factories <@cc.vup>+1%</>, with an \
            additional <@cc.vup>+1%</> for every <@cc.vup>20</> <$cc.bd_mujica><@cc.rem>Passion</></>";
        let (_g, _conv, cons, _cond) = classify(desc, "CONTROL");
        assert_eq!(cons.len(), 1, "expected one consumer, got {}", cons.len());
        assert!(
            cons[0].global,
            "a CC factory-productivity consumer should be global"
        );
        assert_eq!(cons[0].room_type, "MANUFACTURE");
        assert_eq!(cons[0].resource, "bd_mujica");
    }

    #[test]
    fn flat_resource_generation_is_parsed() {
        // "Passion +20" with no per-unit scope is a FLAT generator; the "for every 20 Passion"
        // form is a CONSUMED resource and must NOT be read as flat generation.
        let gen_desc = "<$cc.bd_mujica><@cc.rem>Passion</></> <@cc.vup>+20</>; clue collection speed <@cc.vup>+5%</>";
        let (gens, _c, _co, _cd) = classify(gen_desc, "CONTROL");
        assert!(
            gens.iter()
                .any(|g| g.resource == "bd_mujica" && matches!(g.unit, Unit::Flat)),
            "flat Passion generation should be parsed, got {} gens",
            gens.len()
        );
        let consume_desc = "for every <@cc.vup>8</> <$cc.bd_mujica><@cc.rem>Passion</></>, all \
            Trading Posts' order efficiency <@cc.vup>+1%</>";
        let (gens2, _c2, _co2, _cd2) = classify(consume_desc, "CONTROL");
        assert!(
            gens2.is_empty(),
            "a consumed resource must not be read as flat generation"
        );
    }

    #[test]
    fn passion_threshold_is_not_read_as_generation() {
        // Monster-of-Acting's bandmate downside: "self Morale loss per hour +0.05 when Passion is
        // 40 or higher". The bare 40 is a THRESHOLD, not generation - its lack of a `+` sign must
        // keep it out of the pool (otherwise the combo's pool inflates by a phantom +40).
        let desc = "self Morale loss per hour <@cc.vdown>+0.05</> when Passion is \
            <@cc.kw>40</> or higher";
        let (gens, _c, _co, _cd) = classify(desc, "CONTROL");
        assert!(
            gens.is_empty(),
            "a 'Passion is N' threshold must not be flat generation, got {} gens",
            gens.len()
        );
    }

    #[test]
    fn dual_role_op_generates_and_consumes_one_resource() {
        // Monster of Acting: GENERATES Passion +20 AND, as a Control-Center op, globally consumes
        // it for trading. One buff yields one flat generator and one global consumer of the SAME
        // resource - so the bundle can seat it once with its +20 counted co-present.
        let desc = "<$cc.bd_mujica><@cc.rem>Passion</></> <@cc.vup>+20</>; for every <@cc.vup>8</> \
            <$cc.bd_mujica><@cc.rem>Passion</></>, self Morale consumed per hour <@cc.vdown>+0.01</> \
            and all Trading Posts' order efficiency <@cc.vup>+1%</> (Only the strongest effect of this type takes place)";
        let (gens, _conv, cons, _cond) = classify(desc, "CONTROL");
        assert_eq!(
            gens.len(),
            1,
            "expected exactly one (+20) generator, got {}",
            gens.len()
        );
        assert!(
            matches!(gens[0].unit, Unit::Flat) && (gens[0].per_unit - 20.0).abs() < 1e-9,
            "the generator should be a flat +20"
        );
        assert_eq!(cons.len(), 1, "expected one global trading consumer");
        assert!(cons[0].global && cons[0].room_type == "TRADING");
        assert_eq!(
            gens[0].resource, cons[0].resource,
            "generator and consumer share the resource"
        );
    }

    #[test]
    fn idols_aura_generation_is_dorm_scaled() {
        // "for every Operator in the Dormitories, Passion +1" scales on resting capacity, with no
        // explicit divisor (the bare "every"), so each resting operator adds one point.
        let desc = "for <@cc.vup>every</> Operator in the Dormitories, \
            <$cc.bd_mujica><@cc.rem>Passion</></> <@cc.vup>+1</>";
        let (gens, _conv, _cons, _cond) = classify(desc, "CONTROL");
        assert_eq!(gens.len(), 1);
        assert!(matches!(gens[0].unit, super::Unit::Resting));
        let ctx = super::BaseContext {
            resting: 17,
            single_dorm: 5,
            recruit_slots: 4,
            sui_count: 5,
        };
        assert!(
            (gens[0].contribution(ctx) - 17.0).abs() < 1e-9,
            "17 resting operators -> +17 Passion"
        );
    }
}
