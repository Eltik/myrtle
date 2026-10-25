//! `OperatorData` built once per operator and game-data load.
//!
//! `OperatorData::new` walks every skill level, talent candidate and module of
//! an operator and, before this cache, ran on a full clone of the `Operator`
//! for every request. It is a pure function of the `Operator`, so a cached
//! value computes exactly what a fresh one would.
//!
//! The cache is keyed by the identity of the `GameData` allocation: a reload
//! swaps in a new `Arc<GameData>`, which is a different key, and the entries
//! of a load nobody holds any more are dropped on the next lookup. A `Weak`
//! keeps the old allocation's address from being reused while its entry
//! exists, so a new load can never alias an old one.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, Weak};

use crate::core::gamedata::types::GameData;

use super::operator_data::OperatorData;

struct Generation {
    owner: Weak<GameData>,
    operators: HashMap<String, Arc<OperatorData>>,
}

static CACHE: Mutex<Vec<Generation>> = Mutex::new(Vec::new());

/// The operator's `OperatorData` for this game-data load, or `None` when the
/// load has no such operator.
pub fn operator_data(game_data: &Arc<GameData>, operator_id: &str) -> Option<Arc<OperatorData>> {
    let owner = Arc::downgrade(game_data);
    {
        let cache = CACHE
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(hit) = cache
            .iter()
            .find(|g| g.owner.ptr_eq(&owner))
            .and_then(|g| g.operators.get(operator_id))
        {
            return Some(Arc::clone(hit));
        }
    }

    // Built outside the lock: two racing misses build twice, harmlessly.
    let operator = game_data.operators.get(operator_id)?;
    let built = Arc::new(OperatorData::new(operator.clone()));

    let mut cache = CACHE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    cache.retain(|g| g.owner.strong_count() > 0);
    let generation = if let Some(i) = cache.iter().position(|g| g.owner.ptr_eq(&owner)) {
        &mut cache[i]
    } else {
        cache.push(Generation {
            owner,
            operators: HashMap::new(),
        });
        cache.last_mut().expect("just pushed")
    };
    Some(Arc::clone(
        generation
            .operators
            .entry(operator_id.to_owned())
            .or_insert(built),
    ))
}

/// A content identity of one game-data load, for cache keys that outlive the
/// process (Redis): SHA-256 over the operator table, the only table the DPS
/// and HPS engines read (`OperatorData::new` takes an `Operator` and nothing
/// else), serialized through `serde_json::Value` so every map is key-sorted
/// and the bytes do not depend on `HashMap` iteration order. 16 hex digits.
///
/// Two loads with identical operators share an identity, which is correct:
/// they compute identical results. Computed once per load and kept while the
/// load is alive, like `operator_data`.
pub fn game_data_identity(game_data: &Arc<GameData>) -> Arc<str> {
    if let Some(hit) = cached_identity(game_data) {
        return hit;
    }
    use sha2::{Digest, Sha256};
    let value = serde_json::to_value(&game_data.operators).unwrap_or_default();
    let bytes = serde_json::to_vec(&value).unwrap_or_default();
    let digest = Sha256::digest(&bytes);
    let id: Arc<str> = Arc::from(hex::encode(&digest[..8]));

    let mut ids = IDENTITIES
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    ids.retain(|(owner, _)| owner.strong_count() > 0);
    if !ids
        .iter()
        .any(|(owner, _)| owner.ptr_eq(&Arc::downgrade(game_data)))
    {
        ids.push((Arc::downgrade(game_data), Arc::clone(&id)));
    }
    id
}

/// The identity when it has already been computed for this load.
pub fn cached_identity(game_data: &Arc<GameData>) -> Option<Arc<str>> {
    let owner = Arc::downgrade(game_data);
    IDENTITIES
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .iter()
        .find(|(o, _)| o.ptr_eq(&owner))
        .map(|(_, id)| Arc::clone(id))
}

static IDENTITIES: Mutex<Vec<(Weak<GameData>, Arc<str>)>> = Mutex::new(Vec::new());
