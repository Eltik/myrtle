//! Where the max-level card stops each operator when it prices to the module
//! level, against real game data: every module of a rarity unlocks at one
//! point (4-star E2 40, 5-star E2 50, 6-star E2 60), an operator without a
//! module takes its rarity's point, and 1 to 3 stars keep their cap.
mod common;

use std::collections::HashMap;

use backend::app::services::max_level::{LevelTarget, module_targets_by_rarity, operator_target};
use common::load_game_data;

#[test]
fn every_rarity_with_modules_unlocks_them_at_one_point() {
    let gd = load_game_data();
    let by_rarity = module_targets_by_rarity(gd);
    let expected: HashMap<i16, (i16, i16)> =
        HashMap::from([(4, (2, 40)), (5, (2, 50)), (6, (2, 60))]);
    assert_eq!(by_rarity, expected, "earliest module unlock per rarity");
}

#[test]
fn module_target_is_the_rarity_point_or_the_cap() {
    let gd = load_game_data();
    let by_rarity = module_targets_by_rarity(gd);
    let mut without_module = 0usize;
    for (id, operator) in &gd.operators {
        let Some(cap) = operator_target(operator, LevelTarget::Max, &by_rarity) else {
            continue;
        };
        let last = operator.phases.last().expect("a phase");
        assert_eq!(
            cap,
            (
                i16::try_from(operator.phases.len() - 1).expect("elite"),
                i16::try_from(last.max_level).expect("level")
            ),
            "{}: max target is the cap",
            id
        );
        let module = operator_target(operator, LevelTarget::Module, &by_rarity).expect("a target");
        let rarity = operator.rarity.to_star_int();
        match by_rarity.get(&rarity) {
            Some(point) => {
                assert_eq!(
                    module, *point,
                    "{}: module target is its rarity's point",
                    id
                );
                if operator.modules.len() <= 1 {
                    without_module += 1;
                }
            }
            None => assert_eq!(
                module, cap,
                "{}: no modules for the rarity, keeps the cap",
                id
            ),
        }
        assert!(module <= cap, "{}: never past the cap", id);
    }
    // The fallback is exercised: some 4-star-plus operators have no module yet.
    assert!(
        without_module > 0,
        "operators without a module take the rarity's point"
    );
}
