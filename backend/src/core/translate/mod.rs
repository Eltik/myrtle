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

#[derive(Debug, Default, Clone)]
pub struct TranslationMemory {
    map: HashMap<String, String>,
}

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
        Self { map }
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
    let en = memory.get(cn.trim())?;
    Some(AutoName {
        text: en.to_string(),
        source: AutoNameSource::Memory,
    })
}

pub fn operator_name(cn: &GameData, en: &GameData, char_id: &str) -> Option<AutoName> {
    if let Some(op) = en.operators.get(char_id) {
        return Some(AutoName {
            text: op.name.clone(),
            source: AutoNameSource::Memory,
        });
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
}
