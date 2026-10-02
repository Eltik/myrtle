//! Change detection for the job sidecars under `derived/`.
//!
//! The pool-detail and event-shop jobs re-fetch every live banner and open shop
//! on each 6 h tick, so nearly every run "fetched" something, and each one ended
//! in a full `perform_reload` of that server even when the answers matched the
//! file byte for byte. These helpers let a job compare the data part of its
//! sidecar before and after a run and reload only on a real change.

use std::sync::Mutex;

use serde::Serialize;
use serde_json::Value;

use crate::core::hypergryph::constants::Server;

/// Servers whose sidecars on disk may be ahead of the loaded `GameData`: a job
/// wrote a change, or a reload failed. Comparing against the file alone would
/// miss both, because the file already holds the new data, so the next run
/// reloads while a server is in here. A successful reload takes it out when it
/// starts reading, so a write that lands during the build stays pending.
static PENDING: Mutex<Vec<Server>> = Mutex::new(Vec::new());

pub fn mark_pending(server: Server) {
    let mut pending = PENDING
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if !pending.contains(&server) {
        pending.push(server);
    }
}

pub fn is_pending(server: Server) -> bool {
    PENDING
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .contains(&server)
}

/// Clears the flag for `server`, returning whether it was set.
pub fn take_pending(server: Server) -> bool {
    let mut pending = PENDING
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let before = pending.len();
    pending.retain(|s| *s != server);
    pending.len() != before
}

/// The comparable form of a sidecar's data part, without the run metadata
/// (`fetched_at` moves on every run and would make every comparison differ).
///
/// `serde_json::Value` maps compare by key, not insertion order, so two
/// `HashMap`s holding the same entries compare equal.
#[derive(Debug, Clone)]
pub struct Snapshot(Option<Value>);

impl Snapshot {
    pub fn of<T: Serialize + ?Sized>(data: &T) -> Self {
        Self(serde_json::to_value(data).ok())
    }

    /// True when the two differ. A side that failed to serialize counts as
    /// changed, so a comparison failure can only cost a reload, never skip one.
    pub fn differs(&self, other: &Self) -> bool {
        match (&self.0, &other.0) {
            (Some(a), Some(b)) => a != b,
            _ => true,
        }
    }
}

/// `SIDECAR_RELOAD_ALWAYS=1` restores the previous behaviour exactly: write the
/// sidecar and reload on every run that fetched anything. Unset, or any other
/// value, compares first.
pub fn reload_always() -> bool {
    parse_reload_always(std::env::var("SIDECAR_RELOAD_ALWAYS").ok().as_deref())
}

fn parse_reload_always(raw: Option<&str>) -> bool {
    match raw {
        None => false,
        Some(v) => v.trim() == "1",
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn pending_is_per_server_and_taken_once() {
        // Bilibili is never touched by the jobs' own tests, so this cannot race them.
        let server = Server::Bilibili;
        assert!(!take_pending(server));
        mark_pending(server);
        mark_pending(server);
        assert!(is_pending(server));
        assert!(!is_pending(Server::EN));
        assert!(take_pending(server));
        assert!(!is_pending(server));
        assert!(!take_pending(server));
    }

    #[test]
    fn insertion_order_does_not_count_as_a_change() {
        let mut a = HashMap::new();
        let mut b = HashMap::new();
        for i in 0..64 {
            a.insert(format!("pool_{i}"), i);
        }
        for i in (0..64).rev() {
            b.insert(format!("pool_{i}"), i);
        }
        assert!(!Snapshot::of(&a).differs(&Snapshot::of(&b)));
    }

    #[test]
    fn a_changed_value_or_key_is_a_change() {
        let a: HashMap<&str, i32> = [("x", 1), ("y", 2)].into();
        let changed: HashMap<&str, i32> = [("x", 1), ("y", 3)].into();
        let added: HashMap<&str, i32> = [("x", 1), ("y", 2), ("z", 0)].into();
        assert!(Snapshot::of(&a).differs(&Snapshot::of(&changed)));
        assert!(Snapshot::of(&a).differs(&Snapshot::of(&added)));
    }

    #[test]
    fn an_unserializable_side_always_differs() {
        // Non-string map keys do not serialize to JSON.
        let bad: HashMap<(i32, i32), i32> = [((1, 2), 3)].into();
        let bad = Snapshot::of(&bad);
        assert!(bad.differs(&bad.clone()));
        assert!(bad.differs(&Snapshot::of(&1)));
    }

    #[test]
    fn always_switch_needs_an_explicit_one() {
        assert!(!parse_reload_always(None));
        assert!(!parse_reload_always(Some("")));
        assert!(!parse_reload_always(Some("0")));
        assert!(!parse_reload_always(Some("true")));
        assert!(parse_reload_always(Some("1")));
        assert!(parse_reload_always(Some(" 1\n")));
    }
}
