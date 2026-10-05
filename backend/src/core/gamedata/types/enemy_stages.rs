//! Enemy -> stages index, built at gamedata init from the level files under
//! `gamedata/levels/`. Serves `/static/enemy-stages` so the frontend need not
//! fetch hundreds of level files to say where an enemy shows up.

use serde::Serialize;
use std::collections::{BTreeMap, HashMap};
use ts_rs::TS;

/// A single appearance of an enemy in a stage.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct EnemyStageRef {
    pub stage_id: String,
    /// Human-readable code, e.g. "0-1", "WD-8".
    pub code: String,
    pub zone_id: String,
    /// Resolved display name for the zone/event (e.g. "Stronghold Protocol:
    /// Alliance", "Annihilation"). `None` falls back to `zone_id` in the UI.
    pub zone_name: Option<String>,
    /// Coarse UI bucket: `"stages"` (main story), `"events"` (side stories /
    /// activities), or `"modes"` (permanent game modes - IS, RA, S.S.S., CC,
    /// Annihilation).
    pub category: String,
    /// Fine group key (`story` / `events` / `annihilation` / `is` / `ra` /
    /// `sss` / `paradox` / `cc` / `supplies` / `other`) for grouped UIs.
    pub group: String,
    pub stage_name: Option<String>,
    /// True for hard-mode / Adverse variants (tough levels, 4★/6★ difficulty).
    pub is_hard: bool,
    /// Total spawned across every wave (sum of SPAWN action counts). 0 means
    /// the enemy is declared for the stage but not directly spawned
    /// (summoned, conditional, or a relative-spawn parent).
    pub count: u32,
}

/// `enemy_id -> appearances`. Built by [`build_enemy_stage_index`].
///
/// [`build_enemy_stage_index`]: super::super::enrich::enemy_stages::build_enemy_stage_index
pub type EnemyStageIndex = HashMap<String, Vec<EnemyStageRef>>;

/// One stage as [`EnemyStageTable`] lists it: every [`EnemyStageRef`] field
/// except the per-enemy `count`.
#[derive(Debug, Clone, Serialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct EnemyStageInfo {
    pub stage_id: String,
    pub code: String,
    pub zone_id: String,
    pub zone_name: Option<String>,
    pub category: String,
    pub group: String,
    pub stage_name: Option<String>,
    pub is_hard: bool,
}

/// [`EnemyStageIndex`] with each stage written once instead of once per enemy
/// in it. Lossless: `refs[enemy][i] = [s, count]` is the index's
/// `[enemy][i]` with `stages[s]` for every field but `count`.
///
/// A stage is deduplicated on all its fields, not on `stageId` alone, because
/// the index keys a stage by its level file's name, which two directories
/// could share.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
pub struct EnemyStageTable {
    pub stages: Vec<EnemyStageInfo>,
    /// `enemy_id -> [[index into stages, count], ...]`, in index order.
    pub refs: BTreeMap<String, Vec<(u32, u32)>>,
}

impl EnemyStageTable {
    /// Enemies are walked in id order so stage indices, and with them the
    /// body and its `ETag`, do not depend on `HashMap` order.
    pub fn from_index(index: &EnemyStageIndex) -> Self {
        let mut enemies: Vec<_> = index.iter().collect();
        enemies.sort_unstable_by_key(|(id, _)| id.as_str());

        let mut table = Self::default();
        let mut seen: HashMap<EnemyStageInfo, u32> = HashMap::new();
        for (enemy_id, appearances) in enemies {
            let refs = appearances
                .iter()
                .map(|r| {
                    let info = EnemyStageInfo {
                        stage_id: r.stage_id.clone(),
                        code: r.code.clone(),
                        zone_id: r.zone_id.clone(),
                        zone_name: r.zone_name.clone(),
                        category: r.category.clone(),
                        group: r.group.clone(),
                        stage_name: r.stage_name.clone(),
                        is_hard: r.is_hard,
                    };
                    let next = u32::try_from(table.stages.len()).unwrap_or(u32::MAX);
                    let slot = *seen.entry(info).or_insert_with_key(|info| {
                        table.stages.push(info.clone());
                        next
                    });
                    (slot, r.count)
                })
                .collect();
            table.refs.insert(enemy_id.clone(), refs);
        }
        table
    }
}

#[cfg(test)]
mod tests {
    use super::{EnemyStageIndex, EnemyStageRef, EnemyStageTable};

    fn stage(stage_id: &str, zone_name: Option<&str>, count: u32) -> EnemyStageRef {
        EnemyStageRef {
            stage_id: stage_id.to_owned(),
            code: stage_id.to_uppercase(),
            zone_id: "main_1".to_owned(),
            zone_name: zone_name.map(str::to_owned),
            category: "stages".to_owned(),
            group: "story".to_owned(),
            stage_name: None,
            is_hard: false,
            count,
        }
    }

    #[test]
    fn table_writes_each_stage_once_and_rebuilds_the_index() {
        let index: EnemyStageIndex = [
            (
                "enemy_b".to_owned(),
                vec![stage("s1", Some("Zone"), 3), stage("s2", None, 0)],
            ),
            ("enemy_a".to_owned(), vec![stage("s1", Some("Zone"), 7)]),
            // Same stage id, different zone name: kept as its own stage.
            ("enemy_c".to_owned(), vec![stage("s1", None, 1)]),
        ]
        .into_iter()
        .collect();

        let table = EnemyStageTable::from_index(&index);
        assert_eq!(table.stages.len(), 3);
        assert_eq!(table.refs["enemy_a"], [(0, 7)]);
        assert_eq!(table.refs["enemy_b"], [(0, 3), (1, 0)]);
        assert_eq!(table.refs["enemy_c"], [(2, 1)]);

        for (enemy, refs) in &index {
            let rebuilt: Vec<_> = table.refs[enemy]
                .iter()
                .map(|&(s, count)| {
                    let info = &table.stages[s as usize];
                    EnemyStageRef {
                        stage_id: info.stage_id.clone(),
                        code: info.code.clone(),
                        zone_id: info.zone_id.clone(),
                        zone_name: info.zone_name.clone(),
                        category: info.category.clone(),
                        group: info.group.clone(),
                        stage_name: info.stage_name.clone(),
                        is_hard: info.is_hard,
                        count,
                    }
                })
                .collect();
            assert_eq!(
                serde_json::to_value(&rebuilt).unwrap(),
                serde_json::to_value(refs).unwrap()
            );
        }
    }
}
