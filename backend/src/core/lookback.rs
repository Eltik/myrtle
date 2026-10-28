//! Account Lookback, the pure half: which period a moment falls in, how a
//! roster compacts into a snapshot, and the owned-operator series rebuilt from
//! obtain times. The database half is `queries::lookback`, the sync hook and
//! the reads `services::lookback`.

use std::collections::HashMap;

use crate::core::gamedata::types::activity::ActivityBasicInfo;
use crate::core::gamedata::types::operator::Operator;
use crate::database::models::lookback::{
    BuildRow, LookbackModule, LookbackOperator, LookbackSummary, OwnershipPoint, StoredBuild,
    StoredOperator,
};

const DAY_SECS: i64 = 86_400;

/// How old the latest event launch may be before periods fall back to weeks.
/// A POLICY, not a derivation: EN's longest gap between two launches from
/// 2024 to 2026 is 26 days (2025-12-11 to 2026-01-06), so this never fires
/// while the game data is current and catches a stale or empty table.
pub const STALE_LAUNCH_SECS: i64 = 28 * DAY_SECS;

/// Snapshots older than this keep only the first of each calendar month.
/// Used by `queries::lookback::prune`, spelled here so the policy is in one place.
pub const KEEP_EVERY_DAYS: i64 = 365;

/// A snapshot period: the moment it opened and the event that opened it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Period {
    /// Unix seconds.
    pub start: i64,
    /// `None` for a week period.
    pub activity_id: Option<String>,
    pub activity_name: Option<String>,
}

/// Whether `activity`'s launch opens a period: an event with stages that is
/// not a one-time competitive mode, so side stories, Intermezzi, Vignettes,
/// reruns and main-story events count, and sign-ins, login events, Contingency
/// Contract, Boss Rush and the like do not. `display_type` is not used: EN files
/// several side stories (act42side, act44side, act45side) under `NONE`.
pub fn is_launch(activity: &ActivityBasicInfo) -> bool {
    activity.has_stage && activity.start_time > 0 && !activity.is_one_time_competitive()
}

/// 00:00 UTC of the Monday of the week holding `now`. 1970-01-01 was a
/// Thursday, three days after a Monday.
pub fn week_start(now: i64) -> i64 {
    let day = now.div_euclid(DAY_SECS);
    (day - (day + 3).rem_euclid(7)) * DAY_SECS
}

/// The period `now` falls in: the latest launch at or before `now`, or the
/// current week once that launch is more than [`STALE_LAUNCH_SECS`] old (or
/// there is none). A launch announced for later is not one yet. Two launches
/// at the same second resolve to the smaller id, so the key never flaps.
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

/// The level cap of `operator_id` at promotion `elite`, from `operators`.
pub fn level_cap(operators: &HashMap<String, Operator>, operator_id: &str, elite: i16) -> Option<i32> {
    let phase = usize::try_from(elite).ok()?;
    operators
        .get(operator_id)?
        .phases
        .get(phase)
        .map(|p| p.max_level)
}

/// The non-roster numbers a summary carries, read beside the rows.
#[derive(Debug, Clone, Default)]
pub struct SummaryExtras {
    pub skins: u32,
    pub total_score: Option<f64>,
    pub grade: Option<String>,
}

/// `rows` as a snapshot: the counts and the stored build. `cap` answers the
/// level cap of an operator at a promotion. Masteries are stored only for an
/// operator with at least one, and locked modules never reach `rows`.
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
        s.modules += modules.len() as u32;
        s.module_stage3 += modules.iter().filter(|(_, stage)| *stage >= 3).count() as u32;
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

/// A stored build in the API's shape.
pub fn expand(build: StoredBuild) -> Vec<LookbackOperator> {
    build
        .operators
        .into_iter()
        .map(
            |StoredOperator(operator_id, elite, level, potential, skill_level, masteries, modules)| {
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

/// The owned count at the end of each UTC day an operator was obtained,
/// oldest first. A non-positive time (never seen on the local copy: 0 of
/// 620,119 rows) is not placed on the line.
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

#[cfg(test)]
mod tests {
    use super::*;

    fn act(id: &str, start: i64, has_stage: bool, kind: &str) -> ActivityBasicInfo {
        ActivityBasicInfo {
            id: id.to_owned(),
            name: format!("{id} name"),
            start_time: start,
            has_stage,
            activity_type: kind.to_owned(),
            ..ActivityBasicInfo::default()
        }
    }

    // 2026-09-16 00:00 UTC, a Wednesday.
    const SEP16: i64 = 1_789_516_800;

    #[test]
    fn week_start_is_monday_utc() {
        // Wednesday 2026-09-16 12:00 -> Monday 2026-09-14 00:00.
        assert_eq!(week_start(SEP16 + 12 * 3600), SEP16 - 2 * DAY_SECS);
        // A Monday maps to itself; the Sunday before maps a week back.
        let monday = SEP16 - 2 * DAY_SECS;
        assert_eq!(week_start(monday), monday);
        assert_eq!(week_start(monday - 1), monday - 7 * DAY_SECS);
        assert_eq!(week_start(0), -3 * DAY_SECS);
    }

    #[test]
    fn the_latest_started_launch_keys_the_period() {
        let acts = [
            act("act41sre", SEP16 - 20 * DAY_SECS, true, "TYPE_ACT9D0"),
            act("act51side", SEP16, true, "TYPE_ACT9D0"),
            // Started later but not a launch: a duel and a sign-in.
            act("act3enemyduel", SEP16 + 7 * DAY_SECS, true, "ENEMY_DUEL"),
            act("act59sign", SEP16 + 8 * DAY_SECS, false, "CHECKIN_ONLY"),
            // A launch announced for the future is not one yet.
            act("act52side", SEP16 + 30 * DAY_SECS, true, "TYPE_ACT9D0"),
        ];
        let p = current_period(&acts, SEP16 + 10 * DAY_SECS);
        assert_eq!(p.start, SEP16);
        assert_eq!(p.activity_id.as_deref(), Some("act51side"));
        assert_eq!(p.activity_name.as_deref(), Some("act51side name"));
        // The second it launches, it is the period.
        assert_eq!(current_period(&acts, SEP16).start, SEP16);
        assert_eq!(current_period(&acts, SEP16 - 1).start, SEP16 - 20 * DAY_SECS);
    }

    #[test]
    fn a_stale_or_missing_launch_falls_back_to_the_week() {
        let acts = [act("act51side", SEP16, true, "TYPE_ACT9D0")];
        let at_limit = SEP16 + STALE_LAUNCH_SECS;
        assert_eq!(current_period(&acts, at_limit).start, SEP16);
        let past = at_limit + 1;
        let p = current_period(&acts, past);
        assert_eq!(p.start, week_start(past));
        assert_eq!(p.activity_id, None);
        assert!(p.start > SEP16, "the week period opens after the launch");
        let none: [ActivityBasicInfo; 0] = [];
        assert_eq!(current_period(&none, SEP16).start, week_start(SEP16));
    }

    #[test]
    fn simultaneous_launches_resolve_to_the_smaller_id() {
        let acts = [
            act("act9", SEP16, true, "TYPE_ACT9D0"),
            act("act10", SEP16, true, "MINISTORY"),
        ];
        let rev = [acts[1].clone(), acts[0].clone()];
        assert_eq!(current_period(&acts, SEP16).activity_id.as_deref(), Some("act10"));
        assert_eq!(current_period(&rev, SEP16).activity_id.as_deref(), Some("act10"));
    }

    fn row(id: &str, elite: i16, level: i16, pot: i16, sl: i16, m: &[i16], modules: &[(&str, i16)]) -> BuildRow {
        BuildRow {
            operator_id: id.to_owned(),
            elite,
            level,
            potential: pot,
            skill_level: sl,
            masteries: m.to_vec(),
            module_ids: modules.iter().map(|(id, _)| (*id).to_owned()).collect(),
            module_levels: modules.iter().map(|(_, s)| *s).collect(),
        }
    }

    #[test]
    fn compact_counts_and_strips_the_build() {
        let rows = [
            row("char_a", 2, 90, 5, 7, &[3, 0, 3], &[("uniequip_002_a", 3), ("uniequip_003_a", 1)]),
            row("char_b", 2, 60, 0, 7, &[1, 2], &[]),
            row("char_c", 1, 55, 2, 4, &[0, 0], &[]),
            row("char_d", 0, 30, 5, 1, &[], &[]),
        ];
        let caps = |id: &str, elite: i16| match (id, elite) {
            ("char_a", 2) => Some(90),
            ("char_b", 2) => Some(80),
            ("char_c", 1) => Some(55),
            _ => None,
        };
        let extras = SummaryExtras {
            skins: 12,
            total_score: Some(0.5),
            grade: Some("A".into()),
        };
        let (s, build) = compact(&rows, extras, caps);
        assert_eq!(
            s,
            LookbackSummary {
                operators: 4,
                elite2: 2,
                // char_a and char_c; char_d has no known cap.
                max_level: 2,
                skill_level7: 2,
                masteries: 9,
                mastery3: 2,
                modules: 2,
                module_stage3: 1,
                max_potential: 2,
                skins: 12,
                total_score: Some(0.5),
                grade: Some("A".into()),
            }
        );
        // Unmastered skills are not stored, mastered ones keep their index.
        assert_eq!(build.operators[0].5, vec![3, 0, 3]);
        assert!(build.operators[2].5.is_empty());
        let json = serde_json::to_string(&build).expect("serializes");
        assert_eq!(
            json,
            r#"{"o":[["char_a",2,90,5,7,[3,0,3],[["uniequip_002_a",3],["uniequip_003_a",1]]],["char_b",2,60,0,7,[1,2],[]],["char_c",1,55,2,4,[],[]],["char_d",0,30,5,1,[],[]]]}"#
        );
        // The stored form reads back to the same build, in the API's shape.
        let back: StoredBuild = serde_json::from_str(&json).expect("reads back");
        let ops = expand(back);
        assert_eq!(ops[0].modules[0], LookbackModule { module_id: "uniequip_002_a".into(), stage: 3 });
        assert_eq!(ops[3].operator_id, "char_d");
    }

    #[test]
    fn a_summary_missing_a_later_field_still_reads() {
        let s: LookbackSummary = serde_json::from_str(r#"{"operators":3}"#).expect("reads");
        assert_eq!(s.operators, 3);
        assert_eq!(s.grade, None);
    }

    #[test]
    fn ownership_steps_once_per_day() {
        let day = SEP16;
        let pts = ownership_series(vec![day + 50, 0, day - DAY_SECS + 10, day + 10, -5]);
        assert_eq!(
            pts,
            vec![
                OwnershipPoint { ts: day - DAY_SECS, count: 1 },
                OwnershipPoint { ts: day, count: 3 },
            ]
        );
        assert!(ownership_series(Vec::new()).is_empty());
    }
}
