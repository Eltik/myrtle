//! Converts room efficiency into actual resource **yield** (LMD / gold / EXP per
//! day), so the optimizer can compare facilities by real value instead of by the
//! abstract efficiency % (which conflates different resources).
//!
//! Grounding (all from `building_data.json` + the in-game economy):
//!   - Factory productivity: `BasicSpeedBuff = 0.01` points per percentage-point
//!     per second ⇒ at total productivity P% the factory accrues `P% × 864`
//!     points/day. Base (operators only) = 100%, buffs add on top.
//!   - Gold bar (item 3003) costs 4320 points ⇒ `20 × (1 + eff/100)` bars/day,
//!     and is worth a flat **500 LMD** when sold in a Trading Post.
//!   - EXP records cost 2700 / 4800 / 10800 points (L1/L2/L3) for 200 / 400 /
//!     1000 EXP ⇒ 6400 / 7200 / 8000 EXP/day at base.
//!   - Trading post: an L3 post at base sells ≈ the gold one factory makes
//!     (~20 bars/day), so the famous "match gold factories to trading posts".
//!   - EXP ↔ LMD: LS-5 (≈247 EXP/sanity) vs CE-5 (≈250 LMD/sanity) ⇒ **1 EXP ≈
//!     1 LMD**. Used to put EXP and LMD on one scale.
//!
//! The LMD yield of the gold→trade loop is `min(gold produced, gold sellable) ×
//! 500` - counting it once and letting the optimizer balance gold factories
//! against trading-post throughput (excess gold factories are better as EXP).

/// LMD per Pure Gold bar (fixed by the game).
pub const GOLD_BAR_LMD: f64 = 500.0;
/// EXP→LMD conversion (sanity-equivalence of LS-5 vs CE-5).
pub const EXP_TO_LMD: f64 = 1.0;
/// Gold bars/day a factory produces at 100% productivity (no buffs).
const FACTORY_GOLD_PER_DAY_BASE: f64 = 20.0;
/// Gold bars/day a Trading Post can sell at 100% productivity (no buffs).
const TRADING_GOLD_SOLD_PER_DAY_BASE: f64 = 20.0;
/// Drones ("Labor") regenerated per day at 100% recovery speed:
/// 86400 s / `LaborRecoverTime` (360 s per drone, `building_data.json`).
const DRONES_PER_DAY_BASE: f64 = 240.0;
/// LMD-equivalent of one drone, valued at its production use: one drone buys
/// `ManufactReduceTimeUnit` (180 s) of factory progress per
/// `ManufactLaborCostUnit` (1), and 180 s of a gold factory is 180/4320 of a
/// bar (`ManufactFormulas["4"].cost_point`) at 500 LMD. Assumes recovered
/// drones are spent on production - the standard endgame use.
const LMD_PER_DRONE: f64 = GOLD_BAR_LMD * 180.0 / 4320.0;

/// EXP/day an `F_EXP` factory produces at 100% productivity, by factory level.
const fn factory_exp_per_day_base(level: i32) -> f64 {
    match level {
        1 => 6400.0,
        2 => 7200.0,
        _ => 8000.0,
    }
}

fn productivity_mult(efficiency_pct: f64) -> f64 {
    1.0 + efficiency_pct / 100.0
}

/// The base resource flows produced by a set of rooms (before the gold→LMD
/// coupling is applied).
#[derive(Debug, Clone, Default)]
pub struct BaseFlows {
    /// Gold bars/day produced by `F_GOLD` factories.
    pub gold_produced: f64,
    /// Gold bars/day the Trading Posts can sell.
    pub gold_sell_capacity: f64,
    /// EXP/day produced by `F_EXP` factories.
    pub exp: f64,
    /// Summed drone-recovery bonus % from Power Plant operators. The plants'
    /// inherent recovery is layout-constant, so only operator buffs move the
    /// objective between assignments.
    pub drone_recovery_pct: f64,
}

impl BaseFlows {
    /// `speed_pct` is order/production speed; `value_pct` is order VALUE (Proviso
    /// etc.) which multiplies how much gold each trading-post order moves.
    pub fn add_room(
        &mut self,
        room_type: &str,
        formula: Option<&str>,
        level: i32,
        speed_pct: f64,
        value_pct: f64,
    ) {
        let mult = productivity_mult(speed_pct);
        match (room_type, formula) {
            ("TRADING", _) => {
                // More speed → more orders; more value → more gold per order.
                self.gold_sell_capacity +=
                    TRADING_GOLD_SOLD_PER_DAY_BASE * mult * productivity_mult(value_pct);
            }
            ("MANUFACTURE", Some("F_GOLD")) => {
                self.gold_produced += FACTORY_GOLD_PER_DAY_BASE * mult;
            }
            ("MANUFACTURE", Some("F_EXP")) => {
                self.exp += factory_exp_per_day_base(level) * mult;
            }
            ("POWER", _) => {
                self.drone_recovery_pct += speed_pct;
            }
            _ => {}
        }
    }

    /// Realized LMD/day from the gold→trade loop: the slower side bottlenecks.
    pub fn realized_lmd(&self) -> f64 {
        self.gold_produced.min(self.gold_sell_capacity) * GOLD_BAR_LMD
    }

    /// Total daily output as a single LMD-equivalent value.
    pub fn total_value(&self) -> f64 {
        self.realized_lmd()
            + self.exp * EXP_TO_LMD
            + self.drone_recovery_pct / 100.0 * DRONES_PER_DAY_BASE * LMD_PER_DRONE
    }
}

/// LMD-equivalent value of a +`pct`% global productivity bonus applied to every room of
/// `room_type` in a base with `room_count` such rooms. Puts Control-Center global buffs
/// (factory gold productivity, trading order efficiency) on one comparable LMD scale so the
/// optimizer can weigh a resource combo (Passion: factory + trading) against an ordinary CC
/// fill on equal terms. Factory bonuses are valued at the gold rate - the combo's "Precious
/// Metals" target and the dominant factory output; an EXP factory's bonus sits close enough
/// (8000 vs 10000 LMD-equivalent/day) that one symmetric rate keeps the comparison fair
/// without re-deriving the gold->trade coupling here. Unknown room types are worth 0.
pub fn global_bonus_value(room_type: &str, room_count: usize, pct: f64) -> f64 {
    let per_room_base = match room_type {
        "MANUFACTURE" => FACTORY_GOLD_PER_DAY_BASE * GOLD_BAR_LMD,
        "TRADING" => TRADING_GOLD_SOLD_PER_DAY_BASE * GOLD_BAR_LMD,
        _ => return 0.0,
    };
    per_room_base * room_count as f64 * pct / 100.0
}

/// A production room's output buffer: how big it is and how long it takes to
/// fill from empty. Once full the room stalls, so `fill_hours` is also "how
/// long you can stay logged out of this room without losing anything".
/// Trading capacity is in ORDERS (the game's order limit plus crew capacity
/// skills); factory capacity in ITEMS of the formula the room runs (its
/// `OutputCapacity` weight budget over the item's weight). A trading post is
/// assumed gold-supplied - if it starves it never fills, so this is the
/// conservative deadline.
#[derive(Debug, Clone)]
pub struct RoomFill {
    pub capacity: i32,
    pub fill_hours: f64,
}

/// Nominal gold moved per LMD-strategy order. `TRADING_GOLD_SOLD_PER_DAY_BASE`
/// = 20 bars/day is 10 such orders/day - the same 2-gold/1000-LMD order the
/// yield model's base rate already assumes.
const GOLD_PER_ORDER: f64 = 2.0;

/// Seconds of production points a room accrues per day at 100%.
const POINTS_PER_DAY: f64 = 86400.0;

pub fn room_fill(
    room_type: &str,
    formula: Option<&str>,
    level: i32,
    speed_pct: f64,
    capacity_bonus: i32,
    building_data: &crate::core::gamedata::types::building::BuildingDataFile,
) -> Option<RoomFill> {
    let mult = productivity_mult(speed_pct);
    #[allow(clippy::cast_sign_loss)]
    let phase = (level.max(1) as usize) - 1;
    match (room_type, formula) {
        ("TRADING", _) => {
            let phases = &building_data.trading_data.phases;
            let base = phases
                .get(phase.min(phases.len().checked_sub(1)?))?
                .order_limit;
            let capacity = (base + capacity_bonus).max(1);
            let orders_per_day = TRADING_GOLD_SOLD_PER_DAY_BASE * mult / GOLD_PER_ORDER;
            Some(RoomFill {
                capacity,
                fill_hours: f64::from(capacity) / orders_per_day * 24.0,
            })
        }
        ("MANUFACTURE", Some(f)) => {
            // The recipe the room runs at this level: the highest matching one
            // its level unlocks (the same rule the EXP-rate table encodes).
            let recipe = building_data
                .manufact_formulas
                .values()
                .filter(|r| r.formula_type == f)
                .filter(|r| {
                    r.require_rooms
                        .iter()
                        .all(|rr| rr.room_id != "MANUFACTURE" || rr.room_level <= level)
                })
                .max_by_key(|r| r.cost_point)?;
            let phases = &building_data.manufact_data.phases;
            let out_cap = phases
                .get(phase.min(phases.len().checked_sub(1)?))?
                .output_capacity;
            let capacity = ((out_cap + capacity_bonus) / recipe.weight.max(1)).max(1);
            #[allow(clippy::cast_precision_loss)]
            let items_per_day = POINTS_PER_DAY / recipe.cost_point as f64 * mult;
            Some(RoomFill {
                capacity,
                fill_hours: f64::from(capacity) / items_per_day * 24.0,
            })
        }
        _ => None,
    }
}

/// Per-room natural yield, for display.
#[derive(Debug, Clone, Default)]
pub struct RoomYield {
    pub lmd_per_day: f64,
    pub gold_per_day: f64,
    pub exp_per_day: f64,
}

/// Standalone per-room yield (no coupling) for display. Trading posts show the
/// LMD they could realize if gold-supplied (sell capacity × 500), with order
/// value (`value_pct`) raising the LMD per order.
pub fn room_yield(
    room_type: &str,
    formula: Option<&str>,
    level: i32,
    speed_pct: f64,
    value_pct: f64,
) -> RoomYield {
    let mult = productivity_mult(speed_pct);
    match (room_type, formula) {
        ("TRADING", _) => RoomYield {
            lmd_per_day: TRADING_GOLD_SOLD_PER_DAY_BASE
                * mult
                * productivity_mult(value_pct)
                * GOLD_BAR_LMD,
            ..Default::default()
        },
        ("MANUFACTURE", Some("F_GOLD")) => RoomYield {
            gold_per_day: FACTORY_GOLD_PER_DAY_BASE * mult,
            ..Default::default()
        },
        ("MANUFACTURE", Some("F_EXP")) => RoomYield {
            exp_per_day: factory_exp_per_day_base(level) * mult,
            ..Default::default()
        },
        _ => RoomYield::default(),
    }
}
