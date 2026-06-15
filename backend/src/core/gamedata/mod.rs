use std::path::Path;

use crate::core::gamedata::{
    assets::AssetIndex,
    enrich::{
        audio::build_operator_audio,
        chibi::{init_chibi_data, init_enemy_chibi_data},
        enemy_stages::build_enemy_stage_index,
        gacha::enrich_banners,
        modules::enrich_modules_global,
        operators::{EnrichCtx, enrich_all_operators, extract_all_drones},
        skills::enrich_all_skills,
        skins::enrich_all_skins,
        voice::enrich_all_voices,
    },
    tables::{DataError, load_table, load_table_or_warn},
    types::{
        GameData,
        activity::ActivityTableFile,
        audio::RawAudioData,
        building::BuildingDataFile,
        campaign::{CampaignRotations, CampaignTableFile},
        climb_tower::ClimbTowerTableFile,
        consts::GameDataConst,
        enemy::{EnemyDatabaseFile, EnemyHandbook, EnemyHandbookTableFile},
        gacha::GachaTableFile,
        handbook::HandbookTableFile,
        material::ItemTableFile,
        medal::{MedalData, MedalTableFile},
        module::{BattleEquipTableFile, UniequipTableFile},
        operator::{CharPatchTable, CharacterTable},
        range::Ranges,
        retro::RetroTableFile,
        roguelike::{RoguelikeGameData, RoguelikeTopicTableFile},
        sandbox_universe::SandboxUniverse,
        skill::SkillTableFile,
        skin::SkinTableFile,
        stage::StageTableFile,
        stage_universe::StageUniverse,
        trust::Favor,
        voice::{Voices, VoicesTableFile},
        zone::ZoneTableFile,
    },
};

pub mod assets;
pub mod enrich;
pub mod profile;
pub mod tables;
pub mod types;

pub fn init_game_data(data_dir: &Path, assets_dir: &Path) -> Result<GameData, DataError> {
    let mut warnings: Vec<String> = Vec::new();

    let assets = AssetIndex::build(assets_dir);

    let char_table: CharacterTable = load_table(data_dir, "character_table")?;
    let mut raw_operators = char_table.characters;

    // Merge Amiya's branch forms (Guard `char_1001_amiya2`, Medic
    // `char_1037_amiya3`) from char_patch_table - Hypergryph stores them
    // separately. Without this, those ids 404 everywhere and grade
    // calculations silently skip them.
    let char_patch: CharPatchTable =
        load_table_or_warn(data_dir, "char_patch_table", &mut warnings);
    for (id, op) in char_patch.patch_chars {
        raw_operators.entry(id).or_insert(op);
    }
    // Re-key `Infos` by every tmpl_id so a lookup with any of Amiya's three
    // form ids resolves to the same group metadata.
    let mut tmpl_groups: std::collections::HashMap<String, _> = std::collections::HashMap::new();
    for info in char_patch.infos.into_values() {
        for id in &info.tmpl_ids {
            tmpl_groups.insert(id.clone(), info.clone());
        }
    }

    let skill_file: SkillTableFile = load_table_or_warn(data_dir, "skill_table", &mut warnings);
    let equip_file: UniequipTableFile =
        load_table_or_warn(data_dir, "uniequip_table", &mut warnings);
    let battle_equip_file: BattleEquipTableFile =
        load_table_or_warn(data_dir, "battle_equip_table", &mut warnings);
    let handbook_file: HandbookTableFile =
        load_table_or_warn(data_dir, "handbook_info_table", &mut warnings);
    let skin_file: SkinTableFile = load_table_or_warn(data_dir, "skin_table", &mut warnings);
    let item_file: ItemTableFile = load_table_or_warn(data_dir, "item_table", &mut warnings);
    let favor: Favor = load_table_or_warn(data_dir, "favor_table", &mut warnings);
    let ranges: Ranges = load_table_or_warn(data_dir, "range_table", &mut warnings);
    let gacha_file: GachaTableFile = load_table_or_warn(data_dir, "gacha_table", &mut warnings);
    let zone_file: ZoneTableFile = load_table_or_warn(data_dir, "zone_table", &mut warnings);
    let stage_file: StageTableFile = load_table_or_warn(data_dir, "stage_table", &mut warnings);
    let medal_file: MedalTableFile = load_table_or_warn(data_dir, "medal_table", &mut warnings);
    let campaign_file: CampaignTableFile =
        load_table_or_warn(data_dir, "campaign_table", &mut warnings);
    let climb_tower_file: ClimbTowerTableFile =
        load_table_or_warn(data_dir, "climb_tower_table", &mut warnings);
    let voice_file: VoicesTableFile = load_table_or_warn(data_dir, "charword_table", &mut warnings);
    let audio_file: RawAudioData = load_table_or_warn(data_dir, "audio_data", &mut warnings);
    let enemy_file: EnemyHandbookTableFile =
        load_table_or_warn(data_dir, "enemy_handbook_table", &mut warnings);
    let building_file: BuildingDataFile =
        load_table_or_warn(data_dir, "building_data", &mut warnings);
    let roguelike_file: RoguelikeTopicTableFile =
        load_table_or_warn(data_dir, "roguelike_topic_table", &mut warnings);
    let activity_file: ActivityTableFile =
        load_table_or_warn(data_dir, "activity_table", &mut warnings);
    let retro_file: RetroTableFile = load_table_or_warn(data_dir, "retro_table", &mut warnings);
    let consts: GameDataConst = load_table_or_warn(data_dir, "gamedata_const", &mut warnings);

    let materials = item_file.into_materials();
    let raw_modules = equip_file.into_raw_modules();
    let battle_equip = battle_equip_file.into_battle_equip();
    let handbook = handbook_file.into_handbook();
    let skins = skin_file.into_skin_data();
    let mut gacha = gacha_file.into_gacha_data();
    enrich_banners(&mut gacha.gacha_pool_client);
    let zones = zone_file.zones;
    let stages = stage_file.stages;
    let mut medals = MedalData::from_table(medal_file);
    let roguelike = RoguelikeGameData::from_table(&roguelike_file);
    let campaign_rotations = CampaignRotations::from_table(campaign_file);
    let retro_linked_acts: std::collections::HashSet<String> = retro_file
        .retro_act_list
        .values()
        .flat_map(|r| r.linked_act_id.iter().cloned())
        .collect();
    let stage_universe = StageUniverse::build(
        &stages,
        &zones,
        &activity_file.basic_info,
        &campaign_rotations,
        &retro_linked_acts,
    );

    let sandbox_perm_raw: serde_json::Value =
        load_table_or_warn(data_dir, "sandbox_perm_table", &mut warnings);
    let sandbox_universe = SandboxUniverse::build(&sandbox_perm_raw);

    let skills = enrich_all_skills(skill_file.skills, &assets);
    let drones = extract_all_drones(&raw_operators);
    let mut skins = skins;
    skins.enriched_skins = enrich_all_skins(&skins.char_skins, &assets);
    let modules = enrich_modules_global(&raw_modules, &battle_equip, &materials, &assets);

    // Voice enrichment
    let enriched_char_words =
        enrich_all_voices(&voice_file.char_words, &voice_file.voice_lang_dict);
    let voices = Voices {
        char_words: enriched_char_words,
        char_extra_words: voice_file.char_extra_words,
        voice_lang_dict: voice_file.voice_lang_dict,
        default_lang_type: voice_file.default_lang_type,
        new_tag_list: voice_file.new_tag_list,
        ..Default::default()
    };

    // Map battle SoundFX banks (deploy/attack/skill sounds, voice barks) to
    // operators by char id, resolving each asset to a playable URL.
    let op_ids: std::collections::HashSet<&str> =
        raw_operators.keys().map(String::as_str).collect();
    let operator_audio = build_operator_audio(&audio_file, &op_ids, &assets);

    let operators = enrich_all_operators(
        &raw_operators,
        &EnrichCtx {
            skills: &skills,
            modules: &raw_modules,
            battle_equip: &battle_equip,
            handbook: &handbook,
            skins: &skins,
            materials: &materials,
            assets: &assets,
            drones: &drones,
            building: &building_file,
            tmpl_groups: &tmpl_groups,
            audio: &operator_audio,
            consts: &consts,
        },
    );

    // Enemy database lives outside excel/, in the levels directory
    let enemy_db_path = assets_dir.join("gamedata/levels/enemydata/enemy_database.json");
    let enemies = if let Ok(enemy_db) = std::fs::File::open(&enemy_db_path)
        .map_err(|e| e.to_string())
        .and_then(|f| {
            serde_json::from_reader::<_, EnemyDatabaseFile>(std::io::BufReader::new(f))
                .map_err(|e| e.to_string())
        }) {
        enrich::enemies::enrich_enemies(enemy_file, &enemy_db, &assets)
    } else {
        warnings.push("enemy_database: file not found or parse error".into());
        EnemyHandbook::from(enemy_file) // fallback without stats
    };

    medals.link_operator_locks(&operators);
    // Drive seasonal/event medal availability from the authoritative schedules
    // (SSS tower seasons + event activity windows) rather than the medal table's
    // own ExpireTimes, which are empty/placeholder for this content.
    medals.link_content_windows(&climb_tower_file.tower_windows(), &activity_file.basic_info);

    // Inverted enemy -> stages index, parsed from per-stage level files.
    let enemy_stage_index = build_enemy_stage_index(&assets_dir.join("gamedata/levels"), &stages);

    for w in &warnings {
        eprintln!("warning: {w}");
    }

    Ok(GameData {
        operators,
        skills,
        materials,
        modules,
        skins,
        handbook,
        ranges,
        favor,
        voices,
        gacha,
        chibis: init_chibi_data(assets_dir),
        enemy_chibis: init_enemy_chibi_data(assets_dir, &enemies),
        zones,
        stages,
        activities: activity_file.basic_info,
        retro_acts: retro_file.retro_act_list,
        medals,
        roguelike,
        enemies,
        enemy_stage_index,
        building: building_file,
        stage_universe,
        sandbox_universe,
        campaign_rotations,
        consts,
    })
}
