#![allow(clippy::unwrap_used, clippy::expect_used)]
use std::collections::HashMap;

use myrtle_engine_release::translate::*;
use myrtle_gamedata::model::GameModel;
use myrtle_gamedata_raw::tables::activity::ActivityBasicInfo;

#[test]
fn memory_pairs_shared_ids_and_skips_identical_or_empty() {
    let mut cn = GameModel::default();
    let mut en = GameModel::default();
    for (id, c, e) in [
        ("act1", "月行水上", "Moonlight on Water"),
        ("act2", "same", "same"),
        ("act3", "空", ""),
    ] {
        cn.activities.insert(
            id.into(),
            ActivityBasicInfo {
                id: id.into(),
                name: c.into(),
                ..Default::default()
            },
        );
        en.activities.insert(
            id.into(),
            ActivityBasicInfo {
                id: id.into(),
                name: e.into(),
                ..Default::default()
            },
        );
    }
    let m = TranslationMemory::build(&cn, &en, &HashMap::new());
    assert_eq!(m.len(), 1);
    assert_eq!(m.get("月行水上"), Some("Moonlight on Water"));
    assert_eq!(m.get("same"), None);
}

#[test]
fn resolve_answers_only_from_memory() {
    let m = TranslationMemory::build(
        &GameModel::default(),
        &GameModel::default(),
        &HashMap::new(),
    );
    assert_eq!(resolve(&m, "未知名称"), None);
    assert_eq!(resolve(&m, "   "), None);
}

#[test]
fn group_names_borrow_the_series_of_a_paired_edition() {
    let m = TranslationMemory::from_pairs(&[
        ("斗争血脉/VIII", "Bloodline of Combat/VIII"),
        ("怪物猎人", "Monster Hunter"),
        ("0011/韵系列/VIII", "0011/Yun/VIII"),
        ("时代/XXXIII", "Epoque/XXXIII"),
        ("时代/XLVIII", "EPOQUE/XLVIII"),
        ("时代/XLIX", "EPOQUE/XLIX"),
        ("忒斯特收藏/XVI", "Test Collection/XV"),
    ]);
    let text = |cn: &str| resolve_group(&m, cn).map(|a| a.text);
    assert_eq!(
        text("斗争血脉/XI").as_deref(),
        Some("Bloodline of Combat/XI")
    );
    assert_eq!(text("怪物猎人/II").as_deref(), Some("Monster Hunter/II"));
    assert_eq!(text("0011/韵系列/X").as_deref(), Some("0011/Yun/X"));
    assert_eq!(text("时代/LIV").as_deref(), Some("EPOQUE/LIV"));
    assert_eq!(
        text("斗争血脉/VIII").as_deref(),
        Some("Bloodline of Combat/VIII")
    );
    assert_eq!(text("忒斯特收藏/XX"), None);
    assert_eq!(text("小马宝莉"), None);
}
