use std::collections::HashMap;
use std::sync::LazyLock;

use regex::Regex;

use myrtle_gamedata_raw::tables::building::Buff;

use super::patterns::{
    RE_CC_CLUE, RE_CC_NAMED_GATE, RE_CC_WITH_OTHER_TAG, RE_COUNT_KEYWORD, RE_FACTION_TOKEN,
    RE_FIRST_FLOAT, RE_FIRST_PCT, RE_FIRST_SIGNED_PCT, RE_INNER_TAG, RE_KW_ANY, RE_KW_BLOCK,
    RE_KW_NUMBER, RE_LAST_PCT, RE_LAST_PCT_INNER, RE_MORALE_RECOVERY, RE_NAMED_GATE_ORD,
    RE_NTH_PCT, RE_ORDER_LIMIT_NEG, RE_ORDER_LIMIT_POS, RE_PER_HOUR_PCT, RE_TAG_KEYWORD,
    RE_VDOWN_PCT, RE_VUP_NUMBER,
};
use super::{BuffResolutionStrategy, NamedCharGrant, OrderEffect};
use crate::util::buff_family;

pub(super) fn room_type_from_global_label(label: &str) -> &'static str {
    if label.starts_with("Trading") {
        "TRADING"
    } else {
        "MANUFACTURE"
    }
}

pub(super) fn formula_for_product(name: &str) -> Option<&'static str> {
    let n = name.to_lowercase();
    if n.contains("battle record") {
        Some("F_EXP")
    } else if n.contains("precious metal") || n.contains("pure gold") {
        Some("F_GOLD")
    } else if n.contains("originium") {
        Some("F_DIAMOND")
    } else {
        None
    }
}

pub(super) fn parse_count_keyword(desc: &str) -> Option<String> {
    let cap = RE_COUNT_KEYWORD.captures(desc)?;
    let token = first_token(&cap[1]);
    (!token.is_empty() && !token.chars().next().is_some_and(|c| c.is_ascii_digit()))
        .then_some(token)
}

pub(super) fn parse_skill_conversion(desc: &str) -> Option<(Vec<String>, String)> {
    if !(desc.contains("considered") && desc.contains("skill")) {
        return None;
    }
    let kws: Vec<String> = RE_KW_ANY
        .captures_iter(desc)
        .map(|c| first_token(&c[1]))
        .filter(|t| !t.is_empty())
        .collect();
    if kws.len() < 2 {
        return None;
    }
    let (to_token, from) = kws.split_last()?;
    Some((from.to_vec(), to_token.clone()))
}

pub(super) fn parse_scaling_cap(desc: &str) -> Option<f64> {
    let lower = desc.to_lowercase();
    let idx = lower.find("max")?;
    RE_LAST_PCT
        .find_iter(&desc[idx..])
        .next()
        .and_then(|m| RE_LAST_PCT_INNER.captures(m.as_str()))
        .and_then(|c| c[1].parse().ok())
}

pub(super) fn plain_text(desc: &str) -> String {
    static RE_TAG: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<[^>]*>").expect("valid regex"));
    RE_TAG.replace_all(desc, "").into_owned()
}

pub(super) fn defaulted_trade_threshold(buffs: &HashMap<String, Buff>) -> Option<u32> {
    static RE_DEFAULTED_RULE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"Pure Gold traded is less than (\d+), it will be considered a Defaulted trade")
            .expect("valid regex")
    });
    buffs
        .values()
        .filter(|b| b.room_type == "TRADING")
        .find_map(|b| {
            RE_DEFAULTED_RULE
                .captures(&plain_text(&b.description))
                .and_then(|c| c[1].parse().ok())
        })
}

/// `(shape, pure_gold)`
pub(super) fn order_value_shape(
    desc: &str,
    defaulted_below: Option<u32>,
) -> Option<(OrderEffect, bool)> {
    static RE_HIGH_LMD: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"traded is higher than (\d+)[^.]*?increase the LMD gained by \+(\d+)")
            .expect("valid regex")
    });
    static RE_DEFAULTED_GOLD: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"Defaulted trade, Pure Gold traded \+(\d+)").expect("valid regex")
    });
    let text = plain_text(desc);
    if let Some(c) = RE_HIGH_LMD.captures(&text) {
        let above = c[1].parse().ok()?;
        let lmd = c[2].parse().ok()?;
        Some((OrderEffect::HighOrderLmdBonus { above, lmd }, false))
    } else if let Some(c) = RE_DEFAULTED_GOLD.captures(&text) {
        let bonus = c[1].parse().ok()?;
        let effect = defaulted_below.map_or(OrderEffect::Enabler, |below| {
            OrderEffect::DefaultedGoldBonus { below, bonus }
        });
        Some((effect, true))
    } else if text.contains("higher-yield") {
        Some((
            OrderEffect::HigherYieldChance {
                strong: !text.contains("slightly"),
            },
            false,
        ))
    } else if text.contains("Pure Gold") || text.contains("Defaulted trade") {
        Some((OrderEffect::Enabler, true)) // rules, no payoff
    } else {
        None
    }
}

pub(super) fn first_token(s: &str) -> String {
    s.trim()
        .split([' ', '-'])
        .next()
        .unwrap_or("")
        .to_lowercase()
}

pub(super) fn parse_crew_tag_global(desc: &str) -> Option<BuffResolutionStrategy> {
    let tag = RE_CC_WITH_OTHER_TAG.captures(desc)?[1].to_lowercase();
    let lower = plain_text(desc).to_lowercase();
    if !lower.contains("to the control center") {
        return None;
    }
    let target_room = if lower.contains("all factories") {
        "MANUFACTURE"
    } else if lower.contains("all trading posts") {
        "TRADING"
    } else {
        return None;
    };
    Some(BuffResolutionStrategy::GlobalEffectWithCrewTag {
        target_room: target_room.to_string(),
        bonus_pct: parse_first_pct(desc)?,
        tag,
        min_others: 1,
    })
}

pub(super) fn non_stacking_priority_families(
    desc: &str,
    buffs: &HashMap<String, Buff>,
) -> Vec<String> {
    let plain = plain_text(desc);
    let Some(start) = plain.find("does not stack with ") else {
        return Vec::new();
    };
    let rest = &plain[start + "does not stack with ".len()..];
    let end = rest
        .find(" and takes priority")
        .or_else(|| rest.find(')'))
        .unwrap_or(rest.len());
    let name = rest[..end].trim();
    if name.is_empty() {
        return Vec::new();
    }
    let mut families: Vec<String> = buffs
        .iter()
        .filter(|(_, b)| b.buff_name.eq_ignore_ascii_case(name))
        .map(|(id, _)| buff_family(id).to_string())
        .collect();
    families.sort();
    families.dedup();
    families
}

pub(super) fn parse_first_pct(desc: &str) -> Option<f64> {
    RE_FIRST_PCT.captures(desc).and_then(|c| c[1].parse().ok())
}

pub(super) fn parse_reception_ceiling(desc: &str) -> Option<f64> {
    let idx = desc.to_lowercase().find("maximum of")?;
    parse_first_pct_from(desc, idx)
}

pub(super) fn parse_first_pct_from(desc: &str, start: usize) -> Option<f64> {
    desc.get(start..).and_then(parse_first_pct)
}

/// First keyword naming a known operator, and the byte offset past it
pub(super) fn find_operator_keyword(
    desc: &str,
    name_to_char: &HashMap<String, String>,
) -> Option<(String, usize)> {
    for cap in RE_KW_BLOCK.captures_iter(desc) {
        let block = cap.get(0)?;
        let name = RE_INNER_TAG.replace_all(&cap[1], "").trim().to_lowercase();
        if name_to_char.contains_key(&name) {
            return Some((name, block.end()));
        }
    }
    None
}

pub(super) fn find_all_operator_char_ids(
    desc: &str,
    name_to_char: &HashMap<String, String>,
) -> (Vec<String>, usize) {
    let mut ids = Vec::new();
    let mut last_end = 0;
    for cap in RE_KW_BLOCK.captures_iter(desc) {
        let name = RE_INNER_TAG.replace_all(&cap[1], "").trim().to_lowercase();
        if let Some(char_id) = name_to_char.get(&name) {
            ids.push(char_id.clone());
            if let Some(block) = cap.get(0) {
                last_end = block.end();
            }
        }
    }
    (ids, last_end)
}

pub(super) fn room_type_from_label(label: &str) -> Option<&'static str> {
    Some(match label {
        "Factory" => "MANUFACTURE",
        "Trading Post" => "TRADING",
        "Control Center" => "CONTROL",
        "Power Plant" => "POWER",
        "Training Room" => "TRAINING",
        "Dormitory" => "DORMITORY",
        "Reception Room" => "MEETING",
        "Office" => "HIRE",
        "Workshop" => "WORKSHOP",
        _ => return None,
    })
}

pub(super) fn parse_named_char_room_grants(
    desc: &str,
    name_to_char: &HashMap<String, String>,
) -> Option<Vec<NamedCharGrant>> {
    let grants: Vec<NamedCharGrant> = RE_CC_NAMED_GATE
        .captures_iter(desc)
        .filter_map(|c| {
            let char_id = name_to_char.get(&c[1].to_lowercase())?.clone();
            let target_room = room_type_from_label(&c[2])?;
            let payload = &c[3];
            let order_limit = RE_NAMED_GATE_ORD
                .captures(payload)
                .and_then(|o| o[1].parse::<f64>().ok())
                .unwrap_or(0.0);
            let nonprod_pct = RE_CC_CLUE
                .captures(payload)
                .and_then(|n| n[1].parse::<f64>().ok())
                .unwrap_or(0.0);
            (order_limit != 0.0 || nonprod_pct != 0.0).then(|| NamedCharGrant {
                char_id,
                target_room: target_room.to_string(),
                order_limit,
                nonprod_pct,
            })
        })
        .collect();
    (!grants.is_empty()).then_some(grants)
}

pub(super) fn parse_layout_count_branch(desc: &str) -> Option<BuffResolutionStrategy> {
    static RE_BRANCH: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(
            r"if ([A-Za-z]+) is greater than or equal to ([A-Za-z]+), all (Trading Posts|Factories)' (?:order efficiency|productivity) \+([\d.]+)%; if ([A-Za-z]+) is greater than ([A-Za-z]+), all (Trading Posts|Factories)' (?:order efficiency|productivity) \+([\d.]+)%",
        )
        .expect("valid regex")
    });
    static RE_TERM_NAME: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"<\$cc\.(bd_[a-z0-9_]+)><@cc\.kw>([^<]+)</>").expect("valid regex")
    });
    let text = plain_text(desc);
    let c = RE_BRANCH.captures(&text)?;
    if c[1] != c[6] || c[2] != c[5] {
        return None;
    }
    let ids: HashMap<String, String> = RE_TERM_NAME
        .captures_iter(desc)
        .map(|m| (m[2].to_string(), m[1].to_string()))
        .collect();
    let a_term = ids.get(&c[1])?.clone();
    let b_term = ids.get(&c[2])?.clone();
    let room_of = |label: &str| room_type_from_global_label(label).to_string();
    Some(BuffResolutionStrategy::LayoutCountBranch {
        a_term,
        b_term,
        ge_room: room_of(&c[3]),
        ge_pct: c[4].parse().ok()?,
        gt_room: room_of(&c[7]),
        gt_pct: c[8].parse().ok()?,
    })
}

pub(super) fn parse_room_presence_gated_global(desc: &str) -> Option<BuffResolutionStrategy> {
    static RE_GATE: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(
            r"if there are (\d+) or more Operation Platforms assigned to ([A-Za-z ]+?)s?, all (Trading Posts|Factories)' (?:order efficiency|productivity) \+([\d.]+)%",
        )
        .expect("valid regex")
    });
    let text = plain_text(desc);
    let c = RE_GATE.captures(&text)?;
    Some(BuffResolutionStrategy::RoomPresenceGatedGlobal {
        required_faction: "robot".to_string(),
        required_count: c[1].parse().ok()?,
        room_type: room_type_from_label(&c[2])?.to_string(),
        target_room: room_type_from_global_label(&c[3]).to_string(),
        bonus_pct: c[4].parse().ok()?,
    })
}

pub(super) fn parse_room_presence_gate(
    desc: &str,
    base_efficiency: f64,
    name_to_char: &HashMap<String, String>,
) -> Option<BuffResolutionStrategy> {
    static RE_GATE_CHAR: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(
            r"(?:[Aa]nd|[Ii]f) <@cc\.kw>([^<]+)</> is (?:assigned to be the Trainer in|assigned to|in) (?:a|an|the) (Factory|Trading Post|Control Center|Power Plant|Training Room|Dormitory|Reception Room|Office|Workshop)",
        )
        .expect("valid regex")
    });
    static RE_GATE_FACTION: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(
            r"[Ii]f another <\$cc\.(?:g|tag)\.([a-z0-9]+)><@cc\.kw>[^<]+</></> (?:Operator )?is assigned to (?:a|an|the) (Factory|Trading Post|Control Center|Power Plant|Training Room)",
        )
        .expect("valid regex")
    });
    if let Some(c) = RE_GATE_CHAR.captures(desc) {
        let room_type = room_type_from_label(&c[2])?;
        let end = c.get(0)?.end();
        return Some(BuffResolutionStrategy::ConditionalOnRoomPresence {
            required_char_ids: name_to_char
                .get(&c[1].to_lowercase())
                .cloned()
                .into_iter()
                .collect(),
            required_faction: None,
            required_count: 1,
            room_type: room_type.to_string(),
            base_efficiency,
            bonus_efficiency: parse_first_pct_from(desc, end).unwrap_or(0.0),
        });
    }
    if let Some(c) = RE_GATE_FACTION.captures(desc) {
        let room_type = room_type_from_label(&c[2])?;
        let end = c.get(0)?.end();
        return Some(BuffResolutionStrategy::ConditionalOnRoomPresence {
            required_char_ids: Vec::new(),
            required_faction: Some(c[1].to_lowercase()),
            required_count: 2,
            room_type: room_type.to_string(),
            base_efficiency,
            bonus_efficiency: parse_first_pct_from(desc, end).unwrap_or(0.0),
        });
    }
    None
}

pub(super) fn find_faction_token(desc: &str) -> Option<(String, usize)> {
    let m = RE_FACTION_TOKEN.captures(desc)?;
    let token = m[1].to_lowercase();
    Some((token, m.get(0)?.end()))
}

/// `(percent, units)` of a per-capacity rate
pub(super) fn parse_per_capacity_rate(desc: &str) -> Option<(f64, f64)> {
    static RE_PER_CAPACITY: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"for every\s*(?:<[^>]+>)?\s*(\d+)\s*(?:</>)?\s*max Drone capacity")
            .expect("valid regex")
    });
    let per_pct = parse_first_pct(desc)?;
    let units: f64 = RE_PER_CAPACITY
        .captures(desc)?
        .get(1)?
        .as_str()
        .parse()
        .ok()?;
    (units > 0.0).then_some((per_pct, units))
}

pub(super) fn parse_first_float(desc: &str) -> Option<f64> {
    RE_FIRST_FLOAT
        .captures(desc)
        .and_then(|c| c[1].parse().ok())
}

pub(super) fn parse_morale_recovery(desc: &str) -> Option<f64> {
    if let Some(c) = RE_MORALE_RECOVERY.captures(desc) {
        return c[1].parse().ok();
    }
    if desc.contains("<$cc.bd_") {
        return Some(0.0);
    }
    parse_first_float(desc)
}

pub(super) fn parse_tag_keyword(desc: &str) -> Option<String> {
    let kw = RE_TAG_KEYWORD.captures(desc)?.get(1)?.as_str();
    if kw.contains('.') && kw.chars().all(|c| c.is_ascii_alphabetic() || c == '.') {
        let collapsed: String = kw
            .chars()
            .filter(char::is_ascii_alphabetic)
            .map(|c| c.to_ascii_lowercase())
            .collect();
        if !collapsed.is_empty() {
            return Some(collapsed);
        }
    }
    let first: String = kw
        .chars()
        .take_while(char::is_ascii_alphanumeric)
        .map(|c| c.to_ascii_lowercase())
        .collect();
    (!first.is_empty()).then_some(first)
}

pub(super) fn parse_last_pct(desc: &str) -> Option<f64> {
    RE_LAST_PCT.find_iter(desc).last().and_then(|m| {
        RE_LAST_PCT_INNER
            .captures(m.as_str())
            .and_then(|c| c[1].parse().ok())
    })
}

pub(super) fn parse_kw_number(desc: &str) -> Option<f64> {
    RE_KW_NUMBER.captures(desc).and_then(|c| c[1].parse().ok())
}

pub(super) fn parse_first_vdown_pct(desc: &str) -> Option<f64> {
    RE_VDOWN_PCT.captures(desc).and_then(|c| c[1].parse().ok())
}

pub(super) fn parse_per_hour_pct(desc: &str) -> Option<f64> {
    RE_PER_HOUR_PCT.captures(desc).and_then(|c| {
        c.get(1)
            .or_else(|| c.get(2))
            .and_then(|m| m.as_str().parse().ok())
    })
}

pub(super) fn parse_first_signed_pct(desc: &str) -> Option<f64> {
    RE_FIRST_SIGNED_PCT
        .captures(desc)
        .and_then(|c| c[1].parse().ok())
}

pub(super) fn parse_order_limit(desc: &str) -> Option<i32> {
    if let Some(cap) = RE_ORDER_LIMIT_POS.captures(desc) {
        return cap[1].parse::<i32>().ok();
    }
    if let Some(cap) = RE_ORDER_LIMIT_NEG.captures(desc) {
        return cap[1].parse::<i32>().ok().map(|v| -v);
    }
    None
}

/// The Nth `<@cc.vup>` percentage, 0-indexed
pub(super) fn parse_nth_pct(desc: &str, n: usize) -> Option<f64> {
    RE_NTH_PCT.find_iter(desc).nth(n).and_then(|m| {
        RE_LAST_PCT_INNER
            .captures(m.as_str())
            .and_then(|c| c[1].parse().ok())
    })
}

pub(super) fn parse_first_vup_number(desc: &str) -> Option<f64> {
    RE_VUP_NUMBER.captures(desc).and_then(|c| c[1].parse().ok())
}
