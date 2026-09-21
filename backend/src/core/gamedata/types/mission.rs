//! The mission table's periodical rewards: what the daily and weekly mission
//! chests pay, read for the LMD an account earns from dailies alone.
//!
//! `DailyMissionPeriodInfo` names the reward groups live in a date window
//! (one group for weekdays, one for weekends, by `Period` = days of the
//! week); `PeriodicalRewards` holds every daily chest by group;
//! `WeeklyRewards` every weekly chest with its own date window. LMD is item
//! `4001`. A chest pays once per period, so a day's LMD is the sum over
//! the live group's chests, and a weekly chest pays a seventh per day.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::serde_helpers::deserialize_fb_map_or_default;

/// The game's LMD item id.
pub const LMD_ITEM_ID: &str = "4001";

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MissionTableFile {
    #[serde(default)]
    pub daily_mission_period_info: Vec<DailyPeriod>,
    #[serde(default, deserialize_with = "deserialize_fb_map_or_default")]
    pub periodical_rewards: HashMap<String, PeriodicalReward>,
    #[serde(default, deserialize_with = "deserialize_fb_map_or_default")]
    pub weekly_rewards: HashMap<String, PeriodicalReward>,
}

/// One date window of daily missions and the reward groups it uses by day
/// of the week.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DailyPeriod {
    #[serde(default)]
    pub start_time: i64,
    #[serde(default)]
    pub end_time: i64,
    #[serde(default)]
    pub period_list: Vec<DailyPeriodGroup>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DailyPeriodGroup {
    #[serde(default)]
    pub reward_group_id: String,
    /// Days of the week (1 = Monday .. 7 = Sunday) this group serves.
    #[serde(default)]
    pub period: Vec<u8>,
}

/// One chest: what it pays. Weekly chests also carry their own date
/// window; daily ones are windowed by their group's period.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PeriodicalReward {
    #[serde(default)]
    pub group_id: String,
    #[serde(default)]
    pub rewards: Vec<RewardItem>,
    #[serde(default)]
    pub begin_time: i64,
    #[serde(default)]
    pub end_time: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct RewardItem {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub count: i64,
}

/// The mission rewards the account can earn on a schedule.
#[derive(Debug, Clone, Default)]
pub struct MissionData {
    pub daily_periods: Vec<DailyPeriod>,
    pub daily_rewards: HashMap<String, PeriodicalReward>,
    pub weekly_rewards: HashMap<String, PeriodicalReward>,
}

impl MissionData {
    pub fn from_table(file: MissionTableFile) -> Self {
        Self {
            daily_periods: file.daily_mission_period_info,
            daily_rewards: file.periodical_rewards,
            weekly_rewards: file.weekly_rewards,
        }
    }

    /// LMD the daily chests pay on an average day at `now` (unix seconds):
    /// each live group's LMD weighted by the days of the week it serves.
    /// `None` when no daily period covers `now`.
    pub fn daily_lmd_per_day(&self, now: i64) -> Option<f64> {
        let period = self
            .daily_periods
            .iter()
            .find(|p| p.start_time <= now && now <= p.end_time)?;
        let mut total = 0.0;
        for group in &period.period_list {
            let lmd: i64 = self
                .daily_rewards
                .values()
                .filter(|r| r.group_id == group.reward_group_id)
                .map(|r| lmd_of(&r.rewards))
                .sum();
            #[allow(clippy::cast_precision_loss)]
            let days = group.period.len() as f64;
            #[allow(clippy::cast_precision_loss)]
            let lmd = lmd as f64;
            total += lmd * days / 7.0;
        }
        Some(total)
    }

    /// LMD the live weekly chests pay per day (their week's total over 7).
    /// `None` when no weekly chest is live at `now`.
    pub fn weekly_lmd_per_day(&self, now: i64) -> Option<f64> {
        let live: Vec<&PeriodicalReward> = self
            .weekly_rewards
            .values()
            .filter(|r| r.begin_time <= now && now <= r.end_time)
            .collect();
        if live.is_empty() {
            return None;
        }
        let lmd: i64 = live.iter().map(|r| lmd_of(&r.rewards)).sum();
        #[allow(clippy::cast_precision_loss)]
        Some(lmd as f64 / 7.0)
    }
}

fn lmd_of(rewards: &[RewardItem]) -> i64 {
    rewards
        .iter()
        .filter(|r| r.id == LMD_ITEM_ID)
        .map(|r| r.count)
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chest(group: &str, lmd: i64, begin: i64, end: i64) -> PeriodicalReward {
        PeriodicalReward {
            group_id: group.into(),
            rewards: vec![
                RewardItem {
                    id: LMD_ITEM_ID.into(),
                    count: lmd,
                },
                RewardItem {
                    id: "2001".into(),
                    count: 4,
                },
            ],
            begin_time: begin,
            end_time: end,
        }
    }

    #[test]
    fn daily_lmd_weights_weekday_and_weekend_groups_by_their_days() {
        let data = MissionData {
            daily_periods: vec![DailyPeriod {
                start_time: 100,
                end_time: 200,
                period_list: vec![
                    DailyPeriodGroup {
                        reward_group_id: "wk".into(),
                        period: vec![1, 2, 3, 4, 5],
                    },
                    DailyPeriodGroup {
                        reward_group_id: "we".into(),
                        period: vec![6, 7],
                    },
                ],
            }],
            daily_rewards: HashMap::from([
                ("a".to_string(), chest("wk", 700, 0, 0)),
                ("b".to_string(), chest("wk", 700, 0, 0)),
                ("c".to_string(), chest("we", 1400, 0, 0)),
            ]),
            weekly_rewards: HashMap::new(),
        };
        // Weekdays 1400 x 5/7, weekends 1400 x 2/7: 1400 a day.
        assert!((data.daily_lmd_per_day(150).unwrap() - 1400.0).abs() < 1e-9);
        assert!(
            data.daily_lmd_per_day(300).is_none(),
            "outside every period"
        );
    }

    #[test]
    fn weekly_lmd_is_the_live_chests_over_seven_days() {
        let data = MissionData {
            daily_periods: Vec::new(),
            daily_rewards: HashMap::new(),
            weekly_rewards: HashMap::from([
                ("old".to_string(), chest("g1", 7000, 0, 99)),
                ("live".to_string(), chest("g3", 7000, 100, 200)),
                ("live2".to_string(), chest("g3", 14000, 100, 200)),
            ]),
        };
        assert!((data.weekly_lmd_per_day(150).unwrap() - 3000.0).abs() < 1e-9);
        assert!((data.weekly_lmd_per_day(50).unwrap() - 1000.0).abs() < 1e-9);
        assert!(data.weekly_lmd_per_day(300).is_none());
    }
}
