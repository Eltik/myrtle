use std::collections::{BTreeMap, HashMap};

use myrtle_db::repos::roster::SyncUserData;
use myrtle_engine_squad::policy::SQUAD_SIZE;
use myrtle_gamedata::model::GameModel;
use myrtle_gamedata::model::stage_evidence::{StageEvidenceIndex, fold_into_records};
use myrtle_gamedata_raw::tables::campaign::CampaignRotations;
use myrtle_gamedata_raw::tables::module::{ModuleType, Modules};
use myrtle_types::Server;
use serde::Deserialize;
use serde_json::{Map, Value, json};

use crate::AppError;

pub const EXPEDITED_PLAN_ITEM: &str = "7002";
pub const UNIVERSAL_CERTIFICATE_ITEM: &str = "classic_normal_ticket";

type Object = Map<String, Value>;

#[derive(Deserialize)]
struct SyncDataResponse {
    user: Option<GameUser>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GameUser {
    status: Option<PlayerStatus>,
    troop: Option<Troop>,
    inventory: Option<Object>,
    consumable: Option<Object>,
    skin: Option<SkinStore>,
    medal: Option<MedalStore>,
    dungeon: Option<Value>,
    building: Option<Value>,
    #[serde(rename = "rlv2")]
    roguelike: Option<Value>,
    #[serde(rename = "sandboxPerm")]
    sandbox_perm: Option<Value>,
    #[serde(rename = "checkIn")]
    checkin: Option<CheckIn>,
    social: Option<Social>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Social {
    assist_char_list: Option<Vec<Option<Seat>>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Seat {
    char_inst_id: Option<i64>,
    skill_index: Option<i64>,
    current_equip: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PlayerStatus {
    nick_name: Option<String>,
    level: Option<i64>,
    avatar: Option<Avatar>,
    secretary: Option<String>,
    secretary_skin_id: Option<String>,
    resume: Option<String>,
    exp: Option<i64>,
    #[serde(rename = "diamondShard")]
    orundum: Option<i64>,
    gold: Option<i64>,
    ap: Option<i64>,
    max_ap: Option<i64>,
    gacha_ticket: Option<i64>,
    ten_gacha_ticket: Option<i64>,
    classic_gacha_ticket: Option<i64>,
    classic_ten_gacha_ticket: Option<i64>,
    recruit_license: Option<i64>,
    social_point: Option<i64>,
    hgg_shard: Option<i64>,
    lgg_shard: Option<i64>,
    practice_ticket: Option<i64>,
    instant_finish_ticket: Option<i64>,
    classic_shard: Option<i64>,
    #[serde(rename = "monthlySubscriptionEndTime")]
    monthly_sub_end: Option<i64>,
    register_ts: Option<i64>,
    last_online_ts: Option<i64>,
    friend_num_limit: Option<i64>,
    main_stage_progress: Option<String>,
}

#[derive(Deserialize)]
struct Avatar {
    id: Option<String>,
}

#[derive(Deserialize)]
struct Troop {
    chars: Option<Object>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TroopSquad {
    squad_id: Option<String>,
    name: Option<String>,
    slots: Option<Vec<Option<Seat>>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TroopChar {
    char_id: Option<String>,
    evolve_phase: Option<i64>,
    level: Option<i64>,
    exp: Option<i64>,
    potential_rank: Option<i64>,
    main_skill_lvl: Option<i64>,
    favor_point: Option<i64>,
    skin: Option<String>,
    default_skill_index: Option<i64>,
    current_equip: Option<String>,
    gain_time: Option<i64>,
    voice_lan: Option<String>,
    current_tmpl: Option<String>,
    skills: Option<Vec<TroopSkill>>,
    equip: Option<Object>,
    tmpl: Option<HashMap<String, TroopTmpl>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TroopTmpl {
    skin_id: Option<String>,
    default_skill_index: Option<i64>,
    current_equip: Option<String>,
    skills: Option<Vec<TroopSkill>>,
    equip: Option<Object>,
    evolve_phase: Option<i64>,
    level: Option<i64>,
    exp: Option<i64>,
    main_skill_lvl: Option<i64>,
    favor_point: Option<i64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TroopSkill {
    specialize_level: Option<i64>,
}

#[derive(Deserialize)]
struct EquipEntry {
    level: Option<i64>,
    locked: Option<i64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SkinStore {
    character_skins: Option<Object>,
}

#[derive(Deserialize)]
struct MedalStore {
    medals: Option<Object>,
}

#[derive(Deserialize)]
struct CheckIn {
    #[serde(rename = "checkInHistory")]
    history: Option<Vec<i16>>,
    #[serde(rename = "showCount")]
    show_count: Option<i64>,
    #[serde(rename = "checkInGroupId")]
    group_id: Option<String>,
    #[serde(rename = "checkInRewardIndex")]
    reward_index: Option<i64>,
    #[serde(rename = "canCheckIn")]
    can_check_in: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct SyncRows {
    pub nickname: String,
    pub nick_number: Option<String>,
    pub level: i16,
    pub avatar_id: Option<String>,
    pub secretary: Option<String>,
    pub secretary_skin_id: Option<String>,
    pub resume_id: Option<String>,
    pub operators: Value,
    pub skills: Value,
    pub modules: Value,
    pub items: Value,
    pub skins: Value,
    pub status: Value,
    pub stages: Value,
    pub roguelike: Value,
    pub sandbox: Value,
    pub medals: Value,
    pub building: Value,
    pub checkin: Value,
    pub supports: Value,
    pub enemies: Value,
    pub game_squads: Value,
}

impl SyncRows {
    #[must_use]
    pub fn input<'a>(&'a self, uid: &'a str, server: Server) -> SyncUserData<'a> {
        SyncUserData {
            uid,
            server,
            nickname: &self.nickname,
            level: self.level,
            avatar_id: self.avatar_id.as_deref(),
            secretary: self.secretary.as_deref(),
            secretary_skin_id: self.secretary_skin_id.as_deref(),
            resume_id: self.resume_id.as_deref(),
            operators: &self.operators,
            skills: &self.skills,
            modules: &self.modules,
            items: &self.items,
            skins: &self.skins,
            status: &self.status,
            stages: &self.stages,
            roguelike: &self.roguelike,
            sandbox: &self.sandbox,
            medals: &self.medals,
            building: &self.building,
            checkin: &self.checkin,
            supports: &self.supports,
            nick_number: self.nick_number.as_deref(),
            enemies: &self.enemies,
            game_squads: &self.game_squads,
        }
    }
}

pub fn rows(
    raw: &Value,
    model: &GameModel,
    campaign: &CampaignRotations,
    now: i64,
) -> Result<SyncRows, AppError> {
    let data = SyncDataResponse::deserialize(raw).map_err(AppError::internal)?;
    let user = data
        .user
        .ok_or_else(|| AppError::bad_request("missing user data in response"))?;
    let status = user.status.as_ref();
    let text = |f: fn(&PlayerStatus) -> Option<&String>| status.and_then(f).cloned();
    let chars = troop_chars(user.troop.as_ref());

    let mut status_json = extract_status(status);
    status_json["originite"] = extract_originite(raw).map_or(Value::Null, Value::from);
    let mut stages = user
        .dungeon
        .as_ref()
        .and_then(|d| d.get("stages"))
        .cloned()
        .unwrap_or_default();
    merge_campaign_clears(&mut stages, raw, campaign);
    merge_inferred_clears(&mut stages, raw, &model.stage_evidence);
    merge_cow_level(&mut stages, raw);

    Ok(SyncRows {
        nickname: text(|s| s.nick_name.as_ref()).unwrap_or_default(),
        nick_number: extract_nick_number(raw),
        level: status
            .and_then(|s| s.level)
            .map_or(0, |l| i16::try_from(l).unwrap_or(i16::MAX)),
        avatar_id: status
            .and_then(|s| s.avatar.as_ref())
            .and_then(|a| a.id.clone()),
        secretary: text(|s| s.secretary.as_ref()),
        secretary_skin_id: text(|s| s.secretary_skin_id.as_ref()),
        resume_id: text(|s| s.resume.as_ref()),
        operators: extract_operators(&chars),
        skills: extract_skills(&chars),
        modules: extract_modules(&chars),
        items: extract_items(
            user.inventory.as_ref(),
            user.consumable.as_ref(),
            &[
                (
                    EXPEDITED_PLAN_ITEM,
                    status.and_then(|s| s.instant_finish_ticket),
                ),
                (
                    UNIVERSAL_CERTIFICATE_ITEM,
                    status.and_then(|s| s.classic_shard),
                ),
            ],
            now,
        ),
        skins: extract_skins(user.skin.as_ref()),
        status: status_json,
        stages,
        roguelike: extract_roguelike(user.roguelike.as_ref()),
        sandbox: user.sandbox_perm.unwrap_or_default(),
        medals: extract_medals(user.medal.as_ref()),
        building: user.building.unwrap_or_default(),
        checkin: extract_checkin(user.checkin.as_ref()),
        supports: extract_supports(user.troop.as_ref(), user.social.as_ref()),
        enemies: raw
            .pointer("/user/dexNav/enemy/enemies")
            .cloned()
            .unwrap_or_else(|| json!({})),
        game_squads: extract_game_squads(raw.pointer("/user/troop"), &model.modules),
    })
}

#[must_use]
pub fn extract_originite(raw: &Value) -> Option<i64> {
    let number = |key: &str| {
        raw.pointer(&format!("/user/status/{key}"))
            .and_then(Value::as_i64)
    };
    match (number("payDiamond"), number("freeDiamond")) {
        (None, None) => None,
        (pay, free) => Some(pay.unwrap_or(0) + free.unwrap_or(0)),
    }
}

#[must_use]
pub fn extract_nick_number(raw: &Value) -> Option<String> {
    [
        "/user/status/nickNumber",
        "/user/social/nickNumber",
        "/user/nickNumber",
    ]
    .into_iter()
    .filter_map(|path| raw.pointer(path))
    .find_map(|candidate| match candidate {
        Value::String(s) if !s.is_empty() => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    })
}

fn troop_chars(troop: Option<&Troop>) -> Vec<(String, TroopChar)> {
    troop
        .and_then(|t| t.chars.as_ref())
        .into_iter()
        .flat_map(Map::values)
        .filter_map(|raw| {
            let c = TroopChar::deserialize(raw).ok()?;
            Some((c.char_id.clone()?, c))
        })
        .collect()
}

fn forms(
    chars: &[(String, TroopChar)],
) -> impl Iterator<Item = (&str, &TroopChar, Option<&TroopTmpl>)> {
    chars.iter().flat_map(|(id, c)| {
        let tmpl = c.tmpl.as_ref().filter(|t| !t.is_empty());
        let own = tmpl.is_none().then_some((id.as_str(), c, None));
        tmpl.into_iter()
            .flatten()
            .map(move |(form_id, form)| (form_id.as_str(), c, Some(form)))
            .chain(own)
    })
}

fn extract_operators(chars: &[(String, TroopChar)]) -> Value {
    forms(chars)
        .map(|(id, c, form)| {
            let pick = |f: fn(&TroopTmpl) -> Option<i64>, fallback: Option<i64>| {
                form.and_then(f).or(fallback)
            };
            json!({
                "operator_id": id,
                "elite": pick(|f| f.evolve_phase, c.evolve_phase).unwrap_or(0),
                "level": pick(|f| f.level, c.level).unwrap_or(1),
                "exp": pick(|f| f.exp, c.exp).unwrap_or(0),
                "potential": c.potential_rank.unwrap_or(0),
                "skill_level": pick(|f| f.main_skill_lvl, c.main_skill_lvl).unwrap_or(1),
                "favor_point": pick(|f| f.favor_point, c.favor_point).unwrap_or(0),
                "skin_id": form.and_then(|f| f.skin_id.clone()).or_else(|| c.skin.clone()).unwrap_or_default(),
                "default_skill": pick(|f| f.default_skill_index, c.default_skill_index).unwrap_or(0),
                "current_equip": form.and_then(|f| f.current_equip.clone()).or_else(|| c.current_equip.clone()),
                "obtained_at": c.gain_time.unwrap_or(0),
                "voice_lan": c.voice_lan,
                "current_tmpl": c.current_tmpl,
            })
        })
        .collect()
}

fn extract_skills(chars: &[(String, TroopChar)]) -> Value {
    forms(chars)
        .flat_map(|(id, c, form)| {
            let skills = form.map_or(c.skills.as_deref(), |f| f.skills.as_deref());
            skills
                .unwrap_or_default()
                .iter()
                .enumerate()
                .map(move |(i, s)| {
                    json!({
                        "operator_id": id,
                        "skill_index": i,
                        "specialize_level": s.specialize_level.unwrap_or(0),
                    })
                })
        })
        .collect()
}

fn extract_modules(chars: &[(String, TroopChar)]) -> Value {
    forms(chars)
        .flat_map(|(id, c, form)| {
            let equip = form.map_or(c.equip.as_ref(), |f| f.equip.as_ref());
            equip
                .into_iter()
                .flatten()
                .filter(|(module_id, _)| !module_id.starts_with("uniequip_001_")) // the default badge every operator owns
                .filter_map(move |(module_id, data)| {
                    let entry = EquipEntry::deserialize(data).ok()?;
                    Some(json!({
                        "operator_id": id,
                        "module_id": module_id,
                        "module_level": entry.level.unwrap_or(0),
                        "locked": entry.locked.is_some_and(|l| l != 0),
                    }))
                })
        })
        .collect()
}

#[must_use]
pub fn extract_items(
    inventory: Option<&Object>,
    consumable: Option<&Object>,
    status_counters: &[(&str, Option<i64>)],
    now: i64,
) -> Value {
    let mut quantities: BTreeMap<&str, i64> = inventory
        .into_iter()
        .flatten()
        .map(|(id, qty)| (id.as_str(), qty.as_i64().unwrap_or(0)))
        .collect();
    for (id, instances) in consumable.into_iter().flatten() {
        let total: i64 = instances
            .as_object()
            .into_iter()
            .flat_map(Map::values)
            .filter(|inst| {
                let expiry = inst.get("ts").and_then(Value::as_i64).unwrap_or(-1);
                expiry < 0 || expiry > now
            })
            .filter_map(|inst| inst.get("count").and_then(Value::as_i64))
            .sum();
        if total > 0 {
            *quantities.entry(id.as_str()).or_insert(0) += total;
        }
    }
    for (id, count) in status_counters {
        if let Some(count) = count.filter(|n| *n > 0) {
            *quantities.entry(id).or_insert(0) += count;
        }
    }
    quantities
        .into_iter()
        .map(|(id, quantity)| json!({ "item_id": id, "quantity": quantity }))
        .collect()
}

fn extract_skins(skin: Option<&SkinStore>) -> Value {
    skin.and_then(|s| s.character_skins.as_ref())
        .into_iter()
        .flatten()
        .map(|(id, data)| {
            let obtained = data.get("obtainedAt").and_then(Value::as_i64);
            json!({ "skin_id": id, "obtained_at": obtained.unwrap_or(0) })
        })
        .collect()
}

fn extract_status(status: Option<&PlayerStatus>) -> Value {
    let Some(s) = status else {
        return json!({});
    };
    let n = |v: Option<i64>| v.unwrap_or(0);
    json!({
        "exp": n(s.exp),
        "orundum": n(s.orundum),
        "orundum_shard": 0,
        "lmd": n(s.gold),
        "sanity": n(s.ap),
        "max_sanity": n(s.max_ap),
        "gacha_tickets": n(s.gacha_ticket),
        "ten_pull_tickets": n(s.ten_gacha_ticket),
        "classic_gacha_tickets": n(s.classic_gacha_ticket),
        "classic_ten_pull_tickets": n(s.classic_ten_gacha_ticket),
        "recruit_permits": n(s.recruit_license),
        "social_point": n(s.social_point),
        "hgg_shard": n(s.hgg_shard),
        "lgg_shard": n(s.lgg_shard),
        "practice_tickets": n(s.practice_ticket),
        "gold": n(s.gold),
        "monthly_sub_end": n(s.monthly_sub_end),
        "register_ts": n(s.register_ts),
        "last_online_ts": n(s.last_online_ts),
        "main_stage_progress": s.main_stage_progress.as_deref().unwrap_or(""),
        "resume": s.resume.as_deref().unwrap_or(""),
        "friend_num_limit": n(s.friend_num_limit),
    })
}

fn extract_checkin(checkin: Option<&CheckIn>) -> Value {
    let Some(c) = checkin else {
        return json!({ "history": [] });
    };
    json!({
        "history": c.history.clone().unwrap_or_default(),
        "cumulative_signin": c.show_count.unwrap_or(0),
        "group_id": c.group_id,
        "reward_index": c.reward_index.unwrap_or(0),
        "can_check_in": c.can_check_in.is_some_and(|v| v != 0),
    })
}

fn extract_medals(medal: Option<&MedalStore>) -> Value {
    medal
        .and_then(|m| m.medals.as_ref())
        .into_iter()
        .flatten()
        .filter_map(|(id, data)| {
            let val = data.get("val").cloned().unwrap_or(Value::Null);
            let ts = |key: &str| data.get(key).and_then(Value::as_i64).unwrap_or(0);
            let (fts, rts) = (ts("fts"), ts("rts"));
            is_medal_earned(&val, fts, rts)
                .then(|| json!({ "medal_id": id, "val": val, "first_ts": fts, "reach_ts": rts }))
        })
        .collect()
}

#[must_use]
pub fn is_medal_earned(val: &Value, fts: i64, rts: i64) -> bool {
    if rts > 0 {
        return true;
    }
    rts == -1
        && fts > 0
        && val.as_array().is_some_and(|conditions| {
            conditions.iter().all(|c| {
                c.as_array()
                    .and_then(|pair| Some(pair.first()?.as_i64()? >= pair.get(1)?.as_i64()?))
                    .unwrap_or(false)
            })
        })
}

fn extract_supports(troop: Option<&Troop>, social: Option<&Social>) -> Value {
    let chars = troop.and_then(|t| t.chars.as_ref());
    social
        .and_then(|s| s.assist_char_list.as_ref())
        .into_iter()
        .flatten()
        .enumerate()
        .filter_map(|(slot, seat)| {
            let seat = seat.as_ref()?;
            let entry = chars?.get(&seat.char_inst_id?.to_string())?;
            let parsed = TroopChar::deserialize(entry).ok()?;
            Some(json!({
                "slot": slot,
                "operator_id": parsed.char_id?,
                "skin_id": parsed.skin.filter(|s| !s.is_empty()),
                "skill_index": seat.skill_index.unwrap_or(0),
                "current_equip": seat.current_equip,
            }))
        })
        .collect()
}

#[must_use]
pub fn extract_game_squads(troop: Option<&Value>, modules: &Modules) -> Value {
    let Some(squads) = troop
        .and_then(|t| t.get("squads"))
        .and_then(Value::as_object)
    else {
        return json!([]);
    };
    let char_ids: HashMap<&str, &str> = troop
        .and_then(|t| t.get("chars"))
        .and_then(Value::as_object)
        .into_iter()
        .flatten()
        .filter_map(|(inst, entry)| {
            let id = entry.get("charId")?.as_str().filter(|id| !id.is_empty())?;
            Some((inst.as_str(), id))
        })
        .collect();
    let initial = |id: &str| {
        modules
            .equip_dict
            .get(id)
            .is_some_and(|m| m.module_type == ModuleType::Initial)
    };
    squads
        .values()
        .enumerate()
        .filter_map(|(position, raw)| {
            let squad = TroopSquad::deserialize(raw).ok()?;
            let index = squad
                .squad_id
                .as_deref()
                .and_then(|id| id.parse::<i16>().ok())
                .or_else(|| i16::try_from(position).ok())?;
            let mut slots: Vec<Value> = squad
                .slots
                .unwrap_or_default()
                .into_iter()
                .take(SQUAD_SIZE)
                .map(|seat| {
                    let Some(seat) = seat else {
                        return Value::Null;
                    };
                    let Some(char_id) = seat
                        .char_inst_id
                        .and_then(|inst| char_ids.get(inst.to_string().as_str()).copied())
                    else {
                        return Value::Null;
                    };
                    let module_id = seat.current_equip.filter(|e| !e.is_empty() && !initial(e));
                    json!({
                        "char_id": char_id,
                        "skill_index": seat.skill_index.unwrap_or(0),
                        "module_id": module_id,
                    })
                })
                .collect();
            slots.resize(SQUAD_SIZE, Value::Null);
            Some(json!({
                "index": index,
                "name": squad.name.unwrap_or_default(),
                "slots": slots,
            }))
        })
        .collect()
}

pub fn merge_campaign_clears(stages: &mut Value, raw: &Value, campaign: &CampaignRotations) {
    let Some(instances) = raw
        .pointer("/user/campaignsV2/instances")
        .and_then(Value::as_object)
        .filter(|i| !i.is_empty())
    else {
        return;
    };
    if !stages.is_object() {
        *stages = json!({});
    }
    let Some(map) = stages.as_object_mut() else {
        return;
    };
    for (stage_id, instance) in instances {
        if map.contains_key(stage_id) {
            continue;
        }
        let kills = instance
            .get("maxKills")
            .and_then(Value::as_i64)
            .unwrap_or(0);
        if kills <= 0 {
            continue;
        }
        let full = campaign
            .kill_max(stage_id)
            .is_some_and(|max| kills >= i64::from(max));
        map.insert(
            stage_id.clone(),
            json!({
                "stageId": stage_id,
                "state": if full { 3 } else { 2 },
                "completeTimes": 1,
                "practiceTimes": 0,
                "campaignMaxKills": kills,
            }),
        );
    }
}

pub fn merge_cow_level(stages: &mut Value, raw: &Value) {
    let Some(cow) = raw
        .pointer("/user/dungeon/cowLevel")
        .and_then(Value::as_object)
    else {
        return;
    };
    let Some(map) = stages.as_object_mut() else {
        return;
    };
    for (id, entry) in cow {
        let Some(fts) = entry.get("fts").and_then(Value::as_i64).filter(|t| *t > 0) else {
            continue;
        };
        if let Some(record) = map.get_mut(id).and_then(Value::as_object_mut) {
            record.insert("cowFirstTs".to_owned(), Value::from(fts));
        }
    }
}

fn merge_inferred_clears(stages: &mut Value, raw: &Value, evidence: &StageEvidenceIndex) {
    let Some(user) = raw.get("user") else {
        return;
    };
    let bounds = evidence.infer(user);
    let folded = fold_into_records(stages, &bounds);
    tracing::info!(
        bounded = bounds.len(),
        folded,
        "sync: inferred clears folded into stage records"
    );
}

fn extract_roguelike(rlv2: Option<&Value>) -> Value {
    rlv2.and_then(|v| v.get("outer"))
        .and_then(Value::as_object)
        .into_iter()
        .flatten()
        .map(|(theme_id, progress)| json!({ "theme_id": theme_id, "progress": progress }))
        .collect()
}
