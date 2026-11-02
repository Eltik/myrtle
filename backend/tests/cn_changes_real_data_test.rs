//! The CN changes diff over the real EN and CN tables. Skips when
//! `assets/output/cn` is absent (CI carries EN only).
//!
//! `cargo test --test cn_changes_real_data_test -- --nocapture` prints every
//! flagged operator and the count per change kind.

mod common;

use std::collections::BTreeMap;

use backend::core::release::operator_changes::{OperatorChange, SharedTexts, cn_changes};
use common::load_server_with_en_art as load;

fn kind(change: &OperatorChange) -> &'static str {
    match change {
        OperatorChange::ModuleAdded { .. } => "module_added",
        OperatorChange::ModuleChanged { .. } => "module_changed",
        OperatorChange::SkillAdded { .. } => "skill_added",
        OperatorChange::SkillValuesChanged { .. } => "skill_values_changed",
        OperatorChange::TalentsChanged => "talents_changed",
        OperatorChange::TraitChanged => "trait_changed",
        OperatorChange::StatsChanged => "stats_changed",
        OperatorChange::RangeChanged => "range_changed",
        OperatorChange::PotentialsChanged => "potentials_changed",
        OperatorChange::TrustBonusChanged => "trust_bonus_changed",
    }
}

#[test]
fn cn_changes_flag_real_differences_only() {
    let (Some(en), Some(cn)) = (load("en"), load("cn")) else {
        eprintln!("assets/output/en or cn absent, skipping");
        return;
    };
    let shared = en
        .operators
        .keys()
        .filter(|id| cn.operators.contains_key(*id))
        .count();
    let changed = cn_changes(&en, &cn);

    let mut by_kind: BTreeMap<&str, usize> = BTreeMap::new();
    for op in &changed {
        println!("{} {:?}", op.id, op.changes);
        for change in &op.changes {
            *by_kind.entry(kind(change)).or_default() += 1;
        }
    }
    println!(
        "flagged {} of {shared} shared operators: {by_kind:?}",
        changed.len()
    );

    // Every flagged operator is on both servers and carries at least one change.
    for op in &changed {
        assert!(en.operators.contains_key(&op.id) && cn.operators.contains_key(&op.id));
        assert!(!op.changes.is_empty());
    }
    // A module CN adds is absent from EN's module list and present in CN's.
    for op in &changed {
        for change in &op.changes {
            if let OperatorChange::ModuleAdded { module_id, .. } = change {
                let has = |gd: &backend::core::gamedata::types::GameData| {
                    gd.operators[&op.id]
                        .modules
                        .iter()
                        .any(|m| &m.module.uni_equip_id == module_id)
                };
                assert!(!has(&en) && has(&cn), "{}: {module_id}", op.id);
            }
        }
    }
    // The skill and module comparisons see real levels and phases on both
    // sides: an enrichment that dropped them would compare empty to empty.
    for gd in [&en, &cn] {
        let shu = &gd.operators["char_2025_shu"];
        assert!(
            shu.skills
                .iter()
                .all(|s| { s.static_data.as_ref().is_some_and(|d| d.levels.len() == 10) })
        );
        assert!(shu.modules.iter().any(|m| m.data.phases.len() == 3));
    }
    // Text differs on every shared operator; a diff that read it would flag
    // nearly all of them.
    assert!(
        changed.len() * 4 < shared,
        "{} of {shared} flagged: is text leaking into the comparison?",
        changed.len()
    );
}

/// `CN_CHANGES_DUMP=<path>` also writes the full response there.
#[test]
fn cn_only_modules_borrow_text_from_shared_modules() {
    let (Some(en), Some(cn)) = (load("en"), load("cn")) else {
        eprintln!("assets/output/en or cn absent, skipping");
        return;
    };
    let shared = SharedTexts::build(&en, &cn);
    println!(
        "{} shared texts, {} ambiguous dropped",
        shared.texts.len(),
        shared.ambiguous
    );
    let changed = cn_changes(&en, &cn);
    for op in &changed {
        for change in &op.changes {
            if let OperatorChange::ModuleAdded {
                module_id,
                translations,
            } = change
            {
                println!("{module_id}: {}", translations.len());
                for t in translations {
                    println!("  {} => {}", t.cn, t.en);
                }
            }
        }
    }
    if let Ok(path) = std::env::var("CN_CHANGES_DUMP") {
        std::fs::write(path, serde_json::to_string(&changed).unwrap()).unwrap();
    }

    // Shu's GUA-Y adds a damage reduction another module both servers have
    // words the same way.
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
