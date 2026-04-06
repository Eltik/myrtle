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
                else if prefix.contains("_variable") {
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
