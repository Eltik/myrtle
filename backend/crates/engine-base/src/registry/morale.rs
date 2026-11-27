use std::collections::HashMap;
use std::sync::LazyLock;

use regex::Regex;

use myrtle_gamedata_raw::tables::building::Buff;

use super::patterns::{RE_CC_WITH, RE_MORALE_DECREASE, RE_MORALE_INCREASE};

pub fn parse_morale_drain_increase(desc: &str) -> Option<f64> {
    RE_MORALE_INCREASE
        .captures(desc)
        .and_then(|c| c[1].parse().ok())
}

pub fn parse_morale_drain_decrease(desc: &str) -> Option<f64> {
    RE_MORALE_DECREASE
        .captures(desc)
        .and_then(|c| c[1].parse().ok())
}

/// Morale effect on a different co-seated operator
#[derive(Clone, Debug, PartialEq)]
pub enum TargetedMoraleEffect {
    Rider { target: Option<String>, delta: f64 },
    NegatesOwnLoss { target: Option<String> },
    NegatesFactionOwnLoss { faction: String },
    SelfAuraImmunity,
    SelfFormulaDrain { targets: Vec<String>, delta: f64 },
}

pub(super) static RE_TARGETED_RIDER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"Morale consumed per hour by <@cc\.kw>([^<]+)</>\s*<@cc\.(?:vup|vdown)>\+?([\d.]+)</>",
    )
    .expect("valid regex")
});
pub(super) static RE_NEGATES_OWN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"ignores? the self Morale loss effect").expect("valid regex"));
pub(super) static RE_NEGATES_FACTION: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"<@cc\.kw>remove</> any Morale reduction effects from <\$cc\.g\.([a-z0-9_]+)>")
        .expect("valid regex")
});
pub(super) static RE_AURA_IMMUNITY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"<@cc\.kw>ignore</> the effects of any Operators stationed in (?:that|the) Factory that would affect the Morale consumption of <@cc\.kw>this Operator",
    )
    .expect("valid regex")
});
pub(super) static RE_FORMULA_DRAIN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"Morale consumed when producing <@cc\.kw>[^<]+</> is reduced by <@cc\.vup>-([\d.]+)</>",
    )
    .expect("valid regex")
});

pub fn has_targeted_morale_effect(desc: &str) -> bool {
    RE_TARGETED_RIDER.is_match(desc)
        || RE_NEGATES_OWN.is_match(desc)
        || RE_NEGATES_FACTION.is_match(desc)
        || RE_AURA_IMMUNITY.is_match(desc)
        || RE_FORMULA_DRAIN.is_match(desc)
}

pub fn targeted_morale_effects(
    buffs: &HashMap<String, Buff>,
    name_to_char: &HashMap<String, String>,
) -> HashMap<String, TargetedMoraleEffect> {
    let mut out = HashMap::new();
    for (buff_id, buff) in buffs {
        if let Some(c) = RE_TARGETED_RIDER.captures(&buff.description) {
            out.insert(
                buff_id.clone(),
                TargetedMoraleEffect::Rider {
                    target: name_to_char.get(&c[1].to_lowercase()).cloned(),
                    delta: c[2].parse().unwrap_or(0.0),
                },
            );
        } else if RE_NEGATES_OWN.is_match(&buff.description) {
            let target = RE_CC_WITH
                .captures(&buff.description)
                .and_then(|w| name_to_char.get(&w[1].to_lowercase()).cloned());
            out.insert(
                buff_id.clone(),
                TargetedMoraleEffect::NegatesOwnLoss { target },
            );
        } else if let Some(c) = RE_NEGATES_FACTION.captures(&buff.description) {
            out.insert(
                buff_id.clone(),
                TargetedMoraleEffect::NegatesFactionOwnLoss {
                    faction: c[1].to_lowercase(),
                },
            );
        } else if RE_AURA_IMMUNITY.is_match(&buff.description) {
            out.insert(buff_id.clone(), TargetedMoraleEffect::SelfAuraImmunity);
        } else if let Some(c) = RE_FORMULA_DRAIN.captures(&buff.description) {
            out.insert(
                buff_id.clone(),
                TargetedMoraleEffect::SelfFormulaDrain {
                    targets: buff.targets.clone(),
                    delta: -c[1].parse().unwrap_or(0.0),
                },
            );
        }
    }
    out
}

pub fn parse_morale_loss_increase(desc: &str) -> Option<f64> {
    static RE_MORALE_LOSS: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"[Ss]elf Morale loss per hour <@cc\.vdown>\+([\d.]+)</>").expect("valid regex")
    });
    let c = RE_MORALE_LOSS.captures(desc)?;
    let start = c.get(0)?.start();
    let mut from = start.saturating_sub(120);
    while from > 0 && !desc.is_char_boundary(from) {
        from -= 1;
    }
    if desc[from..start].contains("together with") {
        return None;
    }
    c[1].parse().ok()
}
