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
            _ => {}
        }
    }

    /// Realized LMD/day from the gold→trade loop: the slower side bottlenecks.
    pub fn realized_lmd(&self) -> f64 {
        self.gold_produced.min(self.gold_sell_capacity) * GOLD_BAR_LMD
    }

    /// Total daily output as a single LMD-equivalent value.
    pub fn total_value(&self) -> f64 {
        self.realized_lmd() + self.exp * EXP_TO_LMD
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
