//! Real game data for the tests that read `GAME_DATA_DIR`.
//!
//! The DPS/HPS parity suites need the operators the old
//! `core::gamedata::init_game_data` built (`backend/src/core/gamedata/mod.rs:220-430`),
//! but `gamedata::compile` is not lifted yet. [`dps_operators`] rebuilds the
//! operator fields the DPS engine and the parity suites read, from the same five
//! excel tables, with the old code's joins copied from
//! `backend/src/core/gamedata/enrich/{operators,skills,modules}.rs`:
//! `id`, `rarity`, `profession`, `sub_profession_id`, `position`, `phases`,
//! `skills` (with `static`), `talents`, `potential_ranks`, `favor_key_frames`,
//! `modules` (with `data`), `drones`. It also copies every other field that the old
//! code took straight from `RawOperator`. Fields the old loader derived from
//! other tables or the art tree (handbook, profile, artists, base skills, audio,
//! portrait, skin, faction names, profession and sub-profession names, item icons,
//! module images, phase LMD/EXP costs) stay at their defaults. When
//! `gamedata::compile` lands, the suites switch to it and this module goes.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use myrtle_gamedata_raw::tables::module::{BattleEquip, Module, RawModule, RawModules};
use myrtle_gamedata_raw::tables::module::{BattleEquipTableFile, UniequipTableFile};
use myrtle_gamedata_raw::tables::obtain::ObtainChannel;
use myrtle_gamedata_raw::tables::operator::{
    CharPatchInfo, CharPatchTable, CharacterTable, Drone, EnrichedSkill, LevelUpCostCond,
    LevelUpCostItem, Operator, OperatorModule, OperatorSkillRef, RawOperator, SkillStatic,
};
use myrtle_gamedata_raw::tables::skill::{Skill, SkillTableFile};

/// `GAME_DATA_DIR`, or the repo's EN extract, whatever directory the tests run from.
#[must_use]
pub fn excel_dir() -> PathBuf {
    std::env::var("GAME_DATA_DIR")
        .unwrap_or_else(|_| {
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../../assets/output/en/gamedata/excel"
            )
            .into()
        })
        .into()
}

fn load<T: serde::de::DeserializeOwned>(dir: &Path, table: &str) -> Result<T, String> {
    let path = dir.join(format!("{table}.json"));
    let bytes = std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    serde_json::from_slice(&bytes).map_err(|e| format!("{table}: {e}"))
}

/// Like the old `load_table_or_warn` (`tables.rs:275`): a missing or broken
/// optional table is its default.
fn load_or_default<T: serde::de::DeserializeOwned + Default>(dir: &Path, table: &str) -> T {
    load(dir, table).unwrap_or_default()
}

/// `favor_table`, as the old loader read it (`mod.rs:258`, `load_table_or_warn`),
/// for the grade tests' trust dimension.
pub fn favor(excel_dir: &Path) -> Result<myrtle_gamedata_raw::tables::trust::Favor, String> {
    load(excel_dir, "favor_table")
}

/// Every `char_*` operator in `excel_dir`, keyed by id, as described in the
/// module doc. `character_table` is required, as in the old loader; the other
/// four tables fall back to empty.
pub fn dps_operators(excel_dir: &Path) -> Result<HashMap<String, Operator>, String> {
    // mod.rs:229-247: Amiya's branch forms come from char_patch_table.
    let char_table: CharacterTable = load(excel_dir, "character_table")?;
    let mut raw_operators = char_table.characters;
    let char_patch: CharPatchTable = load_or_default(excel_dir, "char_patch_table");
    for (id, op) in char_patch.patch_chars {
        raw_operators.entry(id).or_insert(op);
    }
    let mut tmpl_groups: HashMap<String, CharPatchInfo> = HashMap::new();
    for info in char_patch.infos.into_values() {
        for id in &info.tmpl_ids {
            tmpl_groups.insert(id.clone(), info.clone());
        }
    }

    let skill_file: SkillTableFile = load_or_default(excel_dir, "skill_table");
    let equip_file: UniequipTableFile = load_or_default(excel_dir, "uniequip_table");
    let battle_equip_file: BattleEquipTableFile = load_or_default(excel_dir, "battle_equip_table");
    let raw_modules = equip_file.into_raw_modules();
    let battle_equip = battle_equip_file.into_battle_equip();

    // enrich/skills.rs:16-39 enrich_all_skills, without the icon path.
    let skills: HashMap<String, Skill> = skill_file
        .skills
        .into_iter()
        .map(|(id, raw)| {
            let skill = Skill {
                id: Some(id.clone()),
                skill_id: raw.skill_id,
                icon_id: raw.icon_id,
                image: None,
                hidden: raw.hidden,
                levels: raw.levels,
            };
            (id, skill)
        })
        .collect();
    let drones = extract_all_drones(&raw_operators);

    // enrich/operators.rs:53-65 enrich_all_operators.
    Ok(raw_operators
        .iter()
        .filter(|(id, _)| id.starts_with("char_"))
        .map(|(id, raw)| {
            let enriched = enrich_operator(
                id,
                raw,
                &skills,
                &raw_modules,
                &battle_equip,
                &drones,
                &tmpl_groups,
            );
            (id.clone(), enriched)
        })
        .collect())
}

/// enrich/operators.rs:67-110, verbatim but for `par_iter` -> `iter`.
fn extract_all_drones(raw_operators: &HashMap<String, RawOperator>) -> HashMap<String, Drone> {
    raw_operators
        .iter()
        .filter(|(id, _)| !id.starts_with("char_"))
        .map(|(id, raw)| {
            let drone = Drone {
                id: Some(id.clone()),
                name: raw.name.clone(),
                description: raw.description.clone().unwrap_or_default(),
                can_use_general_potential_item: raw.can_use_general_potential_item,
                can_use_activity_potential_item: raw.can_use_activity_potential_item,
                potential_item_id: raw.potential_item_id.clone(),
                activity_potential_item_id: raw.activity_potential_item_id.clone(),
                classic_potential_item_id: raw.classic_potential_item_id.clone(),
                nation_id: raw.nation_id.clone(),
                group_id: raw.group_id.clone(),
                team_id: raw.team_id.clone(),
                display_number: raw.display_number.clone(),
                appellation: raw.appellation.clone(),
                position: raw.position.clone(),
                tag_list: raw.tag_list.clone().unwrap_or_default(),
                item_usage: raw.item_usage.clone(),
                item_desc: raw.item_desc.clone(),
                item_obtain_approach: raw.item_obtain_approach.clone(),
                is_not_obtainable: raw.is_not_obtainable,
                is_sp_char: raw.is_sp_char,
                max_potential_level: raw.max_potential_level,
                rarity: raw.rarity.clone(),
                profession: raw.profession.to_raw_str().to_owned(),
                sub_profession_id: raw.sub_profession_id.clone().unwrap_or_default(),
                trait_data: raw.trait_data.clone(),
                phases: raw.phases.clone(),
                skills: raw.skills.clone(),
                display_token_dict: raw.display_token_dict.clone(),
                talents: raw.talents.clone().unwrap_or_default(),
                potential_ranks: raw.potential_ranks.clone(),
                favor_key_frames: None,
                all_skill_lvlup: raw.all_skill_lvlup.clone(),
                modules: Vec::new(),
            };
            (id.clone(), drone)
        })
        .collect()
}

/// enrich/operators.rs:112-212 `enrich_operator`, the fields copied from the raw
/// row plus skills, modules and drones; see the module doc for what is left out.
fn enrich_operator(
    id: &str,
    raw: &RawOperator,
    skills: &HashMap<String, Skill>,
    modules: &RawModules,
    battle_equip: &BattleEquip,
    drones: &HashMap<String, Drone>,
    tmpl_groups: &HashMap<String, CharPatchInfo>,
) -> Operator {
    let operator_drones = resolve_drones(id, raw, drones);
    let enriched_skills = enrich_operator_skills(&raw.skills, skills);
    let operator_modules = get_operator_modules(id, modules, battle_equip);
    let (tmpl_ids, tmpl_default) = match tmpl_groups.get(id) {
        Some(info) => (Some(info.tmpl_ids.clone()), Some(info.default.clone())),
        None => (None, None),
    };

    Operator {
        id: Some(id.to_owned()),
        name: raw.name.clone(),
        description: raw.description.clone().unwrap_or_default(),
        can_use_general_potential_item: raw.can_use_general_potential_item,
        can_use_activity_potential_item: raw.can_use_activity_potential_item,
        potential_item_id: raw.potential_item_id.clone().unwrap_or_default(),
        activity_potential_item_id: raw.activity_potential_item_id.clone(),
        classic_potential_item_id: raw.classic_potential_item_id.clone(),
        nation_id: raw.nation_id.clone().unwrap_or_default(),
        main_power: raw.main_power.clone(),
        sub_power: raw.sub_power.clone(),
        group_id: raw.group_id.clone(),
        team_id: raw.team_id.clone(),
        display_number: raw.display_number.clone().unwrap_or_default(),
        appellation: raw.appellation.clone(),
        position: raw.position.clone(),
        tag_list: raw.tag_list.clone().unwrap_or_default(),
        item_usage: raw.item_usage.clone().unwrap_or_default(),
        item_desc: raw.item_desc.clone().unwrap_or_default(),
        item_obtain_approach: raw.item_obtain_approach.clone().unwrap_or_default(),
        obtain_channel: raw
            .item_obtain_approach
            .as_deref()
            .and_then(ObtainChannel::classify),
        is_not_obtainable: raw.is_not_obtainable,
        is_sp_char: raw.is_sp_char,
        max_potential_level: raw.max_potential_level,
        rarity: raw.rarity.clone(),
        profession: raw.profession.clone(),
        sub_profession_id: raw.sub_profession_id.clone().unwrap_or_default(),
        trait_data: raw.trait_data.clone(),
        phases: raw.phases.clone(),
        skills: enriched_skills,
        display_token_dict: raw.display_token_dict.clone(),
        drones: operator_drones,
        talents: raw.talents.clone().unwrap_or_default(),
        potential_ranks: raw.potential_ranks.clone(),
        favor_key_frames: raw.favor_key_frames.clone().unwrap_or_default(),
        all_skill_level_up: raw.all_skill_lvlup.clone(),
        modules: operator_modules,
        tmpl_ids,
        tmpl_default,
        ..Operator::default()
    }
}

/// enrich/operators.rs:256-294, verbatim.
fn resolve_drones(id: &str, raw: &RawOperator, drones: &HashMap<String, Drone>) -> Vec<Drone> {
    let mut result: Vec<Drone> = Vec::new();

    if let Some(dict) = &raw.display_token_dict {
        for key in dict.keys() {
            if let Some(drone) = drones.get(key) {
                result.push(drone.clone());
            }
        }
    }

    for skill in &raw.skills {
        if let Some(ref token_key) = skill.override_token_key
            && let Some(drone) = drones.get(token_key)
            && !result.iter().any(|d| d.id == drone.id)
        {
            result.push(drone.clone());
        }
    }

    if result.is_empty()
        && let Some(name_part) = id.split('_').nth(2)
    {
        let pattern = format!("_{name_part}_");
        result = drones
            .iter()
            .filter(|(key, _)| key.starts_with("token_") && key.contains(&pattern))
            .map(|(_, drone)| drone.clone())
            .collect();
    }

    result.sort_by(|a, b| {
        let a_id = a.id.as_deref().unwrap_or("");
        let b_id = b.id.as_deref().unwrap_or("");
        a_id.cmp(b_id)
    });

    result
}

/// enrich/skills.rs:41-73 `enrich_operator_skills`; mastery cost items keep id,
/// count and type, without the icon lookup.
fn enrich_operator_skills(
    skill_refs: &[OperatorSkillRef],
    skill_table: &HashMap<String, Skill>,
) -> Vec<EnrichedSkill> {
    skill_refs
        .iter()
        .filter_map(|skill_ref| {
            let skill_id = skill_ref.skill_id.as_ref()?;

            let static_data = skill_table.get(skill_id).map(|skill| SkillStatic {
                levels: skill.levels.clone(),
                skill_id: skill.skill_id.clone(),
                icon_id: skill.icon_id.clone(),
                hidden: skill.hidden,
                image: skill.image.clone(),
            });

            let level_up_cost_cond = skill_ref
                .level_up_cost_cond
                .iter()
                .map(|cond| LevelUpCostCond {
                    unlock_cond: cond.unlock_cond.clone(),
                    lvl_up_time: cond.lvl_up_time,
                    level_up_cost: cond
                        .level_up_cost
                        .iter()
                        .map(|cost| LevelUpCostItem {
                            id: cost.id.clone(),
                            count: cost.count,
                            item_type: cost.item_type.clone(),
                            icon_id: None,
                            image: None,
                        })
                        .collect(),
                })
                .collect();

            Some(EnrichedSkill {
                skill_id: skill_id.clone(),
                override_prefab_key: skill_ref.override_prefab_key.clone(),
                override_token_key: skill_ref.override_token_key.clone(),
                level_up_cost_cond,
                unlock_cond: skill_ref.unlock_cond.clone(),
                static_data,
            })
        })
        .collect()
}

/// enrich/modules.rs:16-59 `get_operator_modules`, verbatim but for the
/// materials and asset parameters.
fn get_operator_modules(
    char_id: &str,
    modules: &RawModules,
    battle_equip: &BattleEquip,
) -> Vec<OperatorModule> {
    let mut out: Vec<OperatorModule> = modules
        .equip_dict
        .values()
        .filter(|m| match &m.tmpl_id {
            Some(tmpl) => tmpl == char_id,
            None => m.char_id == char_id,
        })
        .map(|raw| {
            let data = battle_equip
                .get(&raw.uni_equip_id)
                .cloned()
                .unwrap_or_default();

            let module = enrich_module(raw);
            OperatorModule { module, data }
        })
        .collect();

    out.sort_by(|a, b| {
        uniequip_number(&a.module.uni_equip_id)
            .cmp(&uniequip_number(&b.module.uni_equip_id))
            .then_with(|| a.module.uni_equip_id.cmp(&b.module.uni_equip_id))
    });
    out
}

/// enrich/modules.rs:63-69, verbatim.
fn uniequip_number(uni_equip_id: &str) -> i32 {
    uni_equip_id
        .split('_')
        .nth(1)
        .and_then(|n| n.parse::<i32>().ok())
        .unwrap_or(i32::MAX)
}

/// enrich/modules.rs:93-120 `enrich_module`, without the image and the item
/// cost icons (`item_cost` stays `None`; no DPS path reads it).
fn enrich_module(raw: &RawModule) -> Module {
    Module {
        id: Some(raw.uni_equip_id.clone()),
        uni_equip_id: raw.uni_equip_id.clone(),
        uni_equip_name: raw.uni_equip_name.clone(),
        uni_equip_icon: raw.uni_equip_icon.clone(),
        image: None,
        uni_equip_desc: raw.uni_equip_desc.clone(),
        type_icon: raw.type_icon.clone(),
        type_name1: raw.type_name1.clone(),
        type_name2: raw.type_name2.clone(),
        equip_shining_color: raw.equip_shining_color.clone(),
        show_evolve_phase: raw.show_evolve_phase.clone(),
        unlock_evolve_phase: raw.unlock_evolve_phase.clone(),
        char_id: raw.char_id.clone(),
        tmpl_id: raw.tmpl_id.clone(),
        show_level: raw.show_level,
        unlock_level: raw.unlock_level,
        unlock_favor_point: raw.unlock_favor_point,
        mission_list: raw.mission_list.clone(),
        item_cost: None,
        module_type: raw.module_type.clone(),
        uni_equip_get_time: raw.uni_equip_get_time,
        char_equip_order: raw.char_equip_order,
    }
}

// ---- Lifted for engine-base (lift-4, Oct 25 2026). The base engine reads one
// derived operator field `dps_operators` leaves at its default: `profile`
// (registry/tags.rs faction_tags_of, `basic_info.race`). [`base_tables`] adds it
// with the old join (enrich/operators.rs:122-127, enrich/handbook.rs:6-17) and
// the old parser copied verbatim from enrich/profile.rs:10-19, 116-125, 152-302
// (`align_profiles` and `relabel`, the KR/JP reference pass, are not needed for
// EN and are not copied). `building.layout_terms` is derived from the consts
// table the way `gamedata::compile` derives it.

use myrtle_gamedata_raw::tables::building::BuildingDataFile;
use myrtle_gamedata_raw::tables::consts::GameDataConst;
use myrtle_gamedata_raw::tables::handbook::{
    BasicInfo, HandbookStoryTextAudio, HandbookTableFile, OperatorBirthPlace, OperatorGender,
    OperatorProfile, OperatorRace, PhysicalExam,
};

/// The three tables the base engine reads (`operators`, `building_data`,
/// `gamedata_const`), as the old loader built them (`core/gamedata/mod.rs:254-255,
/// 274-275, 316, 333`).
pub fn base_tables(
    excel_dir: &Path,
) -> Result<(HashMap<String, Operator>, BuildingDataFile, GameDataConst), String> {
    let mut operators = dps_operators(excel_dir)?;
    let handbook =
        load_or_default::<HandbookTableFile>(excel_dir, "handbook_info_table").into_handbook();
    let char_patch: CharPatchTable = load_or_default(excel_dir, "char_patch_table");
    let mut tmpl_default: HashMap<String, String> = HashMap::new();
    for info in char_patch.infos.into_values() {
        for id in &info.tmpl_ids {
            tmpl_default.insert(id.clone(), info.default.clone());
        }
    }
    for (id, op) in &mut operators {
        // Amiya's forms share handbook/profile lore: fall back to the template default id.
        let handbook_id = tmpl_default.get(id).map_or(id.as_str(), String::as_str);
        op.profile = handbook
            .handbook_dict
            .get(handbook_id)
            .and_then(|item| parse_operator_profile(&item.story_text_audio));
    }
    let mut building: BuildingDataFile = load_or_default(excel_dir, "building_data");
    let consts: GameDataConst = load_or_default(excel_dir, "gamedata_const");
    building.layout_terms = myrtle_gamedata::index::base_terms::layout_term_rooms(&consts);
    Ok((operators, building, consts))
}

#[must_use]
pub fn parse_operator_profile(
    story_text_audio: &[HandbookStoryTextAudio],
) -> Option<OperatorProfile> {
    let basic_text = section_text(story_text_audio, 0)?;
    let physical_text = section_text(story_text_audio, 1)?;
    Some(OperatorProfile {
        basic_info: parse_basic_info(basic_text),
        physical_exam: parse_physical_exam(physical_text),
    })
}

fn section_text(story_text_audio: &[HandbookStoryTextAudio], section: usize) -> Option<&str> {
    Some(
        story_text_audio
            .get(section)?
            .stories
            .first()?
            .story_text
            .as_str(),
    )
}

fn parse_basic_info(text: &str) -> BasicInfo {
    let mut info = BasicInfo::default();

    for line in text.lines() {
        let Some((key, value)) = parse_bracketed(line) else {
            continue;
        };
        match key {
            "Code Name" | "Codename" | "代号" => info.code_name = value.to_owned(),
            "Gender" | "性别" => info.gender = parse_gender(value),
            "Combat Experience" | "战斗经验" => info.combat_experience = value.to_owned(),
            "Place of Birth" | "出身地" => info.place_of_birth = parse_birthplace(value),
            "Date of Birth" | "生日" => info.date_of_birth = value.to_owned(),
            "Race" | "种族" => info.race = parse_race(value),
            "Height" | "身高" => info.height = value.to_owned(),
            "Infection Status" | "矿石病感染情况" => {
                info.infection_status = value.to_owned();
            }
            _ => {}
        }
    }
    info
}

fn parse_physical_exam(text: &str) -> PhysicalExam {
    let mut exam = PhysicalExam::default();

    for line in text.lines() {
        let Some((key, value)) = parse_bracketed(line) else {
            continue;
        };
        match key {
            "Physical Strength" | "物理强度" => exam.physical_strength = value.to_owned(),
            "Mobility" | "战场机动" => exam.mobility = value.to_owned(),
            "Physical Resilience" | "生理耐受" => exam.physical_resilience = value.to_owned(),
            "Tactical Acumen" | "战术规划" => exam.tactical_acumen = value.to_owned(),
            "Combat Skill" | "Combat Skills" | "战斗技巧" => {
                exam.combat_skill = value.to_owned();
            }
            // Both misspellings are EN's own, on 3 operators (2026-10-06).
            "Originium Arts Assimilation"
            | "Originium Arts Assilimation"
            | "Originium ArtsAssimilation"
            | "源石技艺适应性" => {
                exam.originium_arts_assimilation = value.to_owned();
            }
            _ => {}
        }
    }
    exam
}

fn parse_bracketed(line: &str) -> Option<(&str, &str)> {
    let line = line.trim();
    if line.starts_with('[') {
        let close = line.find(']')?;
        Some((&line[1..close], line[close + 1..].trim()))
    } else if line.starts_with('【') {
        let close = line.find('】')?;
        let open_len = '【'.len_utf8();
        let close_len = '】'.len_utf8();
        Some((&line[open_len..close], line[close + close_len..].trim()))
    } else {
        None
    }
}

fn parse_gender(s: &str) -> OperatorGender {
    match s {
        "Female" | "女" => OperatorGender::Female,
        "Male" | "男" => OperatorGender::Male,
        "Male]" | "男]" => OperatorGender::MaleBugged,
        "Conviction" | "断罪" => OperatorGender::Conviction,
        _ => OperatorGender::Unknown,
    }
}

fn parse_birthplace(s: &str) -> OperatorBirthPlace {
    match s {
        "未公开" | "Undisclosed" => OperatorBirthPlace::Undisclosed,
        "东国" | "東国" | "Higashi" => OperatorBirthPlace::Higashi,
        "卡西米尔" | "Kazimierz" => OperatorBirthPlace::Kazimierz,
        "维多利亚" | "Victoria" => OperatorBirthPlace::Victoria,
        "雷姆必拓" | "Rim Billiton" => OperatorBirthPlace::RimBilliton,
        "莱塔尼亚" | "Leithanien" => OperatorBirthPlace::Leithanien,
        "玻利瓦尔" | "Bolívar" | "Bolivar" => OperatorBirthPlace::Bolivar,
        "萨尔贡" | "Sargon" => OperatorBirthPlace::Sargon,
        "谢拉格" | "Kjerag" => OperatorBirthPlace::Kjerag,
        "哥伦比亚" | "Columbia" => OperatorBirthPlace::Columbia,
        "萨米" | "Sami" => OperatorBirthPlace::Sami,
        "伊比利亚" | "Iberia" => OperatorBirthPlace::Iberia,
        "卡兹戴尔" | "Kazdel" => OperatorBirthPlace::Kazdel,
        "米诺斯" | "Minos" => OperatorBirthPlace::Minos,
        "龙门" | "Lungmen" => OperatorBirthPlace::Lungmen,
        "叙拉古" | "Siracusa" => OperatorBirthPlace::Siracusa,
        "炎国" | "炎" | "Yan" => OperatorBirthPlace::Yan,
        "乌萨斯" | "Ursus" => OperatorBirthPlace::Ursus,
        "汐斯塔" | "Siesta" => OperatorBirthPlace::Siesta,
        "阿戈尔" | "Aegir" | "Ægir" => OperatorBirthPlace::Aegir,
        "杜林" | "Durin" => OperatorBirthPlace::Durin,
        "拉特兰" | "Laterano" => OperatorBirthPlace::Laterano,
        "沃尔珀" | "Vouivre" => OperatorBirthPlace::Vouivre,
        "罗德岛" | "Rhodes Island" => OperatorBirthPlace::RhodesIsland,
        "远东" | "Far East" => OperatorBirthPlace::FarEast,
        _ => OperatorBirthPlace::Unknown,
    }
}

fn parse_race(s: &str) -> OperatorRace {
    match s {
        "未公开" | "Undisclosed" => OperatorRace::Undisclosed,
        "札拉克" | "Zalak" => OperatorRace::Zalak,
        "鬼" | "Oni" => OperatorRace::Oni,
        "萨弗拉" | "Savra" => OperatorRace::Savra,
        "杜林" | "Durin" => OperatorRace::Durin,
        "库兰塔" | "Kuranta" => OperatorRace::Kuranta,
        "沃尔珀" | "Vouivre" => OperatorRace::Vouivre,
        "黎博利" | "Liberi" => OperatorRace::Liberi,
        "菲林" | "Feline" => OperatorRace::Feline,
        "卡特斯" | "Cautus" => OperatorRace::Cautus,
        "佩洛" | "Perro" => OperatorRace::Perro,
        "雷普罗巴" | "Reproba" => OperatorRace::Reproba,
        "萨科塔" | "Sankta" => OperatorRace::Sankta,
        "萨卡兹" | "Sarkaz" => OperatorRace::Sarkaz,
        "瓦伊凡" | "Vulpo" => OperatorRace::Vulpo,
        "依拉菲亚" | "Elafia" => OperatorRace::Elafia,
        "斐迪亚" | "Phidia" => OperatorRace::Phidia,
        "阿戈尔" | "Aegir" | "Ægir" => OperatorRace::Aegir,
        "阿纳缇" | "Anaty" => OperatorRace::Anaty,
        "依特拉" | "Itra" => OperatorRace::Itra,
        "古龙" | "Archosauria" => OperatorRace::Archosauria,
        "鲁珀" | "Lupo" => OperatorRace::Lupo,
        "菲亚特" | "Forte" => OperatorRace::Forte,
        "乌萨斯" | "Ursus" => OperatorRace::Ursus,
        "佩特拉姆" | "Petram" => OperatorRace::Petram,
        "角峰" | "Cerato" => OperatorRace::Cerato,
        "卡普里尼" | "Caprinae" => OperatorRace::Caprinae,
        "德拉克" | "Draco" => OperatorRace::Draco,
        "阿努拉" | "Anura" => OperatorRace::Anura,
        "阿纳萨" | "Anasa" => OperatorRace::Anasa,
        "卡特斯/奇美拉" | "Cautus/Chimera" => OperatorRace::CautusChimera,
        "麒麟" | "Kylin" => OperatorRace::Kylin,
        "披毛" | "Pilosa" => OperatorRace::Pilosa,
        "曼提柯" | "Manticore" => OperatorRace::Manticore,
        "龙" | "Lung" => OperatorRace::Lung,
        "阿斯兰" | "Aslan" => OperatorRace::Aslan,
        "精灵" | "Elf" => OperatorRace::Elf,
        _ => OperatorRace::Unknown,
    }
}
