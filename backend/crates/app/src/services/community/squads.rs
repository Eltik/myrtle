use std::collections::HashMap;

use chrono::{DateTime, Utc};
use myrtle_db::repos::squads::{self, GameSquadRow, SavedSquadRow};
use myrtle_engine_squad::policy::{
    BUILD_FIELD_MAX, DEPLOY_LIMIT_MAX, NAME_MAX_CHARS, SLOT_LIMIT, SQUAD_SIZE, STAGE_ID_MAX,
    START_DP_MAX,
};
use myrtle_engine_squad::rules::{SkillRules, module_ranks};
use myrtle_engine_squad::types::SquadMemberBuild;
use myrtle_engine_squad::{SquadAnalysis, SquadAnalyzeRequest, SquadData};
use myrtle_gamedata::index::squad::{ThreatThresholds, playable};
use myrtle_gamedata::model::GameModel;
use myrtle_gamedata_raw::tables::module::ModuleType;
use myrtle_types::{Server, UserId};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::policy::Subject;
use crate::{AppContext, AppError};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct SquadBuild {
    pub phase: i32,
    pub level: i32,
    pub potential: i32,
    pub skill_level: i32,
    pub mastery: i32,
    pub module_level: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct SquadMember {
    pub char_id: String,
    pub skill_index: i32,
    pub module_id: Option<String>,
    pub build: Option<SquadBuild>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct SavedSquadInput {
    pub name: String,
    pub members: Vec<Option<SquadMember>>,
    pub support: Option<SquadMember>,
    pub stage_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct SavedSquad {
    pub slot: i32,
    pub name: String,
    pub members: Vec<Option<SquadMember>>,
    pub support: Option<SquadMember>,
    pub stage_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct GameSquadSlot {
    pub char_id: String,
    pub skill_index: i32,
    pub module_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct GameSquad {
    pub index: i32,
    pub name: String,
    pub slots: Vec<Option<GameSquadSlot>>,
    pub synced_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct SquadsResponse {
    pub saved: Vec<Option<SavedSquad>>,
    pub synced: Vec<GameSquad>,
    pub slot_limit: i32,
    pub squad_size: i32,
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct SquadRules {
    pub threats: ThreatThresholds,
    pub normal_of: HashMap<String, String>,
}

pub fn rules(ctx: &AppContext, server: Server) -> Result<SquadRules, AppError> {
    let snapshot = ctx.snapshots.or_default(server)?;
    let squad = &snapshot.model.squad;
    Ok(SquadRules {
        threats: squad.threats.clone(),
        normal_of: squad.normal_of.clone(),
    })
}

pub async fn get(ctx: &AppContext, subject: &Subject) -> Result<SquadsResponse, AppError> {
    let user = signed_in(subject)?;
    let mut conn = ctx.db.conn().await?;
    let rows = squads::list_saved(&mut conn, user).await?;
    let synced = squads::list_game_squads(&mut conn, user).await?;

    let mut saved: Vec<Option<SavedSquad>> = vec![None; SQUAD_SLOTS];
    for row in rows {
        let squad = saved_squad(row);
        if let Some(entry) = usize::try_from(squad.slot - 1)
            .ok()
            .and_then(|i| saved.get_mut(i))
        {
            *entry = Some(squad);
        }
    }
    Ok(SquadsResponse {
        saved,
        synced: synced.into_iter().map(game_squad).collect(),
        slot_limit: SLOT_LIMIT,
        squad_size: SQUAD_SIZE as i32,
    })
}

pub async fn save(
    ctx: &AppContext,
    subject: &Subject,
    slot: i32,
    input: SavedSquadInput,
    server: Server,
) -> Result<SavedSquad, AppError> {
    let user = signed_in(subject)?;
    let slot = checked_slot(slot)?;
    let squad = checked(&ctx.snapshots.or_default(server)?.model, slot, input)?;
    let members = serde_json::to_value(&squad.members).map_err(AppError::internal)?;
    let support = squad
        .support
        .as_ref()
        .map(serde_json::to_value)
        .transpose()
        .map_err(AppError::internal)?;
    let row = squads::upsert_saved(
        &mut *ctx.db.conn().await?,
        user,
        slot,
        &squad.name,
        &members,
        support.as_ref(),
        squad.stage_id.as_deref(),
    )
    .await?;
    Ok(saved_squad(row))
}

pub async fn delete(ctx: &AppContext, subject: &Subject, slot: i32) -> Result<(), AppError> {
    let user = signed_in(subject)?;
    let slot = checked_slot(slot)?;
    if squads::delete_saved(&mut *ctx.db.conn().await?, user, slot).await? {
        Ok(())
    } else {
        Err(AppError::NotFound(None))
    }
}

pub async fn analyze(
    ctx: &AppContext,
    request: SquadAnalyzeRequest,
    server: Server,
) -> Result<SquadAnalysis, AppError> {
    check_seats(
        &request.members,
        request.support.as_ref(),
        check_member_build,
    )?;
    range("start_dp", request.start_dp, 0, START_DP_MAX)?;
    if let Some(limit) = request.deploy_limit {
        range("deploy_limit", limit, 0, DEPLOY_LIMIT_MAX)?;
    }
    let data = ctx.snapshots.or_default(server)?;
    let english = ctx.snapshots.get(Server::EN);
    ctx.cpu
        .run("squad_analyze", move || {
            myrtle_engine_squad::analyze(
                &SquadData::from(&data.model),
                english.as_ref().map(|en| &en.model.squad.text),
                &request,
            )
        })
        .await
}

const SQUAD_SLOTS: usize = SLOT_LIMIT as usize;

#[derive(Debug)]
pub struct CheckedSquad {
    pub name: String,
    pub members: Vec<Option<SquadMember>>,
    pub support: Option<SquadMember>,
    pub stage_id: Option<String>,
}

fn signed_in(subject: &Subject) -> Result<UserId, AppError> {
    subject.user_id().ok_or(AppError::Unauthorized)
}

pub fn checked_slot(slot: i32) -> Result<i16, AppError> {
    if (1..=SLOT_LIMIT).contains(&slot)
        && let Ok(v) = i16::try_from(slot)
    {
        return Ok(v);
    }
    Err(AppError::BadRequest(format!(
        "slot must be 1 to {SLOT_LIMIT}, got {slot}"
    )))
}

pub fn checked(
    data: &GameModel,
    slot: i16,
    input: SavedSquadInput,
) -> Result<CheckedSquad, AppError> {
    let name = input.name.trim();
    max_chars("name", name, NAME_MAX_CHARS)?;
    let name = if name.is_empty() {
        format!("Squad {slot}")
    } else {
        name.to_owned()
    };

    check_seats(&input.members, input.support.as_ref(), |field, member| {
        check_member(data, field, member)
    })?;
    let mut members = input.members;
    members.resize_with(SQUAD_SIZE, || None);

    let stage_id = input
        .stage_id
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty());
    if let Some(stage_id) = &stage_id {
        max_chars("stage_id", stage_id, STAGE_ID_MAX)?;
        let valid = stage_id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '#' | '-'));
        if !valid {
            return Err(AppError::bad_request(
                "stage_id may hold only letters, digits, `_`, `#` and `-`",
            ));
        }
    }

    Ok(CheckedSquad {
        name,
        members,
        support: input.support,
        stage_id,
    })
}

fn check_seats<T>(
    members: &[Option<T>],
    support: Option<&T>,
    check: impl Fn(&str, &T) -> Result<(), AppError>,
) -> Result<(), AppError> {
    if members.len() > SQUAD_SIZE {
        return Err(AppError::BadRequest(format!(
            "members: at most {SQUAD_SIZE} seats, got {}",
            members.len()
        )));
    }
    for (i, member) in members.iter().enumerate() {
        if let Some(member) = member {
            check(&format!("members[{i}]"), member)?;
        }
    }
    if let Some(support) = support {
        check("support", support)?;
    }
    Ok(())
}

fn check_member(data: &GameModel, field: &str, member: &SquadMember) -> Result<(), AppError> {
    let operator = data
        .operators
        .get(&member.char_id)
        .filter(|op| playable(op))
        .ok_or_else(|| {
            AppError::BadRequest(format!("{field}: unknown operator `{}`", member.char_id))
        })?;

    let skill_count = i32::try_from(operator.skills.len()).unwrap_or(i32::MAX);
    let skill_ok = if skill_count == 0 {
        member.skill_index == 0
    } else {
        (0..skill_count).contains(&member.skill_index)
    };
    if !skill_ok {
        return Err(AppError::BadRequest(format!(
            "{field}: skill_index {} is not one of `{}`'s skills",
            member.skill_index, member.char_id
        )));
    }

    let module = match &member.module_id {
        Some(module_id) => Some(
            operator
                .modules
                .iter()
                .find(|m| {
                    m.module.module_type == ModuleType::Advanced
                        && m.module.uni_equip_id == *module_id
                })
                .ok_or_else(|| {
                    AppError::BadRequest(format!(
                        "{field}: `{module_id}` is not one of `{}`'s modules",
                        member.char_id
                    ))
                })?,
        ),
        None => None,
    };

    if let Some(build) = &member.build {
        let rules = SkillRules::of(operator);
        let skill = usize::try_from(member.skill_index).unwrap_or(0);
        let max_phase = i32::try_from(operator.phases.len()).unwrap_or(1) - 1;
        range(&format!("{field}.build.phase"), build.phase, 0, max_phase)?;
        let level_cap = usize::try_from(build.phase)
            .ok()
            .and_then(|p| operator.phases.get(p))
            .map_or(1, |p| p.max_level);
        range(&format!("{field}.build.level"), build.level, 1, level_cap)?;
        let max_pot = i32::try_from(operator.potential_ranks.len()).unwrap_or(0);
        range(
            &format!("{field}.build.potential"),
            build.potential,
            0,
            max_pot,
        )?;
        range(
            &format!("{field}.build.skill_level"),
            build.skill_level,
            1,
            rules.max_rank(),
        )?;
        range(
            &format!("{field}.build.mastery"),
            build.mastery,
            0,
            rules.mastery_cap(skill, build.skill_level),
        )?;
        range(
            &format!("{field}.build.module_level"),
            build.module_level,
            0,
            module.map_or(0, module_ranks),
        )?;
    }
    Ok(())
}

pub fn check_member_build(field: &str, member: &SquadMemberBuild) -> Result<(), AppError> {
    for (name, value) in [
        ("phase", member.phase),
        ("level", member.level),
        ("potential", member.potential),
        ("skill_index", member.skill_index),
        ("skill_level", member.skill_level),
        ("mastery", member.mastery),
        ("module_level", member.module_level),
    ] {
        range(&format!("{field}.{name}"), value, 0, BUILD_FIELD_MAX)?;
    }
    Ok(())
}

pub fn range(field: &str, value: i32, min: i32, max: i32) -> Result<(), AppError> {
    if (min..=max).contains(&value) {
        Ok(())
    } else {
        Err(AppError::BadRequest(format!(
            "{field} must be {min} to {max}, got {value}"
        )))
    }
}

fn max_chars(field: &str, value: &str, max: usize) -> Result<(), AppError> {
    if value.chars().count() > max {
        return Err(AppError::BadRequest(format!("{field} max {max} chars")));
    }
    Ok(())
}

fn saved_squad(row: SavedSquadRow) -> SavedSquad {
    SavedSquad {
        slot: i32::from(row.slot),
        name: row.name,
        members: seats(row.members),
        support: row.support.and_then(|s| serde_json::from_value(s).ok()),
        stage_id: row.stage_id,
        created_at: row.created_at,
        updated_at: row.updated_at,
    }
}

fn game_squad(row: GameSquadRow) -> GameSquad {
    GameSquad {
        index: i32::from(row.squad_index),
        name: row.name,
        slots: seats(row.slots),
        synced_at: row.synced_at,
    }
}

#[must_use]
pub fn seats<T: DeserializeOwned>(value: Value) -> Vec<Option<T>> {
    let mut out: Vec<Option<T>> = match value {
        Value::Array(items) => items
            .into_iter()
            .take(SQUAD_SIZE)
            .map(|item| serde_json::from_value::<Option<T>>(item).ok().flatten())
            .collect(),
        _ => Vec::new(),
    };
    out.resize_with(SQUAD_SIZE, || None);
    out
}
