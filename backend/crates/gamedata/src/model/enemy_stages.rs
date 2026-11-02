use serde::Serialize;
use std::collections::{BTreeMap, HashMap};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnemyStageRef {
    pub stage_id: String,
    /// e.g. "0-1", "WD-8".
    pub code: String,
    pub zone_id: String,
    /// Resolved display name for the zone/event (e.g. "Stronghold Protocol:
    /// Alliance", "Annihilation")
    pub zone_name: Option<String>,
    /// Coarse UI bucket: `"stages"` (main story), `"events"` (side stories /
    /// activities), or `"modes"` (permanent modes: IS, RA, S.S.S., CC,
    /// Annihilation)
    pub category: String,
    /// Fine group key (`story` / `events` / `annihilation` / `is` / `ra` /
    /// `sss` / `paradox` / `cc` / `supplies` / `other`)
    pub group: String,
    pub stage_name: Option<String>,
    /// True for hard-mode / Adverse variants (tough levels, 4★/6★ difficulty).
    pub is_hard: bool,
    /// Total spawned across every wave (sum of SPAWN action counts)
    pub count: u32,
    #[serde(skip)]
    pub battle: Option<StageEnemyBattle>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct StageEnemyBattle {
    pub level: i32,
    pub def: Option<f64>,
    pub res: Option<f64>,
}

pub type EnemyStageIndex = HashMap<String, Vec<EnemyStageRef>>;

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
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

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnemyStageTable {
    pub stages: Vec<EnemyStageInfo>,
    pub refs: BTreeMap<String, Vec<(u32, u32)>>,
}

impl EnemyStageTable {
    #[must_use]
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
