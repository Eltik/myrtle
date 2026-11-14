use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::align::PoolPair;
use myrtle_gamedata::model::GameModel;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AutoNameSource {
    Memory,
    Appellation,
    Override,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoName {
    pub text: String,
    pub source: AutoNameSource,
}

impl AutoName {
    pub fn memory(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            source: AutoNameSource::Memory,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct TranslationMemory {
    /// Trimmed CN name -> EN name
    map: HashMap<String, String>,
    /// Outfit series by CN prefix (斗争血脉 -> Bloodline of Combat)
    series: HashMap<String, String>,
}

/// Splits `斗争血脉/XI` into the series and its numeral
fn split_series(name: &str) -> (&str, Option<&str>) {
    match name.rsplit_once('/') {
        Some((series, numeral)) => (series, Some(numeral)),
        None => (name, None),
    }
}

fn series_of(map: &HashMap<String, String>) -> HashMap<String, String> {
    let mut votes: HashMap<&str, HashMap<&str, usize>> = HashMap::new();
    for (c, e) in map {
        let ((cs, cn), (es, en)) = (split_series(c), split_series(e));
        if cn == en && usable(cs, es) {
            *votes.entry(cs).or_default().entry(es).or_default() += 1;
        }
    }
    votes
        .into_iter()
        .filter_map(|(c, tally)| {
            let best = tally
                .into_iter()
                .max_by(|a, b| a.1.cmp(&b.1).then(b.0.cmp(a.0)))?;
            Some((c.to_string(), best.0.to_string()))
        })
        .collect()
}

fn usable(cn: &str, en: &str) -> bool {
    let (cn, en) = (cn.trim(), en.trim());
    !cn.is_empty() && !en.is_empty() && cn != en
}

impl TranslationMemory {
    #[must_use]
    pub fn build(cn: &GameModel, en: &GameModel, aligned: &HashMap<String, PoolPair>) -> Self {
        let mut map: HashMap<String, String> = HashMap::new();
        let mut put = |c: &str, e: &str| {
            if usable(c, e) {
                map.entry(c.trim().to_string())
                    .or_insert_with(|| e.trim().to_string());
            }
        };
        for (id, a) in &cn.activities {
            if let Some(b) = en.activities.get(id) {
                put(&a.name, &b.name);
            }
        }
        for (id, a) in &cn.retro_acts {
            if let Some(b) = en.retro_acts.get(id) {
                put(&a.name, &b.name);
            }
        }
        for (id, a) in &cn.skins.char_skins {
            if let Some(b) = en.skins.char_skins.get(id) {
                if let (Some(x), Some(y)) = (&a.display_skin.skin_name, &b.display_skin.skin_name) {
                    put(x, y);
                }
                put(
                    &a.display_skin.skin_group_name,
                    &b.display_skin.skin_group_name,
                );
                if let (Some(x), Some(y)) = (
                    &a.display_skin.obtain_approach,
                    &b.display_skin.obtain_approach,
                ) {
                    put(x, y);
                }
            }
        }
        for (id, a) in &cn.skins.brand_list {
            if let Some(b) = en.skins.brand_list.get(id) {
                put(&a.brand_name, &b.brand_name);
            }
        }
        let en_pools: HashMap<&str, &str> = en
            .gacha
            .gacha_pool_client
            .iter()
            .map(|p| (p.gacha_pool_id.as_str(), p.gacha_pool_name.as_str()))
            .collect();
        for p in &cn.gacha.gacha_pool_client {
            if let Some(pair) = aligned.get(&p.gacha_pool_id)
                && let Some(en_name) = en_pools.get(pair.en_pool_id.as_str())
            {
                put(&p.gacha_pool_name, en_name);
            }
        }
        let series = series_of(&map);
        Self { map, series }
    }

    #[must_use]
    pub fn from_pairs(pairs: &[(&str, &str)]) -> Self {
        let map: HashMap<String, String> = pairs
            .iter()
            .map(|(c, e)| ((*c).to_string(), (*e).to_string()))
            .collect();
        let series = series_of(&map);
        Self { map, series }
    }

    pub fn get(&self, cn: &str) -> Option<&str> {
        self.map.get(cn.trim()).map(String::as_str)
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.map.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}

pub fn resolve(memory: &TranslationMemory, cn: &str) -> Option<AutoName> {
    memory.get(cn).map(AutoName::memory)
}

#[must_use]
pub fn resolve_group(memory: &TranslationMemory, cn: &str) -> Option<AutoName> {
    resolve(memory, cn).or_else(|| {
        let (series, numeral) = split_series(cn.trim());
        let en = memory.series.get(series)?;
        Some(AutoName::memory(
            numeral.map_or_else(|| en.clone(), |n| format!("{en}/{n}")),
        ))
    })
}

#[must_use]
pub fn operator_name(cn: &GameModel, en: &GameModel, char_id: &str) -> Option<AutoName> {
    if let Some(op) = en.operators.get(char_id) {
        return Some(AutoName::memory(op.name.clone()));
    }
    let op = cn.operators.get(char_id)?;
    let app = op.appellation.trim();
    if app.is_empty() {
        return None;
    }
    Some(AutoName {
        text: app.to_string(),
        source: AutoNameSource::Appellation,
    })
}
