//! Squad Builder: the account's four saved squads, the in-game squads from the
//! last roster sync, and the per-squad analysis.
//!
//! A saved squad is read and written whole. Operator ids, skills, modules and
//! every build number are checked against the reader's server's game data on
//! save, each bound the operator's own; a stored squad whose operator later
//! leaves the data still loads, since reads never re-validate.

use serde::Serialize;
use ts_rs::TS;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::app::cpu;
use crate::app::error::ApiError;
use crate::app::state::AppState;
use crate::app::validation::validate_length;
use crate::core::gamedata::types::GameData;
use crate::core::gamedata::types::module::ModuleType;
use crate::core::hypergryph::constants::Server;
use crate::core::squad::index::{ThreatThresholds, playable};
use crate::core::squad::policy::{
    BUILD_FIELD_MAX, DEPLOY_LIMIT_MAX, NAME_MAX_CHARS, SLOT_LIMIT, SQUAD_SIZE, STAGE_ID_MAX,
    START_DP_MAX,
};
use crate::core::squad::rules::{SkillRules, module_ranks};
use crate::core::squad::{
    self,
    types::{SquadAnalysis, SquadAnalyzeRequest, SquadMemberBuild},
};
use crate::database::models::squad::{
    GameSquadRow, SavedSquad, SavedSquadInput, SquadMember, SquadsResponse,
};
use crate::database::queries::squads as queries;

/// What the page reads once per server before any squad exists: where a
/// stage's enemies start to count as a threat, and each challenge stage's
/// normal stage.
#[derive(TS, ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Serialize)]
pub struct SquadRules {
    pub threats: ThreatThresholds,
    /// Challenge stage id -> its normal stage id, the inverse of
    /// `stage_table` `HardStagedId`.
    pub normal_of: std::collections::HashMap<String, String>,
}

/// `GET /squads/rules`.
pub fn rules(state: &AppState, server: Server) -> SquadRules {
    let data = state.game_data(server);
    SquadRules {
        threats: data.squad.threats.clone(),
        normal_of: data.squad.normal_of.clone(),
    }
}

/// The caller's saved squads, one entry per slot, and their synced in-game
/// squads in game order.
pub async fn get(state: &AppState, user_id: Uuid) -> Result<SquadsResponse, ApiError> {
    let mut conn = state.db.acquire().await?;
    let rows = queries::list_saved(&mut conn, user_id).await?;
    let synced = queries::list_game_squads(&mut conn, user_id).await?;

    let mut saved: Vec<Option<SavedSquad>> = vec![None; slot_count()];
    for row in rows {
        let squad = row.into_saved();
        if let Some(entry) = usize::try_from(squad.slot - 1)
            .ok()
            .and_then(|i| saved.get_mut(i))
        {
            *entry = Some(squad);
        }
    }
    Ok(SquadsResponse {
        saved,
        synced: synced
            .into_iter()
            .map(GameSquadRow::into_game_squad)
            .collect(),
        slot_limit: SLOT_LIMIT,
        squad_size: SQUAD_SIZE as i32,
    })
}

/// Write `input` into `slot`, replacing whatever was there.
///
/// # Errors
/// `400` naming the first rule broken.
pub async fn save(
    state: &AppState,
    user_id: Uuid,
    slot: i32,
    input: SavedSquadInput,
    server: Server,
) -> Result<SavedSquad, ApiError> {
    let slot = checked_slot(slot)?;
    let data = state.game_data(server);
    let squad = checked(&data, slot, input)?;
    let mut conn = state.db.acquire().await?;
    let row = queries::upsert_saved(
        &mut conn,
        user_id,
        slot,
        &squad.name,
        &squad.members,
        squad.support.as_ref(),
        squad.stage_id.as_deref(),
    )
    .await?;
    Ok(row.into_saved())
}

/// Empty `slot`.
///
/// # Errors
/// `404` when the slot holds no squad.
pub async fn delete(state: &AppState, user_id: Uuid, slot: i32) -> Result<(), ApiError> {
    let slot = checked_slot(slot)?;
    let mut conn = state.db.acquire().await?;
    if queries::delete_saved(&mut conn, user_id, slot).await? {
        Ok(())
    } else {
        Err(ApiError::NotFound)
    }
}

/// Analyse one squad on the reader's `server`. Unknown operator ids are
/// skipped by the analysis and every build number is clamped to what the
/// operator has, so only shapes and sanity caps are checked here.
///
/// The numbers (costs, blackboards, stats) come from `server`'s data. The
/// English text rules' readings come from the EN server's data, keyed by ids
/// (`squad::index`): an operator EN does not have yet comes back in
/// `unread_seats` instead of being skipped.
///
/// # Errors
/// `400` naming the first rule broken; `503` when the CPU queue is full.
pub async fn analyze(
    state: &AppState,
    request: SquadAnalyzeRequest,
    server: Server,
) -> Result<SquadAnalysis, ApiError> {
    check_seats(
        &request.members,
        request.support.as_ref(),
        check_member_build,
    )?;
    range("start_dp", request.start_dp, 0, START_DP_MAX)?;
    if let Some(limit) = request.deploy_limit {
        range("deploy_limit", limit, 0, DEPLOY_LIMIT_MAX)?;
    }
    let data = state.game_data(server);
    let english = state
        .try_server_data(Server::EN)
        .map(|sd| sd.game_data.load_full());
    cpu::run("squad_analyze", move || {
        squad::analyze_with(&data, english.as_deref(), &request)
    })
    .await
}

/// A squad ready to store.
struct CheckedSquad {
    name: String,
    members: Vec<Option<SquadMember>>,
    support: Option<SquadMember>,
    stage_id: Option<String>,
}

fn slot_count() -> usize {
    usize::try_from(SLOT_LIMIT).unwrap_or_default()
}

fn checked_slot(slot: i32) -> Result<i16, ApiError> {
    if (1..=SLOT_LIMIT).contains(&slot)
        && let Ok(v) = i16::try_from(slot)
    {
        return Ok(v);
    }
    Err(ApiError::BadRequest(format!(
        "slot must be 1 to {SLOT_LIMIT}, got {slot}"
    )))
}

/// `input` trimmed, padded to twelve seats, and every rule checked against
/// `data`.
fn checked(data: &GameData, slot: i16, input: SavedSquadInput) -> Result<CheckedSquad, ApiError> {
    let name = input.name.trim();
    validate_length("name", name, NAME_MAX_CHARS)?;
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
        validate_length("stage_id", stage_id, STAGE_ID_MAX)?;
        let valid = stage_id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '#' | '-'));
        if !valid {
            return Err(ApiError::BadRequest(
                "stage_id may hold only letters, digits, `_`, `#` and `-`".into(),
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

/// At most twelve seats, then `check` on each filled seat (`members[i]`) and
/// the support, in that order.
fn check_seats<T>(
    members: &[Option<T>],
    support: Option<&T>,
    check: impl Fn(&str, &T) -> Result<(), ApiError>,
) -> Result<(), ApiError> {
    if members.len() > SQUAD_SIZE {
        return Err(ApiError::BadRequest(format!(
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

/// A seat's operator must be a playable operator in `data`, its skill one the
/// operator has, its module one of the operator's advanced modules (the
/// initial module means no module and is sent as `None`), and its build
/// within the operator's own bounds.
fn check_member(data: &GameData, field: &str, member: &SquadMember) -> Result<(), ApiError> {
    let operator = data
        .operators
        .get(&member.char_id)
        .filter(|op| playable(op))
        .ok_or_else(|| {
            ApiError::BadRequest(format!("{field}: unknown operator `{}`", member.char_id))
        })?;

    let skill_count = i32::try_from(operator.skills.len()).unwrap_or(i32::MAX);
    let skill_ok = if skill_count == 0 {
        member.skill_index == 0
    } else {
        (0..skill_count).contains(&member.skill_index)
    };
    if !skill_ok {
        return Err(ApiError::BadRequest(format!(
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
                    ApiError::BadRequest(format!(
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
        let module_max = module.map_or(0, module_ranks);
        range(
            &format!("{field}.build.module_level"),
            build.module_level,
            0,
            module_max,
        )?;
    }
    Ok(())
}

/// Sanity caps on an analyze request's numbers. The analysis clamps each one
/// to the operator's own bounds, so these only turn away garbage.
fn check_member_build(field: &str, member: &SquadMemberBuild) -> Result<(), ApiError> {
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

fn range(field: &str, value: i32, min: i32, max: i32) -> Result<(), ApiError> {
    if (min..=max).contains(&value) {
        Ok(())
    } else {
        Err(ApiError::BadRequest(format!(
            "{field} must be {min} to {max}, got {value}"
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build(value: i32) -> SquadMemberBuild {
        SquadMemberBuild {
            char_id: "char_002_amiya".into(),
            phase: value,
            level: value,
            potential: value,
            skill_index: value,
            skill_level: value,
            mastery: value,
            module_id: None,
            module_level: value,
        }
    }

    fn member(char_id: &str) -> SquadMember {
        SquadMember {
            char_id: char_id.into(),
            skill_index: 0,
            module_id: None,
            build: None,
        }
    }

    fn input(members: Vec<Option<SquadMember>>, stage_id: Option<&str>) -> SavedSquadInput {
        SavedSquadInput {
            name: "  ".into(),
            members,
            support: None,
            stage_id: stage_id.map(str::to_string),
        }
    }

    fn message<T>(r: Result<T, ApiError>) -> String {
        match r {
            Err(ApiError::BadRequest(m)) => m,
            Err(e) => panic!("expected a 400, got {e:?}"),
            Ok(_) => panic!("expected a 400, got Ok"),
        }
    }

    #[test]
    fn slot_and_range_messages() {
        assert_eq!(checked_slot(1).unwrap(), 1);
        assert_eq!(checked_slot(4).unwrap(), 4);
        assert_eq!(message(checked_slot(0)), "slot must be 1 to 4, got 0");
        assert_eq!(message(checked_slot(5)), "slot must be 1 to 4, got 5");
        assert!(range("x", 3, 0, 3).is_ok());
        assert_eq!(message(range("x", -1, 0, 3)), "x must be 0 to 3, got -1");
    }

    #[test]
    fn analyze_build_caps() {
        assert!(check_member_build("members[0]", &build(999)).is_ok());
        assert_eq!(
            message(check_member_build("members[0]", &build(1000))),
            "members[0].phase must be 0 to 999, got 1000"
        );
        let mut b = build(1);
        b.mastery = -1;
        assert_eq!(
            message(check_member_build("support", &b)),
            "support.mastery must be 0 to 999, got -1"
        );
    }

    #[test]
    fn saved_squad_checks() {
        let data = GameData::default();
        let ok = checked(&data, 2, input(Vec::new(), Some(" main_01-07 "))).unwrap();
        assert_eq!(ok.name, "Squad 2");
        assert_eq!(ok.members.len(), SQUAD_SIZE);
        assert_eq!(ok.stage_id.as_deref(), Some("main_01-07"));
        assert_eq!(
            checked(&data, 1, input(Vec::new(), Some("   ")))
                .unwrap()
                .stage_id,
            None
        );
        assert_eq!(
            message(checked(&data, 1, input(vec![None; 13], None))),
            "members: at most 12 seats, got 13"
        );
        assert_eq!(
            message(checked(
                &data,
                1,
                input(vec![None, Some(member("char_x"))], None)
            )),
            "members[1]: unknown operator `char_x`"
        );
        let mut with_support = input(Vec::new(), None);
        with_support.support = Some(member("char_y"));
        assert_eq!(
            message(checked(&data, 1, with_support)),
            "support: unknown operator `char_y`"
        );
        assert_eq!(
            message(checked(&data, 1, input(Vec::new(), Some("a b")))),
            "stage_id may hold only letters, digits, `_`, `#` and `-`"
        );
    }
}
