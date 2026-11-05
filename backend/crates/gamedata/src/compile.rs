use std::collections::{HashMap, HashSet};

use myrtle_gamedata_raw::{
    RawGameData,
    tables::{
        activity::{
            FARM_ARCHIVE, FarmArchive, FarmStage, farm_stages_by_activity,
            mission_tokens_by_activity, op_stages_by_activity, share_between_twins,
        },
        audio::MusicBanks,
        building::BuildingDataFile,
        campaign::CampaignRotations,
        enemy::EnemyHandbook,
        gacha::profession_names,
        medal::MedalData,
        mission::MissionData,
        operator::{CharPatchInfo, CharPatchTable, RawOperator},
        roguelike::RoguelikeGameData,
        voice::{Voices, VoicesTableFile},
        zone::{Zone, ZoneTableFile},
    },
};
use myrtle_types::Server;

use crate::{
    assets::AssetIndex,
    enrich::{
        audio::{build_enemy_audio, build_operator_audio},
        chibi::{init_chibi_data, init_enemy_chibi_data},
        enemies::{enemy_sound_keys, enrich_enemies},
        enemy_stages::build_enemy_stage_index,
        gacha::{enrich_banners, mark_returning_pools},
        modules::enrich_modules_global,
        operators::{EnrichCtx, enrich_all_operators, extract_all_drones},
        skills::enrich_all_skills,
        skins::enrich_all_skins,
        stage_class::{ModeTables, StageClassifier},
        stage_index::build_stage_index,
        voice::enrich_all_voices,
    },
    index::{
        base_terms::layout_term_rooms,
        squad::{SquadIndex, SquadIndexData},
    },
    model::{
        GameModel, GameVersion, MODEL_SCHEMA_VERSION, ModelMeta, sandbox_universe::SandboxUniverse,
        skin::skin_data, stage_evidence::StageEvidenceIndex, stage_universe::StageUniverse,
    },
};

#[derive(Debug, Clone)]
pub struct CompileOptions {
    pub server: Server,
    pub game_version: GameVersion,
    pub profile_align: bool,
    pub obtain_channel_align: bool,
}

impl CompileOptions {
    #[must_use]
    pub fn new(server: Server) -> Self {
        Self {
            server,
            game_version: GameVersion::default(),
            profile_align: true,
            obtain_channel_align: true,
        }
    }
}

pub struct Compiled {
    pub model: GameModel,
    pub report: CompileReport,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DerivedFile {
    pub rel_path: &'static str,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Default)]
pub struct CompileReport {
    pub warnings: Vec<String>,
    pub derived_files: Vec<DerivedFile>,
}

#[derive(Debug, thiserror::Error)]
pub enum CompileError {
    #[error("{pass}: raw data is for {found:?}, options say {expected:?}")]
    ServerMismatch {
        pass: &'static str,
        expected: Server,
        found: Server,
    },
}

pub fn compile(raw: RawGameData, opts: &CompileOptions) -> Result<Compiled, CompileError> {
    if let Some(found) = raw.server
        && found != opts.server
    {
        return Err(CompileError::ServerMismatch {
            pass: "meta",
            expected: opts.server,
            found,
        });
    }

    let mut report = CompileReport::default();
    let RawGameData {
        server: _,
        character,
        item,
        stage,
        activity,
        gacha,
        building,
        medal,
        roguelike,
        sandbox,
        enemy,
        story,
        audio,
        autochess,
        level,
        asset_paths,
        skipped,
        warnings,
    } = raw;

    let assets = AssetIndex::build(&asset_paths, &story.sprites, &story.scripts);
    drop(story.scripts);

    let (raw_operators, tmpl_groups) =
        merge_char_patch(character.characters.characters, character.patch);

    let activity_op_stages =
        op_stages_by_activity(&stage.stages.stages, &activity.activities.zone_to_activity);
    let (activity_farm_stages, farm_archive) = merge_farm_archive(
        activity.farm_archive,
        farm_stages_by_activity(
            &stage.stages.stages,
            &activity.activities.zone_to_activity,
            &item.items.items,
        ),
    );
    report.derived_files.extend(farm_archive);
    let activity_mission_tokens = mission_tokens_by_activity(
        &activity.activities.mission_data,
        &activity.activities.activity_items,
    );

    let building = BuildingDataFile {
        layout_terms: layout_term_rooms(&character.consts),
        ..building.building
    };

    let materials = item.items.into_materials();
    let raw_modules = character.uniequip.into_raw_modules();
    let battle_equip = character.battle_equip.into_battle_equip();
    let handbook = character.handbook.into_handbook();
    let mut skins = skin_data(character.skins);
    let mut gacha_data = gacha.gacha.into_gacha_data();
    let profession_names = profession_names(&gacha_data.gacha_tags);
    enrich_banners(
        &mut gacha_data.gacha_pool_client,
        gacha.pool_details.as_ref(),
    );
    mark_returning_pools(
        &mut gacha_data.gacha_pool_client,
        &gacha.open_server.returning_pool_ids(),
    );
    let (zones, zone_chapters, zone_open_times) = split_zones(stage.zones);
    let storylines = stage.stages.storylines;
    let storyline_story_sets = stage.stages.storyline_story_sets;
    let stages = stage.stages.stages;
    let mut medals = MedalData::from_table(medal.medals);
    let roguelike_tables = roguelike;
    let roguelike = RoguelikeGameData::from_table(&roguelike_tables.topics);
    let campaign_rotations = CampaignRotations::from_table(stage.campaigns);
    let retro_linked_acts: HashSet<String> = stage
        .retro
        .retro_act_list
        .values()
        .flat_map(|r| r.linked_act_id.iter().cloned())
        .collect();
    let stage_universe = StageUniverse::build(
        &stages,
        &zones,
        &activity.activities.basic_info,
        &campaign_rotations,
        &retro_linked_acts,
        &activity.activities.optional_stage_ids(),
        &activity.activities.live_only_stage_ids(),
    );

    let sandbox_universe = SandboxUniverse::build_reporting(&sandbox.perm, &mut report.warnings);

    let skills = enrich_all_skills(character.skills.skills, &assets);
    let drones = extract_all_drones(&raw_operators);
    skins.enriched_skins = enrich_all_skins(&skins.char_skins, &assets);
    let modules = enrich_modules_global(&raw_modules, &battle_equip, &materials, &assets);

    let voices = build_voices(audio.voices);

    let op_ids: HashSet<&str> = raw_operators.keys().map(String::as_str).collect();
    let operator_audio = build_operator_audio(&audio.audio, &op_ids, &assets);

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
            building: &building,
            tmpl_groups: &tmpl_groups,
            audio: &operator_audio,
            consts: &character.consts,
            factions: &character.handbook_team.teams,
            profession_names: &profession_names,
        },
    );

    let (enemies, enemy_prefab_keys, enemy_skill_keys) = if let Some(enemy_db) = &enemy.database {
        let (prefab_keys, skill_keys) = enemy_sound_keys(enemy_db);
        (
            enrich_enemies(enemy.handbook, enemy_db, &assets),
            prefab_keys,
            skill_keys,
        )
    } else {
        report
            .warnings
            .push("enemy_database: file not found or parse error".into());
        (
            EnemyHandbook::from(enemy.handbook), // fallback without stats
            HashMap::new(),
            HashMap::new(),
        )
    };

    let enemy_audio = build_enemy_audio(
        &audio.audio,
        enemies.enemy_data.keys().map(String::as_str),
        &enemy_prefab_keys,
        &enemy_skill_keys,
        &op_ids,
        &assets,
    );

    medals.link_operator_locks(&operators);
    medals.link_content_windows(
        &stage.climb_tower.tower_windows(),
        &activity.activities.basic_info,
    );

    let (enemy_stage_index, stage_index, mode_levels) = {
        let classifier = StageClassifier::new(
            &stages,
            &zones,
            &activity.activities.basic_info,
            &ModeTables {
                roguelike: &roguelike_tables.topics,
                towers: &stage.climb_tower.towers,
                sandbox: &sandbox.table,
                sandbox_perm: &sandbox.perm,
                crisis_v2: &stage.crisis_v2,
                handbook_stages: &handbook.handbook_stage_data,
            },
        );
        let enemies_by_stage = build_enemy_stage_index(&level, &activity.boss_stages, &classifier);
        let (index, modes) = build_stage_index(
            &asset_paths,
            &level,
            &classifier,
            &stages,
            &zones,
            &activity.activities.basic_info,
        );
        (enemies_by_stage, index, modes)
    };
    drop(level);

    let stage_evidence =
        StageEvidenceIndex::build(&activity.activities.mission_data, &medals.medals, &stages);

    let chibis = init_chibi_data(&asset_paths);
    let enemy_chibis = init_enemy_chibi_data(&asset_paths, &enemies);

    let mut table_warnings = warnings;
    table_warnings.extend(report.warnings.iter().cloned());

    let mut model = GameModel {
        meta: ModelMeta {
            server: opts.server,
            game_version: opts.game_version.clone(),
            schema_version: MODEL_SCHEMA_VERSION,
            skipped_tables: skipped,
            table_warnings,
        },
        operators,
        skills,
        materials,
        modules,
        skins,
        handbook,
        ranges: character.ranges,
        favor: character.favor,
        voices,
        zones,
        stages,
        activities: activity.activities.basic_info,
        retro_acts: stage.retro.retro_act_list,
        story_reviews: story.review.story_reviews,
        story_archives: story.review_meta,
        mission_archives: activity.mission_archives.mission_archives,
        music: MusicBanks::from_raw(&audio.audio),
        chapters: stage.chapters.chapters,
        zone_chapters,
        zone_open_times,
        storylines,
        storyline_story_sets,
        activity_op_stages,
        activity_farm_stages,
        activity_mission_tokens,
        event_shops: activity.event_shops,
        activity_skin_refs: activity.skin_refs,
        skin_listings: item.shop.into_skin_listings(),
        recommend_tags: item.shop.recommend_tags(),
        skin_windows: item.shop.into_skin_windows(),
        medals,
        roguelike,
        enemies,
        enemy_audio,
        enemy_stage_index,
        stage_index,
        mode_levels,
        building,
        stage_universe,
        stage_evidence,
        sandbox_universe,
        campaign_rotations,
        consts: character.consts,
        missions: MissionData::from_table(activity.missions),
        gacha: gacha_data,
        factions: character.handbook_team.teams,
        home_backgrounds: item.display_meta.into_home_backgrounds(),
        profession_names,
        autochess_bonds: autochess.autochess.auto_chess_data.bond_info_dict,
        chibis,
        enemy_chibis,
        assets,
        ..GameModel::default()
    };
    model.squad = SquadIndex::build(&SquadIndexData {
        operators: &model.operators,
        gacha: &model.gacha,
        enemies: &model.enemies,
        stages: &model.stages,
        factions: &model.factions,
    });

    Ok(Compiled { model, report })
}

fn merge_char_patch(
    mut raw_operators: HashMap<String, RawOperator>,
    char_patch: CharPatchTable,
) -> (HashMap<String, RawOperator>, HashMap<String, CharPatchInfo>) {
    // Merge Amiya's branch forms (Guard `char_1001_amiya2`, Medic `char_1037_amiya3`) from char_patch_table
    for (id, op) in char_patch.patch_chars {
        raw_operators.entry(id).or_insert(op);
    }
    let mut tmpl_groups: HashMap<String, CharPatchInfo> = HashMap::new();
    for info in char_patch.infos.into_values() {
        for id in &info.tmpl_ids {
            tmpl_groups.insert(id.clone(), info.clone());
        }
    }
    (raw_operators, tmpl_groups)
}

fn split_zones(
    zone_file: ZoneTableFile,
) -> (
    HashMap<String, Zone>,
    HashMap<String, String>,
    HashMap<String, i64>,
) {
    let mut zone_chapters: HashMap<String, String> = HashMap::new();
    let mut zone_open_times: HashMap<String, i64> = HashMap::new();
    for (zone_id, info) in zone_file.mainline_addition_info {
        if info.zone_open_time > 0 {
            zone_open_times.insert(zone_id.clone(), info.zone_open_time);
        }
        zone_chapters.insert(zone_id, info.chapter_id);
    }
    (zone_file.zones, zone_chapters, zone_open_times)
}

fn merge_farm_archive(
    archive: FarmArchive,
    live: HashMap<String, Vec<FarmStage>>,
) -> (HashMap<String, Vec<FarmStage>>, Option<DerivedFile>) {
    let mut merged: HashMap<String, Vec<FarmStage>> = match archive {
        FarmArchive::Loaded(parsed) => parsed,
        FarmArchive::Absent => HashMap::new(),
        FarmArchive::Unreadable => return (live, None),
    };
    let before = merged.len();
    for (act, stages) in live {
        if !stages.is_empty() {
            merged.insert(act, stages);
        }
    }
    share_between_twins(&mut merged);
    let write = if merged.len() == before {
        None
    } else {
        serde_json::to_string(&merged).ok().map(|json| DerivedFile {
            rel_path: FARM_ARCHIVE,
            bytes: json.into_bytes(),
        })
    };
    (merged, write)
}

fn build_voices(voice_file: VoicesTableFile) -> Voices {
    let enriched_char_words =
        enrich_all_voices(&voice_file.char_words, &voice_file.voice_lang_dict);
    Voices {
        char_words: enriched_char_words,
        char_extra_words: voice_file.char_extra_words,
        voice_lang_dict: voice_file.voice_lang_dict,
        default_lang_type: voice_file.default_lang_type,
        new_tag_list: voice_file.new_tag_list,
        ..Default::default()
    }
}
