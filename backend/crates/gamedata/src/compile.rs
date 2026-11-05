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

#[cfg(test)]
mod tests {
    use myrtle_gamedata_raw::{
        integrity::SkippedTable,
        tables::{
            chapter::MainlineAdditionInfo,
            enemy::EnemyDatabaseFile,
            gacha::GachaPoolClient,
            gacha_detail::{
                DetailInfo, GachaPoolDetail, PerCharEntry, PoolDetailFile, RARITY_RANK_5,
                RARITY_RANK_6, UpCharInfo,
            },
            voice::{CharExtraWord, RawVoice, VoiceLang},
        },
    };

    use super::*;

    const MISSING_ENEMY_DATABASE: &str = "enemy_database: file not found or parse error";

    fn compile_en(raw: RawGameData) -> Compiled {
        compile(raw, &CompileOptions::new(Server::EN)).expect("raw data compiles")
    }

    fn operator(name: &str) -> RawOperator {
        RawOperator {
            name: name.to_owned(),
            ..RawOperator::default()
        }
    }

    fn patch_table(chars: &[(&str, &str)], infos: &[(&str, &str, &[&str])]) -> CharPatchTable {
        CharPatchTable {
            patch_chars: chars
                .iter()
                .map(|(id, name)| ((*id).to_owned(), operator(name)))
                .collect(),
            infos: infos
                .iter()
                .map(|(key, default, tmpl_ids)| {
                    let info = CharPatchInfo {
                        default: (*default).to_owned(),
                        tmpl_ids: tmpl_ids.iter().map(|id| (*id).to_owned()).collect(),
                    };
                    ((*key).to_owned(), info)
                })
                .collect(),
        }
    }

    fn zone_addition(chapter_id: &str, zone_open_time: i64) -> MainlineAdditionInfo {
        MainlineAdditionInfo {
            chapter_id: chapter_id.to_owned(),
            zone_open_time,
            ..MainlineAdditionInfo::default()
        }
    }

    fn farm(stage_id: &str) -> FarmStage {
        FarmStage {
            stage_id: stage_id.to_owned(),
            code: String::new(),
            ap_cost: 0,
            drops: Vec::new(),
        }
    }

    fn farm_map(acts: &[(&str, &[&str])]) -> HashMap<String, Vec<FarmStage>> {
        acts.iter()
            .map(|(act, stages)| {
                (
                    (*act).to_owned(),
                    stages.iter().copied().map(farm).collect(),
                )
            })
            .collect()
    }

    fn archived(bytes: &[u8]) -> HashMap<String, Vec<FarmStage>> {
        serde_json::from_slice(bytes).expect("archive is JSON")
    }

    #[test]
    fn options_default_to_both_alignments_on() {
        let opts = CompileOptions::new(Server::CN);
        assert_eq!(opts.server, Server::CN);
        assert_eq!(opts.game_version, GameVersion::default());
        assert!(opts.profile_align);
        assert!(opts.obtain_channel_align);
    }

    #[test]
    fn raw_data_from_another_server_is_rejected() {
        let raw = RawGameData {
            server: Some(Server::CN),
            ..RawGameData::default()
        };
        let Err(err) = compile(raw, &CompileOptions::new(Server::EN)) else {
            panic!("a server mismatch must not compile");
        };
        assert!(matches!(
            err,
            CompileError::ServerMismatch {
                pass: "meta",
                expected: Server::EN,
                found: Server::CN,
            }
        ));
        assert_eq!(
            err.to_string(),
            format!(
                "meta: raw data is for {:?}, options say {:?}",
                Server::CN,
                Server::EN
            )
        );
    }

    #[test]
    fn raw_data_for_the_same_server_compiles() {
        let raw = RawGameData {
            server: Some(Server::EN),
            ..RawGameData::default()
        };
        assert_eq!(compile_en(raw).model.meta.server, Server::EN);
    }

    #[test]
    fn raw_data_without_a_server_takes_the_one_from_the_options() {
        let compiled = compile(RawGameData::default(), &CompileOptions::new(Server::CN))
            .expect("raw data compiles");
        assert_eq!(compiled.model.meta.server, Server::CN);
    }

    #[test]
    fn meta_carries_the_options_and_the_load_results() {
        let skipped = SkippedTable {
            table: "activity_table".to_owned(),
            since: "2026-10-20T00:00:00Z".to_owned(),
            res_version: None,
        };
        let raw = RawGameData {
            skipped: vec![skipped.clone()],
            ..RawGameData::default()
        };
        let game_version = GameVersion {
            client: "2.6.81".to_owned(),
            res: "26-10-14".to_owned(),
        };
        let opts = CompileOptions {
            game_version: game_version.clone(),
            ..CompileOptions::new(Server::EN)
        };
        let meta = compile(raw, &opts).expect("raw data compiles").model.meta;
        assert_eq!(meta.game_version, game_version);
        assert_eq!(meta.schema_version, MODEL_SCHEMA_VERSION);
        assert_eq!(meta.skipped_tables, vec![skipped]);
    }

    #[test]
    fn a_missing_enemy_database_is_reported_and_the_table_warnings_list_it_last() {
        let raw = RawGameData {
            warnings: vec!["voice_table: parse error".to_owned()],
            ..RawGameData::default()
        };
        let compiled = compile_en(raw);
        let report = &compiled.report.warnings;
        let table = &compiled.model.meta.table_warnings;
        assert_eq!(
            report.last().map(String::as_str),
            Some(MISSING_ENEMY_DATABASE)
        );
        assert_eq!(table[0], "voice_table: parse error");
        assert_eq!(&table[1..], report.as_slice());
    }

    #[test]
    fn a_present_enemy_database_raises_no_warning() {
        let mut raw = RawGameData::default();
        raw.enemy.database = Some(EnemyDatabaseFile::default());
        let warnings = compile_en(raw).report.warnings;
        assert!(!warnings.iter().any(|w| w == MISSING_ENEMY_DATABASE));
    }

    #[test]
    fn empty_raw_data_compiles_to_an_unloaded_model() {
        let compiled = compile_en(RawGameData::default());
        assert!(!compiled.model.is_loaded());
        assert!(compiled.report.derived_files.is_empty());
    }

    #[test]
    fn patch_forms_reach_the_operator_table() {
        let mut raw = RawGameData::default();
        raw.character
            .characters
            .characters
            .insert("char_002_amiya".to_owned(), operator("Amiya"));
        raw.character.patch = patch_table(
            &[("char_1001_amiya2", "Amiya (Guard)")],
            &[(
                "char_002_amiya",
                "char_002_amiya",
                &["char_002_amiya", "char_1001_amiya2"],
            )],
        );
        let operators = compile_en(raw).model.operators;
        assert_eq!(operators.len(), 2);
        assert!(operators.contains_key("char_002_amiya"));
        assert!(operators.contains_key("char_1001_amiya2"));
    }

    #[test]
    fn zone_tables_are_split_into_the_model() {
        let mut raw = RawGameData::default();
        raw.stage
            .zones
            .zones
            .insert("main_1".to_owned(), Zone::default());
        raw.stage.zones.mainline_addition_info.insert(
            "main_1".to_owned(),
            zone_addition("chapter_1", 1_600_000_000),
        );
        let model = compile_en(raw).model;
        assert!(model.zones.contains_key("main_1"));
        assert_eq!(model.zone_chapters["main_1"], "chapter_1");
        assert_eq!(model.zone_open_times["main_1"], 1_600_000_000);
    }

    #[test]
    fn a_growing_farm_archive_lands_in_the_report_and_the_model() {
        let mut raw = RawGameData::default();
        raw.activity.farm_archive = FarmArchive::Loaded(farm_map(&[("act1side", &["a"])]));
        let compiled = compile_en(raw);
        assert_eq!(compiled.model.activity_farm_stages["act1sre"], [farm("a")]);
        let [file] = compiled.report.derived_files.as_slice() else {
            panic!("one derived file expected");
        };
        assert_eq!(file.rel_path, FARM_ARCHIVE);
        assert_eq!(archived(&file.bytes), compiled.model.activity_farm_stages);
    }

    #[test]
    fn an_unreadable_farm_archive_produces_no_derived_file() {
        let mut raw = RawGameData::default();
        raw.activity.farm_archive = FarmArchive::Unreadable;
        assert!(compile_en(raw).report.derived_files.is_empty());
    }

    #[test]
    fn banner_details_only_feature_the_top_two_rarities() {
        let entry = |rarity_rank, char_id: &str| PerCharEntry {
            rarity_rank,
            char_id_list: vec![char_id.to_owned(), String::new()],
            ..PerCharEntry::default()
        };
        let detail = GachaPoolDetail {
            detail_info: DetailInfo {
                up_char_info: Some(UpCharInfo {
                    per_char_list: vec![
                        entry(RARITY_RANK_6, "char_six"),
                        entry(RARITY_RANK_5, "char_five"),
                        entry(RARITY_RANK_5 - 1, "char_four"),
                    ],
                }),
                ..DetailInfo::default()
            },
            ..GachaPoolDetail::default()
        };
        let mut raw = RawGameData::default();
        raw.gacha.gacha.gacha_pool_client = vec![
            GachaPoolClient {
                gacha_pool_id: "with_detail".to_owned(),
                ..GachaPoolClient::default()
            },
            GachaPoolClient {
                gacha_pool_id: "without_detail".to_owned(),
                ..GachaPoolClient::default()
            },
        ];
        raw.gacha.pool_details = Some(PoolDetailFile {
            pools: HashMap::from([("with_detail".to_owned(), detail)]),
            ..PoolDetailFile::default()
        });

        let pools = compile_en(raw).model.gacha.gacha_pool_client;
        assert_eq!(pools[0].featured6, ["char_six"]);
        assert_eq!(pools[0].featured5, ["char_five"]);
        assert_eq!(pools[0].featured_source, "static+api");
        assert!(pools[1].featured6.is_empty());
        assert_eq!(pools[1].featured_source, "static");
    }

    #[test]
    fn patch_forms_never_replace_a_base_operator() {
        let base = HashMap::from([("char_002_amiya".to_owned(), operator("Base"))]);
        let patch = patch_table(
            &[
                ("char_002_amiya", "Patched"),
                ("char_1001_amiya2", "Amiya (Guard)"),
            ],
            &[],
        );
        let (operators, _) = merge_char_patch(base, patch);
        assert_eq!(operators["char_002_amiya"].name, "Base");
        assert_eq!(operators["char_1001_amiya2"].name, "Amiya (Guard)");
    }

    #[test]
    fn every_template_id_maps_to_its_patch_group() {
        let patch = patch_table(
            &[],
            &[(
                "group",
                "char_002_amiya",
                &["char_002_amiya", "char_1001_amiya2"],
            )],
        );
        let (operators, groups) = merge_char_patch(HashMap::new(), patch);
        assert!(operators.is_empty());
        assert_eq!(groups.len(), 2);
        assert_eq!(groups["char_002_amiya"].default, "char_002_amiya");
        assert_eq!(groups["char_1001_amiya2"].default, "char_002_amiya");
    }

    #[test]
    fn split_zones_keeps_the_chapter_of_a_zone_without_an_open_time() {
        let file = ZoneTableFile {
            zones: HashMap::from([("main_1".to_owned(), Zone::default())]),
            mainline_addition_info: HashMap::from([
                ("main_1".to_owned(), zone_addition("chapter_1", -1)),
                ("main_2".to_owned(), zone_addition("chapter_1", 0)),
                ("main_3".to_owned(), zone_addition("chapter_2", 1)),
            ]),
        };
        let (zones, chapters, open_times) = split_zones(file);
        assert_eq!(zones.len(), 1);
        assert_eq!(chapters.len(), 3);
        assert_eq!(chapters["main_1"], "chapter_1");
        assert_eq!(open_times, HashMap::from([("main_3".to_owned(), 1)]));
    }

    #[test]
    fn no_archive_and_no_live_stages_writes_nothing() {
        let (merged, write) = merge_farm_archive(FarmArchive::Absent, HashMap::new());
        assert!(merged.is_empty());
        assert_eq!(write, None);
    }

    #[test]
    fn a_new_activity_rewrites_the_archive_with_the_merged_result() {
        let archive = FarmArchive::Loaded(farm_map(&[("act1", &["old"])]));
        let (merged, write) = merge_farm_archive(archive, farm_map(&[("act2", &["new"])]));
        assert_eq!(merged, farm_map(&[("act1", &["old"]), ("act2", &["new"])]));
        let file = write.expect("archive grew");
        assert_eq!(file.rel_path, FARM_ARCHIVE);
        assert_eq!(archived(&file.bytes), merged);
    }

    #[test]
    fn live_stages_win_but_only_a_new_activity_rewrites_the_archive() {
        let archive = FarmArchive::Loaded(farm_map(&[("act1", &["old"])]));
        let (merged, write) = merge_farm_archive(archive, farm_map(&[("act1", &["new"])]));
        assert_eq!(merged, farm_map(&[("act1", &["new"])]));
        assert_eq!(write, None);
    }

    #[test]
    fn an_activity_gone_from_the_live_tables_stays_archived() {
        let archive = FarmArchive::Loaded(farm_map(&[("act1", &["old"])]));
        let (merged, write) = merge_farm_archive(archive, farm_map(&[("act1", &[])]));
        assert_eq!(merged, farm_map(&[("act1", &["old"])]));
        assert_eq!(write, None);
    }

    #[test]
    fn an_empty_live_list_never_creates_an_activity() {
        let (merged, write) = merge_farm_archive(FarmArchive::Absent, farm_map(&[("act1", &[])]));
        assert!(merged.is_empty());
        assert_eq!(write, None);
    }

    #[test]
    fn a_rerun_twin_counts_as_growth_and_is_archived() {
        let archive = FarmArchive::Loaded(farm_map(&[("act1side", &["a"])]));
        let (merged, write) = merge_farm_archive(archive, HashMap::new());
        assert_eq!(
            merged,
            farm_map(&[("act1side", &["a"]), ("act1sre", &["a"])])
        );
        assert_eq!(archived(&write.expect("twin added").bytes), merged);
    }

    #[test]
    fn an_unreadable_archive_returns_live_as_is_and_is_never_overwritten() {
        let live = farm_map(&[("act1side", &["a"]), ("act2", &[])]);
        let (merged, write) = merge_farm_archive(FarmArchive::Unreadable, live.clone());
        assert_eq!(merged, live);
        assert_eq!(write, None);
    }

    #[test]
    fn build_voices_enriches_lines_and_passes_the_rest_through() {
        let raw_voice = RawVoice {
            char_word_id: "char_002_amiya_CN_001".to_owned(),
            word_key: "char_002_amiya".to_owned(),
            char_id: "char_002_amiya".to_owned(),
            voice_id: "CN_001".to_owned(),
            ..RawVoice::default()
        };
        let voices = build_voices(VoicesTableFile {
            char_words: HashMap::from([("char_002_amiya_CN_001".to_owned(), raw_voice)]),
            char_extra_words: HashMap::from([("extra".to_owned(), CharExtraWord::default())]),
            voice_lang_dict: HashMap::from([("char_002_amiya".to_owned(), VoiceLang::default())]),
            default_lang_type: "JP".to_owned(),
            new_tag_list: vec!["tag".to_owned()],
        });
        let line = &voices.char_words["char_002_amiya_CN_001"];
        assert_eq!(line.char_id, "char_002_amiya");
        assert_eq!(line.voice_id, "CN_001");
        assert!(voices.char_extra_words.contains_key("extra"));
        assert!(voices.voice_lang_dict.contains_key("char_002_amiya"));
        assert_eq!(voices.default_lang_type, "JP");
        assert_eq!(voices.new_tag_list, ["tag"]);
    }
}
