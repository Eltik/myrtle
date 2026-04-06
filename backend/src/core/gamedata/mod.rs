use std::path::Path;

use crate::core::gamedata::{
    assets::AssetIndex,
    enrich::{
        chibi::init_chibi_data,
        modules::enrich_modules_global,
        operators::{EnrichCtx, enrich_all_operators, extract_all_drones},
        skills::enrich_all_skills,
        skins::enrich_all_skins,
        voice::enrich_all_voices,
    },
    tables::{DataError, load_table, load_table_or_warn},
    types::{
        GameData,
        building::BuildingDataFile,
        enemy::{EnemyDatabaseFile, EnemyHandbook, EnemyHandbookTableFile},
        gacha::GachaTableFile,
        handbook::HandbookTableFile,
        material::ItemTableFile,
        medal::{MedalData, MedalTableFile},
        module::{BattleEquipTableFile, UniequipTableFile},
        operator::CharacterTable,
        range::Ranges,
        roguelike::RoguelikeGameData,
        skill::SkillTableFile,
        skin::SkinTableFile,
        stage::StageTableFile,
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
    let raw_operators = char_table.characters;

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
    let voice_file: VoicesTableFile = load_table_or_warn(data_dir, "charword_table", &mut warnings);
    let enemy_file: EnemyHandbookTableFile =
        load_table_or_warn(data_dir, "enemy_handbook_table", &mut warnings);
    let building_file: BuildingDataFile =
        load_table_or_warn(data_dir, "building_data", &mut warnings);

    let materials = item_file.into_materials();
    let raw_modules = equip_file.into_raw_modules();
    let battle_equip = battle_equip_file.into_battle_equip();
    let handbook = handbook_file.into_handbook();
    let skins = skin_file.into_skin_data();
    let gacha = gacha_file.into_gacha_data();
    let zones = zone_file.zones;
    let stages = stage_file.stages;
    let medals = MedalData::from_table(medal_file);
    let roguelike = RoguelikeGameData::with_known_values();

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
        zones,
        stages,
        medals,
        roguelike,
        enemies,
        building: building_file,
    })
}
