//! Module ordering and identity.
//!
//! `get_operator_modules` builds each operator's list by iterating
//! `equip_dict`, a `HashMap`. Rust seeds those per process, so before this was
//! sorted the same operator came back in a different order after every restart:
//! stable while the process lived, different the next time it started. Anything
//! downstream reading position - a default selection, a dropdown, a "Mod N"
//! label - therefore named a different module on each deploy.
//!
//! The fix is a total order, so these tests assert the property that makes the
//! output reproducible rather than any one arrangement: a total order has
//! exactly one valid sequence, so asserting the list already matches it proves
//! the result cannot vary with the hash seed.

mod common;

use common::load_game_data;

/// Key the list is required to be in. The uniequip number, not
/// `char_equip_order`: the latter reads [0, 2, 1] on 17 operators and would put
/// uniequip_003 ahead of _002. `uni_equip_id` breaks ties so the order is total.
fn order_key(m: &backend::core::gamedata::types::operator::OperatorModule) -> (i32, &str) {
    let id = m.module.uni_equip_id.as_str();
    let n = id
        .split('_')
        .nth(1)
        .and_then(|p| p.parse::<i32>().ok())
        .unwrap_or(i32::MAX);
    (n, id)
}

#[test]
fn operator_modules_are_in_a_total_deterministic_order() {
    let gd = load_game_data();

    let mut checked = 0usize;
    let mut multi = 0usize;
    for (char_id, operator) in &gd.operators {
        let got: Vec<_> = operator.modules.iter().map(order_key).collect();
        let mut want = got.clone();
        want.sort_unstable();

        assert_eq!(
            got, want,
            "{char_id}: modules are not in (char_equip_order, uni_equip_id) order. \
             This list is built from a HashMap, so an unsorted list is a different \
             list on the next restart."
        );

        checked += 1;
        if operator.modules.len() > 1 {
            multi += 1;
        }
    }

    assert!(checked > 0, "no operators loaded");
    assert!(
        multi > 100,
        "expected many multi-module operators, saw {multi} - is the fixture right?"
    );
}

#[test]
fn module_order_key_is_unique_per_operator() {
    // If two modules on one operator shared a key the sort would not be total
    // and the hash seed could still leak through.
    let gd = load_game_data();
    for (char_id, operator) in &gd.operators {
        let mut keys: Vec<_> = operator.modules.iter().map(order_key).collect();
        let before = keys.len();
        keys.sort_unstable();
        keys.dedup();
        assert_eq!(
            keys.len(),
            before,
            "{char_id}: duplicate (char_equip_order, uni_equip_id) key - ordering is not total"
        );
    }
}

#[test]
fn every_module_belongs_to_the_operator_that_lists_it() {
    // Guards the filter as well as the sort: a module must not migrate between
    // operators, which is the other way a designator ends up on the wrong entry.
    let gd = load_game_data();
    for (char_id, operator) in &gd.operators {
        for m in &operator.modules {
            let owner = m.module.tmpl_id.as_deref().unwrap_or(&m.module.char_id);
            assert_eq!(
                owner, char_id,
                "module {} is listed under {char_id} but belongs to {owner}",
                m.module.uni_equip_id
            );
        }
    }
}

/// The assertion that actually matters: the order we serve is the order the game
/// shows. `char_equip` is the game's own per-character module list, so this pins
/// the ordering to the source of truth rather than to whichever key we picked.
/// A stable but wrong order would pass the tests above and fail this one.
#[test]
fn operator_modules_match_the_games_own_char_equip_order() {
    let gd = load_game_data();
    let mut compared = 0usize;

    for (char_id, operator) in &gd.operators {
        let Some(expected) = gd.modules.char_equip.get(char_id) else {
            continue;
        };
        // `char_equip` lists ids the current data may not all carry; compare on
        // the intersection, in the game's sequence.
        let known: Vec<&str> = expected
            .iter()
            .filter(|id| {
                operator
                    .modules
                    .iter()
                    .any(|m| &m.module.uni_equip_id == *id)
            })
            .map(String::as_str)
            .collect();
        if known.len() < 2 {
            continue;
        }
        let got: Vec<&str> = operator
            .modules
            .iter()
            .map(|m| m.module.uni_equip_id.as_str())
            .filter(|id| known.contains(id))
            .collect();

        assert_eq!(
            got, known,
            "{char_id}: served order does not match the game's char_equip order"
        );
        compared += 1;
    }

    assert!(
        compared > 300,
        "expected to compare hundreds of operators, compared {compared}"
    );
}
