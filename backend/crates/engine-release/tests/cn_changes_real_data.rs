//! The CN changes diff over the real EN and CN tables. Skips without both extracts.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use myrtle_engine_release::operator_changes::{OperatorChange, cn_changes, shared_operator_count};
use myrtle_gamedata::model::GameModel;
use myrtle_testkit::fixtures::character_model;
use myrtle_types::Server;

fn both() -> Option<(&'static GameModel, &'static GameModel)> {
    let pair = character_model(Server::EN).zip(character_model(Server::CN));
    if pair.is_none() {
        eprintln!("assets/output/en or cn absent, skipping");
    }
    pair
}

#[test]
fn cn_changes_flag_real_differences_only() {
    let Some((en, cn)) = both() else {
        return;
    };
    let shared = shared_operator_count(en, cn);
    let changed = cn_changes(en, cn);

    for op in &changed {
        assert!(en.operators.contains_key(&op.id) && cn.operators.contains_key(&op.id));
        assert!(!op.changes.is_empty(), "{}", op.id);
        for change in &op.changes {
            if let OperatorChange::ModuleAdded { module_id, .. } = change {
                let has = |gd: &GameModel| {
                    gd.operators[&op.id]
                        .modules
                        .iter()
                        .any(|m| &m.module.uni_equip_id == module_id)
                };
                assert!(!has(en) && has(cn), "{}: {module_id}", op.id);
            }
        }
    }
    // An enrichment that dropped levels or phases would compare empty to empty.
    for gd in [en, cn] {
        let shu = &gd.operators["char_2025_shu"];
        assert!(
            shu.skills
                .iter()
                .all(|s| s.static_data.as_ref().is_some_and(|d| d.levels.len() == 10))
        );
        assert!(shu.modules.iter().any(|m| m.data.phases.len() == 3));
    }
    // Text differs on every shared operator, so a diff that read it would flag nearly all.
    assert!(
        changed.len() * 4 < shared,
        "{} of {shared} flagged: is text leaking into the comparison?",
        changed.len()
    );
}

#[test]
fn cn_only_modules_borrow_text_from_shared_modules() {
    let Some((en, cn)) = both() else {
        return;
    };
    let changed = cn_changes(en, cn);
    let shu = changed.iter().find(|o| o.id == "char_2025_shu").unwrap();
    assert!(
        shu.changes.iter().any(|c| matches!(
            c,
            OperatorChange::ModuleAdded { translations, .. }
                if translations.iter().any(|t| t.cn == "受到的伤害减少<@ba.kw>15%</>"
                    && t.en == "Reduces damage taken by <@ba.kw>15%</>")
        )),
        "{:?}",
        shu.changes
    );
}
