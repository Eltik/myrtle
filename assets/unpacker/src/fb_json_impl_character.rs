//! FlatBufferToJson implementations for character_table types
//!
//! This module implements the FlatBufferToJson trait for all types
//! in character_table_generated.rs

use crate::fb_json_macros::{to_pascal_case, EnumToJson, FlatBufferToJson};
use crate::generated_fbs::character_table_generated::*;
use serde_json::{json, Map, Value};

// ============================================
// Enum implementations
// ============================================

impl_enum_to_json!(enum__Torappu_SpecialOperatorTargetType);
impl_enum_to_json!(enum__Torappu_BuildableType);
impl_enum_to_json!(enum__Torappu_RarityRank);
impl_enum_to_json!(enum__Torappu_ProfessionCategory);
impl_enum_to_json!(enum__Torappu_EvolvePhase);
impl_enum_to_json!(enum__Torappu_ItemType);
impl_enum_to_json!(enum__Torappu_CharacterData_PotentialRank_TypeEnum);
impl_enum_to_json!(enum__Torappu_AttributeType);
impl_enum_to_json!(enum__Torappu_AttributeModifierData_AttributeModifier_FormulaItemType);
impl_enum_to_json!(enum__Torappu_AbnormalFlag);
impl_enum_to_json!(enum__Torappu_AbnormalCombo);

// ============================================
// Simple nested types
// ============================================

impl_fb_to_json!(clz_Torappu_CharacterData_PowerData<'_>,
    nationId: string,
    groupId: string,
    teamId: string,
);

impl_fb_to_json!(clz_Torappu_CharacterData_UnlockCondition<'_>,
    phase: enum,
    level: scalar,
);

impl_fb_to_json!(clz_Torappu_Blackboard_DataPair<'_>,
    key: string,
    value: scalar,
    valueStr: string,
);

// ============================================
// AttributesData (many scalar fields)
// ============================================

impl FlatBufferToJson for clz_Torappu_AttributesData<'_> {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        map.insert(to_pascal_case("maxHp"), json!(self.maxHp()));
        map.insert(to_pascal_case("atk"), json!(self.atk()));
        map.insert(to_pascal_case("def"), json!(self.def()));
        map.insert(
            to_pascal_case("magicResistance"),
            json!(self.magicResistance()),
        );
        map.insert(to_pascal_case("cost"), json!(self.cost()));
        map.insert(to_pascal_case("blockCnt"), json!(self.blockCnt()));
        map.insert(to_pascal_case("moveSpeed"), json!(self.moveSpeed()));
        map.insert(to_pascal_case("attackSpeed"), json!(self.attackSpeed()));
        map.insert(
            to_pascal_case("baseAttackTime"),
            json!(self.baseAttackTime()),
        );
        map.insert(to_pascal_case("respawnTime"), json!(self.respawnTime()));
        map.insert(
            to_pascal_case("hpRecoveryPerSec"),
            json!(self.hpRecoveryPerSec()),
        );
        map.insert(
            to_pascal_case("spRecoveryPerSec"),
            json!(self.spRecoveryPerSec()),
        );
        map.insert(
            to_pascal_case("maxDeployCount"),
            json!(self.maxDeployCount()),
        );
        map.insert(
            to_pascal_case("maxDeckStackCnt"),
            json!(self.maxDeckStackCnt()),
        );
        map.insert(to_pascal_case("tauntLevel"), json!(self.tauntLevel()));
        map.insert(to_pascal_case("massLevel"), json!(self.massLevel()));
        map.insert(
            to_pascal_case("baseForceLevel"),
            json!(self.baseForceLevel()),
        );
        map.insert(to_pascal_case("stunImmune"), json!(self.stunImmune()));
        map.insert(to_pascal_case("silenceImmune"), json!(self.silenceImmune()));
        map.insert(to_pascal_case("sleepImmune"), json!(self.sleepImmune()));
        map.insert(to_pascal_case("frozenImmune"), json!(self.frozenImmune()));
        map.insert(
            to_pascal_case("levitateImmune"),
            json!(self.levitateImmune()),
        );
        map.insert(
            to_pascal_case("disarmedCombatImmune"),
            json!(self.disarmedCombatImmune()),
        );
        map.insert(to_pascal_case("fearedImmune"), json!(self.fearedImmune()));
        map.insert(to_pascal_case("palsyImmune"), json!(self.palsyImmune()));
        map.insert(to_pascal_case("attractImmune"), json!(self.attractImmune()));
        Value::Object(map)
    }
}

// ============================================
// AttributesDeltaData (similar to AttributesData)
// ============================================

impl FlatBufferToJson for clz_Torappu_AttributesDeltaData<'_> {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        map.insert(to_pascal_case("maxHp"), json!(self.maxHp()));
        map.insert(to_pascal_case("atk"), json!(self.atk()));
        map.insert(to_pascal_case("def"), json!(self.def()));
        map.insert(
            to_pascal_case("magicResistance"),
            json!(self.magicResistance()),
        );
        map.insert(to_pascal_case("cost"), json!(self.cost()));
        map.insert(to_pascal_case("blockCnt"), json!(self.blockCnt()));
        map.insert(to_pascal_case("moveSpeed"), json!(self.moveSpeed()));
        map.insert(to_pascal_case("attackSpeed"), json!(self.attackSpeed()));
        map.insert(
            to_pascal_case("baseAttackTime"),
            json!(self.baseAttackTime()),
        );
        map.insert(to_pascal_case("respawnTime"), json!(self.respawnTime()));
        map.insert(
            to_pascal_case("hpRecoveryPerSec"),
            json!(self.hpRecoveryPerSec()),
        );
        map.insert(
            to_pascal_case("spRecoveryPerSec"),
            json!(self.spRecoveryPerSec()),
        );
        map.insert(
            to_pascal_case("maxDeployCount"),
            json!(self.maxDeployCount()),
        );
        map.insert(
            to_pascal_case("maxDeckStackCnt"),
            json!(self.maxDeckStackCnt()),
        );
        map.insert(to_pascal_case("tauntLevel"), json!(self.tauntLevel()));
        map.insert(to_pascal_case("massLevel"), json!(self.massLevel()));
        map.insert(
            to_pascal_case("baseForceLevel"),
            json!(self.baseForceLevel()),
        );
        map.insert(to_pascal_case("stunImmune"), json!(self.stunImmune()));
        map.insert(to_pascal_case("silenceImmune"), json!(self.silenceImmune()));
        map.insert(to_pascal_case("sleepImmune"), json!(self.sleepImmune()));
        map.insert(to_pascal_case("frozenImmune"), json!(self.frozenImmune()));
        map.insert(
            to_pascal_case("levitateImmune"),
            json!(self.levitateImmune()),
        );
        map.insert(
            to_pascal_case("disarmedCombatImmune"),
            json!(self.disarmedCombatImmune()),
        );
        map.insert(to_pascal_case("fearedImmune"), json!(self.fearedImmune()));
        map.insert(to_pascal_case("palsyImmune"), json!(self.palsyImmune()));
        map.insert(to_pascal_case("attractImmune"), json!(self.attractImmune()));
        Value::Object(map)
    }
}

// ============================================
// KeyFrame types
// ============================================

impl FlatBufferToJson
    for clz_Torappu_KeyFrames_2_KeyFrame_Torappu_AttributesData_Torappu_AttributesData_<'_>
{
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        map.insert(to_pascal_case("level"), json!(self.level()));
        fb_field!(map, "data", self.data(), nested);
        Value::Object(map)
    }
}

impl FlatBufferToJson
    for clz_Torappu_KeyFrames_2_KeyFrame_Torappu_AttributesDeltaData_Torappu_AttributesData_<'_>
{
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        map.insert(to_pascal_case("level"), json!(self.level()));
        fb_field!(map, "data", self.data(), nested);
        Value::Object(map)
    }
}

// ============================================
// ItemBundle
// ============================================

impl FlatBufferToJson for clz_Torappu_ItemBundle<'_> {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        fb_field!(map, "id", self.id(), string);
        map.insert(to_pascal_case("count"), json!(self.count()));
        map.insert(to_pascal_case("type_"), self.type_().to_json_value());
        Value::Object(map)
    }
}

// ============================================
// PhaseData
// ============================================

impl FlatBufferToJson for clz_Torappu_CharacterData_PhaseData<'_> {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        fb_field!(map, "characterPrefabKey", self.characterPrefabKey(), string);
        fb_field!(map, "rangeId", self.rangeId(), string);
        map.insert(to_pascal_case("maxLevel"), json!(self.maxLevel()));
        fb_field!(
            map,
            "attributesKeyFrames",
            self.attributesKeyFrames(),
            vec_nested
        );
        fb_field!(map, "evolveCost", self.evolveCost(), vec_nested);
        Value::Object(map)
    }
}

// ============================================
// Skill-related types
// ============================================

impl FlatBufferToJson for clz_Torappu_CharacterData_MainSkill_SpecializeLevelData<'_> {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        fb_field!(map, "unlockCond", self.unlockCond(), nested);
        map.insert(to_pascal_case("lvlUpTime"), json!(self.lvlUpTime()));
        fb_field!(map, "levelUpCost", self.levelUpCost(), vec_nested);
        Value::Object(map)
    }
}

impl FlatBufferToJson for clz_Torappu_CharacterData_MainSkill<'_> {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        fb_field!(map, "skillId", self.skillId(), string);
        fb_field!(map, "overridePrefabKey", self.overridePrefabKey(), string);
        fb_field!(map, "overrideTokenKey", self.overrideTokenKey(), string);
        fb_field!(map, "levelUpCostCond", self.levelUpCostCond(), vec_nested);
        fb_field!(map, "unlockCond", self.unlockCond(), nested);
        Value::Object(map)
    }
}

impl FlatBufferToJson for clz_Torappu_CharacterData_SkillLevelCost<'_> {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        fb_field!(map, "unlockCond", self.unlockCond(), nested);
        fb_field!(map, "lvlUpCost", self.lvlUpCost(), vec_nested);
        Value::Object(map)
    }
}

// ============================================
// Trait-related types
// ============================================

impl FlatBufferToJson for clz_Torappu_CharacterData_TraitData<'_> {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        fb_field!(map, "unlockCondition", self.unlockCondition(), nested);
        map.insert(
            to_pascal_case("requiredPotentialRank"),
            json!(self.requiredPotentialRank()),
        );
        fb_field!(map, "blackboard", self.blackboard(), vec_nested);
        fb_field!(map, "overrideDescripton", self.overrideDescripton(), string);
        fb_field!(map, "prefabKey", self.prefabKey(), string);
        fb_field!(map, "rangeId", self.rangeId(), string);
        Value::Object(map)
    }
}

impl FlatBufferToJson for clz_Torappu_CharacterData_TraitDataBundle<'_> {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        fb_field!(map, "candidates", self.candidates(), vec_nested);
        Value::Object(map)
    }
}

// ============================================
// Talent-related types
// ============================================

impl FlatBufferToJson for clz_Torappu_TalentData<'_> {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        fb_field!(map, "unlockCondition", self.unlockCondition(), nested);
        map.insert(
            to_pascal_case("requiredPotentialRank"),
            json!(self.requiredPotentialRank()),
        );
        fb_field!(map, "prefabKey", self.prefabKey(), string);
        fb_field!(map, "name", self.name(), string);
        fb_field!(map, "description", self.description(), string);
        fb_field!(map, "rangeId", self.rangeId(), string);
        fb_field!(map, "blackboard", self.blackboard(), vec_nested);
        map.insert(to_pascal_case("isHideTalent"), json!(self.isHideTalent()));
        Value::Object(map)
    }
}

impl FlatBufferToJson for clz_Torappu_CharacterData_TalentDataBundle<'_> {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        fb_field!(map, "candidates", self.candidates(), vec_nested);
        Value::Object(map)
    }
}

// ============================================
// Potential-related types
// ============================================

impl FlatBufferToJson for clz_Torappu_AttributeModifierData_AttributeModifier<'_> {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        map.insert(
            to_pascal_case("attributeType"),
            self.attributeType().to_json_value(),
        );
        map.insert(
            to_pascal_case("formulaItem"),
            self.formulaItem().to_json_value(),
        );
        map.insert(to_pascal_case("value"), json!(self.value()));
        map.insert(
            to_pascal_case("loadFromBlackboard"),
            json!(self.loadFromBlackboard()),
        );
        map.insert(
            to_pascal_case("fetchBaseValueFromSourceEntity"),
            json!(self.fetchBaseValueFromSourceEntity()),
        );
        Value::Object(map)
    }
}

impl FlatBufferToJson for clz_Torappu_AttributeModifierData<'_> {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        // Vector of enum types - convert each to string
        if let Some(flags) = self.abnormalFlags() {
            let arr: Vec<Value> = flags.iter().map(|f| f.to_json_value()).collect();
            map.insert(to_pascal_case("abnormalFlags"), json!(arr));
        }
        if let Some(flags) = self.abnormalImmunes() {
            let arr: Vec<Value> = flags.iter().map(|f| f.to_json_value()).collect();
            map.insert(to_pascal_case("abnormalImmunes"), json!(arr));
        }
        if let Some(flags) = self.abnormalAntis() {
            let arr: Vec<Value> = flags.iter().map(|f| f.to_json_value()).collect();
            map.insert(to_pascal_case("abnormalAntis"), json!(arr));
        }
        if let Some(flags) = self.abnormalCombos() {
            let arr: Vec<Value> = flags.iter().map(|f| f.to_json_value()).collect();
            map.insert(to_pascal_case("abnormalCombos"), json!(arr));
        }
        if let Some(flags) = self.abnormalComboImmunes() {
            let arr: Vec<Value> = flags.iter().map(|f| f.to_json_value()).collect();
            map.insert(to_pascal_case("abnormalComboImmunes"), json!(arr));
        }
        fb_field!(
            map,
            "attributeModifiers",
            self.attributeModifiers(),
            vec_nested
        );
        Value::Object(map)
    }
}

impl FlatBufferToJson for clz_Torappu_ExternalBuff<'_> {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        fb_field!(map, "attributes", self.attributes(), nested);
        Value::Object(map)
    }
}

impl FlatBufferToJson for clz_Torappu_CharacterData_PotentialRank<'_> {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        map.insert(to_pascal_case("type_"), self.type_().to_json_value());
        fb_field!(map, "description", self.description(), string);
        fb_field!(map, "buff", self.buff(), nested);
        fb_field!(map, "equivalentCost", self.equivalentCost(), vec_nested);
        Value::Object(map)
    }
}

// ============================================
// Dict types (key-value entries)
// ============================================

impl FlatBufferToJson for dict__string__bool<'_> {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        map.insert("key".to_string(), json!(self.key()));
        map.insert("value".to_string(), json!(self.value()));
        Value::Object(map)
    }
}

impl FlatBufferToJson for dict__string__clz_Torappu_CharacterData<'_> {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        map.insert("key".to_string(), json!(self.key()));
        if let Some(value) = self.value() {
            map.insert("value".to_string(), value.to_json());
        }
        Value::Object(map)
    }
}

// ============================================
// Main CharacterData type
// ============================================

impl FlatBufferToJson for clz_Torappu_CharacterData<'_> {
    fn to_json(&self) -> Value {
        let mut map = Map::new();

        // String fields
        fb_field!(map, "name", self.name(), string);
        fb_field!(map, "description", self.description(), string);
        fb_field!(map, "spTargetId", self.spTargetId(), string);
        fb_field!(map, "potentialItemId", self.potentialItemId(), string);
        fb_field!(
            map,
            "activityPotentialItemId",
            self.activityPotentialItemId(),
            string
        );
        fb_field!(
            map,
            "classicPotentialItemId",
            self.classicPotentialItemId(),
            string
        );
        fb_field!(map, "nationId", self.nationId(), string);
        fb_field!(map, "groupId", self.groupId(), string);
        fb_field!(map, "teamId", self.teamId(), string);
        fb_field!(map, "displayNumber", self.displayNumber(), string);
        fb_field!(map, "appellation", self.appellation(), string);
        fb_field!(map, "itemUsage", self.itemUsage(), string);
        fb_field!(map, "itemDesc", self.itemDesc(), string);
        fb_field!(map, "itemObtainApproach", self.itemObtainApproach(), string);
        fb_field!(map, "subProfessionId", self.subProfessionId(), string);

        // Scalar fields
        map.insert(to_pascal_case("sortIndex"), json!(self.sortIndex()));
        map.insert(
            to_pascal_case("canUseGeneralPotentialItem"),
            json!(self.canUseGeneralPotentialItem()),
        );
        map.insert(
            to_pascal_case("canUseActivityPotentialItem"),
            json!(self.canUseActivityPotentialItem()),
        );
        map.insert(
            to_pascal_case("isNotObtainable"),
            json!(self.isNotObtainable()),
        );
        map.insert(to_pascal_case("isSpChar"), json!(self.isSpChar()));
        map.insert(
            to_pascal_case("maxPotentialLevel"),
            json!(self.maxPotentialLevel()),
        );

        // Enum fields
        map.insert(
            to_pascal_case("spTargetType"),
            self.spTargetType().to_json_value(),
        );
        map.insert(to_pascal_case("position"), self.position().to_json_value());
        map.insert(to_pascal_case("rarity"), self.rarity().to_json_value());
        map.insert(
            to_pascal_case("profession"),
            self.profession().to_json_value(),
        );

        // Vector of strings
        fb_field!(map, "tagList", self.tagList(), vec_string);

        // Nested types
        fb_field!(map, "mainPower", self.mainPower(), nested);
        fb_field!(map, "subPower", self.subPower(), vec_nested);
        fb_field!(map, "trait_", self.trait_(), nested);

        // Vector of nested types
        fb_field!(map, "phases", self.phases(), vec_nested);
        fb_field!(map, "skills", self.skills(), vec_nested);
        fb_field!(map, "talents", self.talents(), vec_nested);
        fb_field!(map, "potentialRanks", self.potentialRanks(), vec_nested);
        fb_field!(map, "favorKeyFrames", self.favorKeyFrames(), vec_nested);
        fb_field!(map, "allSkillLvlup", self.allSkillLvlup(), vec_nested);

        // Dict/kv vectors - convert to objects
        if let Some(vec) = self.displayTokenDict() {
            let mut kv_map = Map::new();
            for i in 0..vec.len() {
                let entry = vec.get(i);
                kv_map.insert(entry.key().to_string(), json!(entry.value()));
            }
            map.insert(to_pascal_case("displayTokenDict"), Value::Object(kv_map));
        }

        Value::Object(map)
    }
}

// ============================================
// Root SimpleKVTable type
// ============================================

impl FlatBufferToJson for clz_Torappu_SimpleKVTable_clz_Torappu_CharacterData<'_> {
    fn to_json(&self) -> Value {
        let mut result = Map::new();

        if let Some(characters) = self.characters() {
            let mut chars_map = Map::new();
            for i in 0..characters.len() {
                let entry = characters.get(i);
                let key = entry.key().to_string();
                if let Some(value) = entry.value() {
                    chars_map.insert(key, value.to_json());
                }
            }
            result.insert("Characters".to_string(), Value::Object(chars_map));
        }

        Value::Object(result)
    }
}

/// Decode character_table data to clean JSON using direct field access
pub fn decode_character_table_json(data: &[u8]) -> anyhow::Result<Value> {
    let root =
        unsafe { root_as_clz_torappu_simple_kvtable_clz_torappu_character_data_unchecked(data) };
    Ok(root.to_json())
}
