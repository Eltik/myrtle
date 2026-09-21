//! Guards against the quietest failure mode this backend has.
//!
//! `load_table_or_warn` catches a deserialization error, logs one line, and
//! substitutes `T::default()`. The service then serves an empty table as if it
//! were real data: no error, a passing health check, a passing smoke test. In
//! September 2026 six CN items carrying `ClassifyType: "MEMENTO"` (a value the
//! `ItemClass` enum did not name) failed the whole `item_table`, so all 1,556
//! CN items lost their names and icons. It was found by a user noticing a broken
//! image, not by anything in CI.
//!
//! Hypergryph ships new enum values on CN first, so CN is the canary. Point
//! `GAME_DATA_DIR`/`ASSETS_DIR` at a CN extract in a second CI job to get the
//! warning before it reaches production.

mod common;

/// No table may fall back to its default.
///
/// A failure here names the table and the serde error, which is normally enough
/// to spot the culprit: most often an enum missing a variant. The fix is
/// usually `#[serde(other)] Unknown` on that enum rather than adding the one new
/// variant, since the next unnamed value would break it again.
#[test]
fn no_table_falls_back_to_default() {
    let gd = common::load_game_data();
    assert!(
        gd.table_warnings.is_empty(),
        "{} table(s) failed to load and were replaced by empty defaults:\n  {}",
        gd.table_warnings.len(),
        gd.table_warnings.join("\n  ")
    );
}

/// A table can also deserialize "successfully" into nothing. These four carry the
/// site: an empty one is a visible outage, so assert they have real content
/// rather than merely having parsed.
#[test]
fn core_tables_are_populated() {
    let gd = common::load_game_data();
    assert!(
        !gd.operators.is_empty(),
        "character_table produced no operators"
    );
    assert!(
        !gd.materials.items.is_empty(),
        "item_table produced no items"
    );
    assert!(!gd.skills.is_empty(), "skill_table produced no skills");
    assert!(
        !gd.voices.char_words.is_empty(),
        "charword_table produced no voice lines"
    );
}

/// The 2020 vignettes (`act4d0` SW-EV, `act6d5` AF, `act7d5` SA) mount their
/// stage nodes on `main_1..main_6`, so a zone-type rule alone files them as
/// permanent and the Score tab lists a one-time 2020 event as a permanent gap.
/// They are `StageType::Activity`; that must route them to the event pool with
/// the activity's own window, where a closed limited event is not gradeable.
#[test]
fn vignette_stages_in_mainline_zones_are_events_not_permanent() {
    use backend::core::gamedata::types::stage::StageType;
    use backend::core::grade::stages::event::event_is_gradeable;

    let gd = common::load_game_data();
    let universe = &gd.stage_universe;

    let activity_typed_permanent: Vec<&str> = universe
        .permanent
        .iter()
        .filter(|e| gd.stages.get(&e.stage_id).map(|s| &s.stage_type) == Some(&StageType::Activity))
        .map(|e| e.stage_id.as_str())
        .collect();
    assert!(
        activity_typed_permanent.is_empty(),
        "ACTIVITY-typed stages in the permanent pool: {activity_typed_permanent:?}"
    );

    let vignettes: Vec<_> = universe
        .event
        .iter()
        .filter(|e| {
            ["act4d0_", "act6d5_", "act7d5_"]
                .iter()
                .any(|p| e.stage_id.starts_with(p))
        })
        .collect();
    assert_eq!(
        vignettes.len(),
        19,
        "expected the 19 vignette stages in the event pool"
    );

    let now = 1_789_000_000; // 2026-09
    for entry in vignettes {
        assert!(
            !entry.is_permanent,
            "{} must not be a permanent event",
            entry.stage_id
        );
        assert!(
            entry.start_time.is_some() && entry.end_time.is_some(),
            "{} must carry its activity window",
            entry.stage_id
        );
        assert!(
            !event_is_gradeable(entry, now, None, None),
            "{} is a closed one-time event and must not be gradeable",
            entry.stage_id
        );
    }
}

/// `act21side_06_m` (IS-QT, Penguin Logistics Office) is reached only through
/// the `OR` ring `taskRing_Texas_4` in Il Siracusano's hub: its battle task and
/// a story task lock each other, so the player who read the story can never
/// clear the stage. It must be in the pools as optional, never as a gap.
///
/// The set is derived from the hub's own `LogicType`, so it is asserted whole.
/// A new member means a new event shipped the either/or mechanic: verify in
/// game that the stage really is unreachable after the other arm before
/// widening this assertion.
#[test]
fn or_ring_battle_stages_are_optional_never_gaps() {
    let gd = common::load_game_data();
    let universe = &gd.stage_universe;

    let mut optional: Vec<&str> = universe
        .permanent
        .iter()
        .filter(|e| e.optional)
        .map(|e| e.stage_id.as_str())
        .chain(
            universe
                .event
                .iter()
                .filter(|e| e.optional)
                .map(|e| e.stage_id.as_str()),
        )
        .collect();
    optional.sort_unstable();
    assert_eq!(
        optional,
        vec!["act21side_06_m"],
        "the optional set is derived from OR rings alone; a new member is a new event \
         with the either/or mechanic and must be verified in game before this assertion \
         is widened, and a missing member means the derivation stopped reading the hub"
    );

    let sibling_optional = universe
        .event
        .iter()
        .find(|e| e.stage_id == "act21side_06_t")
        .map(|e| e.optional)
        .expect("act21side_06_t is in the event pool");
    assert!(
        !sibling_optional,
        "act21side_06_t sits in a LINEAR ring and is fully reachable; the `_m`/`_t` \
         suffix is not the discriminator"
    );
}
