use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::activity::ActivityBasicInfo;
use super::operator::Operator;

#[derive(Debug, Default)]
pub struct MedalTables {
    /// `medal_table`
    pub medals: MedalTableFile,
}

/// Medal `Template`s whose first `UnlockParam` is a `char_*` id
const OPERATOR_GATED_TEMPLATES: &[&str] = &[
    "GotChars",
    "GotCharsBeforeTime",
    "CharPotential",
    "CharStoryUnlock",
    "CharEvolvePhase",
];

/// Max hops when walking the `origin_medal` chain
const ORIGIN_CHAIN_MAX_DEPTH: usize = 4;

/// Medal gated on a collab operator
#[derive(Debug, Clone, Serialize)]
pub struct OperatorLock {
    pub operator_id: String,
    pub operator_name: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MedalTableFile {
    pub medal_list: Vec<MedalDefinition>,
    pub medal_type_data: Vec<MedalTypeEntry>,
}

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
    pub template: String,
    #[serde(default)]
    pub unlock_param: Vec<String>,
    #[serde(default)]
    pub get_method: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub display_time: i64,
    #[serde(default)]
    pub expire_times: Vec<ExpireTime>,
    #[serde(default)]
    pub medal_reward_group: Value,
    #[serde(default)]
    pub is_hidden: bool,
    /// Base medal of an upgrade variant (e.g. `medal_*_035`), empty if standalone
    #[serde(default)]
    pub origin_medal: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ExpireTime {
    pub start: i64,
    pub end: i64,
    #[serde(rename = "Type_")]
    pub expire_type: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MedalTypeEntry {
    pub key: String,
    pub value: MedalTypeData,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MedalTypeData {
    pub medal_group_id: String,
    pub sort_id: i32,
    pub medal_name: String,
    #[serde(default)]
    pub group_data: Vec<MedalGroup>,
}

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

#[derive(Debug, Clone, Default, Serialize)]
pub struct MedalData {
    pub medals: HashMap<String, MedalDefinition>,
    pub groups: HashMap<String, MedalGroup>,
    /// Medals by type (e.g. "playerMedal" -> `medal_ids`)
    pub medals_by_type: HashMap<String, Vec<String>>,
    /// Medals by rarity (e.g. "T1" -> `medal_ids`)
    pub medals_by_rarity: HashMap<String, Vec<String>>,
    /// Category display names (e.g. "playerMedal" -> "Records Medal")
    pub category_names: HashMap<String, String>,
    /// `medal_id` -> `group_id`, for grouped medals
    pub medal_to_group: HashMap<String, String>,
    /// `medal_id` -> collab operator gating it
    pub operator_locked: HashMap<String, OperatorLock>,
    /// `medal_id` -> SSS tower season windows
    pub tower_windows: HashMap<String, Vec<(i64, i64)>>,
    /// `medal_id` -> event activity window, overrides `ExpireTimes`
    pub event_windows: HashMap<String, EventWindow>,
}

/// Event medal's earnable window
#[derive(Debug, Clone, Copy, Serialize)]
pub struct EventWindow {
    pub start: i64,
    /// Reward-claimable-until timestamp, `0` if no close
    pub close: i64,
    /// One-time mode that never reruns (auto-chess, enemy duel, boss rush, etc.)
    pub one_time: bool,
}

/// Whether a medal can currently be earned
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Obtainability {
    /// Earnable any time
    Permanent,
    /// Time-bound event medal:
    /// `proxy_close_ts > now` = still earnable,
    /// `<= now` = closed, recency-decayed,
    /// `0` = ongoing, no scheduled close
    Event { proxy_close_ts: i64 },
    /// Window entirely in the future
    NotYet,
    /// Never obtainable again
    Unobtainable,
}

impl MedalData {
    #[must_use]
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

    /// Fill [`Self::operator_locked`] from operator-gated medals
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

    /// Lock info for a medal gated on an unobtainable operator
    #[must_use]
    pub fn operator_lock(&self, medal_id: &str) -> Option<&OperatorLock> {
        self.operator_locked.get(medal_id)
    }

    /// Fill [`Self::tower_windows`] and [`Self::event_windows`]
    pub fn link_content_windows(
        &mut self,
        tower_schedule: &HashMap<String, Vec<(i64, i64)>>,
        activities: &HashMap<String, ActivityBasicInfo>,
    ) {
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

        let mut event_windows = HashMap::new();
        for medal in self.medals.values() {
            let Some(suffix) = medal
                .medal_id
                .strip_prefix("medal_activity_")
                .or_else(|| medal.medal_id.strip_prefix("medal_hidden_"))
            else {
                continue;
            };
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

    #[must_use]
    pub fn get_category_name(&self, category: &str) -> String {
        self.category_names
            .get(category)
            .cloned()
            .unwrap_or_else(|| category.to_string())
    }

    /// Classify a medal's current obtainability
    #[must_use]
    pub fn obtainability(&self, medal_id: &str, now: i64) -> Obtainability {
        let Some(medal) = self.medals.get(medal_id) else {
            return Obtainability::Event { proxy_close_ts: 1 };
        };

        if let Some(windows) = self.tower_windows.get(medal_id) {
            return classify_windows(windows, now);
        }

        let group = self.resolve_group(medal);
        let times: &[ExpireTime] =
            group.map_or(&medal.expire_times, |g| g.shared_expire_times.as_slice());

        let base = classify_expire_times(times, group.is_some(), now);

        if let Some(&EventWindow {
            start,
            close,
            one_time,
        }) = self.event_windows.get(medal_id)
        {
            let active = now >= start && (close <= 0 || now <= close);

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
                    return Obtainability::NotYet;
                }
                return Obtainability::Event {
                    proxy_close_ts: close,
                };
            }
        }

        base
    }

    /// Group that determines a medal's obtainability, walking the `origin_medal` chain
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

    /// Display description, walking the `origin_medal` chain for upgrade variants
    #[must_use]
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

/// Classify seasonal content (SSS towers) from its windows, best case wins
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

    if let Some(active) = times
        .iter()
        .find(|e| e.expire_type == "TEMP" && e.start <= now && (e.end == -1 || now <= e.end))
    {
        return Obtainability::Event {
            proxy_close_ts: if active.end > 0 { active.end } else { 0 },
        };
    }

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

    fn data_with_tower_windows(windows: Vec<(i64, i64)>) -> MedalData {
        let mut d = data_with(vec![mk_medal("medal_t", "PassTower", &["tower_n_10"])]);
        d.tower_windows.insert("medal_t".to_string(), windows);
        d
    }

    fn data_with_event_window(window: EventWindow) -> MedalData {
        let mut d = data_with(vec![mk_medal("medal_ev", "", &[])]);
        d.event_windows.insert("medal_ev".to_string(), window);
        d
    }

    #[test]
    fn no_expire_times_is_permanent() {
        let d = data_with(vec![mk_medal("medal_perm", "", &[])]);
        assert_eq!(d.obtainability("medal_perm", NOW), Obtainability::Permanent);
    }

    /// Login-anniversary medals (`JoinGameDays`, "log in for 6 years") carry an
    /// `INIT` `ExpireTimes` window with a real End, open since account creation.
    /// INIT is neither TEMP nor PERM, so it must stay Permanent.
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

    // Event repair: the activity schedule overrides a mislabeled table.

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
        // TeamId null, DisplayNumber "R161", as in the real gamedata.
        let mut lolxh = mk_operator(None, "R161", "Luo Xiaohei");
        lolxh.id = Some("char_4067_lolxh".to_string());
        ops.insert("char_4067_lolxh".to_string(), lolxh);

        d.link_operator_locks(&ops);

        assert!(d.operator_lock("medal_hidden_story_lolxh_1").is_some());
    }
}
