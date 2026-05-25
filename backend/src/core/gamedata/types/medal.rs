//! Medal game data types
//!
//! Contains types for parsing medal_table.json and storing processed medal data
//! for use in user scoring calculations.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Root structure for medal_table.json
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MedalTableFile {
    pub medal_list: Vec<MedalDefinition>,
    pub medal_type_data: Vec<MedalTypeEntry>,
}

/// Individual medal definition from game data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MedalDefinition {
    pub medal_id: String,
    pub medal_name: String,
    pub medal_type: String,
    pub slot_id: i32,
    #[serde(default)]
    pub pre_medal_id_list: Vec<String>,
    pub rarity: String,
    #[serde(default)]
    pub template: String, // Some medals don't have this
    #[serde(default)]
    pub unlock_param: Vec<String>,
    #[serde(default)]
    pub get_method: String, // Some medals don't have this
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub display_time: i64,
    #[serde(default)]
    pub expire_times: Vec<ExpireTime>,
    #[serde(default)]
    pub medal_reward_group: serde_json::Value, // Complex nested structure, use Value for flexibility
    #[serde(default)]
    pub is_hidden: bool,
    /// "Upgrade variant" medals (e.g. `medal_*_035`) reference their base medal here
    /// so the in-game client can reuse its description. Empty when the medal is
    /// standalone.
    #[serde(default)]
    pub origin_medal: String,
}

/// Time window for medal availability
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ExpireTime {
    pub start: i64,
    pub end: i64,
    #[serde(rename = "Type_")]
    pub expire_type: String,
}

/// Medal type/category entry with groups
#[derive(Debug, Clone, Deserialize)]
pub struct MedalTypeEntry {
    pub key: String,
    pub value: MedalTypeData,
}

/// Medal category metadata
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MedalTypeData {
    pub medal_group_id: String,
    pub sort_id: i32,
    pub medal_name: String,
    #[serde(default)]
    pub group_data: Vec<MedalGroup>,
}

/// Medal group (themed set of medals)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MedalGroup {
    pub group_id: String,
    pub group_name: String,
    pub group_desc: String,
    pub medal_id: Vec<String>,
    pub sort_id: i32,
    pub group_back_color: String,
    pub group_get_time: i64,
    #[serde(default)]
    pub shared_expire_times: Vec<ExpireTime>,
}

/// Processed medal data for efficient lookups during scoring
#[derive(Debug, Clone, Default)]
pub struct MedalData {
    /// All medals indexed by medal_id
    pub medals: HashMap<String, MedalDefinition>,
    /// All medal groups indexed by group_id
    pub groups: HashMap<String, MedalGroup>,
    /// Medals organized by type/category (e.g., "playerMedal" -> vec of medal_ids)
    pub medals_by_type: HashMap<String, Vec<String>>,
    /// Medals organized by rarity (e.g., "T1" -> vec of medal_ids)
    pub medals_by_rarity: HashMap<String, Vec<String>>,
    /// Category display names (e.g., "playerMedal" -> "Records Medal")
    pub category_names: HashMap<String, String>,
    /// medal_id -> group_id, populated for medals that belong to a group.
    /// Activity medals are the only type the game groups; everything else
    /// (player / story / camp / etc.) falls back to the medal's own ExpireTimes.
    pub medal_to_group: HashMap<String, String>,
}

/// Whether a medal can currently be earned. Used to keep "dead event" medals out
/// of the permanent-pool denominator so players aren't penalized for medals
/// tied to retired Crisis Contract / multiplayer / collab events.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Obtainability {
    /// Earnable at any time — counts toward the permanent pool.
    Permanent,
    /// Time-bound event medal. `proxy_close_ts` is used for both in-window checks
    /// and recency decay:
    ///   - `proxy_close_ts > now`: still earnable in the active window.
    ///   - `proxy_close_ts <= now`: window has closed; the medal contributes to the
    ///     event pool with recency-decayed weight.
    ///   - `proxy_close_ts == 0`: ongoing event with no scheduled close (full weight).
    Event { proxy_close_ts: i64 },
}

impl MedalData {
    /// Process raw medal table data into indexed structures
    pub fn from_table(table: MedalTableFile) -> Self {
        let mut data = MedalData::default();

        // Index all medals
        for medal in table.medal_list {
            let medal_id = medal.medal_id.clone();
            let medal_type = medal.medal_type.clone();
            let rarity = medal.rarity.clone();

            // Add to medals_by_type
            data.medals_by_type
                .entry(medal_type)
                .or_default()
                .push(medal_id.clone());

            // Add to medals_by_rarity
            data.medals_by_rarity
                .entry(rarity)
                .or_default()
                .push(medal_id.clone());

            // Add to main index
            data.medals.insert(medal_id, medal);
        }

        // Process type data for category names and groups
        for type_entry in table.medal_type_data {
            // Store category display name
            data.category_names
                .insert(type_entry.key.clone(), type_entry.value.medal_name);

            // Index all groups from this type
            for group in type_entry.value.group_data {
                for medal_id in &group.medal_id {
                    data.medal_to_group
                        .insert(medal_id.clone(), group.group_id.clone());
                }
                data.groups.insert(group.group_id.clone(), group);
            }
        }

        data
    }

    /// Get the display name for a medal category
    pub fn get_category_name(&self, category: &str) -> String {
        self.category_names
            .get(category)
            .cloned()
            .unwrap_or_else(|| category.to_string())
    }

    /// Classify a medal's current obtainability. See [`Obtainability`].
    ///
    /// Group-level `SharedExpireTimes` (when present) is authoritative — the
    /// data field on the medal itself can lie about availability (e.g. dead
    /// Crisis Contract groups whose individual medals are stamped `PERM/-1`).
    ///
    /// Upgrade-variant medals (`medal_*_035`, `medal_*_105` — the
    /// "Commemorative Contract Medal II" tier) aren't enrolled in their parent
    /// group directly. We walk the `origin_medal` chain so they inherit their
    /// base medal's group classification; otherwise Pyrite II / Cinder II etc.
    /// would score as permanent even though their event groups are dead.
    pub fn obtainability(&self, medal_id: &str, now: i64) -> Obtainability {
        let Some(medal) = self.medals.get(medal_id) else {
            // Unknown medal — treat as a closed event so it doesn't pollute the
            // permanent pool. End-ts in the deep past forces full decay.
            return Obtainability::Event { proxy_close_ts: 0 };
        };

        let group = self.resolve_group(medal);
        let times: &[ExpireTime] = group
            .map(|g| g.shared_expire_times.as_slice())
            .unwrap_or(&medal.expire_times);

        classify_expire_times(times, group.is_some(), now)
    }

    /// Find the group that determines a medal's obtainability, walking up the
    /// `origin_medal` chain for upgrade variants.
    fn resolve_group(&self, medal: &MedalDefinition) -> Option<&MedalGroup> {
        if let Some(gid) = self.medal_to_group.get(&medal.medal_id) {
            return self.groups.get(gid);
        }
        let mut current = medal;
        for _ in 0..4 {
            if current.origin_medal.is_empty() || current.origin_medal == current.medal_id {
                return None;
            }
            let next = self.medals.get(&current.origin_medal)?;
            if let Some(gid) = self.medal_to_group.get(&next.medal_id) {
                return self.groups.get(gid);
            }
            current = next;
        }
        None
    }

    /// Resolve a medal's display description, walking the `origin_medal` chain
    /// for upgrade-variant medals (e.g. `Commemorative Contract Medal II`)
    /// whose own description field is empty.
    pub fn resolve_description(&self, medal_id: &str) -> String {
        let mut current = medal_id;
        for _ in 0..4 {
            let Some(medal) = self.medals.get(current) else {
                return String::new();
            };
            if !medal.description.trim().is_empty() {
                return medal.description.clone();
            }
            if medal.origin_medal.is_empty() || medal.origin_medal == current {
                return String::new();
            }
            current = &medal.origin_medal;
        }
        String::new()
    }
}

fn classify_expire_times(times: &[ExpireTime], is_grouped: bool, now: i64) -> Obtainability {
    if times.is_empty() {
        return Obtainability::Permanent;
    }

    // Active TEMP window (start <= now <= end, or open-ended End=-1)?
    if let Some(active) = times
        .iter()
        .find(|e| e.expire_type == "TEMP" && e.start <= now && (e.end == -1 || now <= e.end))
    {
        return Obtainability::Event {
            proxy_close_ts: if active.end > 0 { active.end } else { 0 },
        };
    }

    // Future TEMP (not yet started) — treat as currently earnable from start onwards.
    if let Some(future) = times
        .iter()
        .filter(|e| e.expire_type == "TEMP" && e.start > now)
        .min_by_key(|e| e.start)
    {
        return Obtainability::Event {
            proxy_close_ts: if future.end > 0 { future.end } else { 0 },
        };
    }

    let has_perm_open = times.iter().any(|e| e.expire_type == "PERM" && e.end == -1);
    let has_any_temp = times.iter().any(|e| e.expire_type == "TEMP");

    if has_perm_open {
        // TEMP → PERM transition (e.g. Wolumonde): once temp, now permanently
        // earnable through the post-event permanent route.
        if has_any_temp {
            return Obtainability::Permanent;
        }
        // PERM-only: ungrouped medals (player level, story, tower, etc.) are
        // genuinely permanent. Grouped activity medals with this pattern are
        // dead Crisis Contract / multiplayer events — earnable only during
        // their original season and never re-runnable. Route them to the event
        // pool using the group's PERM start as the close-ts proxy so recency
        // decay applies.
        if is_grouped {
            let proxy = times
                .iter()
                .filter(|e| e.expire_type == "PERM")
                .map(|e| e.start)
                .max()
                .unwrap_or(0);
            return Obtainability::Event {
                proxy_close_ts: proxy,
            };
        }
        return Obtainability::Permanent;
    }

    // TEMP-only with all windows closed → past event.
    let last_end = times
        .iter()
        .filter(|e| e.expire_type == "TEMP")
        .map(|e| if e.end > 0 { e.end } else { e.start })
        .max()
        .unwrap_or(0);
    Obtainability::Event {
        proxy_close_ts: last_end,
    }
}
