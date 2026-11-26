use std::collections::BTreeSet;
use std::fs;

use myrtle_dps_transpile::hand::{guard_output, write_output};
use myrtle_dps_transpile::init_state::{
    INIT_STATE_ELSEWHERE, InitStateKind, corrected_source, formula_audit,
};
use myrtle_dps_transpile::paths::{Mode, default_repo, hand_dir};

#[test]
fn no_output_path_is_in_the_hand_layer() {
    for mode in [Mode::Damage, Mode::Healing] {
        for path in [
            mode.formulas_json_path(),
            mode.generated_rs_path(),
            mode.dispatch_rs_path(),
            mode.expected_json_path(),
        ] {
            assert!(guard_output(&path).is_ok(), "{} refused", path.display());
        }
    }
}

/// The gate for an upstream pull: a new class that breaks either detector fails here
#[test]
fn upstream_formulas_have_no_unhandled_init_state_or_name_collision() {
    let mut unhandled = Vec::new();
    let mut elsewhere_used = BTreeSet::new();
    for mode in [Mode::Damage, Mode::Healing] {
        let src = corrected_source(mode, &default_repo()).expect("read upstream formulas");
        let (findings, collisions) = formula_audit(mode, &src);
        for f in &findings {
            match &f.kind {
                InitStateKind::Unhandled(why) => {
                    unhandled.push(format!("{}.{} ({why}): {}", f.class_name, f.field, f.line));
                }
                InitStateKind::Elsewhere(_) => {
                    elsewhere_used.insert((f.class_name.clone(), f.field.clone()));
                }
                _ => {}
            }
        }
        for c in collisions.iter().filter(|c| c.self_assigned) {
            unhandled.push(format!(
                "{}: local {} collides with self.{} and the body rebinds self.{} too",
                c.class_name, c.name, c.name, c.name
            ));
        }
    }
    assert!(
        unhandled.is_empty(),
        "dps-transpile would emit wrong numbers for:\n{}",
        unhandled.join("\n")
    );
    for (class, field, _) in INIT_STATE_ELSEWHERE {
        assert!(
            elsewhere_used.contains(&((*class).to_string(), (*field).to_string())),
            "INIT_STATE_ELSEWHERE {class}.{field} matches nothing upstream: delete it"
        );
    }
}

#[test]
fn guard_refuses_the_hand_layer() {
    for path in [
        hand_dir().join("angelina_alter.rs"),
        hand_dir().join("../hand/./formulas.json"),
        hand_dir().join("../../custom/hand/new_formula.rs"),
    ] {
        let err = guard_output(&path).expect_err("hand layer path accepted");
        assert!(err.to_string().contains("hand-written"), "{err}");
    }
}

#[test]
fn write_output_replaces_contents_and_refuses_the_hand_layer() {
    let dir = tempfile::tempdir().expect("temp dir");
    let target = dir.path().join("nested/out.json");
    write_output(&target, b"first").expect("first write");
    write_output(&target, b"second").expect("second write");
    assert_eq!(fs::read(&target).expect("read back"), b"second");
    let names: Vec<_> = fs::read_dir(target.parent().expect("parent"))
        .expect("list dir")
        .map(|e| e.expect("entry").file_name())
        .collect();
    assert_eq!(names, ["out.json"], "temp file left behind");

    let hand_target = hand_dir().join("formulas.json");
    let before = fs::read(&hand_target).expect("read hand formulas");
    let err = write_output(&hand_target, b"{}").expect_err("hand layer write accepted");
    assert!(err.to_string().contains("hand-written"), "{err}");
    assert_eq!(
        fs::read(&hand_target).expect("reread hand formulas"),
        before
    );
}
