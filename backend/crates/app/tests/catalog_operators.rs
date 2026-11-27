#![allow(clippy::unwrap_used)]

mod common;

use std::collections::HashMap;

use myrtle_app::AppError;
use myrtle_app::services::catalog::operators::{
    self, LevelBucket, ROSTER_FIELDS, fill_levels, roster_rows, voice_actors,
};
use myrtle_app::services::catalog::recruitment::{self, recruitable_names, strip_tags};
use myrtle_gamedata_raw::tables::operator::Operator;
use myrtle_gamedata_raw::tables::voice::{LangType, VoiceLang, VoiceLangDictEntry, Voices};
use myrtle_types::Server;
use serde_json::Value;

const AMIYA: &str = "char_002_amiya";

fn parse(body: &str) -> Value {
    serde_json::from_str(body).unwrap()
}

fn lang(voice_lang_type: LangType, cv_name: &[&str]) -> VoiceLangDictEntry {
    VoiceLangDictEntry {
        voice_lang_type,
        cv_name: cv_name.iter().map(|s| (*s).to_owned()).collect(),
        ..Default::default()
    }
}

#[test]
fn voice_actors_dedupe_across_languages_in_key_order() {
    let dict = HashMap::from([
        ("JP".to_owned(), lang(LangType::Jp, &["Hanazawa Kana"])),
        (
            "CN_MANDARIN".to_owned(),
            lang(LangType::CnMandarin, &["Hanazawa Kana", "Wang Yaxin"]),
        ),
        (
            "EN".to_owned(),
            lang(LangType::En, &["Li Jiaxin", "Kana Hanazawa", "Li Jiaxin"]),
        ),
    ]);
    let voices = Voices {
        voice_lang_dict: HashMap::from([(
            AMIYA.to_owned(),
            VoiceLang {
                char_id: AMIYA.to_owned(),
                dict,
                ..Default::default()
            },
        )]),
        ..Default::default()
    };
    assert_eq!(
        voice_actors(&voices, AMIYA),
        ["Hanazawa Kana", "Wang Yaxin", "Li Jiaxin", "Kana Hanazawa"]
    );
    assert!(voice_actors(&voices, "char_999_none").is_empty());
}

fn operator(id: &str, tmpl_default: Option<&str>) -> Operator {
    Operator {
        id: Some(id.to_owned()),
        name: id.to_uppercase(),
        tmpl_default: tmpl_default.map(str::to_owned),
        ..Default::default()
    }
}

#[test]
fn roster_rows_keep_exactly_the_roster_fields_in_table_key_order() {
    let table = HashMap::from([
        ("char_b".to_owned(), operator("char_b", None)),
        ("char_a".to_owned(), operator("char_a", Some("char_a"))),
    ]);
    let rows = roster_rows(&table).unwrap();
    let ids: Vec<&str> = rows.iter().map(|r| r["id"].as_str().unwrap()).collect();
    assert_eq!(ids, ["char_a", "char_b"]);
    let mut keys: Vec<&str> = rows[0]
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    keys.sort_unstable();
    let mut expected = ROSTER_FIELDS.to_vec();
    expected.sort_unstable();
    assert_eq!(keys, expected);
    assert!(rows[1].get("tmplDefault").is_none());
    assert_eq!(rows[1].as_object().unwrap().len(), ROSTER_FIELDS.len() - 1);
}

#[test]
fn fill_levels_zero_fills_the_strip() {
    let found = [
        LevelBucket { level: 3, users: 7 },
        LevelBucket { level: 1, users: 2 },
    ];
    let users: Vec<(i16, i64)> = fill_levels(&found)
        .iter()
        .map(|b| (b.level, b.users))
        .collect();
    assert_eq!(users, [(0, 0), (1, 2), (2, 0), (3, 7)]);
}

#[tokio::test]
async fn operator_detail_serves_a_known_id_and_404s_an_unknown_one() {
    let Some((ctx, db, _)) = common::context_with_data().await else {
        return;
    };
    let detail = parse(
        &operators::detail(&ctx, Server::EN, AMIYA)
            .await
            .unwrap()
            .body,
    );
    assert_eq!(detail["id"], AMIYA);
    assert_eq!(detail["server"], "en");
    assert!(detail["phases"].is_array());
    assert!(matches!(
        operators::detail(&ctx, Server::EN, "char_none").await,
        Err(AppError::NotFound(None))
    ));
    assert!(matches!(
        operators::detail(&ctx, Server::JP, AMIYA).await,
        Err(AppError::NotFound(_))
    ));

    let resolved = parse(
        &operators::resolve_detail(&ctx, Server::JP, AMIYA)
            .await
            .unwrap()
            .body,
    );
    assert_eq!(resolved["server"], "en");
    assert!(matches!(
        operators::resolve_detail(&ctx, Server::EN, "char_none").await,
        Err(AppError::NotFound(None))
    ));
    let (snapshot, server) = operators::resolve_operator(&ctx, Server::KR, AMIYA).unwrap();
    assert_eq!(server, Server::EN);
    assert!(snapshot.model.operators.contains_key(AMIYA));
    db.close().await;
}

#[tokio::test]
async fn the_index_is_rarest_first_and_cached_with_one_etag() {
    let Some((ctx, db, snapshot)) = common::context_with_data().await else {
        return;
    };
    let first = operators::index(&ctx, Server::EN).await.unwrap();
    let index = parse(&first.body);
    let entries = index.as_array().unwrap();
    assert_eq!(entries.len(), snapshot.model.operators.len());
    let rarities: Vec<u64> = entries
        .iter()
        .map(|e| e["rarity"].as_u64().unwrap())
        .collect();
    assert!(rarities.windows(2).all(|w| w[0] >= w[1]));
    let amiya = entries.iter().find(|e| e["id"] == AMIYA).unwrap();
    assert_eq!(amiya["rarity"], 5);
    assert!(amiya["skillCount"].as_u64().unwrap() >= 3);
    assert!(!amiya["voiceActors"].as_array().unwrap().is_empty());
    let second = operators::index(&ctx, Server::EN).await.unwrap();
    assert_eq!(first.etag, second.etag);
    db.close().await;
}

#[tokio::test]
async fn the_roster_projection_keeps_only_roster_fields() {
    let Some((ctx, db, snapshot)) = common::context_with_data().await else {
        return;
    };
    let roster = parse(&operators::roster(&ctx, Server::EN).await.unwrap().body);
    let rows = roster.as_array().unwrap();
    assert_eq!(rows.len(), snapshot.model.operators.len());
    for row in rows {
        assert!(
            row.as_object()
                .unwrap()
                .keys()
                .all(|k| ROSTER_FIELDS.contains(&k.as_str()))
        );
    }
    let ids: Vec<&str> = rows.iter().filter_map(|r| r["id"].as_str()).collect();
    assert!(ids.windows(2).all(|w| w[0] <= w[1]));
    db.close().await;
}

#[tokio::test]
async fn voices_and_skins_cut_one_operator() {
    let Some((ctx, db, _)) = common::context_with_data().await else {
        return;
    };
    let voices = operators::voices(&ctx, Server::EN, AMIYA).unwrap();
    assert!(!voices.char_words.is_empty());
    assert!(voices.char_words.values().all(|v| v.char_id == AMIYA));
    let skins = operators::skins(&ctx, Server::EN, AMIYA).unwrap();
    assert!(!skins.char_skins.is_empty());
    assert!(
        skins
            .char_skins
            .values()
            .all(|s| s.tmpl_id.as_deref().unwrap_or(&s.char_id) == AMIYA)
    );
    db.close().await;
}

#[tokio::test]
async fn community_stats_are_empty_on_an_empty_database() {
    let Some((ctx, db, snapshot)) = common::context_with_data().await else {
        return;
    };
    let ownership = parse(&operators::ownership(&ctx, Server::EN).await.unwrap().body);
    assert_eq!(ownership["totalUsers"], 0);
    assert!(ownership["counts"].as_object().unwrap().is_empty());
    assert!(ctx.cache.get("operators:ownership:en").await.is_some());

    let stats = parse(
        &operators::build_stats(&ctx, Server::EN, AMIYA)
            .await
            .unwrap()
            .body,
    );
    assert_eq!(stats["operatorId"], AMIYA);
    assert_eq!(stats["minSample"], 50);
    assert!(stats["defaultSkills"].as_array().unwrap().is_empty());
    assert!(stats["masteries"].as_array().unwrap().is_empty());
    assert!(
        ctx.cache
            .get(&snapshot.key(&format!("operators:buildstats:en:{AMIYA}")))
            .await
            .is_some()
    );
    assert!(matches!(
        operators::ownership(&ctx, Server::JP).await,
        Err(AppError::NotFound(_))
    ));
    db.close().await;
}

#[tokio::test]
async fn the_cn_preview_needs_cn_data() {
    let Some((ctx, db, _)) = common::context_with_data().await else {
        return;
    };
    assert!(matches!(
        operators::upcoming(&ctx, Server::EN).await,
        Err(AppError::NotFound(_))
    ));
    assert!(matches!(
        operators::cn_changes(&ctx, Server::EN).await,
        Err(AppError::NotFound(_))
    ));
    db.close().await;
}

#[test]
fn strip_tags_keeps_an_unclosed_bracket() {
    assert_eq!(
        strip_tags("<@rc.eml>Lancet-2</> / Castle-3"),
        "Lancet-2 / Castle-3"
    );
    assert_eq!(strip_tags("a < b"), "a < b");
}

#[test]
fn recruitable_names_skip_headings_and_star_lines() {
    let detail = "<@rc.title>Recruitment rules</>\n\
                  <@rc.subtitle>※Recruitable operators※</>\n\
                  <@rc.em>Note</>\n\
                  ------------------\n\
                  \n\
                  ★\n\
                  ★★★\n\
                  <@rc.eml>Lancet-2</> / Castle-3 / THRM-EX\n\
                  Yato / Noir Corne / Rangers\n\
                  ★★ Steward / Kroos ★";
    let mut names: Vec<_> = recruitable_names(detail).into_iter().collect();
    names.sort_unstable();
    assert_eq!(
        names,
        [
            "Castle-3",
            "Kroos",
            "Lancet-2",
            "Noir Corne",
            "Rangers",
            "Steward",
            "THRM-EX",
            "Yato"
        ]
    );
}

#[tokio::test]
async fn recruitment_lists_the_pool_with_its_tags() {
    let Some((ctx, db, snapshot)) = common::context_with_data().await else {
        return;
    };
    let data = parse(
        &recruitment::recruitment(&ctx, Server::EN)
            .await
            .unwrap()
            .body,
    );
    let tags = data["tags"].as_array().unwrap();
    assert_eq!(tags.len(), snapshot.model.gacha.gacha_tags.len());
    let ops = data["operators"].as_array().unwrap();
    let names = recruitable_names(&snapshot.model.gacha.recruit_detail);
    assert!(ops.len() > 50);
    assert!(
        ops.iter()
            .all(|o| names.contains(o["name"].as_str().unwrap()))
    );
    let ids: Vec<&str> = ops.iter().map(|o| o["id"].as_str().unwrap()).collect();
    assert!(ids.windows(2).all(|w| w[0] < w[1]));
    let lancet = ops.iter().find(|o| o["name"] == "Lancet-2").unwrap();
    assert_eq!(lancet["rarity"], 1);
    assert_eq!(lancet["profession"], "MEDIC");
    assert!(ops.iter().all(|o| o["rarity"] != 0));
    db.close().await;
}
