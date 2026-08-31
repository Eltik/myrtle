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
        stage_class::StageClassifier,
        stage_index::build_stage_index,
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
        gacha_detail::{POOL_DETAIL_FILE_VERSION, PoolDetailFile, pool_detail_path},
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
use crate::core::startup;

pub mod assets;
pub mod enrich;
pub mod profile;
pub mod tables;
pub mod types;

/// Load the optional `gacha/getPoolDetail` sidecar written by
/// [`super::gacha_detail_job`].
///
/// Absence is the normal case, not an error: a deployment with no game service
/// account never writes one, and banners then carry only their blob-derived
/// rate-ups. A *corrupt* or version-mismatched file is a warning and is
/// discarded - gamedata load must never fail because a cache went bad.
fn load_pool_details(assets_dir: &Path, warnings: &mut Vec<String>) -> Option<PoolDetailFile> {
    let path = pool_detail_path(assets_dir);
    if !path.exists() {
        return None;
    }

    let parsed = std::fs::read(&path)
        .map_err(|e| e.to_string())
        .and_then(|bytes| {
            serde_json::from_slice::<PoolDetailFile>(&bytes).map_err(|e| e.to_string())
        });

    match parsed {
        Ok(file) if file.version == POOL_DETAIL_FILE_VERSION => Some(file),
        Ok(file) => {
            warnings.push(format!(
                "gacha_pool_details: version {} != expected {POOL_DETAIL_FILE_VERSION}, ignoring",
                file.version
            ));
            None
        }
        Err(e) => {
            warnings.push(format!("gacha_pool_details: {e}"));
            None
        }
    }
}

/// The excel tables [`init_game_data`] reads, in the order it reads them. Used
/// only to build the boot plan; the loads name their own table.
const BOOT_TABLES: &[&str] = &[
    "character_table",
    "char_patch_table",
    "skill_table",
    "uniequip_table",
    "battle_equip_table",
    "handbook_info_table",
    "skin_table",
    "item_table",
    "favor_table",
    "range_table",
    "gacha_table",
    "zone_table",
    "stage_table",
    "medal_table",
    "campaign_table",
    "climb_tower_table",
    "charword_table",
    "audio_data",
    "enemy_handbook_table",
    "building_data",
    "roguelike_topic_table",
    "activity_table",
    "retro_table",
    "gamedata_const",
];

/// The steps [`AssetIndex::build`] reports, in order.
const ASSET_STEPS: [&str; 3] = ["textures", "portraits", "audio"];

/// The steps [`init_game_data`] reports, in order, for the boot progress plan.
///
/// Table steps carry a size hint from disk. The tables are wildly unequal - EN's
/// `activity_table` is 2.3 GB against under 20 MB for almost everything else -
/// so weighting them by step count would be badly off until they have been
/// measured.
pub fn boot_steps(data_dir: &Path) -> Vec<startup::StepSpec> {
    let mut steps = ASSET_STEPS.map(startup::StepSpec::new).to_vec();
    steps.extend(BOOT_TABLES.iter().map(|name| table_step(data_dir, name)));
    steps.push(startup::StepSpec::new("derive tables"));
    steps.push(table_step(data_dir, "sandbox_perm_table"));
    steps.extend(
        [
            "sandbox universe",
            "skills",
            "skins",
            "modules",
            "voices",
            "battle audio",
            "operators",
            "enemies",
            "stage classifier",
            "enemy index · scan",
            "enemy index · parse",
            "enemy index · bosses",
            "stage index · art",
            "stage index · stages",
            "stage index · modes",
            "chibis",
            "enemy chibis",
        ]
        .map(startup::StepSpec::new),
    );
    steps
}

fn table_step(data_dir: &Path, name: &'static str) -> startup::StepSpec {
    let step = startup::StepSpec::new(name);
    match startup::table_hint_ms(&data_dir.join(format!("{name}.json"))) {
        Some(ms) => step.hint(ms),
        None => step,
    }
}

pub fn init_game_data(
    data_dir: &Path,
    assets_dir: &Path,
) -> Result<(GameData, AssetIndex), DataError> {
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

    startup::step("derive tables");
    let materials = item_file.into_materials();
    let raw_modules = equip_file.into_raw_modules();
    let battle_equip = battle_equip_file.into_battle_equip();
    let handbook = handbook_file.into_handbook();
    let skins = skin_file.into_skin_data();
    let pool_details = load_pool_details(assets_dir, &mut warnings);
    let mut gacha = gacha_file.into_gacha_data();
    enrich_banners(&mut gacha.gacha_pool_client, pool_details.as_ref());
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
    startup::step("sandbox universe");
    let sandbox_universe = SandboxUniverse::build(&sandbox_perm_raw);

    startup::step("skills");
    let skills = enrich_all_skills(skill_file.skills, &assets);
    let drones = extract_all_drones(&raw_operators);
    startup::step("skins");
    let mut skins = skins;
    skins.enriched_skins = enrich_all_skins(&skins.char_skins, &assets);
    startup::step("modules");
    let modules = enrich_modules_global(&raw_modules, &battle_equip, &materials, &assets);

    // Voice enrichment
    startup::step("voices");
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
    startup::step("battle audio");
    let op_ids: std::collections::HashSet<&str> =
        raw_operators.keys().map(String::as_str).collect();
    let operator_audio = build_operator_audio(&audio_file, &op_ids, &assets);

    startup::step("operators");
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
    startup::step("enemies");
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

    // One classifier for both indexes: building it reads roguelike_topic_table
    // (1.8 GB on CN), climb_tower, sandbox, sandbox_perm, crisis_v2 and
    // handbook_info as loose JSON. Scoped so the borrows of
    // `stages`/`zones`/`activities` end before they move into `GameData`.
    let levels_dir = assets_dir.join("gamedata/levels");
    let (enemy_stage_index, stage_index, mode_levels) = {
        startup::step("stage classifier");
        let classifier = StageClassifier::new(data_dir, &stages, &zones, &activity_file.basic_info);

        // Inverted enemy -> stages index, parsed from per-stage level files.
        let enemies_by_stage = build_enemy_stage_index(&levels_dir, data_dir, &classifier);
        let (index, modes) = build_stage_index(
            assets_dir,
            &levels_dir,
            &classifier,
            &stages,
            &zones,
            &activity_file.basic_info,
        );
        (enemies_by_stage, index, modes)
    };

    startup::step("chibis");
    let chibis = init_chibi_data(assets_dir);
    startup::step("enemy chibis");
    let enemy_chibis = init_enemy_chibi_data(assets_dir, &enemies);

    for w in &warnings {
        tracing::warn!("{w}");
    }

    Ok((
        GameData {
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
            chibis,
            enemy_chibis,
            zones,
            stages,
            activities: activity_file.basic_info,
            retro_acts: retro_file.retro_act_list,
            medals,
            roguelike,
            enemies,
            enemy_stage_index,
            stage_index,
            mode_levels,
            building: building_file,
            stage_universe,
            sandbox_universe,
            campaign_rotations,
            consts,
        },
        assets,
    ))
}
