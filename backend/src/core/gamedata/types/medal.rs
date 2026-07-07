//! Medal game data types
//!
//! Contains types for parsing `medal_table.json` and storing processed medal data
//! for use in user scoring calculations.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::activity::ActivityBasicInfo;
use super::operator::Operator;

/// Medal `Template`s whose first `UnlockParam` is a `char_*` id, i.e. the medal
/// is gated on owning / investing in a specific operator.
const OPERATOR_GATED_TEMPLATES: &[&str] = &[
    "GotChars",
    "GotCharsBeforeTime",
    "CharPotential",
    "CharStoryUnlock",
    "CharEvolvePhase",
];

/// Max hops to follow when walking the `origin_medal` chain of an upgrade-variant
/// medal. Bounds the walk so malformed data that points a medal back into a cycle
/// can't loop forever; the real chains are only one or two links deep.
const ORIGIN_CHAIN_MAX_DEPTH: usize = 4;

/// A medal gated on a collab operator. Whether it's actually *locked* for a
/// given user depends on whether they own the operator - that per-user check
/// happens at scoring / improvements time, not here. This just records the
/// gating operator so consumers don't have to re-resolve it.
#[derive(Debug, Clone)]
pub struct OperatorLock {
    pub operator_id: String,
    pub operator_name: String,
}

/// Root structure for `medal_table.json`
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
    /// All medals indexed by `medal_id`
    pub medals: HashMap<String, MedalDefinition>,
    /// All medal groups indexed by `group_id`
    pub groups: HashMap<String, MedalGroup>,
    /// Medals organized by type/category (e.g., "playerMedal" -> vec of `medal_ids`)
    pub medals_by_type: HashMap<String, Vec<String>>,
    /// Medals organized by rarity (e.g., "T1" -> vec of `medal_ids`)
    pub medals_by_rarity: HashMap<String, Vec<String>>,
    /// Category display names (e.g., "playerMedal" -> "Records Medal")
    pub category_names: HashMap<String, String>,
    /// `medal_id` -> `group_id`, populated for medals that belong to a group.
    /// Activity medals are the only type the game groups; everything else
    /// (player / story / camp / etc.) falls back to the medal's own `ExpireTimes`.
    pub medal_to_group: HashMap<String, String>,
    /// `medal_id` -> the collab operator that gates it. Populated by
    /// [`Self::link_operator_locks`] once operator data is available. A medal here
    /// is only *unobtainable for a given user* if they don't own the operator -
    /// that per-user check lives in scoring / improvements, not in this map.
    pub operator_locked: HashMap<String, OperatorLock>,
    /// `medal_id` -> Stationary Security Service tower season windows, for medals
    /// whose unlock requires a tower that only runs in certain seasons. Their own
    /// `ExpireTimes` are empty (so they'd look permanent), so availability is
    /// driven by the climb-tower schedule instead. Populated by
    /// [`Self::link_content_windows`].
    pub tower_windows: HashMap<String, Vec<(i64, i64)>>,
    /// `medal_id` -> its event activity's window. Used to repair medals whose
    /// `ExpireTimes` mislabel event content as permanent/ongoing - the activity
    /// schedule is authoritative. Populated by [`Self::link_content_windows`].
    pub event_windows: HashMap<String, EventWindow>,
}

/// An event medal's earnable window, resolved from its activity.
#[derive(Debug, Clone, Copy)]
pub struct EventWindow {
    pub start: i64,
    /// Reward-claimable-until timestamp (`0` if the activity has no close).
    pub close: i64,
    /// True for one-time competitive / minigame modes (auto-chess, enemy duel,
    /// boss rush, etc.) that never rerun. Once such a mode is over its medals are
    /// *excluded* from scoring rather than recency-decayed - matching how the
    /// stage universe drops those stages entirely.
    pub one_time: bool,
}

/// Whether a medal can currently be earned. Used to keep "dead event" medals out
/// of the permanent-pool denominator so players aren't penalized for medals
/// tied to retired Crisis Contract / multiplayer / collab events.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Obtainability {
    /// Earnable at any time - counts toward the permanent pool.
    Permanent,
    /// Time-bound event medal. `proxy_close_ts` is used for both in-window checks
    /// and recency decay:
    ///   - `proxy_close_ts > now`: still earnable in the active window.
    ///   - `proxy_close_ts <= now`: window has closed; the medal contributes to the
    ///     event pool with recency-decayed weight.
    ///   - `proxy_close_ts == 0`: ongoing event with no scheduled close (full weight).
    Event { proxy_close_ts: i64 },
    /// Not reachable *yet*: seasonal/event content whose window is entirely in
    /// the future (e.g. an SSS tower season or an event that hasn't started on
    /// this server). It *will* become obtainable later, so it shouldn't count
    /// for or against the user. Excluded from both scoring pools and from the
    /// improvement lists (it isn't an actionable gap right now).
    NotYet,
    /// Never obtainable again: one-time competitive modes that have finished and
    /// aren't rebroadcast, and retired pre-schedule towers with no season entry.
    /// Excluded from both scoring pools; surfaced in the "no longer obtainable"
    /// improvement bucket for reference so players can see it's a dead end.
    Unobtainable,
}

impl MedalData {
    /// Process raw medal table data into indexed structures
    pub fn from_table(table: MedalTableFile) -> Self {
        let mut data = Self::default();

        for medal in table.medal_list {
            let medal_id = medal.medal_id.clone();
            let medal_type = medal.medal_type.clone();
            let rarity = medal.rarity.clone();

            data.medals_by_type
                .entry(medal_type)
                .or_default()
                .push(medal_id.clone());
            data.medals_by_rarity
                .entry(rarity)
                .or_default()
                .push(medal_id.clone());
            data.medals.insert(medal_id, medal);
        }

        for type_entry in table.medal_type_data {
            data.category_names
                .insert(type_entry.key.clone(), type_entry.value.medal_name);

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

    /// Populate [`Self::operator_locked`] by cross-referencing operator-gated
    /// medals against operator obtainability. Call once at load after operators
    /// are built. Idempotent - rebuilds the map each call.
    pub fn link_operator_locks(&mut self, operators: &HashMap<String, Operator>) {
        let mut locked = HashMap::new();
        for medal in self.medals.values() {
            if !OPERATOR_GATED_TEMPLATES.contains(&medal.template.as_str()) {
                continue;
            }
            let Some(char_id) = medal.unlock_param.first() else {
                continue;
            };
            if !char_id.starts_with("char_") {
                continue;
            }
            let Some(op) = operators.get(char_id) else {
                continue;
            };
            if op.is_collab() {
                locked.insert(
                    medal.medal_id.clone(),
                    OperatorLock {
                        operator_id: char_id.clone(),
                        operator_name: op.name.clone(),
                    },
                );
            }
        }
        self.operator_locked = locked;
    }

    /// Lock info for a medal gated on an unobtainable operator, if any.
    pub fn operator_lock(&self, medal_id: &str) -> Option<&OperatorLock> {
        self.operator_locked.get(medal_id)
    }

    /// Populate [`Self::tower_windows`] and [`Self::event_windows`] from the
    /// authoritative schedules. Call once at load after groups are indexed.
    ///
    /// - `tower_schedule`: `tower_id -> [(season_start, season_end)]` from the
    ///   climb-tower (Stationary Security Service) table.
    /// - `activities`: used to map each event medal (via its activity's medal
    ///   group) to the activity's real `(start, close)` window.
    pub fn link_content_windows(
        &mut self,
        tower_schedule: &HashMap<String, Vec<(i64, i64)>>,
        activities: &HashMap<String, ActivityBasicInfo>,
    ) {
        // SSS tower medals (`PassTower`, first unlock param is the tower id) ->
        // that tower's season windows. Their own ExpireTimes are empty/placeholder.
        // Only NORMAL towers (`tower_n_*`) are season-gated; training towers
        // (`tower_tr_*`) are always-available practice and keep their ExpireTimes.
        // Towers with no season entry are the original pre-schedule SSS
        // (`tower_n_01..04`) that were never carried into `SeasonInfos`; they get
        // empty windows, which classify as Unobtainable (retired, not playable)
        // rather than slipping through to Permanent.
        let mut tower_windows = HashMap::new();
        for medal in self.medals.values() {
            if medal.template != "PassTower" {
                continue;
            }
            let Some(tower_id) = medal.unlock_param.first() else {
                continue;
            };
            if !tower_id.starts_with("tower_n_") {
                continue;
            }
            let windows = tower_schedule.get(tower_id).cloned().unwrap_or_default();
            tower_windows.insert(medal.medal_id.clone(), windows);
        }
        self.tower_windows = tower_windows;

        // Event medals -> their activity's (start, close) window, resolved from
        // the medal id: `medal_activity_<suffix>_<n>` / `medal_hidden_<suffix>_<n>`
        // map to activity `<suffix>` or `act<suffix>`. This covers grouped medal
        // sets, `_NNN` upgrade variants, ungrouped special-mode medals (auto-chess,
        // enemy duel, boss rush, ...), AND hidden event medals ("obtain operator
        // X during event Y", event stage-challenge medals) alike - all of which
        // the medal table's own ExpireTimes mislabel as permanent/ongoing.
        // `RewardEndTime` (claimable-until) is preferred over the stage `EndTime`.
        let mut event_windows = HashMap::new();
        for medal in self.medals.values() {
            let Some(suffix) = medal
                .medal_id
                .strip_prefix("medal_activity_")
                .or_else(|| medal.medal_id.strip_prefix("medal_hidden_"))
            else {
                continue;
            };
            // Drop the trailing medal number (`_05`, `_105`, ...) to get the
            // activity suffix.
            let Some((act_suffix, _)) = suffix.rsplit_once('_') else {
                continue;
            };
            let activity = activities
                .get(act_suffix)
                .or_else(|| activities.get(&format!("act{act_suffix}")));
            let Some(act) = activity else {
                continue;
            };
            let close = if act.reward_end_time > 0 {
                act.reward_end_time
            } else {
                act.end_time
            };
            event_windows.insert(
                medal.medal_id.clone(),
                EventWindow {
                    start: act.start_time,
                    close,
                    one_time: act.is_one_time_competitive(),
                },
            );
        }
        self.event_windows = event_windows;
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
    /// Group-level `SharedExpireTimes` (when present) is authoritative - the
    /// data field on the medal itself can lie about availability (e.g. dead
    /// Crisis Contract groups whose individual medals are stamped `PERM/-1`).
    ///
    /// Upgrade-variant medals (`medal_*_035`, `medal_*_105` - the
    /// "Commemorative Contract Medal II" tier) aren't enrolled in their parent
    /// group directly. We walk the `origin_medal` chain so they inherit their
    /// base medal's group classification; otherwise Pyrite II / Cinder II etc.
    /// would score as permanent even though their event groups are dead.
    pub fn obtainability(&self, medal_id: &str, now: i64) -> Obtainability {
        let Some(medal) = self.medals.get(medal_id) else {
            // Unknown medal - treat as a closed event so it doesn't pollute the
            // permanent pool. End-ts in the deep past forces full decay.
            return Obtainability::Event { proxy_close_ts: 0 };
        };

        if let Some(windows) = self.tower_windows.get(medal_id) {
            return classify_windows(windows, now);
        }

        let group = self.resolve_group(medal);
        let times: &[ExpireTime] =
            group.map_or(&medal.expire_times, |g| g.shared_expire_times.as_slice());

        let base = classify_expire_times(times, group.is_some(), now);

        // For event medals the activity schedule is authoritative over the medal
        // table's ExpireTimes, which routinely mislabel event content as
        // permanent (PERM/-1) or ongoing (open-ended TEMP) - e.g. one-time
        // auto-chess / enemy-duel / boss-rush medals, or events whose End is a
        // placeholder. We only override when the base classification claims the
        // medal is permanently/indefinitely earnable; a properly-bounded event
        // window (Event with a real close ts - e.g. a TEMP->PERM medal keyed off
        // its TEMP end, which already covers reruns) is left as-is.
        if let Some(&EventWindow {
            start,
            close,
            one_time,
        }) = self.event_windows.get(medal_id)
        {
            let active = now >= start && (close <= 0 || now <= close);

            // One-time competitive / minigame modes (auto-chess, enemy duel, boss
            // rush, multiplayer, ...) never rerun. While the mode is live the
            // medal is earnable; before it opens it's NotYet; once it's over it's
            // permanently Unobtainable - either way excluded from scoring, matching
            // how the stage universe drops these activities rather than letting the
            // medal linger as a decayed gap. This holds regardless of what the
            // medal's own ExpireTimes claim.
            if one_time {
                if active {
                    return Obtainability::Event {
                        proxy_close_ts: if close > 0 { close } else { 0 },
                    };
                }
                return if now < start {
                    Obtainability::NotYet
                } else {
                    Obtainability::Unobtainable
                };
            }

            // Otherwise the activity schedule only *repairs* medals the table
            // mislabels as permanently/indefinitely earnable; a properly-bounded
            // event window (e.g. a TEMP->PERM medal keyed off its TEMP end, which
            // already covers reruns) is trusted as-is.
            let base_says_open = matches!(
                base,
                Obtainability::Permanent | Obtainability::Event { proxy_close_ts: 0 }
            );
            if base_says_open {
                if active {
                    return Obtainability::Event {
                        proxy_close_ts: if close > 0 { close } else { 0 },
                    };
                }
                if now < start {
                    // Event hasn't started on this server yet - it will run later.
                    return Obtainability::NotYet;
                }
                return Obtainability::Event {
                    proxy_close_ts: close,
                };
            }
        }

        base
    }

    /// Find the group that determines a medal's obtainability, walking up the
    /// `origin_medal` chain for upgrade variants.
    fn resolve_group(&self, medal: &MedalDefinition) -> Option<&MedalGroup> {
        if let Some(gid) = self.medal_to_group.get(&medal.medal_id) {
            return self.groups.get(gid);
        }
        let mut current = medal;
        for _ in 0..ORIGIN_CHAIN_MAX_DEPTH {
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
        for _ in 0..ORIGIN_CHAIN_MAX_DEPTH {
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

/// Classify seasonal content (e.g. SSS towers) from its open/close windows.
/// A tower may appear in several seasons (it gets replicated), so we take the
/// best-case window: currently open beats already-closed beats not-yet-open.
fn classify_windows(windows: &[(i64, i64)], now: i64) -> Obtainability {
    if let Some(&(_, end)) = windows
        .iter()
        .find(|&&(s, e)| s <= now && (e <= 0 || now <= e))
    {
        return Obtainability::Event {
            proxy_close_ts: if end > 0 { end } else { 0 },
        };
    }
    if let Some(last_end) = windows
        .iter()
        .filter(|&&(_, e)| e > 0 && e < now)
        .map(|&(_, e)| e)
        .max()
    {
        return Obtainability::Event {
            proxy_close_ts: last_end,
        };
    }
    // A window exists but is entirely in the future (season scheduled, not yet
    // started) -> will become obtainable. No windows at all -> a retired
    // pre-schedule tower that was never carried into the season schedule and
    // can never run again.
    if windows.is_empty() {
        Obtainability::Unobtainable
    } else {
        Obtainability::NotYet
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

    // Future TEMP (not yet started) - treat as currently earnable from start onwards.
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
    let has_any_perm = times.iter().any(|e| e.expire_type == "PERM");

    if !has_any_temp && !has_any_perm {
        return Obtainability::Permanent;
    }

    if has_perm_open {
        // TEMP + open-ended PERM. The TEMP window is the real earnable period
        // (it spans the event's original run and its rerun); the open-ended PERM
        // entry is a display/collection artifact, NOT a "permanently earnable"
        // signal. These are event medals - typically EX-stage challenge medals
        // ("clear GA-EX-7 with ...") whose stages live in ACTIVITY zones (the
        // event pool, not the permanent archive), so they can't be earned once
        // the event is over. The in-window case already returned above, so here
        // the window has closed: treat as a recency-decayed past event keyed off
        // the TEMP end (covers both grouped sets and `_105` upgrade variants).
        if has_any_temp {
            let last_temp_end = times
                .iter()
                .filter(|e| e.expire_type == "TEMP")
                .map(|e| if e.end > 0 { e.end } else { e.start })
                .max()
                .unwrap_or(0);
            return Obtainability::Event {
                proxy_close_ts: last_temp_end,
            };
        }
        // PERM-only: ungrouped medals (player level, story, tower, etc.) are
        // genuinely permanent. Grouped activity medals with this pattern are
        // dead Crisis Contract / multiplayer events - earnable only during
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

#[cfg(test)]
mod tests {
    use super::*;

    const NOW: i64 = 1_700_000_000;

    /// Minimal medal definition for classification tests.
    fn mk_medal(id: &str, template: &str, unlock_param: &[&str]) -> MedalDefinition {
        MedalDefinition {
            medal_id: id.to_string(),
            medal_name: id.to_string(),
            medal_type: "activityMedal".to_string(),
            slot_id: 0,
            pre_medal_id_list: Vec::new(),
            rarity: "T1".to_string(),
            template: template.to_string(),
            unlock_param: unlock_param.iter().map(|s| (*s).to_string()).collect(),
            get_method: String::new(),
            description: String::new(),
            display_time: 0,
            expire_times: Vec::new(),
            medal_reward_group: serde_json::Value::Null,
            is_hidden: false,
            origin_medal: String::new(),
        }
    }

    fn data_with(medals: Vec<MedalDefinition>) -> MedalData {
        let mut d = MedalData::default();
        for m in medals {
            d.medals.insert(m.medal_id.clone(), m);
        }
        d
    }

    fn mk_operator(team_id: Option<&str>, display_number: &str, name: &str) -> Operator {
        Operator {
            name: name.to_string(),
            team_id: team_id.map(str::to_string),
            display_number: display_number.to_string(),
            ..Default::default()
        }
    }

    /// `MedalData` with a single tower medal whose obtainability is driven by the
    /// given season windows.
    fn data_with_tower_windows(windows: Vec<(i64, i64)>) -> MedalData {
        let mut d = data_with(vec![mk_medal("medal_t", "PassTower", &["tower_n_10"])]);
        d.tower_windows.insert("medal_t".to_string(), windows);
        d
    }

    /// `MedalData` with a single event medal whose obtainability is driven by the
    /// given activity window.
    fn data_with_event_window(window: EventWindow) -> MedalData {
        let mut d = data_with(vec![mk_medal("medal_ev", "", &[])]);
        d.event_windows.insert("medal_ev".to_string(), window);
        d
    }

    // ── Permanent ────────────────────────────────────────────────────────────

    #[test]
    fn no_expire_times_is_permanent() {
        let d = data_with(vec![mk_medal("medal_perm", "", &[])]);
        assert_eq!(d.obtainability("medal_perm", NOW), Obtainability::Permanent);
    }

    /// Login-anniversary medals (`JoinGameDays`, e.g. "log in for 6 years") carry
    /// an `INIT`-type `ExpireTimes` window with a real End - available since account
    /// creation. INIT is neither TEMP nor PERM, so it must stay Permanent rather
    /// than being read as a limited-time event goal.
    #[test]
    fn init_expire_time_is_permanent() {
        let mut medal = mk_medal("medal_player_joingame_07", "JoinGameDays", &["2190"]);
        medal.medal_type = "playerMedal".to_string();
        medal.expire_times = vec![ExpireTime {
            start: -1,
            end: NOW + 1_000_000,
            expire_type: "INIT".to_string(),
        }];
        let d = data_with(vec![medal]);
        assert_eq!(
            d.obtainability("medal_player_joingame_07", NOW),
            Obtainability::Permanent
        );
    }

    // ── Tower / seasonal windows ─────────────────────────────────────────────

    #[test]
    fn tower_in_season_is_active_event() {
        let d = data_with_tower_windows(vec![(NOW - 100, NOW + 100)]);
        assert_eq!(
            d.obtainability("medal_t", NOW),
            Obtainability::Event {
                proxy_close_ts: NOW + 100
            }
        );
    }

    #[test]
    fn tower_between_seasons_is_closed_event() {
        let d = data_with_tower_windows(vec![(NOW - 300, NOW - 100)]);
        assert_eq!(
            d.obtainability("medal_t", NOW),
            Obtainability::Event {
                proxy_close_ts: NOW - 100
            }
        );
    }

    #[test]
    fn tower_future_season_is_not_yet() {
        let d = data_with_tower_windows(vec![(NOW + 100, NOW + 300)]);
        assert_eq!(d.obtainability("medal_t", NOW), Obtainability::NotYet);
    }

    #[test]
    fn retired_tower_with_no_windows_is_unobtainable() {
        // Pre-schedule tower never carried into SeasonInfos -> empty windows.
        let d = data_with_tower_windows(Vec::new());
        assert_eq!(d.obtainability("medal_t", NOW), Obtainability::Unobtainable);
    }

    // ── One-time competitive modes ───────────────────────────────────────────

    #[test]
    fn one_time_mode_active_is_event() {
        let d = data_with_event_window(EventWindow {
            start: NOW - 100,
            close: NOW + 100,
            one_time: true,
        });
        assert_eq!(
            d.obtainability("medal_ev", NOW),
            Obtainability::Event {
                proxy_close_ts: NOW + 100
            }
        );
    }

    #[test]
    fn one_time_mode_future_is_not_yet() {
        let d = data_with_event_window(EventWindow {
            start: NOW + 100,
            close: NOW + 300,
            one_time: true,
        });
        assert_eq!(d.obtainability("medal_ev", NOW), Obtainability::NotYet);
    }

    #[test]
    fn one_time_mode_finished_is_unobtainable() {
        let d = data_with_event_window(EventWindow {
            start: NOW - 300,
            close: NOW - 100,
            one_time: true,
        });
        assert_eq!(
            d.obtainability("medal_ev", NOW),
            Obtainability::Unobtainable
        );
    }

    // ── Repeatable event repair (activity schedule overrides mislabeled table) ─

    #[test]
    fn event_future_window_repairs_to_not_yet() {
        // Table mislabels it Permanent (empty expire times), but the activity
        // hasn't started -> NotYet, not a false permanent.
        let d = data_with_event_window(EventWindow {
            start: NOW + 100,
            close: NOW + 300,
            one_time: false,
        });
        assert_eq!(d.obtainability("medal_ev", NOW), Obtainability::NotYet);
    }

    #[test]
    fn event_closed_window_is_decayed_event() {
        let d = data_with_event_window(EventWindow {
            start: NOW - 300,
            close: NOW - 100,
            one_time: false,
        });
        assert_eq!(
            d.obtainability("medal_ev", NOW),
            Obtainability::Event {
                proxy_close_ts: NOW - 100
            }
        );
    }

    // ── Operator locks ───────────────────────────────────────────────────────

    #[test]
    fn link_operator_locks_catches_collab_only() {
        let d_medals = vec![
            mk_medal("medal_collab", "GotChars", &["char_9001_collab"]),
            mk_medal("medal_regular", "GotChars", &["char_0001_regular"]),
        ];
        let mut d = data_with(d_medals);

        let mut ops = HashMap::new();
        ops.insert(
            "char_9001_collab".to_string(),
            mk_operator(Some("rainbow"), "RB01", "Collab Op"),
        );
        ops.insert(
            "char_0001_regular".to_string(),
            mk_operator(None, "R001", "Regular Op"),
        );

        d.link_operator_locks(&ops);

        assert!(d.operator_lock("medal_collab").is_some());
        assert!(d.operator_lock("medal_regular").is_none());
    }

    /// Luo Xiaohei (`char_4067_lolxh`) is a team-less collab whose `R` display
    /// prefix collides with ordinary Rhodes ops, so `is_collab` catches it via
    /// its curated char id. Its `CharStoryUnlock` medal must be operator-locked.
    #[test]
    fn link_operator_locks_catches_teamless_lolxh_collab() {
        let mut d = data_with(vec![mk_medal(
            "medal_hidden_story_lolxh_1",
            "CharStoryUnlock",
            &["char_4067_lolxh", "story_lolxh_set_1"],
        )]);

        let mut ops = HashMap::new();
        // TeamId null, DisplayNumber "R161" - same shape as the real gamedata.
        let mut lolxh = mk_operator(None, "R161", "Luo Xiaohei");
        lolxh.id = Some("char_4067_lolxh".to_string());
        ops.insert("char_4067_lolxh".to_string(), lolxh);

        d.link_operator_locks(&ops);

        assert!(d.operator_lock("medal_hidden_story_lolxh_1").is_some());
    }
}
