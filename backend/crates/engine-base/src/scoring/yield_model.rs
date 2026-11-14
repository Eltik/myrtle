//! Room efficiency -> resource yield (LMD / gold / EXP per day)

/// LMD per Pure Gold bar
pub const GOLD_BAR_LMD: f64 = 500.0;
pub const EXP_TO_LMD: f64 = 1.0;
/// Gold bars/day a factory produces at 100% productivity
const FACTORY_GOLD_PER_DAY_BASE: f64 = 20.0;
/// Gold bars/day a Trading Post can sell at 100% productivity
const TRADING_GOLD_SOLD_PER_DAY_BASE: f64 = 20.0;
const DRONES_PER_DAY_BASE: f64 = 240.0;
const DRONE_ROOM_DAYS: f64 = 180.0 / 86400.0;

/// `F_EXP` EXP/day at 100% productivity, by factory level
const fn factory_exp_per_day_base(level: i32) -> f64 {
    match level {
        1 => 6400.0,
        2 => 7200.0,
        _ => 8000.0,
    }
}

pub(crate) fn productivity_mult(efficiency_pct: f64) -> f64 {
    1.0 + efficiency_pct / 100.0
}

/// Innate +1% per operator slotted in a factory or post
const INNATE_PCT_PER_OPERATOR: f64 = 1.0;

#[allow(clippy::cast_precision_loss)]
fn innate_pct(crew: usize) -> f64 {
    crew as f64 * INNATE_PCT_PER_OPERATOR
}

pub(crate) fn trading_bars_per_day(level: i32) -> f64 {
    #[allow(clippy::cast_sign_loss)]
    crate::scoring::order_mix::bars_per_day(level.clamp(1, 3) as usize)
}

pub(crate) const TRADING_COLLECTIONS_PER_DAY: f64 = 2.0;

pub(crate) fn trading_avg_gold_per_order(level: i32) -> f64 {
    #[allow(clippy::cast_sign_loss)]
    crate::scoring::order_mix::avg_gold_per_order(level.clamp(1, 3) as usize)
}

/// Orders/day at `mult`, capped by the order limit
fn trading_orders_per_day(level: i32, mult: f64, order_limit: Option<i32>) -> f64 {
    let rate = trading_bars_per_day(level) * mult / trading_avg_gold_per_order(level);
    order_limit.map_or(rate, |limit| {
        rate.min(f64::from(limit.max(1)) * TRADING_COLLECTIONS_PER_DAY)
    })
}

/// Share of a post's peak output that survives its order buffer (<= 1.0)
#[must_use]
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

/// Raw resource flows from a set of rooms, before the gold->LMD coupling
#[derive(Debug, Clone, Default)]
pub struct BaseFlows {
    pub gold_produced: f64,
    pub gold_factories: usize,
    pub gold_sell_capacity: f64,
    /// Each post's `(bars/day it can sell, LMD-per-bar multiplier)`
    pub posts: Vec<(f64, f64)>,
    pub exp: f64,
    pub drone_recovery_pct: f64,
    pub best_gold_mult: f64,
    pub best_exp_rate: f64,
}

/// The day's drones and what they are worth spent on production
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct DroneFlow {
    pub per_day: f64,
    pub to_gold: f64,
    pub to_exp: f64,
    pub gold_bars: f64,
    pub exp: f64,
    pub lmd: f64,
}

impl BaseFlows {
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
                self.gold_produced = FACTORY_GOLD_PER_DAY_BASE.mul_add(mult, self.gold_produced);
                self.best_gold_mult = self.best_gold_mult.max(mult);
            }
            ("MANUFACTURE", Some("F_EXP")) => {
                let rate = factory_exp_per_day_base(level) * mult;
                self.exp += rate;
                self.best_exp_rate = self.best_exp_rate.max(rate);
            }
            ("POWER", _) => {
                self.drone_recovery_pct += speed_pct;
            }
            _ => {}
        }
    }

    /// Realized gold->trade LMD/day: the slower side caps the bars moved
    #[must_use]
    pub fn realized_lmd(&self) -> f64 {
        self.realized_lmd_with(self.gold_produced)
    }

    fn realized_lmd_with(&self, gold: f64) -> f64 {
        if self.gold_sell_capacity <= 0.0 {
            return 0.0;
        }
        let mut supply = if self.gold_factories == 0 {
            f64::INFINITY
        } else {
            gold
        };
        let mut posts = self.posts.clone();
        posts.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        let mut lmd = 0.0;
        for (bars, per_bar) in posts {
            let sold = bars.min(supply);
            lmd = (sold * per_bar).mul_add(GOLD_BAR_LMD, lmd);
            supply -= sold;
            if supply <= 0.0 {
                break;
            }
        }
        lmd
    }

    #[must_use]
    pub fn drones(&self) -> DroneFlow {
        let per_day = DRONES_PER_DAY_BASE * productivity_mult(self.drone_recovery_pct);
        let bars_per_drone = DRONE_ROOM_DAYS * FACTORY_GOLD_PER_DAY_BASE * self.best_gold_mult;
        let exp_per_drone = DRONE_ROOM_DAYS * self.best_exp_rate;
        let slack = if self.gold_factories > 0 {
            (self.gold_sell_capacity - self.gold_produced).max(0.0)
        } else {
            0.0
        };
        let gold_fill = if bars_per_drone > 0.0 {
            (slack / bars_per_drone).min(per_day)
        } else {
            0.0
        };
        let gold_lmd = |drones: f64| {
            self.realized_lmd_with(drones.mul_add(bars_per_drone, self.gold_produced))
                - self.realized_lmd()
        };
        let gold_per_drone = if gold_fill > 0.0 {
            gold_lmd(gold_fill) / gold_fill
        } else {
            0.0
        };
        let to_gold = if gold_per_drone >= exp_per_drone * EXP_TO_LMD {
            gold_fill
        } else {
            0.0
        };
        let to_exp = if exp_per_drone > 0.0 {
            per_day - to_gold
        } else {
            0.0
        };
        DroneFlow {
            per_day,
            to_gold,
            to_exp,
            gold_bars: to_gold * bars_per_drone,
            exp: to_exp * exp_per_drone,
            lmd: gold_lmd(to_gold),
        }
    }

    #[must_use]
    pub fn lmd_per_day(&self) -> f64 {
        self.realized_lmd() + self.drones().lmd
    }

    #[must_use]
    pub fn exp_per_day(&self) -> f64 {
        self.exp + self.drones().exp
    }

    /// The search objective
    #[must_use]
    pub fn total_value(&self) -> f64 {
        (self.drone_recovery_pct / 100.0
                * DRONES_PER_DAY_BASE
                * DRONE_ROOM_DAYS * FACTORY_GOLD_PER_DAY_BASE).mul_add(GOLD_BAR_LMD, self.exp.mul_add(EXP_TO_LMD, self.realized_lmd()))
    }

    /// What the player banks, for display only
    #[must_use]
    pub fn value_with_drones(&self) -> f64 {
        self.exp_per_day().mul_add(EXP_TO_LMD, self.lmd_per_day())
    }
}

/// LMD-equivalent of a +`pct`% global bonus on every `room_type` room
#[must_use]
pub fn global_bonus_value(room_type: &str, room_count: usize, pct: f64) -> f64 {
    let per_room_base = match room_type {
        "MANUFACTURE" => FACTORY_GOLD_PER_DAY_BASE * GOLD_BAR_LMD,
        "TRADING" => TRADING_GOLD_SOLD_PER_DAY_BASE * GOLD_BAR_LMD,
        _ => return 0.0,
    };
    per_room_base * room_count as f64 * pct / 100.0
}

/// A production room's output buffer: size and hours to fill from empty
#[derive(Debug, Clone)]
pub struct RoomFill {
    pub capacity: i32,
    pub fill_hours: f64,
}

const POINTS_PER_DAY: f64 = 86400.0;

#[must_use]
pub fn room_fill(
    room_type: &str,
    formula: Option<&str>,
    level: i32,
    speed_pct: f64,
    capacity_bonus: i32,
    building_data: &myrtle_gamedata_raw::tables::building::BuildingDataFile,
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
            let rarity = crate::scoring::order_mix::rarity_for_level(building_data, level);
            let orders_per_day = TRADING_GOLD_SOLD_PER_DAY_BASE * mult
                / crate::scoring::order_mix::avg_gold_per_order(rarity);
            Some(RoomFill {
                capacity,
                fill_hours: f64::from(capacity) / orders_per_day * 24.0,
            })
        }
        ("MANUFACTURE", Some(f)) => {
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
    #[must_use]
    pub const fn lmd_equivalent(&self) -> f64 {
        self.exp_per_day.mul_add(EXP_TO_LMD, self.gold_per_day.mul_add(GOLD_BAR_LMD, self.lmd_per_day))
    }
}

/// Uncoupled per-room yield for display
#[must_use]
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
