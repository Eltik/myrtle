//! The counting pass over `sandbox_perm_table.json` replaces two
//! `serde_json::Value` parses (`SANDBOX_VALUE_PARSE=1` restores them). Pins
//! that on the real tables the two paths give the IDENTICAL 11 universe counts
//! and the identical set of `(level id, code, name)` triples the stage
//! classifier names Reclamation Algorithm nodes with.
//!
//! Runs on every server tree present under `../assets/output`; a server
//! without the table is skipped, and the test fails if none has it.

use std::path::Path;

use backend::core::gamedata::enrich::stage_class::sandbox_perm_level_named_from_value;
use backend::core::gamedata::tables::load_table;
use backend::core::gamedata::types::sandbox_perm_scan::scan_sandbox_perm;
use backend::core::gamedata::types::sandbox_universe::SandboxUniverse;

#[test]
fn counting_pass_matches_the_value_parse() {
    let mut compared = 0;
    for server in ["en", "cn", "jp", "kr"] {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("../assets/output/{server}/gamedata/excel"));
        if !dir.join("sandbox_perm_table.json").is_file() {
            eprintln!("{server}: no sandbox_perm_table; skipped");
            continue;
        }

        let raw: serde_json::Value = load_table(&dir, "sandbox_perm_table").expect("value parse");
        let old_universe = SandboxUniverse::build(&raw);
        drop(raw);
        let old_named = sandbox_perm_level_named_from_value(&dir);

        let scan = scan_sandbox_perm(&dir).expect("counting pass");

        assert_eq!(
            old_universe, scan.universe,
            "{server}: universe counts differ"
        );
        assert_eq!(
            old_named, scan.level_named,
            "{server}: classifier triples differ"
        );
        assert!(
            scan.universe.max_stages > 0 && scan.universe.max_nodes > 0,
            "{server}: the table carries SANDBOX_V2 counts"
        );
        assert!(
            !scan.level_named.is_empty(),
            "{server}: the table names levels"
        );
        eprintln!(
            "{server}: {:?}, {} level triples",
            scan.universe,
            scan.level_named.len()
        );
        compared += 1;
    }
    assert!(compared > 0, "no server tree carries sandbox_perm_table");
}
