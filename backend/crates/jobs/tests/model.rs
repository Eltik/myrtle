#![allow(clippy::unwrap_used)]

use std::collections::HashSet;

use myrtle_jobs::{JobKind, JobSpec, JobState};

#[test]
fn a_kind_is_stored_under_its_wire_name() {
    for kind in JobKind::ALL {
        let wire = serde_json::to_value(kind).unwrap();
        assert_eq!(wire, kind.as_str(), "{kind:?}");
        assert_eq!(JobKind::parse(kind.as_str()), Some(kind));
    }
    assert_eq!(JobKind::parse("asset_reload"), None);
}

#[test]
fn kind_names_are_distinct() {
    let names: HashSet<&str> = JobKind::ALL.iter().map(|k| k.as_str()).collect();
    assert_eq!(names.len(), JobKind::ALL.len());
}

#[test]
fn every_kind_has_a_lease_and_an_attempt() {
    for kind in JobKind::ALL {
        assert!(kind.lease_secs() > 0, "{kind:?}");
        assert!(kind.max_attempts() > 0, "{kind:?}");
    }
}

#[test]
fn a_state_is_stored_under_its_wire_name() {
    for state in [
        JobState::Queued,
        JobState::Running,
        JobState::Succeeded,
        JobState::Dead,
    ] {
        assert_eq!(serde_json::to_value(state).unwrap(), state.as_str());
        assert_eq!(JobState::parse(state.as_str()), Some(state));
    }
    assert_eq!(JobState::parse("cancelled"), None);
}

#[test]
fn a_new_spec_runs_now_with_an_empty_payload() {
    let spec = JobSpec::new(JobKind::RegradeUser);
    assert_eq!(spec.payload, serde_json::json!({}));
    assert_eq!(spec.priority, 0);
    assert!(spec.run_at.is_none() && spec.dedupe_key.is_none() && spec.server.is_none());
}
