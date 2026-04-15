//! Auto-generated FlatBufferToJson implementations
//! DO NOT EDIT - regenerate with: cargo run --bin generate-fbs

#![allow(unused_imports, unused_variables)]

use crate::fb_json_macros::{EnumToJson, FlatBufferToJson};
use serde_json::{Map, Value, json};
use std::panic::{self, AssertUnwindSafe};

use crate::generated_fbs_yostar::battle_equip_table_generated;
use crate::generated_fbs_yostar::character_table_generated;
use crate::generated_fbs_yostar::ep_breakbuff_table_generated;
use crate::generated_fbs_yostar::token_table_generated;

// ============ Enum Implementations ============

// From battle_equip_table_generated
impl EnumToJson for battle_equip_table_generated::enum__Torappu_UniEquipTarget {
    fn to_json_value(&self) -> Value {
        match self.variant_name() {
            Some(name) => json!(name),
            None => json!(format!("UNKNOWN_{}", self.0)),
        }
    }
}

impl EnumToJson for battle_equip_table_generated::enum__Torappu_EvolvePhase {
    fn to_json_value(&self) -> Value {
        match self.variant_name() {
            Some(name) => json!(name),
            None => json!(format!("UNKNOWN_{}", self.0)),
        }
    }
}

// From character_table_generated
impl EnumToJson for character_table_generated::enum__Torappu_SpecialOperatorTargetType {
    fn to_json_value(&self) -> Value {
        match self.variant_name() {
            Some(name) => json!(name),
            None => json!(format!("UNKNOWN_{}", self.0)),
        }
    }
}

impl EnumToJson for character_table_generated::enum__Torappu_BuildableType {
    fn to_json_value(&self) -> Value {
        match self.variant_name() {
            Some(name) => json!(name),
            None => json!(format!("UNKNOWN_{}", self.0)),
        }
    }
}

impl EnumToJson for character_table_generated::enum__Torappu_RarityRank {
    fn to_json_value(&self) -> Value {
        match self.variant_name() {
            Some(name) => json!(name),
            None => json!(format!("UNKNOWN_{}", self.0)),
        }
    }
}

impl EnumToJson for character_table_generated::enum__Torappu_ProfessionCategory {
    fn to_json_value(&self) -> Value {
        match self.variant_name() {
            Some(name) => json!(name),
            None => json!(format!("UNKNOWN_{}", self.0)),
        }
    }
}

impl EnumToJson for character_table_generated::enum__Torappu_EvolvePhase {
    fn to_json_value(&self) -> Value {
        match self.variant_name() {
            Some(name) => json!(name),
            None => json!(format!("UNKNOWN_{}", self.0)),
        }
    }
}

impl EnumToJson for character_table_generated::enum__Torappu_ItemType {
    fn to_json_value(&self) -> Value {
        match self.variant_name() {
            Some(name) => json!(name),
            None => json!(format!("UNKNOWN_{}", self.0)),
        }
    }
}

impl EnumToJson for character_table_generated::enum__Torappu_CharacterData_PotentialRank_TypeEnum {
    fn to_json_value(&self) -> Value {
        match self.variant_name() {
            Some(name) => json!(name),
            None => json!(format!("UNKNOWN_{}", self.0)),
        }
    }
}

impl EnumToJson for character_table_generated::enum__Torappu_AbnormalFlag {
    fn to_json_value(&self) -> Value {
        match self.variant_name() {
            Some(name) => json!(name),
            None => json!(format!("UNKNOWN_{}", self.0)),
        }
    }
}

impl EnumToJson for character_table_generated::enum__Torappu_AbnormalCombo {
    fn to_json_value(&self) -> Value {
        match self.variant_name() {
            Some(name) => json!(name),
            None => json!(format!("UNKNOWN_{}", self.0)),
        }
    }
}

impl EnumToJson for character_table_generated::enum__Torappu_AttributeType {
    fn to_json_value(&self) -> Value {
        match self.variant_name() {
            Some(name) => json!(name),
            None => json!(format!("UNKNOWN_{}", self.0)),
        }
    }
}

impl EnumToJson for character_table_generated::enum__Torappu_AttributeModifierData_AttributeModifier_FormulaItemType {
    fn to_json_value(&self) -> Value {
        match self.variant_name() {
            Some(name) => json!(name),
            None => json!(format!("UNKNOWN_{}", self.0)),
        }
    }
}

// From token_table_generated
impl EnumToJson for token_table_generated::enum__Torappu_SpecialOperatorTargetType {
    fn to_json_value(&self) -> Value {
        match self.variant_name() {
            Some(name) => json!(name),
            None => json!(format!("UNKNOWN_{}", self.0)),
        }
    }
}

impl EnumToJson for token_table_generated::enum__Torappu_BuildableType {
    fn to_json_value(&self) -> Value {
        match self.variant_name() {
            Some(name) => json!(name),
            None => json!(format!("UNKNOWN_{}", self.0)),
        }
    }
}

impl EnumToJson for token_table_generated::enum__Torappu_RarityRank {
    fn to_json_value(&self) -> Value {
        match self.variant_name() {
            Some(name) => json!(name),
            None => json!(format!("UNKNOWN_{}", self.0)),
        }
    }
}

impl EnumToJson for token_table_generated::enum__Torappu_ProfessionCategory {
    fn to_json_value(&self) -> Value {
        match self.variant_name() {
            Some(name) => json!(name),
            None => json!(format!("UNKNOWN_{}", self.0)),
        }
    }
}

impl EnumToJson for token_table_generated::enum__Torappu_EvolvePhase {
    fn to_json_value(&self) -> Value {
        match self.variant_name() {
            Some(name) => json!(name),
            None => json!(format!("UNKNOWN_{}", self.0)),
        }
    }
}

impl EnumToJson for token_table_generated::enum__Torappu_ItemType {
    fn to_json_value(&self) -> Value {
        match self.variant_name() {
            Some(name) => json!(name),
            None => json!(format!("UNKNOWN_{}", self.0)),
        }
    }
}

impl EnumToJson for token_table_generated::enum__Torappu_CharacterData_PotentialRank_TypeEnum {
    fn to_json_value(&self) -> Value {
        match self.variant_name() {
            Some(name) => json!(name),
            None => json!(format!("UNKNOWN_{}", self.0)),
        }
    }
}

impl EnumToJson for token_table_generated::enum__Torappu_AbnormalFlag {
    fn to_json_value(&self) -> Value {
        match self.variant_name() {
            Some(name) => json!(name),
            None => json!(format!("UNKNOWN_{}", self.0)),
        }
    }
}

impl EnumToJson for token_table_generated::enum__Torappu_AbnormalCombo {
    fn to_json_value(&self) -> Value {
        match self.variant_name() {
            Some(name) => json!(name),
            None => json!(format!("UNKNOWN_{}", self.0)),
        }
    }
}

impl EnumToJson for token_table_generated::enum__Torappu_AttributeType {
    fn to_json_value(&self) -> Value {
        match self.variant_name() {
            Some(name) => json!(name),
            None => json!(format!("UNKNOWN_{}", self.0)),
        }
    }
}

impl EnumToJson
    for token_table_generated::enum__Torappu_AttributeModifierData_AttributeModifier_FormulaItemType
{
    fn to_json_value(&self) -> Value {
        match self.variant_name() {
            Some(name) => json!(name),
            None => json!(format!("UNKNOWN_{}", self.0)),
        }
    }
}

// ============ Struct Implementations ============

// From battle_equip_table_generated
impl FlatBufferToJson
    for battle_equip_table_generated::clz_Torappu_CharacterData_UnlockCondition<'_>
{
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("Phase".to_string(), self.phase().to_json_value()));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("Level".to_string(), json!(self.level())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        Value::Object(map)
    }
}

impl FlatBufferToJson for battle_equip_table_generated::clz_Torappu_Blackboard_DataPair<'_> {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        if let Some(k) = self.key() {
            map.insert("key".to_string(), json!(k));
        }
        map.insert("value".to_string(), json!(self.value()));
        Value::Object(map)
    }
}

impl FlatBufferToJson for battle_equip_table_generated::clz_Torappu_EquipTalentData<'_> {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("DisplayRangeId".to_string(), json!(self.displayRangeId())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.upgradeDescription() {
                return Some(("UpgradeDescription".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("TalentIndex".to_string(), json!(self.talentIndex())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(nested) = self.unlockCondition() {
                return Some(("UnlockCondition".to_string(), nested.to_json()));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some((
                "RequiredPotentialRank".to_string(),
                json!(self.requiredPotentialRank()),
            ));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.prefabKey() {
                return Some(("PrefabKey".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.name() {
                return Some(("Name".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.description() {
                return Some(("Description".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.rangeId() {
                return Some(("RangeId".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(vec) = self.blackboard() {
                assert!(vec.len() <= 10_000_000, "FB vector too large");
                let arr: Vec<Value> = (0..vec.len())
                    .filter_map(|i| {
                        panic::catch_unwind(AssertUnwindSafe(|| vec.get(i).to_json())).ok()
                    })
                    .collect();
                return Some(("Blackboard".to_string(), json!(arr)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.tokenKey() {
                return Some(("TokenKey".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("IsHideTalent".to_string(), json!(self.isHideTalent())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        Value::Object(map)
    }
}

impl FlatBufferToJson for battle_equip_table_generated::clz_Torappu_TalentData<'_> {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(nested) = self.unlockCondition() {
                return Some(("UnlockCondition".to_string(), nested.to_json()));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some((
                "RequiredPotentialRank".to_string(),
                json!(self.requiredPotentialRank()),
            ));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.prefabKey() {
                return Some(("PrefabKey".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.name() {
                return Some(("Name".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.description() {
                return Some(("Description".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.rangeId() {
                return Some(("RangeId".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(vec) = self.blackboard() {
                assert!(vec.len() <= 10_000_000, "FB vector too large");
                let arr: Vec<Value> = (0..vec.len())
                    .filter_map(|i| {
                        panic::catch_unwind(AssertUnwindSafe(|| vec.get(i).to_json())).ok()
                    })
                    .collect();
                return Some(("Blackboard".to_string(), json!(arr)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.tokenKey() {
                return Some(("TokenKey".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("IsHideTalent".to_string(), json!(self.isHideTalent())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        Value::Object(map)
    }
}

impl FlatBufferToJson
    for battle_equip_table_generated::clz_Torappu_CharacterData_EquipTalentDataBundle<'_>
{
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(vec) = self.candidates() {
                assert!(vec.len() <= 10_000_000, "FB vector too large");
                let arr: Vec<Value> = (0..vec.len())
                    .filter_map(|i| {
                        panic::catch_unwind(AssertUnwindSafe(|| vec.get(i).to_json())).ok()
                    })
                    .collect();
                return Some(("Candidates".to_string(), json!(arr)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        Value::Object(map)
    }
}

impl FlatBufferToJson
    for battle_equip_table_generated::clz_Torappu_CharacterData_EquipTraitData<'_>
{
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.additionalDescription() {
                return Some(("AdditionalDescription".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(nested) = self.unlockCondition() {
                return Some(("UnlockCondition".to_string(), nested.to_json()));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some((
                "RequiredPotentialRank".to_string(),
                json!(self.requiredPotentialRank()),
            ));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(vec) = self.blackboard() {
                assert!(vec.len() <= 10_000_000, "FB vector too large");
                let arr: Vec<Value> = (0..vec.len())
                    .filter_map(|i| {
                        panic::catch_unwind(AssertUnwindSafe(|| vec.get(i).to_json())).ok()
                    })
                    .collect();
                return Some(("Blackboard".to_string(), json!(arr)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.overrideDescripton() {
                return Some(("OverrideDescripton".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.prefabKey() {
                return Some(("PrefabKey".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.rangeId() {
                return Some(("RangeId".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        Value::Object(map)
    }
}

impl FlatBufferToJson
    for battle_equip_table_generated::clz_Torappu_CharacterData_EquipTraitDataBundle<'_>
{
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(vec) = self.candidates() {
                assert!(vec.len() <= 10_000_000, "FB vector too large");
                let arr: Vec<Value> = (0..vec.len())
                    .filter_map(|i| {
                        panic::catch_unwind(AssertUnwindSafe(|| vec.get(i).to_json())).ok()
                    })
                    .collect();
                return Some(("Candidates".to_string(), json!(arr)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        Value::Object(map)
    }
}

impl FlatBufferToJson for battle_equip_table_generated::clz_Torappu_BattleUniEquipData<'_> {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.resKey() {
                return Some(("ResKey".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("Target".to_string(), self.target().to_json_value()));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("IsToken".to_string(), json!(self.isToken())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.validInGameTag() {
                return Some(("ValidInGameTag".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.validInMapTag() {
                return Some(("ValidInMapTag".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(nested) = self.addOrOverrideTalentDataBundle() {
                return Some((
                    "AddOrOverrideTalentDataBundle".to_string(),
                    nested.to_json(),
                ));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(nested) = self.overrideTraitDataBundle() {
                return Some(("OverrideTraitDataBundle".to_string(), nested.to_json()));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        Value::Object(map)
    }
}

impl FlatBufferToJson
    for battle_equip_table_generated::dict__string__list_clz_Torappu_Blackboard_DataPair<'_>
{
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        if let Ok(k) = panic::catch_unwind(AssertUnwindSafe(|| self.key())) {
            map.insert("key".to_string(), json!(k));
        }
        if let Some(vec) = self.value() {
            assert!(vec.len() <= 10_000_000, "FB vector too large");
            let arr: Vec<Value> = (0..vec.len()).map(|i| vec.get(i).to_json()).collect();
            map.insert("value".to_string(), json!(arr));
        }
        Value::Object(map)
    }
}

impl FlatBufferToJson for battle_equip_table_generated::clz_Torappu_BattleEquipPerLevelPack<'_> {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("EquipLevel".to_string(), json!(self.equipLevel())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(vec) = self.parts() {
                assert!(vec.len() <= 10_000_000, "FB vector too large");
                let arr: Vec<Value> = (0..vec.len())
                    .filter_map(|i| {
                        panic::catch_unwind(AssertUnwindSafe(|| vec.get(i).to_json())).ok()
                    })
                    .collect();
                return Some(("Parts".to_string(), json!(arr)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(vec) = self.attributeBlackboard() {
                assert!(vec.len() <= 10_000_000, "FB vector too large");
                let arr: Vec<Value> = (0..vec.len())
                    .filter_map(|i| {
                        panic::catch_unwind(AssertUnwindSafe(|| vec.get(i).to_json())).ok()
                    })
                    .collect();
                return Some(("AttributeBlackboard".to_string(), json!(arr)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(vec) = self.tokenAttributeBlackboard() {
                assert!(vec.len() <= 10_000_000, "FB vector too large");
                let arr: Vec<Value> = (0..vec.len())
                    .filter_map(|i| {
                        panic::catch_unwind(AssertUnwindSafe(|| vec.get(i).to_json())).ok()
                    })
                    .collect();
                return Some(("TokenAttributeBlackboard".to_string(), json!(arr)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        Value::Object(map)
    }
}

impl FlatBufferToJson for battle_equip_table_generated::clz_Torappu_BattleEquipPack<'_> {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(vec) = self.phases() {
                assert!(vec.len() <= 10_000_000, "FB vector too large");
                let arr: Vec<Value> = (0..vec.len())
                    .filter_map(|i| {
                        panic::catch_unwind(AssertUnwindSafe(|| vec.get(i).to_json())).ok()
                    })
                    .collect();
                return Some(("Phases".to_string(), json!(arr)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        Value::Object(map)
    }
}

impl FlatBufferToJson
    for battle_equip_table_generated::dict__string__clz_Torappu_BattleEquipPack<'_>
{
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        if let Ok(k) = panic::catch_unwind(AssertUnwindSafe(|| self.key())) {
            map.insert("key".to_string(), json!(k));
        }
        if let Some(v) = self.value() {
            map.insert("value".to_string(), v.to_json());
        }
        Value::Object(map)
    }
}

impl FlatBufferToJson
    for battle_equip_table_generated::clz_Torappu_SimpleKVTable_clz_Torappu_BattleEquipPack<'_>
{
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(vec) = self.equips() {
                assert!(vec.len() <= 10_000_000, "FB vector too large");
                let arr: Vec<Value> = (0..vec.len())
                    .filter_map(|i| {
                        panic::catch_unwind(AssertUnwindSafe(|| vec.get(i).to_json())).ok()
                    })
                    .collect();
                return Some(("Equips".to_string(), json!(arr)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        Value::Object(map)
    }
}

// From character_table_generated
impl FlatBufferToJson for character_table_generated::clz_Torappu_CharacterData_PowerData<'_> {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.nationId() {
                return Some(("NationId".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.groupId() {
                return Some(("GroupId".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.teamId() {
                return Some(("TeamId".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        Value::Object(map)
    }
}

impl FlatBufferToJson for character_table_generated::clz_Torappu_CharacterData_UnlockCondition<'_> {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("Phase".to_string(), self.phase().to_json_value()));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("Level".to_string(), json!(self.level())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        Value::Object(map)
    }
}

impl FlatBufferToJson for character_table_generated::clz_Torappu_Blackboard_DataPair<'_> {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        if let Some(k) = self.key() {
            map.insert("key".to_string(), json!(k));
        }
        map.insert("value".to_string(), json!(self.value()));
        Value::Object(map)
    }
}

impl FlatBufferToJson for character_table_generated::clz_Torappu_CharacterData_TraitData<'_> {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(nested) = self.unlockCondition() {
                return Some(("UnlockCondition".to_string(), nested.to_json()));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some((
                "RequiredPotentialRank".to_string(),
                json!(self.requiredPotentialRank()),
            ));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(vec) = self.blackboard() {
                assert!(vec.len() <= 10_000_000, "FB vector too large");
                let arr: Vec<Value> = (0..vec.len())
                    .filter_map(|i| {
                        panic::catch_unwind(AssertUnwindSafe(|| vec.get(i).to_json())).ok()
                    })
                    .collect();
                return Some(("Blackboard".to_string(), json!(arr)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.overrideDescripton() {
                return Some(("OverrideDescripton".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.prefabKey() {
                return Some(("PrefabKey".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.rangeId() {
                return Some(("RangeId".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        Value::Object(map)
    }
}

impl FlatBufferToJson for character_table_generated::clz_Torappu_CharacterData_TraitDataBundle<'_> {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(vec) = self.candidates() {
                assert!(vec.len() <= 10_000_000, "FB vector too large");
                let arr: Vec<Value> = (0..vec.len())
                    .filter_map(|i| {
                        panic::catch_unwind(AssertUnwindSafe(|| vec.get(i).to_json())).ok()
                    })
                    .collect();
                return Some(("Candidates".to_string(), json!(arr)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        Value::Object(map)
    }
}

impl FlatBufferToJson for character_table_generated::clz_Torappu_AttributesData<'_> {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("MaxHp".to_string(), json!(self.maxHp())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("Atk".to_string(), json!(self.atk())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("Def".to_string(), json!(self.def())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("MagicResistance".to_string(), json!(self.magicResistance())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("Cost".to_string(), json!(self.cost())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("BlockCnt".to_string(), json!(self.blockCnt())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("MoveSpeed".to_string(), json!(self.moveSpeed())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("AttackSpeed".to_string(), json!(self.attackSpeed())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("BaseAttackTime".to_string(), json!(self.baseAttackTime())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("RespawnTime".to_string(), json!(self.respawnTime())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some((
                "HpRecoveryPerSec".to_string(),
                json!(self.hpRecoveryPerSec()),
            ));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some((
                "SpRecoveryPerSec".to_string(),
                json!(self.spRecoveryPerSec()),
            ));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("MaxDeployCount".to_string(), json!(self.maxDeployCount())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("MaxDeckStackCnt".to_string(), json!(self.maxDeckStackCnt())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("TauntLevel".to_string(), json!(self.tauntLevel())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("MassLevel".to_string(), json!(self.massLevel())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("BaseForceLevel".to_string(), json!(self.baseForceLevel())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("StunImmune".to_string(), json!(self.stunImmune())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("SilenceImmune".to_string(), json!(self.silenceImmune())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("SleepImmune".to_string(), json!(self.sleepImmune())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("FrozenImmune".to_string(), json!(self.frozenImmune())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("LevitateImmune".to_string(), json!(self.levitateImmune())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some((
                "DisarmedCombatImmune".to_string(),
                json!(self.disarmedCombatImmune()),
            ));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("FearedImmune".to_string(), json!(self.fearedImmune())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("PalsyImmune".to_string(), json!(self.palsyImmune())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("AttractImmune".to_string(), json!(self.attractImmune())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        Value::Object(map)
    }
}

impl FlatBufferToJson for character_table_generated::clz_Torappu_KeyFrames_2_KeyFrame_Torappu_AttributesData_Torappu_AttributesData_<'_> {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("Level".to_string(), json!(self.level())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(nested) = self.data() {
                return Some(("Data".to_string(), nested.to_json()));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        Value::Object(map)
    }
}

impl FlatBufferToJson for character_table_generated::clz_Torappu_ItemBundle<'_> {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.id() {
                return Some(("Id".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("Count".to_string(), json!(self.count())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("Type_".to_string(), self.type_().to_json_value()));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        Value::Object(map)
    }
}

impl FlatBufferToJson for character_table_generated::clz_Torappu_CharacterData_PhaseData<'_> {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.characterPrefabKey() {
                return Some(("CharacterPrefabKey".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.rangeId() {
                return Some(("RangeId".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("MaxLevel".to_string(), json!(self.maxLevel())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(vec) = self.attributesKeyFrames() {
                assert!(vec.len() <= 10_000_000, "FB vector too large");
                let arr: Vec<Value> = (0..vec.len())
                    .filter_map(|i| {
                        panic::catch_unwind(AssertUnwindSafe(|| vec.get(i).to_json())).ok()
                    })
                    .collect();
                return Some(("AttributesKeyFrames".to_string(), json!(arr)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(vec) = self.evolveCost() {
                assert!(vec.len() <= 10_000_000, "FB vector too large");
                let arr: Vec<Value> = (0..vec.len())
                    .filter_map(|i| {
                        panic::catch_unwind(AssertUnwindSafe(|| vec.get(i).to_json())).ok()
                    })
                    .collect();
                return Some(("EvolveCost".to_string(), json!(arr)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        Value::Object(map)
    }
}

impl FlatBufferToJson
    for character_table_generated::clz_Torappu_CharacterData_MainSkill_SpecializeLevelData<'_>
{
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(nested) = self.unlockCond() {
                return Some(("UnlockCond".to_string(), nested.to_json()));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("LvlUpTime".to_string(), json!(self.lvlUpTime())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(vec) = self.levelUpCost() {
                assert!(vec.len() <= 10_000_000, "FB vector too large");
                let arr: Vec<Value> = (0..vec.len())
                    .filter_map(|i| {
                        panic::catch_unwind(AssertUnwindSafe(|| vec.get(i).to_json())).ok()
                    })
                    .collect();
                return Some(("LevelUpCost".to_string(), json!(arr)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        Value::Object(map)
    }
}

impl FlatBufferToJson for character_table_generated::clz_Torappu_CharacterData_MainSkill<'_> {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.skillId() {
                return Some(("SkillId".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.overridePrefabKey() {
                return Some(("OverridePrefabKey".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.overrideTokenKey() {
                return Some(("OverrideTokenKey".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(vec) = self.levelUpCostCond() {
                assert!(vec.len() <= 10_000_000, "FB vector too large");
                let arr: Vec<Value> = (0..vec.len())
                    .filter_map(|i| {
                        panic::catch_unwind(AssertUnwindSafe(|| vec.get(i).to_json())).ok()
                    })
                    .collect();
                return Some(("LevelUpCostCond".to_string(), json!(arr)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(nested) = self.unlockCond() {
                return Some(("UnlockCond".to_string(), nested.to_json()));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        Value::Object(map)
    }
}

impl FlatBufferToJson for character_table_generated::dict__string__bool<'_> {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        if let Ok(k) = panic::catch_unwind(AssertUnwindSafe(|| self.key())) {
            map.insert("key".to_string(), json!(k));
        }
        map.insert("value".to_string(), json!(self.value()));
        Value::Object(map)
    }
}

impl FlatBufferToJson for character_table_generated::clz_Torappu_TalentData<'_> {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(nested) = self.unlockCondition() {
                return Some(("UnlockCondition".to_string(), nested.to_json()));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some((
                "RequiredPotentialRank".to_string(),
                json!(self.requiredPotentialRank()),
            ));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.prefabKey() {
                return Some(("PrefabKey".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.name() {
                return Some(("Name".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.description() {
                return Some(("Description".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.rangeId() {
                return Some(("RangeId".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(vec) = self.blackboard() {
                assert!(vec.len() <= 10_000_000, "FB vector too large");
                let arr: Vec<Value> = (0..vec.len())
                    .filter_map(|i| {
                        panic::catch_unwind(AssertUnwindSafe(|| vec.get(i).to_json())).ok()
                    })
                    .collect();
                return Some(("Blackboard".to_string(), json!(arr)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.tokenKey() {
                return Some(("TokenKey".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("IsHideTalent".to_string(), json!(self.isHideTalent())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        Value::Object(map)
    }
}

impl FlatBufferToJson
    for character_table_generated::clz_Torappu_CharacterData_TalentDataBundle<'_>
{
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(vec) = self.candidates() {
                assert!(vec.len() <= 10_000_000, "FB vector too large");
                let arr: Vec<Value> = (0..vec.len())
                    .filter_map(|i| {
                        panic::catch_unwind(AssertUnwindSafe(|| vec.get(i).to_json())).ok()
                    })
                    .collect();
                return Some(("Candidates".to_string(), json!(arr)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        Value::Object(map)
    }
}

impl FlatBufferToJson
    for character_table_generated::clz_Torappu_AttributeModifierData_AttributeModifier<'_>
{
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some((
                "AttributeType".to_string(),
                self.attributeType().to_json_value(),
            ));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some((
                "FormulaItem".to_string(),
                self.formulaItem().to_json_value(),
            ));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("Value".to_string(), json!(self.value())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some((
                "LoadFromBlackboard".to_string(),
                json!(self.loadFromBlackboard()),
            ));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some((
                "FetchBaseValueFromSourceEntity".to_string(),
                json!(self.fetchBaseValueFromSourceEntity()),
            ));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        Value::Object(map)
    }
}

impl FlatBufferToJson for character_table_generated::clz_Torappu_AttributeModifierData<'_> {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(vec) = self.abnormalFlags() {
                assert!(vec.len() <= 10_000_000, "FB vector too large");
                let arr: Vec<Value> = vec.iter().map(|e| e.to_json_value()).collect();
                return Some(("AbnormalFlags".to_string(), json!(arr)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(vec) = self.abnormalImmunes() {
                assert!(vec.len() <= 10_000_000, "FB vector too large");
                let arr: Vec<Value> = vec.iter().map(|e| e.to_json_value()).collect();
                return Some(("AbnormalImmunes".to_string(), json!(arr)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(vec) = self.abnormalAntis() {
                assert!(vec.len() <= 10_000_000, "FB vector too large");
                let arr: Vec<Value> = vec.iter().map(|e| e.to_json_value()).collect();
                return Some(("AbnormalAntis".to_string(), json!(arr)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(vec) = self.abnormalCombos() {
                assert!(vec.len() <= 10_000_000, "FB vector too large");
                let arr: Vec<Value> = vec.iter().map(|e| e.to_json_value()).collect();
                return Some(("AbnormalCombos".to_string(), json!(arr)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(vec) = self.abnormalComboImmunes() {
                assert!(vec.len() <= 10_000_000, "FB vector too large");
                let arr: Vec<Value> = vec.iter().map(|e| e.to_json_value()).collect();
                return Some(("AbnormalComboImmunes".to_string(), json!(arr)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(vec) = self.attributeModifiers() {
                assert!(vec.len() <= 10_000_000, "FB vector too large");
                let arr: Vec<Value> = (0..vec.len())
                    .filter_map(|i| {
                        panic::catch_unwind(AssertUnwindSafe(|| vec.get(i).to_json())).ok()
                    })
                    .collect();
                return Some(("AttributeModifiers".to_string(), json!(arr)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        Value::Object(map)
    }
}

impl FlatBufferToJson for character_table_generated::clz_Torappu_ExternalBuff<'_> {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(nested) = self.attributes() {
                return Some(("Attributes".to_string(), nested.to_json()));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        Value::Object(map)
    }
}

impl FlatBufferToJson for character_table_generated::clz_Torappu_CharacterData_PotentialRank<'_> {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("Type_".to_string(), self.type_().to_json_value()));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.description() {
                return Some(("Description".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(nested) = self.buff() {
                return Some(("Buff".to_string(), nested.to_json()));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(vec) = self.equivalentCost() {
                assert!(vec.len() <= 10_000_000, "FB vector too large");
                let arr: Vec<Value> = (0..vec.len())
                    .filter_map(|i| {
                        panic::catch_unwind(AssertUnwindSafe(|| vec.get(i).to_json())).ok()
                    })
                    .collect();
                return Some(("EquivalentCost".to_string(), json!(arr)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        Value::Object(map)
    }
}

impl FlatBufferToJson for character_table_generated::clz_Torappu_AttributesDeltaData<'_> {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("MaxHp".to_string(), json!(self.maxHp())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("Atk".to_string(), json!(self.atk())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("Def".to_string(), json!(self.def())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("MagicResistance".to_string(), json!(self.magicResistance())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("Cost".to_string(), json!(self.cost())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("BlockCnt".to_string(), json!(self.blockCnt())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("MoveSpeed".to_string(), json!(self.moveSpeed())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("AttackSpeed".to_string(), json!(self.attackSpeed())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("BaseAttackTime".to_string(), json!(self.baseAttackTime())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("RespawnTime".to_string(), json!(self.respawnTime())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some((
                "HpRecoveryPerSec".to_string(),
                json!(self.hpRecoveryPerSec()),
            ));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some((
                "SpRecoveryPerSec".to_string(),
                json!(self.spRecoveryPerSec()),
            ));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("MaxDeployCount".to_string(), json!(self.maxDeployCount())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("MaxDeckStackCnt".to_string(), json!(self.maxDeckStackCnt())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("TauntLevel".to_string(), json!(self.tauntLevel())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("MassLevel".to_string(), json!(self.massLevel())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("BaseForceLevel".to_string(), json!(self.baseForceLevel())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("StunImmune".to_string(), json!(self.stunImmune())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("SilenceImmune".to_string(), json!(self.silenceImmune())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("SleepImmune".to_string(), json!(self.sleepImmune())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("FrozenImmune".to_string(), json!(self.frozenImmune())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("LevitateImmune".to_string(), json!(self.levitateImmune())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some((
                "DisarmedCombatImmune".to_string(),
                json!(self.disarmedCombatImmune()),
            ));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("FearedImmune".to_string(), json!(self.fearedImmune())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("PalsyImmune".to_string(), json!(self.palsyImmune())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("AttractImmune".to_string(), json!(self.attractImmune())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        Value::Object(map)
    }
}

impl FlatBufferToJson for character_table_generated::clz_Torappu_KeyFrames_2_KeyFrame_Torappu_AttributesDeltaData_Torappu_AttributesData_<'_> {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("Level".to_string(), json!(self.level())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(nested) = self.data() {
                return Some(("Data".to_string(), nested.to_json()));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        Value::Object(map)
    }
}

impl FlatBufferToJson for character_table_generated::clz_Torappu_CharacterData_SkillLevelCost<'_> {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(nested) = self.unlockCond() {
                return Some(("UnlockCond".to_string(), nested.to_json()));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(vec) = self.lvlUpCost() {
                assert!(vec.len() <= 10_000_000, "FB vector too large");
                let arr: Vec<Value> = (0..vec.len())
                    .filter_map(|i| {
                        panic::catch_unwind(AssertUnwindSafe(|| vec.get(i).to_json())).ok()
                    })
                    .collect();
                return Some(("LvlUpCost".to_string(), json!(arr)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        Value::Object(map)
    }
}

impl FlatBufferToJson for character_table_generated::clz_Torappu_CharacterData<'_> {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.name() {
                return Some(("Name".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.description() {
                return Some(("Description".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("SortIndex".to_string(), json!(self.sortIndex())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some((
                "SpTargetType".to_string(),
                self.spTargetType().to_json_value(),
            ));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.spTargetId() {
                return Some(("SpTargetId".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some((
                "CanUseGeneralPotentialItem".to_string(),
                json!(self.canUseGeneralPotentialItem()),
            ));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some((
                "CanUseActivityPotentialItem".to_string(),
                json!(self.canUseActivityPotentialItem()),
            ));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.potentialItemId() {
                return Some(("PotentialItemId".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.activityPotentialItemId() {
                return Some(("ActivityPotentialItemId".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.classicPotentialItemId() {
                return Some(("ClassicPotentialItemId".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.nationId() {
                return Some(("NationId".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.groupId() {
                return Some(("GroupId".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.teamId() {
                return Some(("TeamId".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(nested) = self.mainPower() {
                return Some(("MainPower".to_string(), nested.to_json()));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(vec) = self.subPower() {
                assert!(vec.len() <= 10_000_000, "FB vector too large");
                let arr: Vec<Value> = (0..vec.len())
                    .filter_map(|i| {
                        panic::catch_unwind(AssertUnwindSafe(|| vec.get(i).to_json())).ok()
                    })
                    .collect();
                return Some(("SubPower".to_string(), json!(arr)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.displayNumber() {
                return Some(("DisplayNumber".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.appellation() {
                return Some(("Appellation".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("Position".to_string(), self.position().to_json_value()));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(vec) = self.tagList() {
                assert!(vec.len() <= 10_000_000, "FB vector too large");
                let arr: Vec<Value> = (0..vec.len()).map(|i| json!(vec.get(i))).collect();
                return Some(("TagList".to_string(), json!(arr)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.itemUsage() {
                return Some(("ItemUsage".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.itemDesc() {
                return Some(("ItemDesc".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.itemObtainApproach() {
                return Some(("ItemObtainApproach".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("IsNotObtainable".to_string(), json!(self.isNotObtainable())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("IsSpChar".to_string(), json!(self.isSpChar())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some((
                "MaxPotentialLevel".to_string(),
                json!(self.maxPotentialLevel()),
            ));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("Rarity".to_string(), self.rarity().to_json_value()));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("Profession".to_string(), self.profession().to_json_value()));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.subProfessionId() {
                return Some(("SubProfessionId".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(nested) = self.trait_() {
                return Some(("Trait_".to_string(), nested.to_json()));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(vec) = self.phases() {
                assert!(vec.len() <= 10_000_000, "FB vector too large");
                let arr: Vec<Value> = (0..vec.len())
                    .filter_map(|i| {
                        panic::catch_unwind(AssertUnwindSafe(|| vec.get(i).to_json())).ok()
                    })
                    .collect();
                return Some(("Phases".to_string(), json!(arr)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(vec) = self.skills() {
                assert!(vec.len() <= 10_000_000, "FB vector too large");
                let arr: Vec<Value> = (0..vec.len())
                    .filter_map(|i| {
                        panic::catch_unwind(AssertUnwindSafe(|| vec.get(i).to_json())).ok()
                    })
                    .collect();
                return Some(("Skills".to_string(), json!(arr)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(vec) = self.displayTokenDict() {
                assert!(vec.len() <= 10_000_000, "FB vector too large");
                let arr: Vec<Value> = (0..vec.len())
                    .filter_map(|i| {
                        panic::catch_unwind(AssertUnwindSafe(|| vec.get(i).to_json())).ok()
                    })
                    .collect();
                return Some(("DisplayTokenDict".to_string(), json!(arr)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(vec) = self.talents() {
                assert!(vec.len() <= 10_000_000, "FB vector too large");
                let arr: Vec<Value> = (0..vec.len())
                    .filter_map(|i| {
                        panic::catch_unwind(AssertUnwindSafe(|| vec.get(i).to_json())).ok()
                    })
                    .collect();
                return Some(("Talents".to_string(), json!(arr)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(vec) = self.potentialRanks() {
                assert!(vec.len() <= 10_000_000, "FB vector too large");
                let arr: Vec<Value> = (0..vec.len())
                    .filter_map(|i| {
                        panic::catch_unwind(AssertUnwindSafe(|| vec.get(i).to_json())).ok()
                    })
                    .collect();
                return Some(("PotentialRanks".to_string(), json!(arr)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(vec) = self.favorKeyFrames() {
                assert!(vec.len() <= 10_000_000, "FB vector too large");
                let arr: Vec<Value> = (0..vec.len())
                    .filter_map(|i| {
                        panic::catch_unwind(AssertUnwindSafe(|| vec.get(i).to_json())).ok()
                    })
                    .collect();
                return Some(("FavorKeyFrames".to_string(), json!(arr)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(vec) = self.allSkillLvlup() {
                assert!(vec.len() <= 10_000_000, "FB vector too large");
                let arr: Vec<Value> = (0..vec.len())
                    .filter_map(|i| {
                        panic::catch_unwind(AssertUnwindSafe(|| vec.get(i).to_json())).ok()
                    })
                    .collect();
                return Some(("AllSkillLvlup".to_string(), json!(arr)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        Value::Object(map)
    }
}

impl FlatBufferToJson for character_table_generated::dict__string__clz_Torappu_CharacterData<'_> {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        if let Ok(k) = panic::catch_unwind(AssertUnwindSafe(|| self.key())) {
            map.insert("key".to_string(), json!(k));
        }
        if let Some(v) = self.value() {
            map.insert("value".to_string(), v.to_json());
        }
        Value::Object(map)
    }
}

impl FlatBufferToJson
    for character_table_generated::clz_Torappu_SimpleKVTable_clz_Torappu_CharacterData<'_>
{
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(vec) = self.characters() {
                assert!(vec.len() <= 10_000_000, "FB vector too large");
                let arr: Vec<Value> = (0..vec.len())
                    .filter_map(|i| {
                        panic::catch_unwind(AssertUnwindSafe(|| vec.get(i).to_json())).ok()
                    })
                    .collect();
                return Some(("Characters".to_string(), json!(arr)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        Value::Object(map)
    }
}

// From ep_breakbuff_table_generated
impl FlatBufferToJson for ep_breakbuff_table_generated::clz_Torappu_EPBreakBuffData<'_> {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some((
                "ElementBreakDuration".to_string(),
                json!(self.elementBreakDuration()),
            ));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(vec) = self.elementBuffs() {
                assert!(vec.len() <= 10_000_000, "FB vector too large");
                let arr: Vec<Value> = (0..vec.len()).map(|i| json!(vec.get(i))).collect();
                return Some(("ElementBuffs".to_string(), json!(arr)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        Value::Object(map)
    }
}

impl FlatBufferToJson
    for ep_breakbuff_table_generated::dict__string__clz_Torappu_EPBreakBuffData<'_>
{
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        if let Ok(k) = panic::catch_unwind(AssertUnwindSafe(|| self.key())) {
            map.insert("key".to_string(), json!(k));
        }
        if let Some(v) = self.value() {
            map.insert("value".to_string(), v.to_json());
        }
        Value::Object(map)
    }
}

impl FlatBufferToJson
    for ep_breakbuff_table_generated::clz_Torappu_SimpleKVTable_clz_Torappu_EPBreakBuffData<'_>
{
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(vec) = self.ep_breakbuffs() {
                assert!(vec.len() <= 10_000_000, "FB vector too large");
                let arr: Vec<Value> = (0..vec.len())
                    .filter_map(|i| {
                        panic::catch_unwind(AssertUnwindSafe(|| vec.get(i).to_json())).ok()
                    })
                    .collect();
                return Some(("Ep_breakbuffs".to_string(), json!(arr)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        Value::Object(map)
    }
}

// From token_table_generated
impl FlatBufferToJson for token_table_generated::clz_Torappu_CharacterData_PowerData<'_> {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.nationId() {
                return Some(("NationId".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.groupId() {
                return Some(("GroupId".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.teamId() {
                return Some(("TeamId".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        Value::Object(map)
    }
}

impl FlatBufferToJson for token_table_generated::clz_Torappu_CharacterData_UnlockCondition<'_> {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("Phase".to_string(), self.phase().to_json_value()));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("Level".to_string(), json!(self.level())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        Value::Object(map)
    }
}

impl FlatBufferToJson for token_table_generated::clz_Torappu_Blackboard_DataPair<'_> {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        if let Some(k) = self.key() {
            map.insert("key".to_string(), json!(k));
        }
        map.insert("value".to_string(), json!(self.value()));
        Value::Object(map)
    }
}

impl FlatBufferToJson for token_table_generated::clz_Torappu_CharacterData_TraitData<'_> {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(nested) = self.unlockCondition() {
                return Some(("UnlockCondition".to_string(), nested.to_json()));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some((
                "RequiredPotentialRank".to_string(),
                json!(self.requiredPotentialRank()),
            ));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(vec) = self.blackboard() {
                assert!(vec.len() <= 10_000_000, "FB vector too large");
                let arr: Vec<Value> = (0..vec.len())
                    .filter_map(|i| {
                        panic::catch_unwind(AssertUnwindSafe(|| vec.get(i).to_json())).ok()
                    })
                    .collect();
                return Some(("Blackboard".to_string(), json!(arr)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.overrideDescripton() {
                return Some(("OverrideDescripton".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.prefabKey() {
                return Some(("PrefabKey".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.rangeId() {
                return Some(("RangeId".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        Value::Object(map)
    }
}

impl FlatBufferToJson for token_table_generated::clz_Torappu_CharacterData_TraitDataBundle<'_> {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(vec) = self.candidates() {
                assert!(vec.len() <= 10_000_000, "FB vector too large");
                let arr: Vec<Value> = (0..vec.len())
                    .filter_map(|i| {
                        panic::catch_unwind(AssertUnwindSafe(|| vec.get(i).to_json())).ok()
                    })
                    .collect();
                return Some(("Candidates".to_string(), json!(arr)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        Value::Object(map)
    }
}

impl FlatBufferToJson for token_table_generated::clz_Torappu_AttributesData<'_> {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("MaxHp".to_string(), json!(self.maxHp())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("Atk".to_string(), json!(self.atk())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("Def".to_string(), json!(self.def())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("MagicResistance".to_string(), json!(self.magicResistance())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("Cost".to_string(), json!(self.cost())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("BlockCnt".to_string(), json!(self.blockCnt())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("MoveSpeed".to_string(), json!(self.moveSpeed())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("AttackSpeed".to_string(), json!(self.attackSpeed())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("BaseAttackTime".to_string(), json!(self.baseAttackTime())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("RespawnTime".to_string(), json!(self.respawnTime())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some((
                "HpRecoveryPerSec".to_string(),
                json!(self.hpRecoveryPerSec()),
            ));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some((
                "SpRecoveryPerSec".to_string(),
                json!(self.spRecoveryPerSec()),
            ));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("MaxDeployCount".to_string(), json!(self.maxDeployCount())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("MaxDeckStackCnt".to_string(), json!(self.maxDeckStackCnt())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("TauntLevel".to_string(), json!(self.tauntLevel())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("MassLevel".to_string(), json!(self.massLevel())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("BaseForceLevel".to_string(), json!(self.baseForceLevel())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("StunImmune".to_string(), json!(self.stunImmune())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("SilenceImmune".to_string(), json!(self.silenceImmune())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("SleepImmune".to_string(), json!(self.sleepImmune())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("FrozenImmune".to_string(), json!(self.frozenImmune())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("LevitateImmune".to_string(), json!(self.levitateImmune())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some((
                "DisarmedCombatImmune".to_string(),
                json!(self.disarmedCombatImmune()),
            ));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("FearedImmune".to_string(), json!(self.fearedImmune())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("PalsyImmune".to_string(), json!(self.palsyImmune())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("AttractImmune".to_string(), json!(self.attractImmune())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        Value::Object(map)
    }
}

impl FlatBufferToJson for token_table_generated::clz_Torappu_KeyFrames_2_KeyFrame_Torappu_AttributesData_Torappu_AttributesData_<'_> {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("Level".to_string(), json!(self.level())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(nested) = self.data() {
                return Some(("Data".to_string(), nested.to_json()));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        Value::Object(map)
    }
}

impl FlatBufferToJson for token_table_generated::clz_Torappu_ItemBundle<'_> {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.id() {
                return Some(("Id".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("Count".to_string(), json!(self.count())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("Type_".to_string(), self.type_().to_json_value()));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        Value::Object(map)
    }
}

impl FlatBufferToJson for token_table_generated::clz_Torappu_CharacterData_PhaseData<'_> {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.characterPrefabKey() {
                return Some(("CharacterPrefabKey".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.rangeId() {
                return Some(("RangeId".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("MaxLevel".to_string(), json!(self.maxLevel())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(vec) = self.attributesKeyFrames() {
                assert!(vec.len() <= 10_000_000, "FB vector too large");
                let arr: Vec<Value> = (0..vec.len())
                    .filter_map(|i| {
                        panic::catch_unwind(AssertUnwindSafe(|| vec.get(i).to_json())).ok()
                    })
                    .collect();
                return Some(("AttributesKeyFrames".to_string(), json!(arr)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(vec) = self.evolveCost() {
                assert!(vec.len() <= 10_000_000, "FB vector too large");
                let arr: Vec<Value> = (0..vec.len())
                    .filter_map(|i| {
                        panic::catch_unwind(AssertUnwindSafe(|| vec.get(i).to_json())).ok()
                    })
                    .collect();
                return Some(("EvolveCost".to_string(), json!(arr)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        Value::Object(map)
    }
}

impl FlatBufferToJson
    for token_table_generated::clz_Torappu_CharacterData_MainSkill_SpecializeLevelData<'_>
{
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(nested) = self.unlockCond() {
                return Some(("UnlockCond".to_string(), nested.to_json()));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("LvlUpTime".to_string(), json!(self.lvlUpTime())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(vec) = self.levelUpCost() {
                assert!(vec.len() <= 10_000_000, "FB vector too large");
                let arr: Vec<Value> = (0..vec.len())
                    .filter_map(|i| {
                        panic::catch_unwind(AssertUnwindSafe(|| vec.get(i).to_json())).ok()
                    })
                    .collect();
                return Some(("LevelUpCost".to_string(), json!(arr)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        Value::Object(map)
    }
}

impl FlatBufferToJson for token_table_generated::clz_Torappu_CharacterData_MainSkill<'_> {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.skillId() {
                return Some(("SkillId".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.overridePrefabKey() {
                return Some(("OverridePrefabKey".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.overrideTokenKey() {
                return Some(("OverrideTokenKey".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(vec) = self.levelUpCostCond() {
                assert!(vec.len() <= 10_000_000, "FB vector too large");
                let arr: Vec<Value> = (0..vec.len())
                    .filter_map(|i| {
                        panic::catch_unwind(AssertUnwindSafe(|| vec.get(i).to_json())).ok()
                    })
                    .collect();
                return Some(("LevelUpCostCond".to_string(), json!(arr)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(nested) = self.unlockCond() {
                return Some(("UnlockCond".to_string(), nested.to_json()));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        Value::Object(map)
    }
}

impl FlatBufferToJson for token_table_generated::dict__string__bool<'_> {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        if let Ok(k) = panic::catch_unwind(AssertUnwindSafe(|| self.key())) {
            map.insert("key".to_string(), json!(k));
        }
        map.insert("value".to_string(), json!(self.value()));
        Value::Object(map)
    }
}

impl FlatBufferToJson for token_table_generated::clz_Torappu_TalentData<'_> {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(nested) = self.unlockCondition() {
                return Some(("UnlockCondition".to_string(), nested.to_json()));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some((
                "RequiredPotentialRank".to_string(),
                json!(self.requiredPotentialRank()),
            ));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.prefabKey() {
                return Some(("PrefabKey".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.name() {
                return Some(("Name".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.description() {
                return Some(("Description".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.rangeId() {
                return Some(("RangeId".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(vec) = self.blackboard() {
                assert!(vec.len() <= 10_000_000, "FB vector too large");
                let arr: Vec<Value> = (0..vec.len())
                    .filter_map(|i| {
                        panic::catch_unwind(AssertUnwindSafe(|| vec.get(i).to_json())).ok()
                    })
                    .collect();
                return Some(("Blackboard".to_string(), json!(arr)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.tokenKey() {
                return Some(("TokenKey".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("IsHideTalent".to_string(), json!(self.isHideTalent())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        Value::Object(map)
    }
}

impl FlatBufferToJson for token_table_generated::clz_Torappu_CharacterData_TalentDataBundle<'_> {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(vec) = self.candidates() {
                assert!(vec.len() <= 10_000_000, "FB vector too large");
                let arr: Vec<Value> = (0..vec.len())
                    .filter_map(|i| {
                        panic::catch_unwind(AssertUnwindSafe(|| vec.get(i).to_json())).ok()
                    })
                    .collect();
                return Some(("Candidates".to_string(), json!(arr)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        Value::Object(map)
    }
}

impl FlatBufferToJson
    for token_table_generated::clz_Torappu_AttributeModifierData_AttributeModifier<'_>
{
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some((
                "AttributeType".to_string(),
                self.attributeType().to_json_value(),
            ));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some((
                "FormulaItem".to_string(),
                self.formulaItem().to_json_value(),
            ));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("Value".to_string(), json!(self.value())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some((
                "LoadFromBlackboard".to_string(),
                json!(self.loadFromBlackboard()),
            ));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some((
                "FetchBaseValueFromSourceEntity".to_string(),
                json!(self.fetchBaseValueFromSourceEntity()),
            ));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        Value::Object(map)
    }
}

impl FlatBufferToJson for token_table_generated::clz_Torappu_AttributeModifierData<'_> {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(vec) = self.abnormalFlags() {
                assert!(vec.len() <= 10_000_000, "FB vector too large");
                let arr: Vec<Value> = vec.iter().map(|e| e.to_json_value()).collect();
                return Some(("AbnormalFlags".to_string(), json!(arr)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(vec) = self.abnormalImmunes() {
                assert!(vec.len() <= 10_000_000, "FB vector too large");
                let arr: Vec<Value> = vec.iter().map(|e| e.to_json_value()).collect();
                return Some(("AbnormalImmunes".to_string(), json!(arr)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(vec) = self.abnormalAntis() {
                assert!(vec.len() <= 10_000_000, "FB vector too large");
                let arr: Vec<Value> = vec.iter().map(|e| e.to_json_value()).collect();
                return Some(("AbnormalAntis".to_string(), json!(arr)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(vec) = self.abnormalCombos() {
                assert!(vec.len() <= 10_000_000, "FB vector too large");
                let arr: Vec<Value> = vec.iter().map(|e| e.to_json_value()).collect();
                return Some(("AbnormalCombos".to_string(), json!(arr)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(vec) = self.abnormalComboImmunes() {
                assert!(vec.len() <= 10_000_000, "FB vector too large");
                let arr: Vec<Value> = vec.iter().map(|e| e.to_json_value()).collect();
                return Some(("AbnormalComboImmunes".to_string(), json!(arr)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(vec) = self.attributeModifiers() {
                assert!(vec.len() <= 10_000_000, "FB vector too large");
                let arr: Vec<Value> = (0..vec.len())
                    .filter_map(|i| {
                        panic::catch_unwind(AssertUnwindSafe(|| vec.get(i).to_json())).ok()
                    })
                    .collect();
                return Some(("AttributeModifiers".to_string(), json!(arr)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        Value::Object(map)
    }
}

impl FlatBufferToJson for token_table_generated::clz_Torappu_ExternalBuff<'_> {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(nested) = self.attributes() {
                return Some(("Attributes".to_string(), nested.to_json()));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        Value::Object(map)
    }
}

impl FlatBufferToJson for token_table_generated::clz_Torappu_CharacterData_PotentialRank<'_> {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("Type_".to_string(), self.type_().to_json_value()));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.description() {
                return Some(("Description".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(nested) = self.buff() {
                return Some(("Buff".to_string(), nested.to_json()));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(vec) = self.equivalentCost() {
                assert!(vec.len() <= 10_000_000, "FB vector too large");
                let arr: Vec<Value> = (0..vec.len())
                    .filter_map(|i| {
                        panic::catch_unwind(AssertUnwindSafe(|| vec.get(i).to_json())).ok()
                    })
                    .collect();
                return Some(("EquivalentCost".to_string(), json!(arr)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        Value::Object(map)
    }
}

impl FlatBufferToJson for token_table_generated::clz_Torappu_AttributesDeltaData<'_> {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("MaxHp".to_string(), json!(self.maxHp())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("Atk".to_string(), json!(self.atk())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("Def".to_string(), json!(self.def())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("MagicResistance".to_string(), json!(self.magicResistance())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("Cost".to_string(), json!(self.cost())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("BlockCnt".to_string(), json!(self.blockCnt())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("MoveSpeed".to_string(), json!(self.moveSpeed())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("AttackSpeed".to_string(), json!(self.attackSpeed())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("BaseAttackTime".to_string(), json!(self.baseAttackTime())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("RespawnTime".to_string(), json!(self.respawnTime())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some((
                "HpRecoveryPerSec".to_string(),
                json!(self.hpRecoveryPerSec()),
            ));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some((
                "SpRecoveryPerSec".to_string(),
                json!(self.spRecoveryPerSec()),
            ));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("MaxDeployCount".to_string(), json!(self.maxDeployCount())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("MaxDeckStackCnt".to_string(), json!(self.maxDeckStackCnt())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("TauntLevel".to_string(), json!(self.tauntLevel())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("MassLevel".to_string(), json!(self.massLevel())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("BaseForceLevel".to_string(), json!(self.baseForceLevel())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("StunImmune".to_string(), json!(self.stunImmune())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("SilenceImmune".to_string(), json!(self.silenceImmune())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("SleepImmune".to_string(), json!(self.sleepImmune())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("FrozenImmune".to_string(), json!(self.frozenImmune())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("LevitateImmune".to_string(), json!(self.levitateImmune())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some((
                "DisarmedCombatImmune".to_string(),
                json!(self.disarmedCombatImmune()),
            ));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("FearedImmune".to_string(), json!(self.fearedImmune())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("PalsyImmune".to_string(), json!(self.palsyImmune())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("AttractImmune".to_string(), json!(self.attractImmune())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        Value::Object(map)
    }
}

impl FlatBufferToJson for token_table_generated::clz_Torappu_KeyFrames_2_KeyFrame_Torappu_AttributesDeltaData_Torappu_AttributesData_<'_> {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("Level".to_string(), json!(self.level())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(nested) = self.data() {
                return Some(("Data".to_string(), nested.to_json()));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        Value::Object(map)
    }
}

impl FlatBufferToJson for token_table_generated::clz_Torappu_CharacterData_SkillLevelCost<'_> {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(nested) = self.unlockCond() {
                return Some(("UnlockCond".to_string(), nested.to_json()));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(vec) = self.lvlUpCost() {
                assert!(vec.len() <= 10_000_000, "FB vector too large");
                let arr: Vec<Value> = (0..vec.len())
                    .filter_map(|i| {
                        panic::catch_unwind(AssertUnwindSafe(|| vec.get(i).to_json())).ok()
                    })
                    .collect();
                return Some(("LvlUpCost".to_string(), json!(arr)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        Value::Object(map)
    }
}

impl FlatBufferToJson for token_table_generated::clz_Torappu_CharacterData<'_> {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.name() {
                return Some(("Name".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.description() {
                return Some(("Description".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("SortIndex".to_string(), json!(self.sortIndex())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some((
                "SpTargetType".to_string(),
                self.spTargetType().to_json_value(),
            ));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.spTargetId() {
                return Some(("SpTargetId".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some((
                "CanUseGeneralPotentialItem".to_string(),
                json!(self.canUseGeneralPotentialItem()),
            ));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some((
                "CanUseActivityPotentialItem".to_string(),
                json!(self.canUseActivityPotentialItem()),
            ));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.potentialItemId() {
                return Some(("PotentialItemId".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.activityPotentialItemId() {
                return Some(("ActivityPotentialItemId".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.classicPotentialItemId() {
                return Some(("ClassicPotentialItemId".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.nationId() {
                return Some(("NationId".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.groupId() {
                return Some(("GroupId".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.teamId() {
                return Some(("TeamId".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(nested) = self.mainPower() {
                return Some(("MainPower".to_string(), nested.to_json()));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(vec) = self.subPower() {
                assert!(vec.len() <= 10_000_000, "FB vector too large");
                let arr: Vec<Value> = (0..vec.len())
                    .filter_map(|i| {
                        panic::catch_unwind(AssertUnwindSafe(|| vec.get(i).to_json())).ok()
                    })
                    .collect();
                return Some(("SubPower".to_string(), json!(arr)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.displayNumber() {
                return Some(("DisplayNumber".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.appellation() {
                return Some(("Appellation".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("Position".to_string(), self.position().to_json_value()));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(vec) = self.tagList() {
                assert!(vec.len() <= 10_000_000, "FB vector too large");
                let arr: Vec<Value> = (0..vec.len()).map(|i| json!(vec.get(i))).collect();
                return Some(("TagList".to_string(), json!(arr)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.itemUsage() {
                return Some(("ItemUsage".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.itemDesc() {
                return Some(("ItemDesc".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.itemObtainApproach() {
                return Some(("ItemObtainApproach".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("IsNotObtainable".to_string(), json!(self.isNotObtainable())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("IsSpChar".to_string(), json!(self.isSpChar())));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some((
                "MaxPotentialLevel".to_string(),
                json!(self.maxPotentialLevel()),
            ));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("Rarity".to_string(), self.rarity().to_json_value()));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            return Some(("Profession".to_string(), self.profession().to_json_value()));
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(v) = self.subProfessionId() {
                return Some(("SubProfessionId".to_string(), json!(v)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(nested) = self.trait_() {
                return Some(("Trait_".to_string(), nested.to_json()));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(vec) = self.phases() {
                assert!(vec.len() <= 10_000_000, "FB vector too large");
                let arr: Vec<Value> = (0..vec.len())
                    .filter_map(|i| {
                        panic::catch_unwind(AssertUnwindSafe(|| vec.get(i).to_json())).ok()
                    })
                    .collect();
                return Some(("Phases".to_string(), json!(arr)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(vec) = self.skills() {
                assert!(vec.len() <= 10_000_000, "FB vector too large");
                let arr: Vec<Value> = (0..vec.len())
                    .filter_map(|i| {
                        panic::catch_unwind(AssertUnwindSafe(|| vec.get(i).to_json())).ok()
                    })
                    .collect();
                return Some(("Skills".to_string(), json!(arr)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(vec) = self.displayTokenDict() {
                assert!(vec.len() <= 10_000_000, "FB vector too large");
                let arr: Vec<Value> = (0..vec.len())
                    .filter_map(|i| {
                        panic::catch_unwind(AssertUnwindSafe(|| vec.get(i).to_json())).ok()
                    })
                    .collect();
                return Some(("DisplayTokenDict".to_string(), json!(arr)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(vec) = self.talents() {
                assert!(vec.len() <= 10_000_000, "FB vector too large");
                let arr: Vec<Value> = (0..vec.len())
                    .filter_map(|i| {
                        panic::catch_unwind(AssertUnwindSafe(|| vec.get(i).to_json())).ok()
                    })
                    .collect();
                return Some(("Talents".to_string(), json!(arr)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(vec) = self.potentialRanks() {
                assert!(vec.len() <= 10_000_000, "FB vector too large");
                let arr: Vec<Value> = (0..vec.len())
                    .filter_map(|i| {
                        panic::catch_unwind(AssertUnwindSafe(|| vec.get(i).to_json())).ok()
                    })
                    .collect();
                return Some(("PotentialRanks".to_string(), json!(arr)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(vec) = self.favorKeyFrames() {
                assert!(vec.len() <= 10_000_000, "FB vector too large");
                let arr: Vec<Value> = (0..vec.len())
                    .filter_map(|i| {
                        panic::catch_unwind(AssertUnwindSafe(|| vec.get(i).to_json())).ok()
                    })
                    .collect();
                return Some(("FavorKeyFrames".to_string(), json!(arr)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(vec) = self.allSkillLvlup() {
                assert!(vec.len() <= 10_000_000, "FB vector too large");
                let arr: Vec<Value> = (0..vec.len())
                    .filter_map(|i| {
                        panic::catch_unwind(AssertUnwindSafe(|| vec.get(i).to_json())).ok()
                    })
                    .collect();
                return Some(("AllSkillLvlup".to_string(), json!(arr)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        Value::Object(map)
    }
}

impl FlatBufferToJson for token_table_generated::dict__string__clz_Torappu_CharacterData<'_> {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        if let Ok(k) = panic::catch_unwind(AssertUnwindSafe(|| self.key())) {
            map.insert("key".to_string(), json!(k));
        }
        if let Some(v) = self.value() {
            map.insert("value".to_string(), v.to_json());
        }
        Value::Object(map)
    }
}

impl FlatBufferToJson
    for token_table_generated::clz_Torappu_SimpleKVTable_clz_Torappu_CharacterData<'_>
{
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        if let Ok(Some((k, v))) = panic::catch_unwind(AssertUnwindSafe(|| {
            if let Some(vec) = self.characters() {
                assert!(vec.len() <= 10_000_000, "FB vector too large");
                let arr: Vec<Value> = (0..vec.len())
                    .filter_map(|i| {
                        panic::catch_unwind(AssertUnwindSafe(|| vec.get(i).to_json())).ok()
                    })
                    .collect();
                return Some(("Characters".to_string(), json!(arr)));
            }
            #[allow(unreachable_code)]
            None
        })) {
            map.insert(k, v);
        }
        Value::Object(map)
    }
}
