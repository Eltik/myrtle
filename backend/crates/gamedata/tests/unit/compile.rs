use std::collections::HashMap;

use myrtle_gamedata_raw::{
    RawGameData,
    integrity::SkippedTable,
    tables::{
        activity::{FARM_ARCHIVE, FarmArchive, FarmStage},
        chapter::MainlineAdditionInfo,
        enemy::EnemyDatabaseFile,
        gacha::GachaPoolClient,
        gacha_detail::{
            DetailInfo, GachaPoolDetail, PerCharEntry, PoolDetailFile, RARITY_RANK_5,
            RARITY_RANK_6, UpCharInfo,
        },
        operator::{CharPatchInfo, CharPatchTable, RawOperator},
        voice::{CharExtraWord, RawVoice, VoiceLang, VoicesTableFile},
        zone::{Zone, ZoneTableFile},
    },
};
use myrtle_types::Server;

use crate::compile::*;
use crate::model::{GameVersion, MODEL_SCHEMA_VERSION};

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
