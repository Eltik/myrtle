use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::serde_helpers::deserialize_fb_map_or_default;

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

/// Date window of daily missions and its reward groups by day of week
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
    /// Days of week served (1 = Monday .. 7 = Sunday)
    #[serde(default)]
    pub period: Vec<u8>,
}

/// One chest. Weekly chests carry their own date window
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

/// Scheduled mission rewards
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

    /// Average daily-chest LMD per day at `now` (unix seconds), `None` if no period covers `now`
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

    /// Live weekly-chest LMD per day (week total / 7), `None` if none live at `now`
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
