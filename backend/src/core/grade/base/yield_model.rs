//! Room efficiency -> resource yield (LMD / gold / EXP per day), so facilities
//! compare by value rather than by efficiency % across different resources.
//!
//! Grounding (`building_data.json` + the in-game economy):
//!   - Factory: `BasicSpeedBuff = 0.01` points per pp per second, so P% accrues
//!     `P% × 864` points/day. Operators-only base = 100%, buffs on top.
//!   - Gold bar (item 3003) costs 4320 points -> `20 × (1 + eff/100)` bars/day,
//!     sells for a flat **500 LMD** in a Trading Post.
//!   - EXP records cost 2700 / 4800 / 10800 points (L1/L2/L3) for 200 / 400 /
//!     1000 EXP -> 6400 / 7200 / 8000 EXP/day at base.
//!   - An L3 post at base sells about one factory's gold (~20 bars/day), hence
//!     "match gold factories to trading posts".
//!   - LS-5 (≈247 EXP/sanity) vs CE-5 (≈250 LMD/sanity) -> **1 EXP ≈ 1 LMD**.
//!
//! Gold->trade LMD is `min(gold produced, gold sellable) × 500`, counted once;
//! the optimizer balances factories against post throughput (surplus gold
//! factories are better as EXP).

/// LMD per Pure Gold bar (fixed by the game).
pub const GOLD_BAR_LMD: f64 = 500.0;
/// Sanity-equivalence of LS-5 vs CE-5.
pub const EXP_TO_LMD: f64 = 1.0;
/// Gold bars/day a factory produces at 100% productivity (no buffs).
const FACTORY_GOLD_PER_DAY_BASE: f64 = 20.0;
/// Gold bars/day a Trading Post can sell at 100% productivity (no buffs).
const TRADING_GOLD_SOLD_PER_DAY_BASE: f64 = 20.0;
/// Drones regenerated per day at 100% recovery: 86400 s / `LaborRecoverTime`
/// (360 s, `building_data.json`).
const DRONES_PER_DAY_BASE: f64 = 240.0;
/// One drone buys `ManufactReduceTimeUnit` (180 s) of factory progress per
/// `ManufactLaborCostUnit` (1); 180 s of gold is 180/4320 of a bar
/// (`ManufactFormulas["4"].cost_point`) at 500 LMD. Assumes drones go to
/// production, the standard endgame use.
const LMD_PER_DRONE: f64 = GOLD_BAR_LMD * 180.0 / 4320.0;

/// `F_EXP` EXP/day at 100% productivity, by factory level.
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

/// Innate +1% per operator slotted in a factory or post, on top of the shown
/// skill total (the community's "3% from having operators slotted"). Not in the
/// displayed efficiency, so yield only.
const INNATE_PCT_PER_OPERATOR: f64 = 1.0;

#[allow(clippy::cast_precision_loss)]
fn innate_pct(crew: usize) -> f64 {
    crew as f64 * INNATE_PCT_PER_OPERATOR
}

/// Base sell rate at this level (order rarity = level in `TradingData`): the
/// exact order-mix figure, not a round 20.
fn trading_bars_per_day(level: i32) -> f64 {
    #[allow(clippy::cast_sign_loss)]
    super::order_mix::bars_per_day(level.clamp(1, 3) as usize)
}

/// Buffer collections per day: the community sheet's 12-hour convention (shift
/// changes come at least this often). An order limit costs output only when the
/// post fills faster.
const TRADING_COLLECTIONS_PER_DAY: f64 = 2.0;

/// Average Pure Gold per order at this level's order mix.
fn trading_avg_gold_per_order(level: i32) -> f64 {
    #[allow(clippy::cast_sign_loss)]
    super::order_mix::avg_gold_per_order(level.clamp(1, 3) as usize)
}

/// Orders/day at `mult`, capped by the order limit emptied
/// `TRADING_COLLECTIONS_PER_DAY` times (a full buffer stalls until collected).
/// `None` = uncapped.
fn trading_orders_per_day(level: i32, mult: f64, order_limit: Option<i32>) -> f64 {
    let rate = trading_bars_per_day(level) * mult / trading_avg_gold_per_order(level);
    order_limit.map_or(rate, |limit| {
        rate.min(f64::from(limit.max(1)) * TRADING_COLLECTIONS_PER_DAY)
    })
}

/// Share of a post's peak output that survives its order buffer (<= 1.0). The
/// search-score twin of `add_room`'s cap, so a team that cuts the limit below
/// what it fills between collections (Jaye beside +80% of roommates,
/// Degenbrecher without a limit-adder) ranks by what it sells. 1.0 for
/// non-trading rooms and unknown limits.
pub fn trading_cap_factor(
    room_type: &str,
    level: i32,
    speed_pct: f64,
    crew: usize,
    order_limit: Option<i32>,
) -> f64 {
    if room_type != "TRADING" {
        return 1.0;
    }
    let mult = productivity_mult(speed_pct + innate_pct(crew));
    let rate = trading_orders_per_day(level, mult, None);
    if rate <= 0.0 {
        return 1.0;
    }
    trading_orders_per_day(level, mult, order_limit) / rate
}

/// Raw resource flows from a set of rooms, before the gold->LMD coupling.
#[derive(Debug, Clone, Default)]
pub struct BaseFlows {
    /// Gold bars/day produced by `F_GOLD` factories.
    pub gold_produced: f64,
    /// `F_GOLD` factories seen. With none, posts sell stock (gold handed out outside
    /// the base) and the coupling can't bind; with idle ones, posts sell nothing.
    pub gold_factories: usize,
    /// Gold bars/day the Trading Posts can sell.
    pub gold_sell_capacity: f64,
    /// Each post's `(bars/day it can sell, LMD-per-bar multiplier)`. The multiplier
    /// is value that pays more per bar without drawing more gold (Tequila's "+500 LMD
    /// above 3 gold"); Proviso's bonus is bars from stock and sits in the capacity
    /// instead (base expert, 2026-09-08). Per post, so a short supply sells where
    /// it pays most.
    pub posts: Vec<(f64, f64)>,
    /// EXP/day produced by `F_EXP` factories.
    pub exp: f64,
    /// Summed drone-recovery % from Power Plant operators. Plants' innate recovery
    /// is layout-constant, so only operator buffs move the objective.
    pub drone_recovery_pct: f64,
}

impl BaseFlows {
    /// `speed_pct` = order/production speed; `value_pct` = order VALUE (LMD/h over a
    /// bare post); `gold_pct` = its gold-throughput part (Pure Gold/h over a bare
    /// post), see `order_mix`. `order_limit` = the post's final limit
    /// (`RoomTotals::order_limit`); `None` prices uncapped.
    #[allow(clippy::too_many_arguments)]
    pub fn add_room(
        &mut self,
        room_type: &str,
        formula: Option<&str>,
        level: i32,
        speed_pct: f64,
        gold_pct: f64,
        value_pct: f64,
        crew: usize,
        order_limit: Option<i32>,
    ) {
        let mult = productivity_mult(speed_pct + innate_pct(crew));
        match (room_type, formula) {
            ("TRADING", _) => {
                // Speed -> more orders, bounded by the buffer (`trading_orders_per_day`).
                // Value splits: the gold part is more bars per order, drawn from stock
                // (Proviso: a defaulted 2-gold order trades 4 bars), so it widens sell capacity
                // and is bounded by factory gold; the rest is more LMD per bar (Tequila's
                // rider) and pays even when gold-starved.
                let bars = trading_orders_per_day(level, mult, order_limit)
                    * trading_avg_gold_per_order(level)
                    * productivity_mult(gold_pct);
                self.gold_sell_capacity += bars;
                self.posts.push((
                    bars,
                    productivity_mult(value_pct) / productivity_mult(gold_pct),
                ));
            }
            ("MANUFACTURE", Some("F_GOLD")) => {
                self.gold_factories += 1;
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

    /// Realized gold->trade LMD/day: the slower side caps the bars moved. A short
    /// supply sells at the best-paying posts first - a gold-short player delivers
    /// the best orders and leaves the rest standing - so a post's unsellable extra
    /// capacity (Proviso's bonus bars on a starved base) never dilutes a premium
    /// post's take.
    pub fn realized_lmd(&self) -> f64 {
        if self.gold_sell_capacity <= 0.0 {
            return 0.0;
        }
        let mut supply = if self.gold_factories == 0 {
            f64::INFINITY
        } else {
            self.gold_produced
        };
        let mut posts = self.posts.clone();
        posts.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        let mut lmd = 0.0;
        for (bars, per_bar) in posts {
            let sold = bars.min(supply);
            lmd += sold * per_bar * GOLD_BAR_LMD;
            supply -= sold;
            if supply <= 0.0 {
                break;
            }
        }
        lmd
    }

    pub fn total_value(&self) -> f64 {
        self.realized_lmd()
            + self.exp * EXP_TO_LMD
            + self.drone_recovery_pct / 100.0 * DRONES_PER_DAY_BASE * LMD_PER_DRONE
    }
}

/// LMD-equivalent of a +`pct`% global bonus on every `room_type` room
/// (`room_count` of them). Puts CC globals on the LMD scale so a resource combo
/// (Passion: factory + trading) weighs fairly against an ordinary CC fill.
/// Factory bonuses use the gold rate ("Precious Metals" target, dominant
/// output); EXP is close enough (8000 vs 10000/day) to skip re-deriving the
/// coupling. Unknown room types are worth 0.
pub fn global_bonus_value(room_type: &str, room_count: usize, pct: f64) -> f64 {
    let per_room_base = match room_type {
        "MANUFACTURE" => FACTORY_GOLD_PER_DAY_BASE * GOLD_BAR_LMD,
        "TRADING" => TRADING_GOLD_SOLD_PER_DAY_BASE * GOLD_BAR_LMD,
        _ => return 0.0,
    };
    per_room_base * room_count as f64 * pct / 100.0
}

/// A production room's output buffer: size and hours to fill from empty. Full
/// means stalled, so `fill_hours` is how long you can stay away losslessly.
/// Trading capacity is in ORDERS (order limit + crew capacity skills); factory
/// in ITEMS of the running formula (`OutputCapacity` weight over item weight).
/// Posts are assumed gold-supplied; a starved one never fills, so this is the
/// conservative deadline.
#[derive(Debug, Clone)]
pub struct RoomFill {
    pub capacity: i32,
    pub fill_hours: f64,
}

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
            // Orders/day follow the order mix: an L1 post fills with more, smaller orders
            // than an L3.
            let rarity = super::order_mix::rarity_for_level(building_data, level);
            let orders_per_day = TRADING_GOLD_SOLD_PER_DAY_BASE * mult
                / super::order_mix::avg_gold_per_order(rarity);
            Some(RoomFill {
                capacity,
                fill_hours: f64::from(capacity) / orders_per_day * 24.0,
            })
        }
        ("MANUFACTURE", Some(f)) => {
            // The highest recipe this level unlocks (same rule as the EXP-rate table).
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

#[derive(Debug, Clone, Default)]
pub struct RoomYield {
    pub lmd_per_day: f64,
    pub gold_per_day: f64,
    pub exp_per_day: f64,
}

impl RoomYield {
    /// One LMD-equivalent figure at the objective's rates (`GOLD_BAR_LMD`,
    /// `EXP_TO_LMD`).
    pub fn lmd_equivalent(&self) -> f64 {
        self.lmd_per_day + self.gold_per_day * GOLD_BAR_LMD + self.exp_per_day * EXP_TO_LMD
    }
}

/// Uncoupled per-room yield for display. Posts show LMD if gold-supplied (sell
/// capacity × 500), with `value_pct` raising LMD per order.
pub fn room_yield(
    room_type: &str,
    formula: Option<&str>,
    level: i32,
    speed_pct: f64,
    value_pct: f64,
    crew: usize,
    order_limit: Option<i32>,
) -> RoomYield {
    let mult = productivity_mult(speed_pct + innate_pct(crew));
    match (room_type, formula) {
        ("TRADING", _) => RoomYield {
            lmd_per_day: trading_orders_per_day(level, mult, order_limit)
                * trading_avg_gold_per_order(level)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_slashed_order_limit_caps_a_post_at_its_collections() {
        // A level-1 post at +100% sells 40 bars/day uncapped. A buffer of one
        // order emptied twice a day moves two orders; a buffer the post can't
        // fill between collections costs nothing.
        let rate = trading_bars_per_day(1) * productivity_mult(100.0);
        let mut free = BaseFlows::default();
        free.add_room("TRADING", None, 1, 100.0, 0.0, 0.0, 0, None);
        assert!((free.gold_sell_capacity - rate).abs() < 1e-9);
        let mut capped = BaseFlows::default();
        capped.add_room("TRADING", None, 1, 100.0, 0.0, 0.0, 0, Some(1));
        let expect = TRADING_COLLECTIONS_PER_DAY * trading_avg_gold_per_order(1);
        assert!((capped.gold_sell_capacity - expect).abs() < 1e-9);
        let mut roomy = BaseFlows::default();
        roomy.add_room("TRADING", None, 1, 100.0, 0.0, 0.0, 0, Some(100));
        assert!((roomy.gold_sell_capacity - rate).abs() < 1e-9);
    }

    #[test]
    fn posts_without_any_gold_factory_sell_from_stock() {
        // No gold factory at all: the coupling can't bind, the post sells at
        // capacity. One gold factory making nothing: the post sells nothing.
        let mut stock = BaseFlows::default();
        stock.add_room("TRADING", None, 1, 100.0, 0.0, 0.0, 0, None);
        assert!((stock.realized_lmd() - 40.0 * GOLD_BAR_LMD).abs() < 1e-6);
        let mut idle = BaseFlows::default();
        idle.add_room("TRADING", None, 1, 100.0, 0.0, 0.0, 0, None);
        idle.add_room("MANUFACTURE", Some("F_GOLD"), 3, -100.0, 0.0, 0.0, 0, None);
        assert!(idle.realized_lmd().abs() < 1e-6);
    }

    #[test]
    fn proviso_moves_bars_from_stock_and_tequila_pays_more_per_bar() {
        // Proviso-class value (+55% LMD, +55% gold): one post at +200% could move 93
        // bars/day, but a base making 44 sells 44 at 500 each; her bonus bars come
        // from stock (base expert, 2026-09-08). One gold factory at +120% = 44/day.
        let mut starved = BaseFlows::default();
        starved.add_room("MANUFACTURE", Some("F_GOLD"), 3, 120.0, 0.0, 0.0, 0, None);
        starved.add_room("TRADING", None, 1, 200.0, 55.0, 55.0, 0, None);
        assert!((starved.realized_lmd() - 44.0 * GOLD_BAR_LMD).abs() < 1e-6);
        let mut rich = BaseFlows::default();
        rich.add_room("MANUFACTURE", Some("F_GOLD"), 3, 4900.0, 0.0, 0.0, 0, None);
        rich.add_room("TRADING", None, 1, 200.0, 55.0, 55.0, 0, None);
        assert!((rich.realized_lmd() - 60.0 * 1.55 * GOLD_BAR_LMD).abs() < 1e-6);

        // Tequila-class value (+24% LMD, +0% gold): the same 44 bars pay 24% more;
        // the rider is LMD, not gold, so starvation doesn't touch it.
        let mut tequila = BaseFlows::default();
        tequila.add_room("MANUFACTURE", Some("F_GOLD"), 3, 120.0, 0.0, 0.0, 0, None);
        tequila.add_room("TRADING", None, 1, 200.0, 0.0, 24.0, 0, None);
        assert!((tequila.realized_lmd() - 44.0 * 1.24 * GOLD_BAR_LMD).abs() < 1e-6);
    }
}
