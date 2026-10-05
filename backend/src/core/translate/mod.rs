use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::core::gamedata::types::GameData;
use crate::core::release::align::PoolPair;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS, utoipa::ToSchema)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum AutoNameSource {
    Memory,
    Appellation,
    Override,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
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
    /// Trimmed CN name -> EN name.
    map: HashMap<String, String>,
    /// Outfit series by CN prefix (斗争血脉 -> Bloodline of Combat), read off the
    /// paired group names, so a CN-only edition reads as its series plus its numeral.
    series: HashMap<String, String>,
}

/// Splits `斗争血脉/XI` into the series and its numeral; a name with no `/` is a series alone.
fn split_series(name: &str) -> (&str, Option<&str>) {
    match name.rsplit_once('/') {
        Some((series, numeral)) => (series, Some(numeral)),
        None => (name, None),
    }
}

/// Each CN series takes the EN prefix most of its paired editions use, ties to
/// the lexically first, so 时代 settles on EPOQUE over the odd Epoque.
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

/// A pair worth remembering: both sides non-blank and the EN side actually differs.
fn usable(cn: &str, en: &str) -> bool {
    let (cn, en) = (cn.trim(), en.trim());
    !cn.is_empty() && !en.is_empty() && cn != en
}

impl TranslationMemory {
    pub fn build(cn: &GameData, en: &GameData, aligned: &HashMap<String, PoolPair>) -> Self {
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

    #[cfg(test)]
    fn from_pairs(pairs: &[(&str, &str)]) -> Self {
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

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}

pub fn resolve(memory: &TranslationMemory, cn: &str) -> Option<AutoName> {
    memory.get(cn).map(AutoName::memory)
}

/// An outfit group's name: the paired name when EN has this edition, else the
/// EN series with the CN numeral (怪物猎人/II -> Monster Hunter/II).
pub fn resolve_group(memory: &TranslationMemory, cn: &str) -> Option<AutoName> {
    resolve(memory, cn).or_else(|| {
        let (series, numeral) = split_series(cn.trim());
        let en = memory.series.get(series)?;
        Some(AutoName::memory(
            numeral.map_or_else(|| en.clone(), |n| format!("{en}/{n}")),
        ))
    })
}

pub fn operator_name(cn: &GameData, en: &GameData, char_id: &str) -> Option<AutoName> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::gamedata::types::activity::ActivityBasicInfo;

    #[test]
    fn memory_pairs_shared_ids_and_skips_identical_or_empty() {
        let mut cn = GameData::new();
        let mut en = GameData::new();
        for (id, c, e) in [
            ("act1", "月行水上", "Moonlight on Water"),
            ("act2", "same", "same"),
            ("act3", "空", ""),
        ] {
            cn.activities.insert(
                id.into(),
                ActivityBasicInfo {
                    id: id.into(),
                    name: c.into(),
                    ..Default::default()
                },
            );
            en.activities.insert(
                id.into(),
                ActivityBasicInfo {
                    id: id.into(),
                    name: e.into(),
                    ..Default::default()
                },
            );
        }
        let m = TranslationMemory::build(&cn, &en, &HashMap::new());
        assert_eq!(m.len(), 1);
        assert_eq!(m.get("月行水上"), Some("Moonlight on Water"));
        assert_eq!(m.get("same"), None);
    }

    #[test]
    fn resolve_answers_only_from_memory() {
        let m = TranslationMemory::build(&GameData::new(), &GameData::new(), &HashMap::new());
        assert_eq!(resolve(&m, "未知名称"), None);
        assert_eq!(resolve(&m, "   "), None);
    }

    #[test]
    fn group_names_borrow_the_series_of_a_paired_edition() {
        let m = TranslationMemory::from_pairs(&[
            ("斗争血脉/VIII", "Bloodline of Combat/VIII"),
            ("怪物猎人", "Monster Hunter"),
            ("0011/韵系列/VIII", "0011/Yun/VIII"),
            ("时代/XXXIII", "Epoque/XXXIII"),
            ("时代/XLVIII", "EPOQUE/XLVIII"),
            ("时代/XLIX", "EPOQUE/XLIX"),
            ("忒斯特收藏/XVI", "Test Collection/XV"),
        ]);
        let text = |cn: &str| resolve_group(&m, cn).map(|a| a.text);
        assert_eq!(
            text("斗争血脉/XI").as_deref(),
            Some("Bloodline of Combat/XI")
        );
        assert_eq!(text("怪物猎人/II").as_deref(), Some("Monster Hunter/II"));
        assert_eq!(text("0011/韵系列/X").as_deref(), Some("0011/Yun/X"));
        assert_eq!(text("时代/LIV").as_deref(), Some("EPOQUE/LIV"));
        assert_eq!(
            text("斗争血脉/VIII").as_deref(),
            Some("Bloodline of Combat/VIII")
        );
        // A pair whose numerals disagree says nothing about the series.
        assert_eq!(text("忒斯特收藏/XX"), None);
        assert_eq!(text("小马宝莉"), None);
    }
}
