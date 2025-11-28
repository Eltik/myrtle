//! FlatBufferToJson implementations for enemy_database types
//!
//! This module implements the FlatBufferToJson trait for all types
//! in enemy_database_generated.rs

use crate::fb_json_macros::{to_pascal_case, EnumToJson, FlatBufferToJson};
use crate::generated_fbs::enemy_database_generated::*;
use serde_json::{json, Map, Value};

// ============================================
// Enum implementations
// ============================================

impl_enum_to_json!(enum__Torappu_SourceApplyWay);
impl_enum_to_json!(enum__Torappu_MotionMode);
impl_enum_to_json!(enum__Torappu_EnemyLevelType);
impl_enum_to_json!(enum__Torappu_SpType);

// ============================================
// Undefinable wrapper types
// These types wrap values with an m_defined flag
// ============================================

impl FlatBufferToJson for clz_Torappu_Undefinable_1_System_String_<'_> {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        map.insert(to_pascal_case("m_defined"), json!(self.m_defined()));
        if let Some(v) = self.m_value() {
            map.insert(to_pascal_case("m_value"), json!(v));
        }
        Value::Object(map)
    }
}

impl FlatBufferToJson for clz_Torappu_Undefinable_1_System_Int32_<'_> {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        map.insert(to_pascal_case("m_defined"), json!(self.m_defined()));
        map.insert(to_pascal_case("m_value"), json!(self.m_value()));
        Value::Object(map)
    }
}

impl FlatBufferToJson for clz_Torappu_Undefinable_1_System_Single_<'_> {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        map.insert(to_pascal_case("m_defined"), json!(self.m_defined()));
        map.insert(to_pascal_case("m_value"), json!(self.m_value()));
        Value::Object(map)
    }
}

impl FlatBufferToJson for clz_Torappu_Undefinable_1_System_Boolean_<'_> {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        map.insert(to_pascal_case("m_defined"), json!(self.m_defined()));
        map.insert(to_pascal_case("m_value"), json!(self.m_value()));
        Value::Object(map)
    }
}

impl FlatBufferToJson for clz_Torappu_Undefinable_1_Torappu_SourceApplyWay_<'_> {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        map.insert(to_pascal_case("m_defined"), json!(self.m_defined()));
        map.insert(to_pascal_case("m_value"), self.m_value().to_json_value());
        Value::Object(map)
    }
}

impl FlatBufferToJson for clz_Torappu_Undefinable_1_Torappu_MotionMode_<'_> {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        map.insert(to_pascal_case("m_defined"), json!(self.m_defined()));
        map.insert(to_pascal_case("m_value"), self.m_value().to_json_value());
        Value::Object(map)
    }
}

impl FlatBufferToJson for clz_Torappu_Undefinable_1_Torappu_EnemyLevelType_<'_> {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        map.insert(to_pascal_case("m_defined"), json!(self.m_defined()));
        map.insert(to_pascal_case("m_value"), self.m_value().to_json_value());
        Value::Object(map)
    }
}

impl FlatBufferToJson for clz_Torappu_Undefinable_1_System_String___<'_> {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        map.insert(to_pascal_case("m_defined"), json!(self.m_defined()));
        if let Some(arr) = self.m_value() {
            let values: Vec<Value> = (0..arr.len()).map(|i| json!(arr.get(i))).collect();
            map.insert(to_pascal_case("m_value"), json!(values));
        }
        Value::Object(map)
    }
}

// ============================================
// Blackboard DataPair (reuse from character_table if imported)
// ============================================

impl FlatBufferToJson for clz_Torappu_Blackboard_DataPair<'_> {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        fb_field!(map, "key", self.key(), string);
        map.insert(to_pascal_case("value"), json!(self.value()));
        fb_field!(map, "valueStr", self.valueStr(), string);
        Value::Object(map)
    }
}

// ============================================
// AttributesData for enemies
// ============================================

impl FlatBufferToJson for clz_Torappu_EnemyDatabase_AttributesData<'_> {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        fb_field!(map, "maxHp", self.maxHp(), nested);
        fb_field!(map, "atk", self.atk(), nested);
        fb_field!(map, "def", self.def(), nested);
        fb_field!(map, "magicResistance", self.magicResistance(), nested);
        fb_field!(map, "cost", self.cost(), nested);
        fb_field!(map, "blockCnt", self.blockCnt(), nested);
        fb_field!(map, "moveSpeed", self.moveSpeed(), nested);
        fb_field!(map, "attackSpeed", self.attackSpeed(), nested);
        fb_field!(map, "baseAttackTime", self.baseAttackTime(), nested);
        fb_field!(map, "respawnTime", self.respawnTime(), nested);
        fb_field!(map, "hpRecoveryPerSec", self.hpRecoveryPerSec(), nested);
        fb_field!(map, "spRecoveryPerSec", self.spRecoveryPerSec(), nested);
        fb_field!(map, "maxDeployCount", self.maxDeployCount(), nested);
        fb_field!(map, "massLevel", self.massLevel(), nested);
        fb_field!(map, "baseForceLevel", self.baseForceLevel(), nested);
        fb_field!(map, "tauntLevel", self.tauntLevel(), nested);
        fb_field!(map, "epDamageResistance", self.epDamageResistance(), nested);
        fb_field!(map, "epResistance", self.epResistance(), nested);
        fb_field!(
            map,
            "damageHitratePhysical",
            self.damageHitratePhysical(),
            nested
        );
        fb_field!(
            map,
            "damageHitrateMagical",
            self.damageHitrateMagical(),
            nested
        );
        fb_field!(
            map,
            "epBreakRecoverSpeed",
            self.epBreakRecoverSpeed(),
            nested
        );
        fb_field!(map, "stunImmune", self.stunImmune(), nested);
        fb_field!(map, "silenceImmune", self.silenceImmune(), nested);
        fb_field!(map, "sleepImmune", self.sleepImmune(), nested);
        fb_field!(map, "frozenImmune", self.frozenImmune(), nested);
        fb_field!(map, "levitateImmune", self.levitateImmune(), nested);
        fb_field!(
            map,
            "disarmedCombatImmune",
            self.disarmedCombatImmune(),
            nested
        );
        fb_field!(map, "fearedImmune", self.fearedImmune(), nested);
        fb_field!(map, "palsyImmune", self.palsyImmune(), nested);
        fb_field!(map, "attractImmune", self.attractImmune(), nested);
        Value::Object(map)
    }
}

// ============================================
// Skill-related types
// ============================================

impl FlatBufferToJson for clz_Torappu_LevelData_EnemyData_ESkillData<'_> {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        fb_field!(map, "prefabKey", self.prefabKey(), string);
        map.insert(to_pascal_case("priority"), json!(self.priority()));
        map.insert(to_pascal_case("cooldown"), json!(self.cooldown()));
        map.insert(to_pascal_case("initCooldown"), json!(self.initCooldown()));
        map.insert(to_pascal_case("spCost"), json!(self.spCost()));
        fb_field!(map, "blackboard", self.blackboard(), vec_nested);
        Value::Object(map)
    }
}

impl FlatBufferToJson for clz_Torappu_LevelData_EnemyData_ESpData<'_> {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        map.insert(to_pascal_case("spType"), self.spType().to_json_value());
        map.insert(to_pascal_case("maxSp"), json!(self.maxSp()));
        map.insert(to_pascal_case("initSp"), json!(self.initSp()));
        map.insert(to_pascal_case("increment"), json!(self.increment()));
        Value::Object(map)
    }
}

// ============================================
// EnemyData
// ============================================

impl FlatBufferToJson for clz_Torappu_EnemyDatabase_EnemyData<'_> {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        fb_field!(map, "name", self.name(), nested);
        fb_field!(map, "description", self.description(), nested);
        fb_field!(map, "prefabKey", self.prefabKey(), nested);
        fb_field!(map, "attributes", self.attributes(), nested);
        fb_field!(map, "applyWay", self.applyWay(), nested);
        fb_field!(map, "motion", self.motion(), nested);
        fb_field!(map, "enemyTags", self.enemyTags(), nested);
        fb_field!(map, "lifePointReduce", self.lifePointReduce(), nested);
        fb_field!(map, "levelType", self.levelType(), nested);
        fb_field!(map, "rangeRadius", self.rangeRadius(), nested);
        fb_field!(map, "numOfExtraDrops", self.numOfExtraDrops(), nested);
        fb_field!(map, "viewRadius", self.viewRadius(), nested);
        fb_field!(map, "notCountInTotal", self.notCountInTotal(), nested);
        fb_field!(map, "talentBlackboard", self.talentBlackboard(), vec_nested);
        fb_field!(map, "skills", self.skills(), vec_nested);
        fb_field!(map, "spData", self.spData(), nested);
        Value::Object(map)
    }
}

// ============================================
// EnemyLevel
// ============================================

impl FlatBufferToJson for clz_Torappu_EnemyDatabase_EnemyLevel<'_> {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        map.insert(to_pascal_case("level"), json!(self.level()));
        fb_field!(map, "enemyData", self.enemyData(), nested);
        Value::Object(map)
    }
}

// ============================================
// Key-value pair type
// ============================================

impl FlatBufferToJson for kvp__string__list_clz_Torappu_EnemyDatabase_EnemyLevel<'_> {
    fn to_json(&self) -> Value {
        let mut map = Map::new();
        map.insert("Key".to_string(), json!(self.Key()));
        if let Some(values) = self.Value() {
            let arr: Vec<Value> = (0..values.len()).map(|i| values.get(i).to_json()).collect();
            map.insert("Value".to_string(), json!(arr));
        }
        Value::Object(map)
    }
}

// ============================================
// Root EnemyDatabase type
// ============================================

impl FlatBufferToJson for clz_Torappu_EnemyDatabase<'_> {
    fn to_json(&self) -> Value {
        let mut result = Map::new();

        if let Some(enemies) = self.enemies() {
            let mut enemies_map = Map::new();
            for i in 0..enemies.len() {
                let entry = enemies.get(i);
                let key = entry.Key().to_string();
                if let Some(levels) = entry.Value() {
                    let arr: Vec<Value> =
                        (0..levels.len()).map(|j| levels.get(j).to_json()).collect();
                    enemies_map.insert(key, json!(arr));
                }
            }
            result.insert("Enemies".to_string(), Value::Object(enemies_map));
        }

        Value::Object(result)
    }
}

/// Decode enemy_database data to clean JSON using direct field access
pub fn decode_enemy_database_json(data: &[u8]) -> anyhow::Result<Value> {
    let root = unsafe { root_as_clz_torappu_enemy_database_unchecked(data) };
    Ok(root.to_json())
}
