mod dto;

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock};
use std::time::{Duration, Instant};

use dashmap::DashMap;
use myrtle_db::repos::{progress, roster, users};
use myrtle_engine_base::assignment::scope::settle_scoped_proposal;
use myrtle_engine_base::assignment::{
    align_rooms_to_current, compute_current_assignment, compute_live_assignment, morale_recovery,
};
use myrtle_engine_base::registry::resolve_account_facts;
use myrtle_engine_base::rotation::dorms::{dorm_aura_value, dorm_list, dorm_single_value};
use myrtle_engine_base::rotation::shift::{SHIFT_COUNT, recommend_shift_rotation};
use myrtle_engine_base::rotation::sustain_sim::{
    game_morale_drain, project_morale, synced_live_morale,
};
use myrtle_engine_base::rotation::value::{
    StaffingValue, plan_cycle, presets_value, rotation_value,
};
use myrtle_engine_base::scoring::pools::{AcceptedEconomy, optimal_with_bundles, search_economy};
use myrtle_engine_base::scoring::yield_model::room_yield;
use myrtle_engine_base::types::{
    BaseAssignment, OperatorBaseProfile, UserBuilding, UserRoom, live_morale_snapshot,
};
use myrtle_engine_base::{BaseContext, BaseData, is_production_room};
use myrtle_gamedata_raw::tables::building::BuildingDataFile;
use myrtle_types::UserId;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub use self::dto::{
    AssignedOperator, BaseAssignmentDto, DepletedOperatorDto, FacilityOutputDto, MoraleTimelineDto,
    MovedOperator, NonProdEffectDto, RoomAssignmentDto, ShiftDto, ShiftRoomDto, ShiftRotationDto,
    SkillLineDto, SustainabilityDto,
};
pub use self::dto::{
    assigned_operator, base_assignment_to_dto, operator_name, shift_rotation_to_dto,
    static_sustainability,
};
use crate::cache::{CachedJson, key_hash};
use crate::policy::Subject;
use crate::privacy::{self, layout::ProfileTabId};
use crate::snapshots::Snapshot;
use crate::{AppContext, AppError};

const MAX_ROOMS: usize = 40;
const SEARCH_TTL: Duration = Duration::from_mins(5);
const PLAN_TTL: Duration = Duration::from_mins(10);
const PLAN_CAP: usize = 64;
static PLANS: LazyLock<DashMap<String, (Instant, Arc<AcceptedEconomy>)>> =
    LazyLock::new(DashMap::new);
const MAX_MORALE: f64 = 24.0;
const UNMODELED_ROOM_TYPES: [&str; 2] = ["TRAINING", "WORKSHOP"];
const TRAINER_CLASSES: [&str; 8] = [
    "Vanguard",
    "Guard",
    "Defender",
    "Sniper",
    "Caster",
    "Medic",
    "Supporter",
    "Specialist",
];

#[derive(Debug, Clone, Deserialize, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct DraftRoom {
    pub slot_id: String,
    pub room_type: String,
    pub level: i32,
    #[serde(default)]
    pub operators: Vec<String>,
    #[serde(default)]
    pub formula_type: Option<String>,
    #[serde(default)]
    pub comfort: i32,
}

impl DraftRoom {
    fn into_user_room(self) -> UserRoom {
        UserRoom {
            slot_id: self.slot_id,
            room_type: self.room_type,
            level: self.level,
            current_operators: self.operators,
            current_formula: self.formula_type,
            preset_shifts: Vec::new(),
            comfort: self.comfort,
            frozen: false,
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct AccountFactsReq {
    #[serde(default)]
    pub open_recruit_slots: Option<u32>,
    #[serde(default)]
    pub training_class: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EvaluateRequest {
    pub layout: Vec<DraftRoom>,
    #[serde(default)]
    pub claim_interval_hours: Option<f64>,
    #[serde(default)]
    pub ignore_promotion: bool,
    #[serde(default)]
    pub facts: AccountFactsReq,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OptimizeRequest {
    pub layout: Vec<DraftRoom>,
    #[serde(default)]
    pub scope: Vec<String>,
    #[serde(default)]
    pub locked: Vec<String>,
    #[serde(default)]
    pub excluded: Vec<String>,
    #[serde(default)]
    pub ignore_promotion: bool,
    #[serde(default)]
    pub facts: AccountFactsReq,
}

#[derive(Debug, Clone, Deserialize, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct RotationRequest {
    pub layout: Vec<DraftRoom>,
    #[serde(default)]
    pub locked: Vec<String>,
    #[serde(default)]
    pub excluded: Vec<String>,
    #[serde(default)]
    pub ignore_promotion: bool,
    #[serde(default)]
    pub facts: AccountFactsReq,
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct LayoutResponse {
    pub rooms: Vec<DraftRoomDto>,
    pub presets: Vec<SlotPresetsDto>,
    #[serde(default)]
    pub facts: AccountFactsReq,
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct SlotPresetsDto {
    pub slot_id: String,
    pub shifts: Vec<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct DraftRoomDto {
    pub slot_id: String,
    pub room_type: String,
    pub level: i32,
    pub operators: Vec<String>,
    pub formula_type: Option<String>,
    pub comfort: i32,
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct EvaluateResponse {
    pub assignment: BaseAssignmentDto,
    pub power: PowerDto,
    pub sustain: Vec<SustainEntryDto>,
    pub dorms: DormsDto,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub claim: Option<ClaimDto>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unrotated: Option<SustainabilityDto>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub morale_synced_hours_ago: Option<f64>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub trainer_hints: Vec<TrainerHintDto>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub drones: Option<DronesDto>,
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct TrainerHintDto {
    pub operator: AssignedOperator,
    pub value_pct: f64,
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct DronesDto {
    pub current: f64,
    pub max: i32,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full_in_hours: Option<f64>,
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct ClaimDto {
    pub next_full_hours: f64,
    pub intervals: Vec<ClaimIntervalDto>,
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct ClaimIntervalDto {
    pub hours: f64,
    pub lost_lmd_per_day: f64,
    pub lost_gold_per_day: f64,
    pub lost_exp_per_day: f64,
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct DormsDto {
    pub count: usize,
    pub total_levels: i32,
    pub total_capacity: i32,
    pub recovery_per_hour: f64,
    pub per_dorm: Vec<DormDto>,
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct DormDto {
    pub slot_id: String,
    pub level: i32,
    pub capacity: usize,
    pub recovery_per_hour: f64,
    pub occupant_aura_per_hour: f64,
    pub occupant_single_per_hour: f64,
    pub comfort: i32,
    pub comfort_limit: i32,
    pub comfort_upside_per_hour: f64,
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct SustainEntryDto {
    pub operator_id: String,
    pub name: String,
    pub slot_id: String,
    pub room_type: String,
    pub drain_per_hour: f64,
    pub lasts_hours: Option<f64>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub morale: Option<f64>,
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct PowerDto {
    pub generated: i32,
    pub consumed: i32,
    pub net: i32,
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct OptimizeResponse {
    pub proposal: BaseAssignmentDto,
    pub baseline: BaseAssignmentDto,
    pub room_diffs: Vec<RoomDiffDto>,
    pub power: PowerDto,
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct RoomDiffDto {
    pub slot_id: String,
    pub room_type: String,
    pub before: Vec<String>,
    pub after: Vec<String>,
    pub efficiency_before: f64,
    pub efficiency_after: f64,
    pub yield_before: f64,
    pub yield_after: f64,
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct RotationResponse {
    pub rotation: ShiftRotationDto,
    pub shift_count: usize,
    pub comparison: RotationComparisonDto,
    pub cycle: Option<CycleDto>,
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct CycleDto {
    pub legs: Vec<CycleLegDto>,
    pub value: RotationValueDto,
    pub beats_rotation: bool,
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct CycleLegDto {
    pub hours: f64,
    pub rooms: Vec<CycleRoomDto>,
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct CycleRoomDto {
    pub slot_id: String,
    pub room_type: String,
    pub operators: Vec<AssignedOperator>,
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct RotationComparisonDto {
    pub recommended: RotationValueDto,
    pub presets: Option<RotationValueDto>,
    pub matching_cells: usize,
    pub production_cells: usize,
    pub preset_legs_hours: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct RotationValueDto {
    pub production_efficiency: f64,
    pub lmd_per_day: f64,
    pub exp_per_day: f64,
    pub total_value: f64,
}

impl From<StaffingValue> for RotationValueDto {
    fn from(v: StaffingValue) -> Self {
        Self {
            production_efficiency: v.production_efficiency,
            lmd_per_day: v.lmd_per_day,
            exp_per_day: v.exp_per_day,
            total_value: v.total_value,
        }
    }
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct CatalogResponse {
    pub shift_count: usize,
    pub rooms: Vec<CatalogRoomDto>,
    pub formulas: Vec<CatalogFormulaDto>,
    pub slots: Vec<CatalogSlotDto>,
    pub storeys: Vec<CatalogStoreyDto>,
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct CatalogSlotDto {
    pub slot_id: String,
    pub category: String,
    pub storey_id: String,
    pub offset_col: i32,
    pub offset_row: i32,
    pub size_col: i32,
    pub size_row: i32,
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct CatalogStoreyDto {
    pub storey_id: String,
    pub unlock_control_level: i32,
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct CatalogRoomDto {
    pub room_type: String,
    pub name: String,
    pub category: String,
    /// -1 is unlimited
    pub max_count: i32,
    pub size_col: i32,
    pub size_row: i32,
    pub phases: Vec<CatalogPhaseDto>,
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct CatalogPhaseDto {
    pub level: i32,
    pub max_stationed: i32,
    pub electricity: i32,
    pub manpower_cost: i32,
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct CatalogFormulaDto {
    pub formula_type: String,
    pub label: String,
}

struct Planning {
    base: BaseContext,
    synced: Option<Value>,
    facts: AccountFactsReq,
}

fn game_data(ctx: &AppContext) -> Result<Arc<Snapshot>, AppError> {
    ctx.snapshots.or_default(ctx.snapshots.default_server())
}

fn facts_of(stored: Option<Value>) -> AccountFactsReq {
    stored
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default()
}

async fn target(
    ctx: &AppContext,
    subject: &Subject,
    uid: Option<&str>,
) -> Result<(UserId, String), AppError> {
    if let Some(uid) = uid {
        let user =
            privacy::resolve_user_id(ctx, subject, Some(uid), &[ProfileTabId::Optimizer]).await?;
        return Ok((user, uid.to_owned()));
    }
    let uid = privacy::resolve_uid(ctx, subject, None, ProfileTabId::Optimizer).await?;
    Ok((subject.user_id().ok_or(AppError::Unauthorized)?, uid))
}

async fn planning(
    ctx: &AppContext,
    user: UserId,
    data: &BaseData,
    ignore_promotion: bool,
    requested: AccountFactsReq,
) -> Result<Planning, AppError> {
    let mut conn = ctx.db.conn().await?;
    let entries = roster::roster(&mut conn, user).await?;
    let synced = progress::building(&mut conn, user).await?;
    let saved = facts_of(users::base_facts(&mut conn, user).await?);
    drop(conn);
    let facts = AccountFactsReq {
        open_recruit_slots: requested.open_recruit_slots.or(saved.open_recruit_slots),
        training_class: requested.training_class.or(saved.training_class),
    };
    let mut base = BaseContext::build(&entries, data, ignore_promotion);
    if let Some(slots) = facts.open_recruit_slots.filter(|&n| n > 0) {
        base.registry = Arc::new(resolve_account_facts(
            &base.registry,
            &data.building().buffs,
            slots.min(3),
        ));
    }
    Ok(Planning {
        base,
        synced,
        facts,
    })
}

fn validate(layout: &[DraftRoom], building: &BuildingDataFile) -> Result<(), AppError> {
    if layout.is_empty() {
        return Err(AppError::bad_request("layout is empty"));
    }
    if layout.len() > MAX_ROOMS {
        return Err(AppError::bad_request(format!(
            "layout has {} rooms, more than the {MAX_ROOMS} a base can hold",
            layout.len()
        )));
    }
    let mut slots: HashSet<&str> = HashSet::new();
    let mut per_type: HashMap<&str, i32> = HashMap::new();
    for room in layout {
        if !slots.insert(&room.slot_id) {
            return Err(AppError::bad_request(format!(
                "duplicate slot '{}'",
                room.slot_id
            )));
        }
        let Some(def) = building.rooms.get(&room.room_type) else {
            return Err(AppError::bad_request(format!(
                "unknown room type '{}'",
                room.room_type
            )));
        };
        let max_level = i32::try_from(def.phases.len()).unwrap_or(i32::MAX);
        if room.level < 1 || room.level > max_level {
            return Err(AppError::bad_request(format!(
                "room '{}' has level {}, outside 1..={max_level}",
                room.slot_id, room.level
            )));
        }
        let seats = phase(def, room.level).map_or(0, |p| p.max_stationed_num);
        if i32::try_from(room.operators.len()).unwrap_or(i32::MAX) > seats {
            return Err(AppError::bad_request(format!(
                "room '{}' has {} operators but only {seats} seats at level {}",
                room.slot_id,
                room.operators.len(),
                room.level
            )));
        }
        let count = per_type.entry(&room.room_type).or_default();
        *count += 1;
        if def.max_count > 0 && *count > def.max_count {
            return Err(AppError::bad_request(format!(
                "layout has {count} '{}' rooms, more than the {} a base can hold",
                room.room_type, def.max_count
            )));
        }
    }
    let mut placed: HashSet<&str> = HashSet::new();
    for op in layout.iter().flat_map(|r| &r.operators) {
        if !placed.insert(op) {
            return Err(AppError::bad_request(format!(
                "operator '{op}' is stationed in more than one room"
            )));
        }
    }
    Ok(())
}

fn phase(
    def: &myrtle_gamedata_raw::tables::building::RoomDef,
    level: i32,
) -> Option<&myrtle_gamedata_raw::tables::building::RoomPhase> {
    def.phases
        .get(usize::try_from(level.max(1) - 1).unwrap_or(0))
}

fn draft_building(layout: Vec<DraftRoom>, drone_capacity: Option<i32>) -> UserBuilding {
    UserBuilding {
        rooms: layout.into_iter().map(DraftRoom::into_user_room).collect(),
        drone_capacity,
    }
}

pub async fn save_facts(
    ctx: &AppContext,
    subject: &Subject,
    facts: AccountFactsReq,
) -> Result<AccountFactsReq, AppError> {
    let user = subject.user_id().ok_or(AppError::Unauthorized)?;
    let value = serde_json::to_value(&facts).map_err(|_| AppError::bad_request("facts"))?;
    let mut conn = ctx.db.conn().await?;
    users::set_base_facts(&mut conn, user, &value).await?;
    Ok(facts)
}

pub async fn layout(
    ctx: &AppContext,
    subject: &Subject,
    uid: Option<&str>,
) -> Result<LayoutResponse, AppError> {
    let user = privacy::resolve_user_id(ctx, subject, uid, &[ProfileTabId::Optimizer]).await?;
    let mut conn = ctx.db.conn().await?;
    let facts = facts_of(users::base_facts(&mut conn, user).await?);
    let Some(json) = progress::building(&mut conn, user).await? else {
        return Ok(LayoutResponse {
            rooms: Vec::new(),
            presets: Vec::new(),
            facts,
        });
    };
    drop(conn);
    let building = UserBuilding::from_json(&json);
    let mut presets: Vec<SlotPresetsDto> = building
        .rooms
        .iter()
        .filter(|room| room.preset_shifts.iter().any(|shift| !shift.is_empty()))
        .map(|room| SlotPresetsDto {
            slot_id: room.slot_id.clone(),
            shifts: room.preset_shifts.clone(),
        })
        .collect();
    presets.sort_by(|a, b| a.slot_id.cmp(&b.slot_id));
    let mut rooms: Vec<DraftRoomDto> = building
        .rooms
        .into_iter()
        .map(|room| DraftRoomDto {
            slot_id: room.slot_id,
            room_type: room.room_type,
            level: room.level,
            operators: room.current_operators,
            formula_type: room.current_formula,
            comfort: room.comfort,
        })
        .collect();
    rooms.sort_by(|a, b| a.slot_id.cmp(&b.slot_id));
    Ok(LayoutResponse {
        rooms,
        presets,
        facts,
    })
}

pub async fn evaluate(
    ctx: &AppContext,
    subject: &Subject,
    uid: Option<&str>,
    req: EvaluateRequest,
) -> Result<EvaluateResponse, AppError> {
    let user = privacy::resolve_user_id(ctx, subject, uid, &[ProfileTabId::Optimizer]).await?;
    let snapshot = game_data(ctx)?;
    validate(&req.layout, snapshot.base.building())?;
    let _admission = ctx.cpu.admit_light("base_evaluate").await?;
    let planned = planning(
        ctx,
        user,
        &snapshot.base,
        req.ignore_promotion,
        req.facts.clone(),
    )
    .await?;
    let now = chrono::Utc::now().timestamp();
    ctx.cpu
        .offload("base_evaluate", move || {
            evaluate_planned(&snapshot.base, planned, req, now)
        })
        .await
}

fn evaluate_planned(
    data: &BaseData,
    planned: Planning,
    req: EvaluateRequest,
    now_unix: i64,
) -> EvaluateResponse {
    let Planning {
        base,
        synced,
        facts,
    } = planned;
    let building_data = data.building();
    let mut live_morale = HashMap::new();
    let mut synced_bars = HashMap::new();
    let mut morale_synced_hours_ago = None;
    let mut drone_capacity = None;
    if let Some(json) = &synced {
        let snapshots = live_morale_snapshot(json);
        let latest = snapshots.values().map(|s| s.at_unix).max().unwrap_or(0);
        morale_synced_hours_ago = (latest > 0).then(|| hours_between(latest, now_unix));
        let real = UserBuilding::from_json(json);
        live_morale = project_morale(
            &snapshots,
            now_unix,
            &real,
            &base.profiles,
            building_data,
            &base.morale_drains,
        );
        synced_bars = synced_live_morale(json);
        drone_capacity = real.drone_capacity;
    }
    let building = draft_building(req.layout, drone_capacity);
    let assignment = compute_live_assignment(
        &base.profiles,
        &building,
        building_data,
        &base.registry,
        &base.morale_drains,
        None,
        &synced_bars,
    );
    let assignment_dto = base_assignment_to_dto(&assignment, data, &base.profiles, &base.registry);
    let trainer_hints = match facts.training_class.as_deref() {
        Some(class) if building.rooms.iter().any(|r| r.room_type == "TRAINING") => {
            trainer_hints(&base, data, class)
        }
        _ => Vec::new(),
    };
    EvaluateResponse {
        claim: claim_of(&assignment_dto, req.claim_interval_hours),
        unrotated: static_sustainability(
            &building,
            &assignment,
            &base.profiles,
            data,
            &base.registry,
            &base.morale_drains,
            Some(&live_morale),
        ),
        power: power_of(&building, building_data),
        sustain: sustain_of(&building, &base, data, &live_morale),
        dorms: dorms_of(&building, &base, building_data),
        assignment: assignment_dto,
        morale_synced_hours_ago,
        trainer_hints,
        drones: synced
            .as_ref()
            .and_then(|json| drones_of(json.get("status")?.get("labor")?, now_unix)),
    }
}

fn hours_between(from_unix: i64, to_unix: i64) -> f64 {
    (to_unix - from_unix).max(0) as f64 / 3600.0
}

fn drones_of(labor: &Value, now_unix: i64) -> Option<DronesDto> {
    let value = labor.get("value").and_then(Value::as_f64)?;
    let max = i32::try_from(labor.get("maxValue").and_then(Value::as_i64)?).ok()?;
    let buff = labor
        .get("buffSpeed")
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    let at = labor
        .get("lastUpdateTime")
        .and_then(Value::as_i64)
        .unwrap_or(0);
    let rate_per_hour = 240.0 / 24.0 * (1.0 + buff);
    let current = rate_per_hour
        .mul_add(hours_between(at, now_unix), value)
        .min(f64::from(max));
    Some(DronesDto {
        current,
        max,
        full_in_hours: (rate_per_hour > 0.0 && current < f64::from(max))
            .then(|| (f64::from(max) - current) / rate_per_hour),
    })
}

#[must_use]
pub fn claim_of(assignment: &BaseAssignmentDto, custom_hours: Option<f64>) -> Option<ClaimDto> {
    let fills: Vec<(&RoomAssignmentDto, f64)> = assignment
        .rooms
        .iter()
        .filter_map(|r| Some((r, r.fill_hours?)))
        .collect();
    let next_full_hours = fills.iter().map(|(_, f)| *f).min_by(f64::total_cmp)?;
    let mut cadences = vec![6.0, 12.0, 24.0];
    if let Some(h) = custom_hours.filter(|h| h.is_finite() && (1.0..=168.0).contains(h))
        && !cadences.iter().any(|c: &f64| (c - h).abs() < 1e-9)
    {
        cadences.push(h);
        cadences.sort_by(f64::total_cmp);
    }
    let intervals = cadences
        .into_iter()
        .map(|hours| {
            let mut interval = ClaimIntervalDto {
                hours,
                lost_lmd_per_day: 0.0,
                lost_gold_per_day: 0.0,
                lost_exp_per_day: 0.0,
            };
            for (r, fill) in &fills {
                let lost = (1.0 - (fill / hours).min(1.0)).max(0.0);
                match r.room_type.as_str() {
                    "TRADING" => {
                        interval.lost_lmd_per_day =
                            r.yield_lmd_per_day.mul_add(lost, interval.lost_lmd_per_day);
                    }
                    "MANUFACTURE" => {
                        interval.lost_gold_per_day = r
                            .yield_gold_per_day
                            .mul_add(lost, interval.lost_gold_per_day);
                        interval.lost_exp_per_day =
                            r.yield_exp_per_day.mul_add(lost, interval.lost_exp_per_day);
                    }
                    _ => {}
                }
            }
            interval
        })
        .collect();
    Some(ClaimDto {
        next_full_hours,
        intervals,
    })
}

fn power_of(building: &UserBuilding, building_data: &BuildingDataFile) -> PowerDto {
    let (mut generated, mut consumed) = (0, 0);
    for room in &building.rooms {
        let Some(p) = building_data
            .rooms
            .get(&room.room_type)
            .and_then(|def| phase(def, room.level))
        else {
            continue;
        };
        if p.electricity >= 0 {
            generated += p.electricity;
        } else {
            consumed -= p.electricity;
        }
    }
    PowerDto {
        generated,
        consumed,
        net: generated - consumed,
    }
}

fn dorms_of(
    building: &UserBuilding,
    base: &BaseContext,
    building_data: &BuildingDataFile,
) -> DormsDto {
    let by_id: HashMap<&str, &OperatorBaseProfile> = base
        .profiles
        .iter()
        .map(|p| (p.char_id.as_str(), p))
        .collect();
    let rooms: HashMap<&str, &UserRoom> = building
        .rooms
        .iter()
        .filter(|r| r.room_type == "DORMITORY")
        .map(|r| (r.slot_id.as_str(), r))
        .collect();
    let phases = &building_data.dorm_data.phases;
    let factor = building_data.comfort_manpower_recover_factor;
    let per_dorm: Vec<DormDto> = dorm_list(building, building_data)
        .into_iter()
        .map(|d| {
            let room = rooms.get(d.slot_id.as_str());
            let best = |value: fn(
                &OperatorBaseProfile,
                &HashMap<String, myrtle_engine_base::registry::BuffResolutionStrategy>,
                &BuildingDataFile,
            ) -> f64| {
                room.map_or(0.0, |r| {
                    r.current_operators
                        .iter()
                        .filter_map(|id| by_id.get(id.as_str()))
                        .map(|p| value(p, &base.registry, building_data))
                        .fold(0.0, f64::max)
                })
            };
            let comfort = room.map_or(0, |r| r.comfort);
            let idx = usize::try_from(d.level.max(1) - 1)
                .unwrap_or(0)
                .min(phases.len().saturating_sub(1));
            let limit = phases.get(idx).map_or(0, |p| p.decoration_limit);
            let upside = if factor > 0.0 {
                f64::from((limit - comfort).max(0)) / (factor * 100.0)
            } else {
                0.0
            };
            DormDto {
                occupant_aura_per_hour: best(dorm_aura_value),
                occupant_single_per_hour: best(dorm_single_value),
                slot_id: d.slot_id,
                level: d.level,
                capacity: d.capacity,
                recovery_per_hour: d.recovery_per_hour,
                comfort,
                comfort_limit: limit,
                comfort_upside_per_hour: upside,
            }
        })
        .collect();
    DormsDto {
        count: per_dorm.len(),
        total_levels: building.total_dorm_levels(),
        total_capacity: i32::try_from(per_dorm.iter().map(|d| d.capacity).sum::<usize>())
            .unwrap_or(i32::MAX),
        recovery_per_hour: morale_recovery(building),
        per_dorm,
    }
}

fn sustain_of(
    building: &UserBuilding,
    base: &BaseContext,
    data: &BaseData,
    live_morale: &HashMap<String, f64>,
) -> Vec<SustainEntryDto> {
    let by_id: HashMap<&str, &OperatorBaseProfile> = base
        .profiles
        .iter()
        .map(|p| (p.char_id.as_str(), p))
        .collect();
    let mut out = Vec::new();
    for room in building.rooms.iter().filter(|r| r.room_type != "DORMITORY") {
        for char_id in &room.current_operators {
            let Some(profile) = by_id.get(char_id.as_str()) else {
                continue;
            };
            let drain = game_morale_drain(profile, &base.morale_drains);
            let morale = live_morale.get(char_id).copied();
            out.push(SustainEntryDto {
                operator_id: char_id.clone(),
                name: operator_name(char_id, data),
                slot_id: room.slot_id.clone(),
                room_type: room.room_type.clone(),
                drain_per_hour: drain,
                lasts_hours: (drain > 0.0).then(|| morale.unwrap_or(MAX_MORALE) / drain),
                morale,
            });
        }
    }
    out
}

#[must_use]
pub fn trainer_value(desc: &str, class: &str) -> Option<f64> {
    if !desc.contains("Specialization training speed") || desc.contains("for each") {
        return None;
    }
    let pct = desc.match_indices('+').find_map(|(i, _)| {
        let rest = &desc[i + 1..];
        let end = rest
            .find(|c: char| !(c.is_ascii_digit() || c == '.'))
            .unwrap_or(rest.len());
        rest[end..]
            .starts_with('%')
            .then(|| rest[..end].parse::<f64>().ok())?
    })?;
    let mut mentioned = TRAINER_CLASSES
        .iter()
        .filter(|c| desc.contains(**c))
        .peekable();
    let fits = mentioned.peek().is_none() || mentioned.any(|c| *c == class);
    fits.then_some(pct)
}

fn trainer_hints(base: &BaseContext, data: &BaseData, class: &str) -> Vec<TrainerHintDto> {
    if !TRAINER_CLASSES.contains(&class) {
        return Vec::new();
    }
    let buffs = &data.building().buffs;
    let mut ranked: Vec<(&str, f64)> = base
        .profiles
        .iter()
        .filter_map(|p| {
            let best = p
                .available_buffs
                .iter()
                .filter_map(|b| buffs.get(b).filter(|buff| buff.room_type == "TRAINING"))
                .filter_map(|buff| trainer_value(&buff.description, class))
                .fold(0.0_f64, f64::max);
            (best > 0.0).then_some((p.char_id.as_str(), best))
        })
        .collect();
    ranked.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(b.0)));
    ranked
        .into_iter()
        .take(3)
        .map(|(id, value_pct)| TrainerHintDto {
            operator: assigned_operator(id, data),
            value_pct,
        })
        .collect()
}

#[must_use]
pub fn optimize_key(uid: &str, req: &OptimizeRequest) -> String {
    format!(
        "base:optimize:{uid}:{}",
        key_hash(&serde_json::to_vec(req).unwrap_or_default())
    )
}

#[must_use]
pub fn rotation_key(uid: &str, req: &RotationRequest) -> String {
    format!(
        "base:rotation:{uid}:{}",
        key_hash(&serde_json::to_vec(req).unwrap_or_default())
    )
}

pub async fn optimize(
    ctx: &AppContext,
    subject: &Subject,
    uid: Option<&str>,
    req: OptimizeRequest,
) -> Result<CachedJson, AppError> {
    let (user, uid) = target(ctx, subject, uid).await?;
    let snapshot = game_data(ctx)?;
    validate(&req.layout, snapshot.base.building())?;
    let key = snapshot.key(&optimize_key(&uid, &req));
    let owner = ctx.clone();
    ctx.cache
        .json_detached(&key, SEARCH_TTL, move || async move {
            let planned = planning(
                &owner,
                user,
                &snapshot.base,
                req.ignore_promotion,
                req.facts.clone(),
            )
            .await?;
            let response = owner
                .cpu
                .run("base_optimize", move || {
                    optimize_planned(&snapshot, planned, req, &uid)
                })
                .await?;
            serde_json::to_string(&response).map_err(AppError::internal)
        })
        .await
}

fn optimize_planned(
    snapshot: &Snapshot,
    planned: Planning,
    req: OptimizeRequest,
    uid: &str,
) -> OptimizeResponse {
    let data = &snapshot.base;
    let Planning {
        base,
        synced,
        facts,
    } = planned;
    let building_data = data.building();
    let scope: HashSet<String> = req.scope.iter().cloned().collect();
    let mut building = draft_building(
        req.layout.clone(),
        synced
            .as_ref()
            .and_then(|json| UserBuilding::from_json(json).drone_capacity),
    );
    for room in &mut building.rooms {
        room.frozen = !scope.is_empty()
            && !scope.contains(&room.slot_id)
            && is_production_room(&room.room_type);
    }
    let candidates = base.profiles_excluding(&req.excluded);
    let economy = search_economy(&candidates, &building, building_data, &base.registry);
    let pins = with_pins(
        build_pins(&req.layout, &req.scope, &req.locked),
        economy.pins,
    );
    let baseline = compute_current_assignment(
        &base.profiles,
        &building,
        building_data,
        &base.registry,
        &base.morale_drains,
        None,
    );
    let key = snapshot.key(&plan_cache_key(
        uid,
        &req.layout,
        &req.scope,
        &req.locked,
        &req.excluded,
        req.ignore_promotion,
        &facts,
    ));
    let optimal = plan(key, || {
        optimal_with_bundles(
            &candidates,
            &building,
            building_data,
            &base.registry,
            &economy.registry,
            &base.morale_drains,
            &pins,
        )
    })
    .optimal
    .clone();
    let mut proposal = if scope.is_empty() {
        optimal
    } else {
        settle_scoped_proposal(
            &base.profiles,
            &building,
            building_data,
            &base.registry,
            &base.morale_drains,
            &scope,
            &baseline,
            &optimal,
        )
    };
    align_rooms_to_current(&building, &mut proposal);
    OptimizeResponse {
        room_diffs: diff_rooms(&req.layout, &baseline, &proposal),
        power: power_of(&building, building_data),
        baseline: base_assignment_to_dto(&baseline, data, &base.profiles, &base.registry),
        proposal: base_assignment_to_dto(&proposal, data, &base.profiles, &base.registry),
    }
}

pub async fn rotation(
    ctx: &AppContext,
    subject: &Subject,
    uid: Option<&str>,
    req: RotationRequest,
) -> Result<CachedJson, AppError> {
    let (user, uid) = target(ctx, subject, uid).await?;
    let snapshot = game_data(ctx)?;
    validate(&req.layout, snapshot.base.building())?;
    let key = snapshot.key(&rotation_key(&uid, &req));
    let owner = ctx.clone();
    ctx.cache
        .json_detached(&key, SEARCH_TTL, move || async move {
            let planned = planning(
                &owner,
                user,
                &snapshot.base,
                req.ignore_promotion,
                req.facts.clone(),
            )
            .await?;
            let response = owner
                .cpu
                .run("base_rotation", move || {
                    rotation_planned(&snapshot, planned, req, &uid)
                })
                .await?;
            serde_json::to_string(&response).map_err(AppError::internal)
        })
        .await
}

fn rotation_planned(
    snapshot: &Snapshot,
    planned: Planning,
    req: RotationRequest,
    uid: &str,
) -> RotationResponse {
    let data = &snapshot.base;
    let Planning {
        base,
        synced,
        facts,
    } = planned;
    let building_data = data.building();
    let pins = build_pins(&req.layout, &[], &req.locked);
    let key = snapshot.key(&plan_cache_key(
        uid,
        &req.layout,
        &[],
        &req.locked,
        &req.excluded,
        req.ignore_promotion,
        &facts,
    ));
    let mut building = draft_building(req.layout, None);
    if let Some(real) = synced.as_ref().map(UserBuilding::from_json) {
        building.drone_capacity = real.drone_capacity;
        for room in &mut building.rooms {
            if let Some(saved) = real
                .rooms
                .iter()
                .find(|r| r.slot_id == room.slot_id && r.room_type == room.room_type)
            {
                room.preset_shifts.clone_from(&saved.preset_shifts);
            }
        }
    }
    let candidates = base.profiles_excluding(&req.excluded);
    let economy = search_economy(&candidates, &building, building_data, &base.registry);
    let pins = with_pins(pins, economy.pins);
    let accepted = plan(key, || {
        optimal_with_bundles(
            &candidates,
            &building,
            building_data,
            &base.registry,
            &economy.registry,
            &base.morale_drains,
            &pins,
        )
    });
    let plan = recommend_shift_rotation(
        &candidates,
        &building,
        building_data,
        &accepted.registry,
        &base.morale_drains,
        &accepted.pins,
    );
    let recommended = rotation_value(
        &plan,
        &base.profiles,
        &building,
        building_data,
        &base.registry,
        &base.morale_drains,
    )
    .unwrap_or_default();
    let presets = presets_value(
        &base.profiles,
        &building,
        building_data,
        &base.registry,
        &base.morale_drains,
    );
    let cycle = plan_cycle(
        &accepted.optimal,
        &candidates,
        &building,
        building_data,
        &base.registry,
        &accepted.registry,
        &base.morale_drains,
        &accepted.pins,
    )
    .map(|c| CycleDto {
        legs: c
            .legs
            .iter()
            .zip(&c.legs_hours)
            .map(|(rooms, &hours)| CycleLegDto {
                hours,
                rooms: rooms
                    .iter()
                    .map(|(slot_id, ops)| CycleRoomDto {
                        room_type: building
                            .rooms
                            .iter()
                            .find(|r| &r.slot_id == slot_id)
                            .map(|r| r.room_type.clone())
                            .unwrap_or_default(),
                        slot_id: slot_id.clone(),
                        operators: ops.iter().map(|id| assigned_operator(id, data)).collect(),
                    })
                    .collect(),
            })
            .collect(),
        value: c.value.into(),
        beats_rotation: c.value.total_value > recommended.total_value,
    });
    let rotation = shift_rotation_to_dto(
        &plan,
        data,
        &candidates,
        &building,
        &base.registry,
        &base.morale_drains,
    );
    let production: Vec<&ShiftRoomDto> = rotation
        .shifts
        .iter()
        .flat_map(|shift| &shift.rooms)
        .filter(|r| r.active && is_production_room(&r.room_type))
        .collect();
    let comparison = RotationComparisonDto {
        recommended: recommended.into(),
        presets: presets.as_ref().map(|p| p.value.into()),
        matching_cells: production
            .iter()
            .filter(|r| r.matches || r.equivalent)
            .count(),
        production_cells: production.len(),
        preset_legs_hours: presets.map(|p| p.legs_hours).unwrap_or_default(),
    };
    RotationResponse {
        rotation,
        shift_count: SHIFT_COUNT,
        comparison,
        cycle,
    }
}

#[must_use]
pub fn plan_cache_key(
    uid: &str,
    layout: &[DraftRoom],
    scope: &[String],
    locked: &[String],
    excluded: &[String],
    ignore_promotion: bool,
    facts: &AccountFactsReq,
) -> String {
    let sorted = |v: &[String]| {
        let mut v = v.to_vec();
        v.sort();
        v.join(",")
    };
    let mut rooms: Vec<String> = layout
        .iter()
        .map(|r| {
            format!(
                "{}:{}:{}:{}:{}",
                r.slot_id,
                r.room_type,
                r.level,
                r.formula_type.as_deref().unwrap_or(""),
                sorted(&r.operators)
            )
        })
        .collect();
    rooms.sort();
    format!(
        "{uid}|{ignore_promotion}|{}|{}|{}|{}|{}|{}",
        facts.open_recruit_slots.unwrap_or(0),
        facts.training_class.as_deref().unwrap_or(""),
        sorted(scope),
        sorted(locked),
        sorted(excluded),
        rooms.join(";")
    )
}

#[must_use]
pub fn cached_plan(key: &str) -> Option<Arc<AcceptedEconomy>> {
    PLANS
        .get(key)
        .filter(|entry| entry.0.elapsed() < PLAN_TTL)
        .map(|entry| Arc::clone(&entry.1))
}

fn plan(key: String, search: impl FnOnce() -> AcceptedEconomy) -> Arc<AcceptedEconomy> {
    if let Some(hit) = cached_plan(&key) {
        return hit;
    }
    let plan = Arc::new(search());
    if PLANS.len() >= PLAN_CAP {
        PLANS.retain(|_, (at, _)| at.elapsed() < PLAN_TTL);
        let oldest = PLANS
            .iter()
            .min_by_key(|entry| entry.value().0)
            .map(|entry| entry.key().clone());
        if PLANS.len() >= PLAN_CAP
            && let Some(oldest) = oldest
        {
            PLANS.remove(&oldest);
        }
    }
    PLANS.insert(key, (Instant::now(), Arc::clone(&plan)));
    plan
}

fn with_pins(
    mut pins: Vec<(String, String)>,
    extra: Vec<(String, String)>,
) -> Vec<(String, String)> {
    for pin in extra {
        if !pins.iter().any(|(id, _)| *id == pin.0) {
            pins.push(pin);
        }
    }
    pins
}

#[must_use]
pub fn build_pins(
    layout: &[DraftRoom],
    scope: &[String],
    locked: &[String],
) -> Vec<(String, String)> {
    let in_scope: HashSet<&str> = scope.iter().map(String::as_str).collect();
    let locked: HashSet<&str> = locked.iter().map(String::as_str).collect();
    let mut pins = Vec::new();
    for room in layout {
        let frozen = !(in_scope.is_empty() || in_scope.contains(room.slot_id.as_str()));
        let unmodeled = UNMODELED_ROOM_TYPES.contains(&room.room_type.as_str());
        for op in &room.operators {
            if frozen || unmodeled || locked.contains(op.as_str()) {
                pins.push((op.clone(), room.room_type.clone()));
            }
        }
    }
    pins
}

fn diff_rooms(
    draft: &[DraftRoom],
    baseline: &BaseAssignment,
    proposal: &BaseAssignment,
) -> Vec<RoomDiffDto> {
    let yield_of = |r: &myrtle_engine_base::types::RoomAssignment| {
        let y = room_yield(
            &r.room_type,
            r.formula_type.as_deref(),
            r.level,
            r.total_efficiency,
            r.order_value,
            r.operators.len(),
            r.order_limit,
        );
        y.lmd_per_day + y.exp_per_day
    };
    let mut diffs = Vec::new();
    for after in &proposal.rooms {
        let Some(drafted) = draft.iter().find(|r| r.slot_id == after.slot_id) else {
            continue;
        };
        let a: HashSet<&str> = drafted.operators.iter().map(String::as_str).collect();
        let b: HashSet<&str> = after.operators.iter().map(String::as_str).collect();
        if a == b {
            continue;
        }
        let before = baseline.rooms.iter().find(|r| r.slot_id == after.slot_id);
        diffs.push(RoomDiffDto {
            slot_id: after.slot_id.clone(),
            room_type: after.room_type.clone(),
            before: drafted.operators.clone(),
            after: after.operators.clone(),
            efficiency_before: before.map_or(0.0, |r| r.total_efficiency),
            efficiency_after: after.total_efficiency,
            yield_before: before.map_or(0.0, yield_of),
            yield_after: yield_of(after),
        });
    }
    diffs
}

pub fn catalog(ctx: &AppContext) -> Result<CatalogResponse, AppError> {
    let snapshot = game_data(ctx)?;
    let building = snapshot.base.building();
    let mut rooms: Vec<CatalogRoomDto> = building
        .rooms
        .values()
        .map(|def| CatalogRoomDto {
            room_type: def.id.clone(),
            name: def.name.clone(),
            category: def.category.clone(),
            max_count: def.max_count,
            size_col: def.size.col,
            size_row: def.size.row,
            phases: def
                .phases
                .iter()
                .zip(1..)
                .map(|(p, level)| CatalogPhaseDto {
                    level,
                    max_stationed: p.max_stationed_num,
                    electricity: p.electricity,
                    manpower_cost: p.manpower_cost,
                })
                .collect(),
        })
        .collect();
    rooms.sort_by(|a, b| a.room_type.cmp(&b.room_type));
    let formulas = [
        ("F_DIAMOND", "Originium Shard"),
        ("F_EXP", "Battle Records"),
        ("F_GOLD", "Pure Gold"),
    ]
    .into_iter()
    .map(|(formula_type, label)| CatalogFormulaDto {
        formula_type: formula_type.to_owned(),
        label: label.to_owned(),
    })
    .collect();
    let layout = building.layouts.values().next();
    let mut slots: Vec<CatalogSlotDto> = layout
        .map(|l| {
            l.slots
                .values()
                .map(|s| CatalogSlotDto {
                    slot_id: s.id.clone(),
                    category: s.category.clone(),
                    storey_id: s.storey_id.clone(),
                    offset_col: s.offset.col,
                    offset_row: s.offset.row,
                    size_col: s.size.col,
                    size_row: s.size.row,
                })
                .collect()
        })
        .unwrap_or_default();
    slots.sort_by(|a, b| {
        b.offset_row
            .cmp(&a.offset_row)
            .then(a.offset_col.cmp(&b.offset_col))
    });
    let mut storeys: Vec<CatalogStoreyDto> = layout
        .map(|l| {
            l.storeys
                .values()
                .map(|s| CatalogStoreyDto {
                    storey_id: s.id.clone(),
                    unlock_control_level: s.unlock_control_level,
                })
                .collect()
        })
        .unwrap_or_default();
    storeys.sort_by(|a, b| a.storey_id.cmp(&b.storey_id));
    Ok(CatalogResponse {
        shift_count: SHIFT_COUNT,
        rooms,
        formulas,
        slots,
        storeys,
    })
}
