use std::collections::HashMap;
use std::sync::LazyLock;

use regex::Regex;

use myrtle_gamedata_raw::tables::building::Buff;

use super::BuffResolutionStrategy;

/// Registry key of the synthetic [`BuffResolutionStrategy::AccountFacts`]
pub const ACCOUNT_FACTS_KEY: &str = "ACCOUNT_FACTS";

pub(super) static RE_HR_PER_SLOT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?:<@cc\.vup>)?\+([\d.]+)%(?:</>)? HR contacting speed for every Recruit slot")
        .expect("valid regex")
});

/// Re-priced registry copy for player-declared facts
pub fn resolve_account_facts(
    registry: &HashMap<String, BuffResolutionStrategy>,
    buffs: &HashMap<String, Buff>,
    open_recruit_slots: u32,
) -> HashMap<String, BuffResolutionStrategy> {
    let mut out = registry.clone();
    out.insert(
        ACCOUNT_FACTS_KEY.to_string(),
        BuffResolutionStrategy::AccountFacts { open_recruit_slots },
    );
    for (buff_id, strategy) in registry {
        let Some(buff) = buffs.get(buff_id) else {
            continue;
        };
        if let (BuffResolutionStrategy::NonProduction { value }, Some(c)) =
            (strategy, RE_HR_PER_SLOT.captures(&buff.description))
        {
            let per_slot: f64 = c[1].parse().unwrap_or(0.0);
            out.insert(
                buff_id.clone(),
                BuffResolutionStrategy::NonProduction {
                    value: value + per_slot * f64::from(open_recruit_slots),
                },
            );
        }
    }
    out
}
