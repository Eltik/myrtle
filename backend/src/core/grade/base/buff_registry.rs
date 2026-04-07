use std::collections::HashMap;

use regex::Regex;

use crate::core::gamedata::types::building::Buff;

pub enum BuffResolutionStrategy {
    /// Efficiency field is the bonus %. Value = efficiency as f64.
    DirectEfficiency { value: f64 },

    /// Bonus scales with count of a facility type.
    /// e.g. Automation: +X% per Power Plant, nullifies other ops' productivity.
    FacilityCountScaling {
        target_room: String,    // "POWER", "TRADING", "DORMITORY", etc.
        per_unit_pct: f64,      // e.g. 5.0, 10.0, 15.0
        per_level: bool,        // true for dorm-level scaling ("per level of each Dormitory")
        nullifies_others: bool, // true for Automation buffs
    },

    /// Bonus scales with teammates' skills matching a pattern.
    /// e.g. "+5% per Standardization skill in same Factory"
    TeammateSkillScaling {
        target_buff_pattern: String, // prefix to match, e.g. "manu_prod_spd"
        per_match_pct: f64,
    },

    /// Mirrors/multiplies based on teammates' total output.
    /// e.g. Heavenly Reward: "+5% per 5% from others, max +25%"
    TeammateOutputMirroring {
        ratio: f64,   // e.g. 1.0 (1:1 mirror) or 0.5
        cap_pct: f64, // max bonus, e.g. 25.0
    },

    /// Provides both efficiency AND order limit change.
    /// e.g. SilverAsh: "+20% efficiency, +4 order limit"
    /// e.g. Degenbrecher: "+25% efficiency, -6 order limit"
    EfficiencyWithOrderLimit {
        efficiency: f64,
        order_limit: i32, // positive = adds CAP, negative = removes CAP
    },

    /// Scales with total order limit contributions from teammates.
    /// e.g. Degenbrecher E2: "+25% per 5 CAP from teammates, max +100%"
    /// e.g. Jaye E0+1: "+4% per 1 order limit increase from others"
    OrderLimitScaling {
        per_cap_threshold: f64,   // every N CAP
        bonus_per_threshold: f64, // gives this much %
        cap_pct: f64,             // max bonus
    },

    /// Control Center buff that applies globally to all rooms of a type.
    /// e.g. "all Factories +2%"
    GlobalEffect {
        target_room: String, // "MANUFACTURE", "TRADING"
        bonus_pct: f64,
    },

    /// Bonus based on operator faction/tag in the affected rooms.
    /// e.g. "all Knight operators in Factories +7%"
    TagBased {
        tag: String, // "knight", "sarkaz", "abyssal", etc.
        bonus_pct: f64,
        target_room: String,
    },

    /// Morale recovery or morale drain modifier (dormitory/control).
    MoraleModifier {
        recovery_per_hour: f64, // positive = recovery, negative = drain
        is_self_only: bool,     // true for single-target, false for AoE
    },

    /// Only affects capacity/order limits, not speed.
    CapacityOnly,

    /// Non-production facilities (workshop, HR, training, reception).
    /// Store the efficiency or parsed value for secondary scoring.
    NonProduction { value: f64 },

    /// Fallback for truly complex buffs we can't cleanly parse.
    /// Stores a conservative estimate.
    Complex { estimated_pct: f64 },

    /// Efficiency changes over the course of a shift based on time/morale.
    /// Stores the time-averaged value over a full 24hr shift.
    MoraleDecayEfficiency { time_averaged_value: f64 },
}

pub fn build_registry(buffs: &HashMap<String, Buff>) -> HashMap<String, BuffResolutionStrategy> {
    let mut registry: HashMap<String, BuffResolutionStrategy> = HashMap::new();
    for (buff_id, buff) in buffs {
        let prefix = buff_id.split("[").next().unwrap_or(buff_id);
        // "manu_prod_spd&power[000]" → "manu_prod_spd&power"
        // Then check: prefix.contains("&power"), prefix.contains("_variable"), etc.
        let strategy = match buff.room_type.as_str() {
            "WORKSHOP" | "HIRE" | "TRAINING" | "MEETING" => {
                let effiency = if buff.efficiency == 0 {
                    // parse from description
                    0.0
                } else {
                    buff.efficiency as f64
                };
                BuffResolutionStrategy::NonProduction { value: effiency }
            }
            "DORMITORY" => {
                let recovery = parse_first_float(&buff.description).unwrap_or(0.0);
                let desc_lower = buff.description.to_lowercase();
                let is_self_only = desc_lower.contains("self")
                    || desc_lower.contains("oneself")
                    || prefix.contains("_oneself")
                    || prefix.contains("_single");
                BuffResolutionStrategy::MoraleModifier {
                    recovery_per_hour: recovery,
                    is_self_only,
                }
            }
            "CONTROL" => {
                let desc_lower = buff.description.to_lowercase();
                if prefix.contains("_fraction") || prefix.contains("_tag") {
                    // Tag-based
                    let tag = parse_tag_keyword(&buff.description).unwrap_or_default();
                    let bonus = parse_first_pct(&buff.description).unwrap_or(0.0);
                    let target_room = if desc_lower.contains("factor") {
                        "MANUFACTURE"
                    } else if desc_lower.contains("trading") {
                        "TRADING"
                    } else {
                        "MANUFACTURE"
                    };

                    BuffResolutionStrategy::TagBased {
                        tag,
                        bonus_pct: bonus,
                        target_room: target_room.to_string(),
                    }
                } else if prefix.contains("_mp_")
                    || prefix.contains("_cost")
                    || prefix.contains("allCost")
                {
                    // Morale gain
                    let recovery = parse_first_float(&buff.description).unwrap_or(0.0);
                    BuffResolutionStrategy::MoraleModifier {
                        recovery_per_hour: recovery,
                        is_self_only: false,
                    }
                } else if prefix.contains("_prod_")
                    || prefix.contains("_tra_")
                    || prefix.contains("_trade_")
                {
                    // Global production bonus
                    let target_room = if prefix.contains("_prod_") {
                        "MANUFACTURE"
                    } else {
                        "TRADING"
                    };
                    let bonus = parse_first_pct(&buff.description).unwrap_or(0.0);
                    BuffResolutionStrategy::GlobalEffect {
                        target_room: target_room.to_string(),
                        bonus_pct: bonus,
                    }
                } else {
                    // Other buffs
                    let value = parse_first_pct(&buff.description).unwrap_or(0.0);
                    BuffResolutionStrategy::Complex {
                        estimated_pct: value,
                    }
                }
            }
            "MANUFACTURE" | "TRADING" | "POWER" => {
                // Capacity-only
                if (prefix.contains("_limit") || prefix.contains("limit&"))
                    && !prefix.contains("_spd")
                    && buff.efficiency == 0
                {
                    BuffResolutionStrategy::CapacityOnly
                }
                // Morale-decay dependent: efficiency decreases as morale drops
                else if prefix.contains("_reduce")
                    && buff.description.contains("Morale difference")
                {
                    // Pattern: "+X% base, -Y% per Z morale difference"
                    // Peak is in Efficiency field. Penalty: parse from description.
                    // Over a full shift, avg morale difference = 12
                    // (morale goes from 24 to 0, difference goes from 0 to 24, avg = 12)
                    let peak = buff.efficiency as f64;
                    // Parse: "every <@cc.kw>4</> points" → 4, and "-5%" → 5
                    let interval = parse_kw_number(&buff.description).unwrap_or(4.0);
                    let penalty_pct = parse_first_vdown_pct(&buff.description).unwrap_or(5.0);
                    let avg_penalty = (12.0 / interval) * penalty_pct;
                    BuffResolutionStrategy::MoraleDecayEfficiency {
                        time_averaged_value: (peak - avg_penalty).max(0.0),
                    }
                }
                // Morale threshold: activates when morale difference > threshold
                else if prefix.contains("_addition&cost")
                    && buff.description.contains("Morale difference")
                {
                    // Pattern: "+X% when morale difference > T"
                    // Active for (24 - T) / 24 of the shift
                    let bonus = parse_first_pct(&buff.description).unwrap_or(0.0);
                    let threshold = parse_kw_number(&buff.description).unwrap_or(12.0);
                    let active_fraction = (24.0 - threshold) / 24.0;
                    BuffResolutionStrategy::MoraleDecayEfficiency {
                        time_averaged_value: bonus * active_fraction,
                    }
                }
                // Time-ramp: efficiency increases per hour, capped
                else if prefix.contains("_addition") && buff.description.contains("per hour") {
                    // Pattern: "+X% base, +Y% per hour, up to +Z%"
                    let base = buff.efficiency as f64; // starting value (may be 0)
                    // let per_hour = parse_first_pct(&buff.description).unwrap_or(0.0);
                    // Note: parse_first_pct will grab the first %, which for [030] is "20%"
                    // We need the "per hour" value specifically. Use a targeted regex.
                    let per_hr = parse_per_hour_pct(&buff.description).unwrap_or(1.0);
                    let cap = parse_last_pct(&buff.description).unwrap_or(25.0);
                    // Time to reach cap from base: (cap - base) / per_hr hours
                    let ramp_hours = if per_hr > 0.0 {
                        (cap - base) / per_hr
                    } else {
                        24.0
                    };
                    // Over a 24hr shift:
                    // - Ramp phase: average = (base + cap) / 2, duration = min(ramp_hours, 24)
                    // - Plateau phase: value = cap, duration = max(24 - ramp_hours, 0)
                    let ramp_duration = ramp_hours.min(24.0);
                    let plateau_duration = (24.0 - ramp_duration).max(0.0);
                    let avg = ((base + cap) / 2.0 * ramp_duration + cap * plateau_duration) / 24.0;
                    BuffResolutionStrategy::MoraleDecayEfficiency {
                        time_averaged_value: avg,
                    }
                }
                // Efficiency + order limit (e.g. "efficiency +25% and order limit -6")
                // Must come BEFORE the generic buff.efficiency > 0 check
                else if prefix.starts_with("trade_ord_spd&limit") {
                    let efficiency = buff.efficiency as f64;
                    let order_limit = parse_order_limit(&buff.description).unwrap_or(0);
                    BuffResolutionStrategy::EfficiencyWithOrderLimit {
                        efficiency,
                        order_limit,
                    }
                }
                // Order limit scaling: Degenbrecher's "for every 5 order limit increase... +25%, max +100%"
                else if prefix == "trade_ord_spd_variable3" {
                    let threshold = parse_first_vup_number(&buff.description).unwrap_or(5.0);
                    let bonus = parse_nth_pct(&buff.description, 0).unwrap_or(25.0);
                    let cap = parse_last_pct(&buff.description).unwrap_or(100.0);
                    BuffResolutionStrategy::OrderLimitScaling {
                        per_cap_threshold: threshold,
                        bonus_per_threshold: bonus,
                        cap_pct: cap,
                    }
                }
                // Jaye's "Investment Solicitations": "+4% per order limit increase from others"
                else if prefix == "trade_ord_spd_variable" {
                    let per = parse_first_pct(&buff.description).unwrap_or(4.0);
                    BuffResolutionStrategy::OrderLimitScaling {
                        per_cap_threshold: 1.0,
                        bonus_per_threshold: per,
                        cap_pct: f64::MAX,
                    }
                }
                // Direct efficiency
                else if buff.efficiency > 0 {
                    BuffResolutionStrategy::DirectEfficiency {
                        value: buff.efficiency as f64,
                    }
                }
                // Automation, scales with power plant count
                else if prefix.contains("&power") {
                    let per_unit = parse_first_pct(&buff.description).unwrap_or(5.0);
                    BuffResolutionStrategy::FacilityCountScaling {
                        target_room: "POWER".to_string(),
                        per_unit_pct: per_unit,
                        per_level: false,
                        nullifies_others: true,
                    }
                }
                // Dormitory scaling
                else if prefix.contains("&dorm") {
                    let per_unit = parse_first_pct(&buff.description).unwrap_or(1.0);
                    BuffResolutionStrategy::FacilityCountScaling {
                        target_room: "DORMITORY".to_string(),
                        per_unit_pct: per_unit,
                        per_level: true,
                        nullifies_others: false,
                    }
                }
                // Teammate skill scaling (eg. +5% per Standardization skill)
                else if prefix.contains("_skill_spd") {
                    let per_match = parse_first_pct(&buff.description).unwrap_or(5.0);
                    let keyword = parse_tag_keyword(&buff.description).unwrap_or_default();
                    BuffResolutionStrategy::TeammateSkillScaling {
                        target_buff_pattern: keyword,
                        per_match_pct: per_match,
                    }
                }
                // Output mirroring, eg. Heavenly Reward, Champion's Bearing
                else if prefix.contains("_variable2") {
                    let cap = parse_last_pct(&buff.description).unwrap_or(25.0);
                    let per = parse_first_pct(&buff.description).unwrap_or(5.0);
                    BuffResolutionStrategy::TeammateOutputMirroring {
                        ratio: per,
                        cap_pct: cap,
                    }
                }
                // Building-resource dependent (Felvine, Engineering Robots, etc.)
                else if prefix.contains("_bd") {
                    let per_unit = parse_first_pct(&buff.description).unwrap_or(1.0);
                    BuffResolutionStrategy::Complex {
                        estimated_pct: per_unit,
                    }
                }
                // Trading gold-line scaling
                else if prefix.contains("&gold") || prefix.contains("&trade") {
                    let per_unit = parse_first_pct(&buff.description).unwrap_or(5.0);
                    BuffResolutionStrategy::FacilityCountScaling {
                        target_room: if prefix.contains("&gold") {
                            "MANUFACTURE"
                        } else {
                            "TRADING"
                        }
                        .to_string(),
                        per_unit_pct: per_unit,
                        per_level: false,
                        nullifies_others: false,
                    }
                }
                // Fallback
                else {
                    let est = parse_first_pct(&buff.description).unwrap_or(15.0);
                    BuffResolutionStrategy::Complex { estimated_pct: est }
                }
            }
            _ => BuffResolutionStrategy::CapacityOnly,
        };

        registry.insert(buff_id.clone(), strategy);
    }

    registry
}

/// Extract first percentage like "+25%" from description markup
fn parse_first_pct(desc: &str) -> Option<f64> {
    let re = Regex::new(r"<@cc\.vup>\+?([\d.]+)%</>").unwrap();
    re.captures(desc).and_then(|c| c[1].parse().ok())
}

/// Extract first float like "+0.7" from description markup (for morale values)
fn parse_first_float(desc: &str) -> Option<f64> {
    let re = Regex::new(r"<@cc\.vup>\+?([\d.]+)</>").unwrap();
    re.captures(desc).and_then(|c| c[1].parse().ok())
}

/// Extract tag keyword like "Knight" from <@cc.kw>Knight</>
fn parse_tag_keyword(desc: &str) -> Option<String> {
    let re = Regex::new(r"<@cc\.kw>(\w+)</>").unwrap();
    re.captures(desc).map(|c| c[1].to_lowercase())
}

/// Extract the last percentage in description (for cap values)
fn parse_last_pct(desc: &str) -> Option<f64> {
    let re = Regex::new(r"<@cc\.vup>\+?([\d.]+)%</>").unwrap();
    re.find_iter(desc).last().and_then(|m| {
        let inner = Regex::new(r"([\d.]+)%").unwrap();
        inner.captures(m.as_str()).and_then(|c| c[1].parse().ok())
    })
}

/// Parse number from <@cc.kw>4</> pattern
fn parse_kw_number(desc: &str) -> Option<f64> {
    let re = Regex::new(r"<@cc\.kw>(\d+)</>").unwrap();
    re.captures(desc).and_then(|c| c[1].parse().ok())
}

/// Parse first negative percentage from <@cc.vdown>-5%</> or <@cc.vdown>+0.25</>
fn parse_first_vdown_pct(desc: &str) -> Option<f64> {
    let re = Regex::new(r"<@cc\.vdown>[+-]?([\d.]+)%?</>").unwrap();
    re.captures(desc).and_then(|c| c[1].parse().ok())
}

/// Parse "per hour" percentage: "+2% per hour" or "+1% per hour"
fn parse_per_hour_pct(desc: &str) -> Option<f64> {
    let re = Regex::new(r"<@cc\.vup>\+?([\d.]+)%?</>\s*per hour").unwrap();
    re.captures(desc).and_then(|c| c[1].parse().ok())
}

/// Parse order limit from description.
/// Matches "+4" from <@cc.vup>+4</> or "-6" from <@cc.vdown>-6</>
fn parse_order_limit(desc: &str) -> Option<i32> {
    // Try positive first: "order limit <@cc.vup>+4</>"
    let re_pos = Regex::new(r"order limit\s*<@cc\.vup>\+?(\d+)</>").unwrap();
    if let Some(cap) = re_pos.captures(desc) {
        return cap[1].parse::<i32>().ok();
    }
    // Try negative: "order limit <@cc.vdown>-6</>"
    let re_neg = Regex::new(r"order limit\s*<@cc\.vdown>-?(\d+)</>").unwrap();
    if let Some(cap) = re_neg.captures(desc) {
        return cap[1].parse::<i32>().ok().map(|v| -v);
    }
    None
}

/// Parse the Nth <@cc.vup> percentage (0-indexed).
/// Useful when a description has multiple percentage values.
fn parse_nth_pct(desc: &str, n: usize) -> Option<f64> {
    let re = Regex::new(r"<@cc\.vup>\+?([\d.]+)%</>").unwrap();
    re.find_iter(desc).nth(n).and_then(|m| {
        let inner = Regex::new(r"([\d.]+)%").unwrap();
        inner.captures(m.as_str()).and_then(|c| c[1].parse().ok())
    })
}

/// Parse a plain number from <@cc.vup>5</> (no % sign)
fn parse_first_vup_number(desc: &str) -> Option<f64> {
    let re = Regex::new(r"<@cc\.vup>(\d+)</>").unwrap();
    re.captures(desc).and_then(|c| c[1].parse().ok())
}
