//! Hidden factions (`SubPower` ids outside `MainPower`) on the real EN tree.
//!
//! Skips when `assets/output/en` is absent.

mod common;

use common::load_server_with_en_art as load;

fn hidden(data: &backend::core::gamedata::types::GameData, id: &str) -> Vec<String> {
    data.operators[id]
        .hidden_factions
        .iter()
        .map(|f| f.id.clone())
        .collect()
}

#[test]
fn hidden_factions_are_sub_powers_the_card_does_not_show() {
    let Some(en) = load("en") else {
        eprintln!("skipping: assets/output/en missing");
        return;
    };

    let (chars, others): (Vec<&String>, Vec<&String>) = en
        .operators
        .iter()
        .filter(|(_, op)| !op.hidden_factions.is_empty())
        .map(|(id, _)| id)
        .partition(|id| id.starts_with("char_"));
    eprintln!(
        "en operators with a hidden faction: {} char_, others {others:?}",
        chars.len()
    );
    // 80 on the EN tree of 2026-10-11: 78 `character_table` operators plus
    // Amiya's two `char_patch_table` forms, which inherit her `rim`. The
    // three Laterano cosplayer traps carry a `SubPower` too and are not
    // operators. New operators only add to it.
    assert!(chars.len() >= 80, "{}", chars.len());

    assert_eq!(hidden(&en, "char_113_cqbw"), ["rhodes"]);
    assert_eq!(hidden(&en, "char_1028_texas2"), ["siracusa"]);
    // Reed's entry is `tara` + `victoria`; Victoria is already her nation.
    assert_eq!(hidden(&en, "char_261_sddrag"), ["tara"]);
    // Mostima's entry names a group with its nation, both new to her.
    assert_eq!(hidden(&en, "char_213_mostma"), ["penguin", "lungmen"]);
    assert_eq!(hidden(&en, "char_017_huang"), ["yan", "victoria"]);
    // No `SubPower`: nothing hidden, whatever the card shows.
    assert!(hidden(&en, "char_1012_skadi2").is_empty());
    assert!(hidden(&en, "char_003_kalts").is_empty());
    assert!(hidden(&en, "char_456_ash").is_empty());

    let w = &en.operators["char_113_cqbw"].hidden_factions[0];
    assert_eq!(w.name.as_deref(), Some("Rhodes Island"));
}
