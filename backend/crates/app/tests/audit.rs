#![allow(clippy::unwrap_used, clippy::unnecessary_wraps)]

use myrtle_app::audit::{AUDIT_SET_MAX, AuditTierRef, AuditTierRename, ContentAuditDetails};
use uuid::Uuid;

fn tier(n: u128, name: &str) -> AuditTierRef {
    AuditTierRef {
        id: Uuid::from_u128(n),
        name: name.into(),
    }
}

fn rename(n: u128, from: &str, to: &str) -> AuditTierRename {
    AuditTierRename {
        id: Uuid::from_u128(n),
        from: from.into(),
        to: to.into(),
    }
}

fn strings(items: &[&str]) -> Option<Vec<String>> {
    Some(items.iter().map(|s| (*s).to_owned()).collect())
}

#[test]
fn entries_moved_twice_count_once() {
    let mut run = ContentAuditDetails {
        entries_moved: strings(&["operator:a", "operator:b"]),
        ..Default::default()
    };
    run.merge(ContentAuditDetails {
        entries_moved: strings(&["operator:b", "operator:c"]),
        ..Default::default()
    });
    assert_eq!(
        run.entries_moved,
        strings(&["operator:a", "operator:b", "operator:c"])
    );
}

#[test]
fn a_set_stops_growing_at_the_cap() {
    let many: Vec<String> = (0..AUDIT_SET_MAX + 50)
        .map(|i| format!("operator:{i}"))
        .collect();
    let mut run = ContentAuditDetails::default();
    run.merge(ContentAuditDetails {
        entries_added: Some(many),
        ..Default::default()
    });
    assert_eq!(run.entries_added.unwrap().len(), AUDIT_SET_MAX);
}

#[test]
fn renames_keep_the_first_from_and_the_last_to() {
    let mut run = ContentAuditDetails {
        renamed_from: Some("Original".into()),
        tiers_renamed: Some(vec![rename(1, "S", "S+")]),
        ..Default::default()
    };
    run.merge(ContentAuditDetails {
        renamed_from: Some("Second".into()),
        tiers_renamed: Some(vec![rename(1, "S+", "God"), rename(2, "A", "B")]),
        ..Default::default()
    });
    assert_eq!(run.renamed_from.as_deref(), Some("Original"));
    assert_eq!(
        run.tiers_renamed,
        Some(vec![rename(1, "S", "God"), rename(2, "A", "B")])
    );
}

#[test]
fn a_tier_renamed_back_drops_out() {
    let mut run = ContentAuditDetails {
        tiers_renamed: Some(vec![rename(1, "S", "S+")]),
        ..Default::default()
    };
    run.merge(ContentAuditDetails {
        tiers_renamed: Some(vec![rename(1, "S+", "S")]),
        ..Default::default()
    });
    assert_eq!(run.tiers_renamed, None);
}

#[test]
fn a_tier_added_then_renamed_reads_as_added_under_its_latest_name() {
    let mut run = ContentAuditDetails {
        tiers_added: Some(vec![tier(1, "New tier")]),
        ..Default::default()
    };
    run.merge(ContentAuditDetails {
        tiers_renamed: Some(vec![rename(1, "New tier", "S")]),
        ..Default::default()
    });
    assert_eq!(run.tiers_added, Some(vec![tier(1, "S")]));
    assert_eq!(run.tiers_renamed, None);
}

#[test]
fn counts_add_and_latest_scalars_win() {
    let mut run = ContentAuditDetails {
        cells_changed: Some(2),
        listed: Some(true),
        size: Some("3 x 3".into()),
        ..Default::default()
    };
    run.merge(ContentAuditDetails {
        cells_changed: Some(5),
        entries_dropped: Some(4),
        listed: Some(false),
        description_changed: Some(true),
        ..Default::default()
    });
    assert_eq!(run.cells_changed, Some(7));
    assert_eq!(run.entries_dropped, Some(4));
    assert_eq!(run.listed, Some(false));
    assert_eq!(run.size.as_deref(), Some("3 x 3"));
    assert_eq!(run.description_changed, Some(true));
}

#[test]
fn stored_json_holds_only_what_happened() {
    let details = ContentAuditDetails {
        version: Some(3),
        ..Default::default()
    };
    let stored = serde_json::to_value(&details).unwrap();
    assert_eq!(stored, serde_json::json!({ "version": 3 }));
    assert_eq!(
        serde_json::from_value::<ContentAuditDetails>(stored).unwrap(),
        details
    );
}
