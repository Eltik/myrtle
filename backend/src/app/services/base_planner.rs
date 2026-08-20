//! The interactive base planner: score an arbitrary layout, or optimize one
//! under constraints.
//!
//! `/user/improvements` answers one question - "what is the best plan for this
//! player's real base?" - and answers it whole. The planner answers a different
//! one: "what happens if I do THIS?", asked repeatedly against a layout the
//! player is editing in the browser. Both run through the same clause engine
//! (`ledger::score_room` via `assignment`), so a number the planner shows and a
//! number the improvements plan shows can never disagree.
//!
//! These handlers are stateless. The edited layout lives in the client and
//! arrives with every request; nothing about a draft is persisted server-side.

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::app::error::ApiError;
use crate::app::services::improvements::{
    BaseAssignmentDto, ShiftRotationDto, base_assignment_to_dto, shift_rotation_to_dto,
};
use crate::app::state::AppState;
use crate::core::gamedata::types::GameData;
use crate::core::grade::base::assignment::{
    compute_current_assignment, compute_optimal_assignment_with_pins, morale_recovery,
};
use crate::core::grade::base::context::BaseContext;
use crate::core::grade::base::dorms::morale_manager_pin;
use crate::core::grade::base::shift_rotation::{SHIFT_COUNT, recommend_shift_rotation};
use crate::core::grade::base::sustain_sim::game_morale_drain;
use crate::core::grade::base::types::{
    BaseAssignment, OperatorBaseProfile, RoomAssignment, UserBuilding, UserRoom,
};
use crate::core::grade::base::yield_model::room_yield;
use crate::database::models::user::UserProfile;
use crate::database::queries::building::get_building;
use crate::database::queries::roster::get_roster;
use crate::database::queries::users::find_by_uid;

/// One room of a client-drafted layout. Mirrors `UserRoom` minus the bits only
/// the live game data can supply (preset queues).
#[derive(Debug, Clone, Deserialize)]
pub struct DraftRoom {
    pub slot_id: String,
    /// Game room constant: "MANUFACTURE", "TRADING", "POWER", "DORMITORY",
    /// "CONTROL", "MEETING", "HIRE", "TRAINING", "WORKSHOP".
    pub room_type: String,
    pub level: i32,
    /// Who the player has placed here in the draft.
    #[serde(default)]
    pub operators: Vec<String>,
    /// Factory recipe ("`F_GOLD`" / "`F_EXP`" / "`F_DIAMOND`"). None for other rooms.
    #[serde(default)]
    pub formula_type: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EvaluateRequest {
    pub layout: Vec<DraftRoom>,
    /// Plan with every operator's highest base skills, whether or not the
    /// player has promoted them that far. The result is a target to build
    /// toward, not a reading of the base they have today.
    #[serde(default)]
    pub ignore_promotion: bool,
}

/// The player's real base, every built slot of it.
///
/// This exists because a `BaseAssignment` is deliberately NOT a layout: it
/// carries only the rooms the plan has something to say about (production, plus
/// a staffed Control Center). Seeding a planner from it silently drops
/// dormitories, power plants and every support facility - which then breaks the
/// power balance AND the scoring, since dorm levels and facility counts feed
/// the clause engine.
#[derive(Debug, Clone, Serialize)]
pub struct LayoutResponse {
    pub rooms: Vec<DraftRoomDto>,
    /// The player's own saved shift rotation, straight out of `presetQueue`.
    /// Separate from `rooms` because that is the round-trip shape the client
    /// posts back; presets are read-only context and never travel with a draft.
    pub presets: Vec<SlotPresetsDto>,
}

/// One slot's saved rotation: the crew the player has queued for each shift.
#[derive(Debug, Clone, Serialize)]
pub struct SlotPresetsDto {
    pub slot_id: String,
    pub shifts: Vec<Vec<String>>,
}

/// The serialized twin of [`DraftRoom`] - what the client sends back on every
/// evaluate/optimize call.
#[derive(Debug, Clone, Serialize)]
pub struct DraftRoomDto {
    pub slot_id: String,
    pub room_type: String,
    pub level: i32,
    pub operators: Vec<String>,
    pub formula_type: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OptimizeRequest {
    pub layout: Vec<DraftRoom>,
    /// Slot ids the optimizer may restaff. Empty = every room in the layout.
    /// Rooms outside the scope keep their drafted crews AND hold those
    /// operators, so a scoped run can't quietly steal from an untouched room.
    #[serde(default)]
    pub scope: Vec<String>,
    /// Operators the plan must keep exactly where the draft puts them.
    #[serde(default)]
    pub locked: Vec<String>,
    /// Operators the plan may not seat anywhere (injured, saved for elsewhere).
    #[serde(default)]
    pub excluded: Vec<String>,
    /// Plan with every operator's highest base skills, whether or not the
    /// player has promoted them that far. The result is a target to build
    /// toward, not a reading of the base they have today.
    #[serde(default)]
    pub ignore_promotion: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct EvaluateResponse {
    pub assignment: BaseAssignmentDto,
    /// Facility power: generated, consumed, and the net the base runs at.
    pub power: PowerDto,
    /// How long each stationed operator lasts before morale runs out.
    pub sustain: Vec<SustainEntryDto>,
    /// What the dormitory wing is worth to this layout.
    pub dorms: DormsDto,
}

/// The dormitories' contribution. They produce nothing, so they never appear in
/// a `BaseAssignment` - but they set how fast workers recover and how many can
/// rest at once, which is what decides whether a staffing survives its own
/// rhythm. Surfaced separately so "no yield" doesn't read as "not accounted for".
#[derive(Debug, Clone, Serialize)]
pub struct DormsDto {
    pub count: usize,
    /// Sum of dorm levels - the `&dorm&lv` scaling the clause engine reads.
    pub total_levels: i32,
    /// Operators the base can rest at once.
    pub total_capacity: i32,
    /// Morale restored per hour to a resting operator, scaled by dorm levels.
    pub recovery_per_hour: f64,
    /// Each dormitory, BEST FIRST - the order resters fill them. A 2/5/2's
    /// under-leveled dorms show up here as the slower rates they are.
    pub per_dorm: Vec<DormDto>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DormDto {
    pub slot_id: String,
    pub level: i32,
    pub capacity: usize,
    /// Game-true rate for THIS dorm (`DormData.Phases[level]`), before skills.
    pub recovery_per_hour: f64,
    /// The strongest whole-dorm recovery aura among the DRAFTED occupants
    /// ("+X/hr to all Operators in that Dormitory", non-stacking).
    pub occupant_aura_per_hour: f64,
    /// The strongest single-target heal among the drafted occupants
    /// ("+X/hr to another Operator whose Morale is not full").
    pub occupant_single_per_hour: f64,
}

/// One operator's endurance in the room the draft puts them in. Drain is the
/// game's own per-hour figure (including the operator's own morale-cost riders),
/// not a per-room approximation.
#[derive(Debug, Clone, Serialize)]
pub struct SustainEntryDto {
    pub operator_id: String,
    pub name: String,
    pub slot_id: String,
    pub room_type: String,
    /// Morale lost per working hour.
    pub drain_per_hour: f64,
    /// Hours from full morale to empty. `None` = never depletes.
    pub lasts_hours: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PowerDto {
    pub generated: i32,
    pub consumed: i32,
    pub net: i32,
}

/// A rotation is whole-base by construction (a shift covers every room at
/// once), so unlike optimize there is no `scope` - only who must stay put.
#[derive(Debug, Clone, Deserialize)]
pub struct RotationRequest {
    pub layout: Vec<DraftRoom>,
    #[serde(default)]
    pub locked: Vec<String>,
    #[serde(default)]
    pub excluded: Vec<String>,
    /// Plan with every operator's highest base skills, whether or not the
    /// player has promoted them that far. The result is a target to build
    /// toward, not a reading of the base they have today.
    #[serde(default)]
    pub ignore_promotion: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct RotationResponse {
    /// `SHIFT_COUNT` shifts, each room labelled with the squad staffing it, plus
    /// the week-long morale simulation of the resulting rhythm.
    pub rotation: ShiftRotationDto,
    /// Shifts per day the rotation assumes, so the UI never hardcodes it.
    pub shift_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct OptimizeResponse {
    /// The proposed layout, scored.
    pub proposal: BaseAssignmentDto,
    /// The draft as submitted, scored the same way - the honest before-picture
    /// the UI diffs against. Computed here rather than trusted from the client
    /// so both sides of the comparison come from one engine run.
    pub baseline: BaseAssignmentDto,
    /// Only the rooms whose crew actually changed.
    pub room_diffs: Vec<RoomDiffDto>,
    pub power: PowerDto,
}

#[derive(Debug, Clone, Serialize)]
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

/// The facility catalogue, straight out of `building_data`. The client renders
/// capacities, power draw and level caps from this instead of carrying its own
/// copy of numbers the game already publishes.
#[derive(Debug, Clone, Serialize)]
pub struct CatalogResponse {
    /// Shifts in a base day. A game constant, not a property of any one
    /// player - a base whose rooms only queue two presets still runs three
    /// shifts, alternating across them.
    pub shift_count: usize,
    pub rooms: Vec<CatalogRoomDto>,
    pub formulas: Vec<CatalogFormulaDto>,
    /// The floorplan every base shares - what the client draws the board from.
    pub slots: Vec<CatalogSlotDto>,
    pub storeys: Vec<CatalogStoreyDto>,
}

/// One slot of the base floorplan, joined to a player's rooms on `slot_id`.
///
/// Coordinates stay in the game's own half-tile units rather than pixels or
/// grid tracks: how a half-tile becomes a column is the board's decision, and
/// the elevator shafts (1 unit wide where every room is 2) only make sense at
/// this scale.
#[derive(Debug, Clone, Serialize)]
pub struct CatalogSlotDto {
    pub slot_id: String,
    /// Room category this slot accepts, matching `CatalogRoomDto::category`.
    pub category: String,
    pub storey_id: String,
    pub offset_col: i32,
    /// Absolute - the storey's Y-offset is already applied. Counts up from the
    /// bottom of the base, so B4 is 0.
    pub offset_row: i32,
    pub size_col: i32,
    pub size_row: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct CatalogStoreyDto {
    pub storey_id: String,
    /// Control-centre level that unlocks this floor.
    pub unlock_control_level: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct CatalogRoomDto {
    pub room_type: String,
    pub name: String,
    pub category: String,
    /// -1 in game data means unlimited.
    pub max_count: i32,
    pub size_col: i32,
    pub size_row: i32,
    /// Index = level - 1.
    pub phases: Vec<CatalogPhaseDto>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CatalogPhaseDto {
    pub level: i32,
    pub max_stationed: i32,
    /// Positive generates, negative consumes.
    pub electricity: i32,
    pub manpower_cost: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct CatalogFormulaDto {
    pub formula_type: String,
    pub label: String,
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
        }
    }
}

/// Reject a draft before it reaches the engine. A layout is client-supplied, so
/// it gets the same scepticism as any other request body: rooms the game has no
/// definition for, impossible levels, and crews larger than the room's seats are
/// all refused by name rather than silently producing a nonsense score.
fn validate(layout: &[DraftRoom], game_data: &GameData) -> Result<(), ApiError> {
    if layout.is_empty() {
        return Err(ApiError::BadRequest("layout is empty".into()));
    }
    if layout.len() > MAX_ROOMS {
        return Err(ApiError::BadRequest(format!(
            "layout has {} rooms, more than the {MAX_ROOMS} a base can hold",
            layout.len()
        )));
    }

    let mut seen: HashSet<&str> = HashSet::new();
    for room in layout {
        if !seen.insert(room.slot_id.as_str()) {
            return Err(ApiError::BadRequest(format!(
                "duplicate slot '{}'",
                room.slot_id
            )));
        }
        let Some(def) = game_data.building.rooms.get(&room.room_type) else {
            return Err(ApiError::BadRequest(format!(
                "unknown room type '{}'",
                room.room_type
            )));
        };
        let max_level = i32::try_from(def.phases.len()).unwrap_or(i32::MAX);
        if room.level < 1 || room.level > max_level {
            return Err(ApiError::BadRequest(format!(
                "room '{}' has level {}, outside 1..={max_level}",
                room.slot_id, room.level
            )));
        }
        let seats = def
            .phases
            .get((room.level - 1) as usize)
            .map_or(0, |p| p.max_stationed_num);
        if i32::try_from(room.operators.len()).unwrap_or(i32::MAX) > seats {
            return Err(ApiError::BadRequest(format!(
                "room '{}' has {} operators but only {seats} seats at level {}",
                room.slot_id,
                room.operators.len(),
                room.level
            )));
        }
    }

    // One operator, one seat - the engine assumes it, so catch a violation here
    // instead of letting a duplicated operator inflate the score.
    let mut placed: HashSet<&str> = HashSet::new();
    for room in layout {
        for op in &room.operators {
            if !placed.insert(op.as_str()) {
                return Err(ApiError::BadRequest(format!(
                    "operator '{op}' is stationed in more than one room"
                )));
            }
        }
    }
    Ok(())
}

/// A real base tops out well under this; it exists to bound the work a single
/// request can ask the optimizer to do.
const MAX_ROOMS: usize = 40;

fn power_of(building: &UserBuilding, game_data: &GameData) -> PowerDto {
    let mut generated = 0;
    let mut consumed = 0;
    for room in &building.rooms {
        let Some(def) = game_data.building.rooms.get(&room.room_type) else {
            continue;
        };
        let Some(phase) = def.phases.get((room.level.max(1) - 1) as usize) else {
            continue;
        };
        if phase.electricity >= 0 {
            generated += phase.electricity;
        } else {
            consumed += -phase.electricity;
        }
    }
    PowerDto {
        generated,
        consumed,
        net: generated - consumed,
    }
}

/// Load the roster behind `uid`, honouring profile privacy the same way
/// `/user/improvements` does.
async fn context_for(
    state: &AppState,
    uid: &str,
    viewer_id: Option<uuid::Uuid>,
    game_data: &GameData,
    ignore_promotion: bool,
) -> Result<BaseContext, ApiError> {
    let user = profile_for(state, uid, viewer_id).await?;
    let roster = get_roster(&state.db, user.id).await?;
    Ok(BaseContext::build(&roster, game_data, ignore_promotion))
}

/// Resolve `uid` to a profile, honouring privacy the same way
/// `/user/improvements` does.
async fn profile_for(
    state: &AppState,
    uid: &str,
    viewer_id: Option<uuid::Uuid>,
) -> Result<UserProfile, ApiError> {
    let user = find_by_uid(&state.db, uid)
        .await?
        .ok_or(ApiError::NotFound)?;
    let is_own = viewer_id.is_some_and(|id| id == user.id);
    if !is_own && user.public_profile != Some(true) {
        return Err(ApiError::Forbidden);
    }
    Ok(user)
}

/// The player's stationed base, as an editable draft.
pub async fn layout(
    state: &AppState,
    uid: &str,
    viewer_id: Option<uuid::Uuid>,
) -> Result<LayoutResponse, ApiError> {
    let user = profile_for(state, uid, viewer_id).await?;
    let Some(building_json) = get_building(&state.db, user.id).await? else {
        return Ok(LayoutResponse {
            rooms: Vec::new(),
            presets: Vec::new(),
        });
    };

    let building = UserBuilding::from_json(&building_json);

    // An all-empty queue is the game's placeholder for "no rotation saved", so
    // it is dropped here rather than surfacing as a row of blank shifts.
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
        })
        .collect();
    // `roomSlots` is a JSON object, so its iteration order is not the base's.
    // Sort by slot so the board is stable between loads.
    rooms.sort_by(|a, b| a.slot_id.cmp(&b.slot_id));
    Ok(LayoutResponse { rooms, presets })
}

pub async fn evaluate(
    state: &AppState,
    uid: &str,
    viewer_id: Option<uuid::Uuid>,
    req: EvaluateRequest,
) -> Result<EvaluateResponse, ApiError> {
    let game_data = state.default_game_data();
    validate(&req.layout, &game_data)?;

    let ctx = context_for(state, uid, viewer_id, &game_data, req.ignore_promotion).await?;
    let building = UserBuilding {
        rooms: req
            .layout
            .into_iter()
            .map(DraftRoom::into_user_room)
            .collect(),
    };

    // `compute_current_assignment` scores rooms exactly as stationed - which is
    // precisely what "score this draft" means.
    let assignment = compute_current_assignment(
        &ctx.profiles,
        &building,
        &game_data.building,
        &ctx.registry,
        &ctx.morale_drains,
        None,
    );

    Ok(EvaluateResponse {
        power: power_of(&building, &game_data),
        sustain: sustain_of(&building, &ctx, &game_data),
        dorms: dorms_of(&building, &ctx, &game_data),
        assignment: base_assignment_to_dto(&assignment, &game_data, &ctx.profiles, &ctx.registry),
    })
}

fn dorms_of(building: &UserBuilding, ctx: &BaseContext, game_data: &GameData) -> DormsDto {
    use crate::core::grade::base::dorms::{dorm_aura_value, dorm_list, dorm_single_value};
    let by_id: HashMap<&str, &OperatorBaseProfile> = ctx
        .profiles
        .iter()
        .map(|p| (p.char_id.as_str(), p))
        .collect();
    let occupants: HashMap<&str, &Vec<String>> = building
        .rooms
        .iter()
        .filter(|r| r.room_type == "DORMITORY")
        .map(|r| (r.slot_id.as_str(), &r.current_operators))
        .collect();

    let per_dorm: Vec<DormDto> = dorm_list(building, &game_data.building)
        .into_iter()
        .map(|d| {
            let skill_max = |single: bool| -> f64 {
                occupants.get(d.slot_id.as_str()).map_or(0.0, |ids| {
                    ids.iter()
                        .filter_map(|id| by_id.get(id.as_str()))
                        .map(|p| {
                            if single {
                                dorm_single_value(p, &ctx.registry, &game_data.building)
                            } else {
                                dorm_aura_value(p, &ctx.registry, &game_data.building)
                            }
                        })
                        .fold(0.0, f64::max)
                })
            };
            DormDto {
                occupant_aura_per_hour: skill_max(false),
                occupant_single_per_hour: skill_max(true),
                slot_id: d.slot_id,
                level: d.level,
                capacity: d.capacity,
                recovery_per_hour: d.recovery_per_hour,
            }
        })
        .collect();

    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    let total_capacity = per_dorm.iter().map(|d| d.capacity).sum::<usize>() as i32;
    DormsDto {
        count: per_dorm.len(),
        total_levels: building.total_dorm_levels(),
        total_capacity,
        recovery_per_hour: morale_recovery(building),
        per_dorm,
    }
}

/// Full morale, in the game's units. An operator lasts `MAX_MORALE / drain`
/// working hours from a full bar.
const MAX_MORALE: f64 = 24.0;

fn sustain_of(
    building: &UserBuilding,
    ctx: &BaseContext,
    game_data: &GameData,
) -> Vec<SustainEntryDto> {
    let by_id: HashMap<&str, &OperatorBaseProfile> = ctx
        .profiles
        .iter()
        .map(|p| (p.char_id.as_str(), p))
        .collect();

    let mut out = Vec::new();
    for room in &building.rooms {
        // Dormitories are where morale comes back, so "how long until they run
        // out" is not a question about a dorm seat.
        if room.room_type == "DORMITORY" {
            continue;
        }
        for char_id in &room.current_operators {
            let Some(profile) = by_id.get(char_id.as_str()) else {
                continue;
            };
            let drain = game_morale_drain(profile, &ctx.morale_drains);
            out.push(SustainEntryDto {
                operator_id: char_id.clone(),
                name: game_data
                    .operators
                    .get(char_id)
                    .map_or_else(|| char_id.clone(), |o| o.name.clone()),
                slot_id: room.slot_id.clone(),
                room_type: room.room_type.clone(),
                drain_per_hour: drain,
                lasts_hours: (drain > 0.0).then(|| MAX_MORALE / drain),
            });
        }
    }
    out
}

pub async fn optimize(
    state: &AppState,
    uid: &str,
    viewer_id: Option<uuid::Uuid>,
    req: OptimizeRequest,
) -> Result<OptimizeResponse, ApiError> {
    let game_data = state.default_game_data();
    validate(&req.layout, &game_data)?;

    let ctx = context_for(state, uid, viewer_id, &game_data, req.ignore_promotion).await?;

    let building = UserBuilding {
        rooms: req
            .layout
            .iter()
            .cloned()
            .map(DraftRoom::into_user_room)
            .collect(),
    };
    let baseline = compute_current_assignment(
        &ctx.profiles,
        &building,
        &game_data.building,
        &ctx.registry,
        &ctx.morale_drains,
        None,
    );

    let pins = build_pins(&req.layout, &req.scope, &req.locked);
    let candidates = ctx.profiles_excluding(&req.excluded);
    let proposal = compute_optimal_assignment_with_pins(
        &candidates,
        &building,
        &game_data.building,
        &ctx.registry,
        &ctx.morale_drains,
        &pins,
    );

    let room_diffs = diff_rooms(&req.layout, &baseline, &proposal);

    Ok(OptimizeResponse {
        power: power_of(&building, &game_data),
        baseline: base_assignment_to_dto(&baseline, &game_data, &ctx.profiles, &ctx.registry),
        proposal: base_assignment_to_dto(&proposal, &game_data, &ctx.profiles, &ctx.registry),
        room_diffs,
    })
}

/// Rooms the optimizer has no model for at all: it never staffs a Training Room
/// or a Workshop, and no part of the search values a seat in one.
///
/// Note this is NOT simply "everything the optimizer doesn't staff" -
/// dormitories are excluded on purpose. The optimizer never *puts* anyone in a
/// dorm, but a resting operator is legitimately available labor, and the
/// improvements plan treats them that way. Pinning dorm crews here would make
/// the planner propose worse layouts than the Score tab, which is exactly the
/// disagreement this design exists to prevent.
const UNMODELED_ROOM_TYPES: [&str; 2] = ["TRAINING", "WORKSHOP"];

/// A two-squad, three-shift rotation for the drafted layout.
///
/// The draft carries no in-game preset queue, so the DTO's preset-comparison
/// fields (`current`, `swap_in`, `swap_out`, `matches`) are empty by
/// construction - there is nothing to compare a draft against. The planner
/// renders the recommendation, its squad labels and the sustainability verdict,
/// and leaves the preset overlay to the Score tab where a real preset exists.
pub async fn rotation(
    state: &AppState,
    uid: &str,
    viewer_id: Option<uuid::Uuid>,
    req: RotationRequest,
) -> Result<RotationResponse, ApiError> {
    let game_data = state.default_game_data();
    validate(&req.layout, &game_data)?;

    let ctx = context_for(state, uid, viewer_id, &game_data, req.ignore_promotion).await?;
    let building = UserBuilding {
        rooms: req
            .layout
            .iter()
            .cloned()
            .map(DraftRoom::into_user_room)
            .collect(),
    };

    let mut pins = build_pins(&req.layout, &[], &req.locked);
    let candidates = ctx.profiles_excluding(&req.excluded);
    // Same reservation the improvements plan makes: a morale-swap manager
    // (Fiammetta) holds a dormitory seat when the roster runs a
    // morale-conditional generator - the planner and the Score tab must
    // never disagree about her.
    if let Some(pin) = morale_manager_pin(&candidates, &building, &game_data.building)
        && !pins.iter().any(|(id, _)| id == &pin.0)
    {
        pins.push(pin);
    }
    let plan = recommend_shift_rotation(
        &candidates,
        &building,
        &game_data.building,
        &ctx.registry,
        &ctx.morale_drains,
        &pins,
    );

    Ok(RotationResponse {
        rotation: shift_rotation_to_dto(
            &plan,
            &game_data,
            &candidates,
            &building,
            &ctx.registry,
            &ctx.morale_drains,
        ),
        shift_count: SHIFT_COUNT,
    })
}

/// The `(char_id, room_type)` pins a planner run must respect.
///
/// Three reasons an operator gets held in place, all expressed the same way:
///
/// - the room is outside `scope`, so the player asked us not to touch it;
/// - the room type is one the optimizer has no model for (see
///   [`UNMODELED_ROOM_TYPES`]);
/// - the player pinned that operator explicitly.
///
/// Pinning rather than excluding is deliberate: a pinned operator still sits in
/// a room, so their cross-room buffs stay in the scoring. Excluding them would
/// quietly delete those buffs and make a scoped run disagree with a whole-base
/// one. Shared by optimize and rotation so the two never drift.
fn build_pins(layout: &[DraftRoom], scope: &[String], locked: &[String]) -> Vec<(String, String)> {
    let in_scope: HashSet<&str> = scope.iter().map(String::as_str).collect();
    let locked: HashSet<&str> = locked.iter().map(String::as_str).collect();

    let mut pins = Vec::new();
    for room in layout {
        let frozen_room = !(in_scope.is_empty() || in_scope.contains(room.slot_id.as_str()));
        let unmodeled = UNMODELED_ROOM_TYPES.contains(&room.room_type.as_str());
        for op in &room.operators {
            if frozen_room || unmodeled || locked.contains(op.as_str()) {
                pins.push((op.clone(), room.room_type.clone()));
            }
        }
    }
    pins
}

/// What changed, room by room.
///
/// The "before" crew comes from the DRAFT, not from `baseline`: a
/// `BaseAssignment` only carries production rooms and a staffed Control Center,
/// so keying off it silently discards every proposal for a Power Plant, Office
/// or Reception Room. Baseline supplies the efficiency and yield figures where
/// it has them, and zero where the engine reports none.
fn diff_rooms(
    draft: &[DraftRoom],
    baseline: &BaseAssignment,
    proposal: &BaseAssignment,
) -> Vec<RoomDiffDto> {
    let scored_by_slot: HashMap<&str, &RoomAssignment> = baseline
        .rooms
        .iter()
        .map(|r| (r.slot_id.as_str(), r))
        .collect();
    let drafted_by_slot: HashMap<&str, &DraftRoom> =
        draft.iter().map(|r| (r.slot_id.as_str(), r)).collect();

    let mut diffs = Vec::new();
    for after in &proposal.rooms {
        let Some(drafted) = drafted_by_slot.get(after.slot_id.as_str()) else {
            continue;
        };
        let before = scored_by_slot.get(after.slot_id.as_str()).copied();
        // Crew identity, not seat order - reordering the same operators is not a
        // change worth asking the player to review.
        let a: HashSet<&str> = drafted.operators.iter().map(String::as_str).collect();
        let b: HashSet<&str> = after.operators.iter().map(String::as_str).collect();
        if a == b {
            continue;
        }
        let y_before = before.map_or_else(Default::default, |r| {
            room_yield(
                &r.room_type,
                r.formula_type.as_deref(),
                r.level,
                r.total_efficiency,
                r.order_value,
            )
        });
        let y_after = room_yield(
            &after.room_type,
            after.formula_type.as_deref(),
            after.level,
            after.total_efficiency,
            after.order_value,
        );
        diffs.push(RoomDiffDto {
            slot_id: after.slot_id.clone(),
            room_type: after.room_type.clone(),
            before: drafted.operators.clone(),
            after: after.operators.clone(),
            efficiency_before: before.map_or(0.0, |r| r.total_efficiency),
            efficiency_after: after.total_efficiency,
            yield_before: y_before.lmd_per_day + y_before.exp_per_day,
            yield_after: y_after.lmd_per_day + y_after.exp_per_day,
        });
    }
    diffs
}

pub fn catalog(state: &AppState) -> CatalogResponse {
    let game_data = state.default_game_data();

    let mut rooms: Vec<CatalogRoomDto> = game_data
        .building
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
                .enumerate()
                .map(|(i, p)| CatalogPhaseDto {
                    level: i32::try_from(i + 1).unwrap_or(i32::MAX),
                    max_stationed: p.max_stationed_num,
                    electricity: p.electricity,
                    manpower_cost: p.manpower_cost,
                })
                .collect(),
        })
        .collect();
    rooms.sort_by(|a, b| a.room_type.cmp(&b.room_type));

    let mut formulas: Vec<CatalogFormulaDto> = ["F_GOLD", "F_EXP", "F_DIAMOND"]
        .iter()
        .map(|t| CatalogFormulaDto {
            formula_type: (*t).to_string(),
            label: match *t {
                "F_GOLD" => "Pure Gold",
                "F_EXP" => "Battle Records",
                _ => "Originium Shard",
            }
            .to_string(),
        })
        .collect();
    formulas.sort_by(|a, b| a.formula_type.cmp(&b.formula_type));

    // The game keys them, so take whichever is
    // there rather than naming it - a second layout should not blank the board.
    let layout = game_data.building.layouts.values().next();

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
    // Reading order, so the client never has to sort to lay the board out.
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

    CatalogResponse {
        shift_count: SHIFT_COUNT,
        rooms,
        formulas,
        slots,
        storeys,
    }
}
