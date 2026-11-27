use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;

use myrtle_db::repos::users::{self, ProfileRow};
use myrtle_db::repos::{progress, roster, stats};
use myrtle_engine_base::assignment::{
    align_rooms_to_current, compute_live_assignment, compute_sustained_assignment, morale_recovery,
    op_uptime,
};
use myrtle_engine_base::registry::{BuffResolutionStrategy, resolve_account_facts};
use myrtle_engine_base::rotation::shift::recommend_shift_rotation;
use myrtle_engine_base::rotation::sustain_sim::synced_live_morale;
use myrtle_engine_base::scoring::pools::{
    has_morale_conditional_grant, optimal_with_bundles, search_economy,
};
use myrtle_engine_base::types::{OperatorBaseProfile, RotationAssignment, UserBuilding};
use myrtle_engine_base::{BaseContext, BaseData};
use myrtle_engine_grade::StageClears;
use myrtle_engine_grade::improvements::{
    MedalImprovements, OperatorImprovements, RoguelikeThemeImprovement, SandboxImprovements,
    StageImprovements, medal_improvements, operator_improvements, roguelike_improvements,
    sandbox_improvements, stage_improvements,
};
use myrtle_types::roster::RosterEntry;
use myrtle_types::{Server, UserId};
use serde::Serialize;
use serde_json::Value;

use crate::base_planner::{
    AssignedOperator, BaseAssignmentDto, ClaimDto, ShiftRotationDto, SustainabilityDto,
    assigned_operator, base_assignment_to_dto, claim_of, shift_rotation_to_dto,
    static_sustainability,
};
use crate::cache::CachedJson;
use crate::privacy::{self, layout::ProfileTabId};
use crate::services::grading::score_model;
use crate::snapshots::Snapshot;
use crate::{AppContext, AppError, Subject};

const TTL: Duration = Duration::from_mins(5);

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct ImprovementsResponse {
    pub uid: String,
    pub stages: StageImprovements,
    pub roguelike: Vec<RoguelikeThemeImprovement>,
    pub sandbox: SandboxImprovements,
    pub medals: MedalImprovements,
    pub operators: OperatorImprovements,
    pub base: BaseImprovements,
}

#[derive(Debug, Clone, Serialize, Default, ts_rs::TS)]
#[ts(export)]
pub struct BaseImprovements {
    pub current: Option<BaseAssignmentDto>,
    pub optimal: Option<BaseAssignmentDto>,
    pub rotation: Option<RotationDto>,
    pub layout: Vec<RoomLayoutEntry>,
    pub shift_rotation: Option<ShiftRotationDto>,
    pub perception: Option<PerceptionPlanDto>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub claim: Option<ClaimDto>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unrotated: Option<SustainabilityDto>,
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct PerceptionPlanDto {
    pub support: Vec<PerceptionSupportDto>,
    pub consumers: Vec<PerceptionConsumerDto>,
    pub rotation_manager: Option<AssignedOperator>,
    pub needs_rotation_manager: bool,
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct PerceptionSupportDto {
    pub operator: AssignedOperator,
    pub room_type: String,
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct PerceptionConsumerDto {
    pub operator: AssignedOperator,
    pub room_type: String,
    pub bonus_pct: f64,
    pub sustained_pct: f64,
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct RotationDto {
    pub rooms: Vec<RoomRotationDto>,
    pub shared_bench: Vec<AssignedOperator>,
    pub sets: Vec<RotationSetDto>,
    pub sustained_efficiency: f64,
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct RotationSetDto {
    pub rooms: Vec<RotationSetRoomDto>,
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct RotationSetRoomDto {
    pub slot_id: String,
    pub room_type: String,
    pub working: Vec<AssignedOperator>,
    pub resting: Option<AssignedOperator>,
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct RoomRotationDto {
    pub slot_id: String,
    pub room_type: String,
    pub members: Vec<RotationMemberDto>,
    pub backup: Option<AssignedOperator>,
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct RotationMemberDto {
    pub operator: AssignedOperator,
    pub lasts_hours: f64,
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct RoomLayoutEntry {
    pub room_type: String,
    pub count: usize,
    pub levels: Vec<i32>,
}

#[must_use]
pub fn cache_key(uid: &str, updated_at_ms: i64) -> String {
    format!("improvements:{uid}:{updated_at_ms}")
}

pub async fn improvements(
    ctx: &AppContext,
    subject: &Subject,
    uid: Option<&str>,
) -> Result<CachedJson, AppError> {
    let profile = if let Some(uid) = uid {
        privacy::resolve_profile_for_tabs(ctx, subject, uid, &[ProfileTabId::Score])
            .await?
            .0
    } else {
        let user = subject.user_id().ok_or(AppError::Unauthorized)?;
        users::profile(&mut *ctx.db.conn().await?, user)
            .await?
            .ok_or(AppError::Unauthorized)?
    };
    let snapshot = ctx.snapshots.or_default(ctx.snapshots.default_server())?;
    let key = snapshot.key(&cache_key(
        &profile.uid,
        profile.updated_at.timestamp_millis(),
    ));
    let owner = ctx.clone();
    ctx.cache
        .json_detached(&key, TTL, move || async move {
            let body = build(&owner, snapshot, profile).await?;
            serde_json::to_string(&body).map_err(AppError::internal)
        })
        .await
}

async fn build(
    ctx: &AppContext,
    snapshot: Arc<Snapshot>,
    profile: ProfileRow,
) -> Result<ImprovementsResponse, AppError> {
    let user = UserId(profile.id);
    let (
        entries,
        supports,
        clears,
        known,
        themes,
        medals,
        sandbox,
        building,
        open_recruit_slots,
        (population, owners),
    ) = {
        let mut conn = ctx.db.conn().await?;
        let conn = &mut *conn;
        let building = progress::building(conn, user).await?;
        let open_recruit_slots = if building.is_some() {
            users::base_facts(conn, user)
                .await?
                .and_then(|facts| facts.get("open_recruit_slots")?.as_u64())
                .filter(|&n| n > 0)
                .map(|n| u32::try_from(n.min(3)).unwrap_or(3))
        } else {
            None
        };
        (
            roster::roster(conn, user).await?,
            roster::supports(conn, user).await?,
            progress::stage_clears(conn, user).await?,
            progress::known_stage_ids_for_server(conn, user).await?,
            progress::roguelike(conn, user).await?,
            progress::medals(conn, user).await?,
            progress::sandbox(conn, user).await?,
            building,
            open_recruit_slots,
            match Server::parse(&profile.server) {
                Some(server) => stats::medal_ownership(conn, server).await?,
                None => (0, Vec::new()),
            },
        )
    };
    let model = score_model(ctx);
    let now = chrono::Utc::now().timestamp();

    ctx.cpu
        .run("user_improvements", move || {
            let data = &snapshot.grade;
            let support_ids: HashSet<&str> =
                supports.iter().map(|s| s.operator_id.as_str()).collect();
            let owned: HashSet<&str> = entries.iter().map(|e| e.operator_id.as_str()).collect();
            let themes: Vec<(String, Value)> = themes
                .into_iter()
                .map(|r| (r.theme_id, r.progress))
                .collect();
            let medals: Vec<_> = medals
                .into_iter()
                .map(|m| (m.medal_id, m.val, m.first_ts, m.reach_ts))
                .collect();
            let owners: HashMap<String, i64> =
                owners.into_iter().map(|r| (r.medal_id, r.owners)).collect();
            let clears = StageClears {
                clears: clears.clears,
                last_synced_ts: clears.last_synced_ts,
            };
            ImprovementsResponse {
                stages: stage_improvements(
                    data,
                    &snapshot.model.stages,
                    &snapshot.model.campaign_rotations,
                    &clears,
                    &known,
                    now,
                ),
                roguelike: roguelike_improvements(data, &themes),
                sandbox: sandbox_improvements(data, sandbox.as_ref()),
                medals: medal_improvements(data, &medals, &owned, population, &owners, now),
                operators: operator_improvements(&entries, data, &support_ids, model),
                base: building
                    .as_ref()
                    .map_or_else(BaseImprovements::default, |json| {
                        base_improvements(&entries, &snapshot.base, json, open_recruit_slots)
                    }),
                uid: profile.uid,
            }
        })
        .await
}

#[must_use]
pub fn base_improvements(
    roster: &[RosterEntry],
    data: &BaseData,
    building_json: &Value,
    open_recruit_slots: Option<u32>,
) -> BaseImprovements {
    let building = UserBuilding::from_json(building_json);
    if building.is_empty() {
        return BaseImprovements::default();
    }
    let building_data = data.building();
    let mut base = BaseContext::build(roster, data, false);
    if let Some(slots) = open_recruit_slots {
        base.registry = Arc::new(resolve_account_facts(
            &base.registry,
            &building_data.buffs,
            slots,
        ));
    }
    let BaseContext {
        profiles,
        registry,
        morale_drains,
    } = base;
    let has_conditional_generator = profiles
        .iter()
        .any(|op| has_morale_conditional_grant(op, building_data));
    let economy = search_economy(&profiles, &building, building_data, &registry);
    let current = compute_live_assignment(
        &profiles,
        &building,
        building_data,
        &registry,
        &morale_drains,
        None,
        &synced_live_morale(building_json),
    );
    let accepted = optimal_with_bundles(
        &profiles,
        &building,
        building_data,
        &registry,
        &economy.registry,
        &morale_drains,
        &economy.pins,
    );
    let mut optimal = accepted.optimal;
    align_rooms_to_current(&building, &mut optimal);
    let sustained = compute_sustained_assignment(
        &profiles,
        &building,
        building_data,
        &registry,
        &morale_drains,
    );
    let shift_rotation = has_shift_rotation_layout(&building).then(|| {
        let rotation = recommend_shift_rotation(
            &profiles,
            &building,
            building_data,
            &accepted.registry,
            &morale_drains,
            &accepted.pins,
        );
        shift_rotation_to_dto(
            &rotation,
            data,
            &profiles,
            &building,
            &accepted.registry,
            &morale_drains,
        )
    });
    let current_dto = base_assignment_to_dto(&current, data, &profiles, &registry);
    BaseImprovements {
        claim: claim_of(&current_dto, None),
        unrotated: static_sustainability(
            &building,
            &current,
            &profiles,
            data,
            &registry,
            &morale_drains,
            None,
        ),
        current: Some(current_dto),
        optimal: Some(base_assignment_to_dto(
            &optimal,
            data,
            &profiles,
            &accepted.registry,
        )),
        rotation: Some(rotation_dto(&sustained, data)),
        layout: layout_summary(&building),
        shift_rotation,
        perception: economy_plan(
            &registry,
            &accepted.registry,
            &accepted.pins,
            &profiles,
            &building,
            &morale_drains,
            economy.manager.as_deref(),
            has_conditional_generator,
            data,
        ),
    }
}

#[allow(clippy::too_many_arguments)]
fn economy_plan(
    base_registry: &HashMap<String, BuffResolutionStrategy>,
    optimal_registry: &HashMap<String, BuffResolutionStrategy>,
    optimal_pins: &[(String, String)],
    profiles: &[OperatorBaseProfile],
    building: &UserBuilding,
    morale_drains: &HashMap<String, f64>,
    rotation_manager: Option<&str>,
    has_conditional_generator: bool,
    data: &BaseData,
) -> Option<PerceptionPlanDto> {
    let mut seen = HashSet::new();
    let support: Vec<&(String, String)> = optimal_pins
        .iter()
        .filter(|(id, room)| {
            Some(id.as_str()) != rotation_manager
                && room != "MANUFACTURE"
                && room != "TRADING"
                && seen.insert(id.as_str())
        })
        .collect();
    let recovery = morale_recovery(building);
    let sustain_factor = |consumer: &str| {
        let ups: Vec<f64> = support
            .iter()
            .filter(|(id, _)| id != consumer)
            .filter_map(|(id, _)| profiles.iter().find(|p| &p.char_id == id))
            .map(|p| op_uptime(p, morale_drains, recovery))
            .collect();
        if ups.is_empty() {
            1.0
        } else {
            ups.iter().sum::<f64>() / ups.len() as f64
        }
    };

    let mut by_char: HashMap<&str, PerceptionConsumerDto> = HashMap::new();
    for (buff_id, strategy) in optimal_registry {
        let (room_type, pct) = match strategy {
            BuffResolutionStrategy::PoolPayoff { pct } => {
                let Some(buff) = data.building().buffs.get(buff_id) else {
                    continue;
                };
                (buff.room_type.clone(), *pct)
            }
            BuffResolutionStrategy::GlobalEffect {
                target_room,
                bonus_pct,
            } if base_registry.get(buff_id) != Some(strategy) => (target_room.clone(), *bonus_pct),
            _ => continue,
        };
        if pct <= 0.0 {
            continue;
        }
        let Some(owner) = profiles
            .iter()
            .find(|p| p.available_buffs.iter().any(|b| b == buff_id))
        else {
            continue;
        };
        let slot = by_char
            .entry(&owner.char_id)
            .or_insert_with(|| PerceptionConsumerDto {
                operator: assigned_operator(&owner.char_id, data),
                room_type: room_type.clone(),
                bonus_pct: 0.0,
                sustained_pct: 0.0,
            });
        if pct > slot.bonus_pct {
            slot.room_type = room_type;
            slot.bonus_pct = pct;
            slot.sustained_pct = pct * sustain_factor(&owner.char_id);
        }
    }
    if by_char.is_empty() {
        return None;
    }
    let mut consumers: Vec<PerceptionConsumerDto> = by_char.into_values().collect();
    consumers.sort_by(|a, b| b.bonus_pct.total_cmp(&a.bonus_pct));
    Some(PerceptionPlanDto {
        support: support
            .into_iter()
            .map(|(id, room)| PerceptionSupportDto {
                operator: assigned_operator(id, data),
                room_type: room.clone(),
            })
            .collect(),
        consumers,
        rotation_manager: rotation_manager.map(|id| assigned_operator(id, data)),
        needs_rotation_manager: has_conditional_generator && rotation_manager.is_none(),
    })
}

fn has_shift_rotation_layout(building: &UserBuilding) -> bool {
    let count = |room_type: &str| {
        building
            .rooms
            .iter()
            .filter(|r| r.room_type == room_type)
            .count()
    };
    count("TRADING") >= 1 && count("MANUFACTURE") >= 2 && count("POWER") >= 1
}

fn rotation_dto(asn: &RotationAssignment, data: &BaseData) -> RotationDto {
    let op = |id: &str| assigned_operator(id, data);
    RotationDto {
        rooms: asn
            .rooms
            .iter()
            .map(|r| RoomRotationDto {
                slot_id: r.slot_id.clone(),
                room_type: r.room_type.clone(),
                members: r
                    .members
                    .iter()
                    .map(|m| RotationMemberDto {
                        operator: op(&m.operator),
                        lasts_hours: m.lasts_hours,
                    })
                    .collect(),
                backup: r.backup.as_deref().map(op),
            })
            .collect(),
        shared_bench: asn.shared_bench.iter().map(|id| op(id)).collect(),
        sets: asn
            .sets
            .iter()
            .map(|s| RotationSetDto {
                rooms: s
                    .rooms
                    .iter()
                    .map(|r| RotationSetRoomDto {
                        slot_id: r.slot_id.clone(),
                        room_type: r.room_type.clone(),
                        working: r.working.iter().map(|id| op(id)).collect(),
                        resting: r.resting.as_deref().map(op),
                    })
                    .collect(),
            })
            .collect(),
        sustained_efficiency: asn.sustained_efficiency,
    }
}

#[must_use]
pub fn layout_summary(building: &UserBuilding) -> Vec<RoomLayoutEntry> {
    let mut by_type: BTreeMap<&str, Vec<i32>> = BTreeMap::new();
    for room in &building.rooms {
        by_type.entry(&room.room_type).or_default().push(room.level);
    }
    by_type
        .into_iter()
        .map(|(room_type, mut levels)| {
            levels.sort_unstable_by(|a, b| b.cmp(a));
            RoomLayoutEntry {
                room_type: room_type.to_owned(),
                count: levels.len(),
                levels,
            }
        })
        .collect()
}
