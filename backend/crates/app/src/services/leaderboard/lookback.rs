use std::collections::HashMap;

use chrono::{DateTime, Utc};
use myrtle_db::repos::lookback::{self, BuildRow, NewSnapshot, SummaryExtrasRow};
use myrtle_gamedata_raw::tables::activity::ActivityBasicInfo;
use myrtle_gamedata_raw::tables::operator::Operator;
use myrtle_types::{Server, UserId};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::privacy::{self, layout::ProfileTabId};
use crate::{AppContext, AppError, Subject};

const DAY_SECS: i64 = 86_400;
pub const STALE_LAUNCH_SECS: i64 = 28 * DAY_SECS;
pub const KEEP_EVERY_DAYS: i32 = 365;
const TABS: [ProfileTabId; 2] = [ProfileTabId::Lookback, ProfileTabId::Roster];

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct LookbackSummary {
    pub operators: u32,
    pub elite2: u32,
    pub max_level: u32,
    pub skill_level7: u32,
    pub masteries: u32,
    pub mastery3: u32,
    pub modules: u32,
    pub module_stage3: u32,
    pub max_potential: u32,
    pub skins: u32,
    pub total_score: Option<f64>,
    pub grade: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LookbackSnapshot {
    pub id: i64,
    pub period_start: DateTime<Utc>,
    pub taken_at: DateTime<Utc>,
    pub activity_id: Option<String>,
    pub activity_name: Option<String>,
    pub summary: LookbackSummary,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LookbackModule {
    pub module_id: String,
    pub stage: i16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LookbackOperator {
    pub operator_id: String,
    pub elite: i16,
    pub level: i16,
    pub potential: i16,
    pub skill_level: i16,
    pub masteries: Vec<i16>,
    pub modules: Vec<LookbackModule>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LookbackBuild {
    pub taken_at: DateTime<Utc>,
    pub summary: LookbackSummary,
    pub operators: Vec<LookbackOperator>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct OwnershipPoint {
    pub ts: i64,
    pub count: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct LookbackOverview {
    pub snapshots: Vec<LookbackSnapshot>,
    pub ownership: Vec<OwnershipPoint>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredOperator(
    pub String,
    pub i16,
    pub i16,
    pub i16,
    pub i16,
    pub Vec<i16>,
    pub Vec<(String, i16)>,
);

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredBuild {
    #[serde(rename = "o")]
    pub operators: Vec<StoredOperator>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Period {
    pub start: i64,
    pub activity_id: Option<String>,
    pub activity_name: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct SummaryExtras {
    pub skins: u32,
    pub total_score: Option<f64>,
    pub grade: Option<String>,
}

impl From<SummaryExtrasRow> for SummaryExtras {
    fn from(r: SummaryExtrasRow) -> Self {
        Self {
            skins: u32::try_from(r.skins).unwrap_or(0),
            total_score: r.total_score,
            grade: r.grade,
        }
    }
}

#[must_use]
pub fn is_launch(activity: &ActivityBasicInfo) -> bool {
    activity.has_stage && activity.start_time > 0 && !activity.is_one_time_competitive()
}

#[must_use]
pub const fn week_start(now: i64) -> i64 {
    let day = now.div_euclid(DAY_SECS);
    (day - (day + 3).rem_euclid(7)) * DAY_SECS
}

pub fn current_period<'a>(
    activities: impl IntoIterator<Item = &'a ActivityBasicInfo>,
    now: i64,
) -> Period {
    let latest = activities
        .into_iter()
        .filter(|a| is_launch(a) && a.start_time <= now)
        .max_by(|a, b| a.start_time.cmp(&b.start_time).then(b.id.cmp(&a.id)));
    match latest {
        Some(a) if now - a.start_time <= STALE_LAUNCH_SECS => Period {
            start: a.start_time,
            activity_id: Some(a.id.clone()),
            activity_name: Some(a.name.clone()).filter(|n| !n.is_empty()),
        },
        _ => Period {
            start: week_start(now),
            activity_id: None,
            activity_name: None,
        },
    }
}

#[must_use]
pub fn level_cap(
    operators: &HashMap<String, Operator>,
    operator_id: &str,
    elite: i16,
) -> Option<i32> {
    let phase = usize::try_from(elite).ok()?;
    operators
        .get(operator_id)?
        .phases
        .get(phase)
        .map(|p| p.max_level)
}

pub fn compact(
    rows: &[BuildRow],
    extras: SummaryExtras,
    cap: impl Fn(&str, i16) -> Option<i32>,
) -> (LookbackSummary, StoredBuild) {
    let mut s = LookbackSummary {
        skins: extras.skins,
        total_score: extras.total_score,
        grade: extras.grade,
        ..LookbackSummary::default()
    };
    let mut operators = Vec::with_capacity(rows.len());
    for r in rows {
        s.operators += 1;
        s.elite2 += u32::from(r.elite == 2);
        s.max_level += u32::from(cap(&r.operator_id, r.elite) == Some(i32::from(r.level)));
        s.skill_level7 += u32::from(r.skill_level >= 7);
        s.max_potential += u32::from(r.potential >= 5);
        for &m in &r.masteries {
            s.masteries += u32::try_from(m).unwrap_or(0);
            s.mastery3 += u32::from(m >= 3);
        }
        let modules: Vec<(String, i16)> = r
            .module_ids
            .iter()
            .zip(&r.module_levels)
            .filter(|&(_, &stage)| stage > 0)
            .map(|(id, &stage)| (id.clone(), stage))
            .collect();
        for (_, stage) in &modules {
            s.modules += 1;
            s.module_stage3 += u32::from(*stage >= 3);
        }
        let masteries = if r.masteries.iter().any(|&m| m > 0) {
            r.masteries.clone()
        } else {
            Vec::new()
        };
        operators.push(StoredOperator(
            r.operator_id.clone(),
            r.elite,
            r.level,
            r.potential,
            r.skill_level,
            masteries,
            modules,
        ));
    }
    (s, StoredBuild { operators })
}

#[must_use]
pub fn expand(build: StoredBuild) -> Vec<LookbackOperator> {
    build
        .operators
        .into_iter()
        .map(
            |StoredOperator(
                operator_id,
                elite,
                level,
                potential,
                skill_level,
                masteries,
                modules,
            )| {
                LookbackOperator {
                    operator_id,
                    elite,
                    level,
                    potential,
                    skill_level,
                    masteries,
                    modules: modules
                        .into_iter()
                        .map(|(module_id, stage)| LookbackModule { module_id, stage })
                        .collect(),
                }
            },
        )
        .collect()
}

#[must_use]
pub fn ownership_series(mut obtained: Vec<i64>) -> Vec<OwnershipPoint> {
    obtained.retain(|&t| t > 0);
    obtained.sort_unstable();
    let mut out: Vec<OwnershipPoint> = Vec::new();
    for (i, t) in obtained.iter().enumerate() {
        let day = t.div_euclid(DAY_SECS) * DAY_SECS;
        let count = u32::try_from(i + 1).unwrap_or(u32::MAX);
        match out.last_mut() {
            Some(last) if last.ts == day => last.count = count,
            _ => out.push(OwnershipPoint { ts: day, count }),
        }
    }
    out
}

fn decode<T: serde::de::DeserializeOwned>(value: Value) -> Result<T, AppError> {
    serde_json::from_value(value).map_err(AppError::internal)
}

fn encode<T: Serialize>(value: &T) -> Result<Value, AppError> {
    serde_json::to_value(value).map_err(AppError::internal)
}

pub async fn record(ctx: &AppContext, user: UserId, server: Server) -> Result<bool, AppError> {
    let snapshot = ctx.snapshots.or_default(server)?;
    let model = &snapshot.model;
    let period = current_period(model.activities.values(), Utc::now().timestamp());
    let mut conn = ctx.db.conn().await?;
    if lookback::period_taken(&mut conn, user, period.start).await? {
        return Ok(false);
    }
    let Some(extras) = lookback::summary_extras(&mut conn, user).await? else {
        return Ok(false);
    };
    let rows = lookback::build_rows(&mut conn, user).await?;
    let (summary, build) = compact(&rows, extras.into(), |id, elite| {
        level_cap(&model.operators, id, elite)
    });
    let new = NewSnapshot {
        period_start: period.start,
        activity_id: period.activity_id.as_deref(),
        activity_name: period.activity_name.as_deref(),
        summary: &encode(&summary)?,
        build: &encode(&build)?,
    };
    Ok(lookback::insert(&mut conn, user, &new, KEEP_EVERY_DAYS).await?)
}

pub async fn overview(
    ctx: &AppContext,
    subject: &Subject,
    uid: Option<&str>,
) -> Result<LookbackOverview, AppError> {
    let user = privacy::resolve_user_id(ctx, subject, uid, &TABS).await?;
    let mut conn = ctx.db.conn().await?;
    let snapshots = lookback::list(&mut conn, user)
        .await?
        .into_iter()
        .map(|r| {
            Ok(LookbackSnapshot {
                id: r.period_start.timestamp(),
                period_start: r.period_start,
                taken_at: r.taken_at,
                activity_id: r.activity_id,
                activity_name: r.activity_name,
                summary: decode(r.summary)?,
            })
        })
        .collect::<Result<_, AppError>>()?;
    let ownership = ownership_series(lookback::obtained_times(&mut conn, user).await?);
    Ok(LookbackOverview {
        snapshots,
        ownership,
    })
}

pub async fn snapshot(
    ctx: &AppContext,
    subject: &Subject,
    uid: Option<&str>,
    id: i64,
) -> Result<LookbackBuild, AppError> {
    let user = privacy::resolve_user_id(ctx, subject, uid, &TABS).await?;
    let mut conn = ctx.db.conn().await?;
    let row = lookback::get(&mut conn, user, id)
        .await?
        .ok_or(AppError::NotFound(None))?;
    Ok(LookbackBuild {
        taken_at: row.taken_at,
        summary: decode(row.summary)?,
        operators: expand(decode(row.build)?),
    })
}

pub async fn current(
    ctx: &AppContext,
    subject: &Subject,
    uid: Option<&str>,
) -> Result<LookbackBuild, AppError> {
    let user = privacy::resolve_user_id(ctx, subject, uid, &TABS).await?;
    let mut conn = ctx.db.conn().await?;
    let extras = lookback::summary_extras(&mut conn, user)
        .await?
        .ok_or(AppError::NotFound(None))?;
    let rows = lookback::build_rows(&mut conn, user).await?;
    drop(conn);
    let server = Server::from_index(extras.server_id).unwrap_or(Server::EN);
    let snapshot = ctx.snapshots.or_default(server)?;
    let (summary, build) = compact(&rows, extras.into(), |id, elite| {
        level_cap(&snapshot.model.operators, id, elite)
    });
    Ok(LookbackBuild {
        taken_at: Utc::now(),
        summary,
        operators: expand(build),
    })
}
