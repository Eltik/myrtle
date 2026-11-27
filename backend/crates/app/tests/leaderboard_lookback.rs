#![allow(clippy::unwrap_used)]

mod common;

use myrtle_app::services::leaderboard::lookback::{
    LookbackModule, LookbackSummary, OwnershipPoint, STALE_LAUNCH_SECS, StoredBuild, SummaryExtras,
    compact, current_period, expand, overview, ownership_series, record, snapshot, week_start,
};
use myrtle_app::{AppError, Subject};
use myrtle_db::repos::lookback::BuildRow;
use myrtle_db::repos::users::{self, LayoutUpdate, SettingsUpdate};
use myrtle_gamedata_raw::tables::activity::ActivityBasicInfo;
use myrtle_types::{Role, Server};
use serde_json::json;

const DAY: i64 = 86_400;
const SEP16: i64 = 1_789_516_800;

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

#[test]
fn week_start_is_monday_utc() {
    assert_eq!(week_start(SEP16 + 12 * 3600), SEP16 - 2 * DAY);
    let monday = SEP16 - 2 * DAY;
    assert_eq!(week_start(monday), monday);
    assert_eq!(week_start(monday - 1), monday - 7 * DAY);
    assert_eq!(week_start(0), -3 * DAY);
}

#[test]
fn the_latest_started_launch_keys_the_period() {
    let acts = [
        act("act41sre", SEP16 - 20 * DAY, true, "TYPE_ACT9D0"),
        act("act51side", SEP16, true, "TYPE_ACT9D0"),
        act("act3enemyduel", SEP16 + 7 * DAY, true, "ENEMY_DUEL"),
        act("act59sign", SEP16 + 8 * DAY, false, "CHECKIN_ONLY"),
        act("act52side", SEP16 + 30 * DAY, true, "TYPE_ACT9D0"),
    ];
    let p = current_period(&acts, SEP16 + 10 * DAY);
    assert_eq!(p.start, SEP16);
    assert_eq!(p.activity_id.as_deref(), Some("act51side"));
    assert_eq!(p.activity_name.as_deref(), Some("act51side name"));
    assert_eq!(current_period(&acts, SEP16).start, SEP16);
    assert_eq!(current_period(&acts, SEP16 - 1).start, SEP16 - 20 * DAY);
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
    assert!(p.start > SEP16);
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
    assert_eq!(
        current_period(&acts, SEP16).activity_id.as_deref(),
        Some("act10")
    );
    assert_eq!(
        current_period(&rev, SEP16).activity_id.as_deref(),
        Some("act10")
    );
}

fn row(
    id: &str,
    elite: i16,
    level: i16,
    potential: i16,
    skill_level: i16,
    masteries: &[i16],
    modules: &[(&str, i16)],
) -> BuildRow {
    BuildRow {
        operator_id: id.to_owned(),
        elite,
        level,
        potential,
        skill_level,
        masteries: masteries.to_vec(),
        module_ids: modules.iter().map(|(id, _)| (*id).to_owned()).collect(),
        module_levels: modules.iter().map(|(_, s)| *s).collect(),
    }
}

#[test]
fn compact_counts_and_strips_the_build() {
    let rows = [
        row(
            "char_a",
            2,
            90,
            5,
            7,
            &[3, 0, 3],
            &[("uniequip_002_a", 3), ("uniequip_003_a", 1)],
        ),
        row("char_b", 2, 60, 0, 7, &[1, 2], &[]),
        row("char_c", 1, 55, 2, 4, &[0, 0], &[("uniequip_002_c", 0)]),
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
    assert_eq!(build.operators[0].5, vec![3, 0, 3]);
    assert!(build.operators[2].5.is_empty());
    let json = serde_json::to_string(&build).unwrap();
    assert_eq!(
        json,
        r#"{"o":[["char_a",2,90,5,7,[3,0,3],[["uniequip_002_a",3],["uniequip_003_a",1]]],["char_b",2,60,0,7,[1,2],[]],["char_c",1,55,2,4,[],[]],["char_d",0,30,5,1,[],[]]]}"#
    );
    let back: StoredBuild = serde_json::from_str(&json).unwrap();
    let ops = expand(back);
    assert_eq!(
        ops[0].modules[0],
        LookbackModule {
            module_id: "uniequip_002_a".into(),
            stage: 3
        }
    );
    assert_eq!(ops[3].operator_id, "char_d");
}

#[test]
fn a_summary_missing_a_later_field_still_reads() {
    let s: LookbackSummary = serde_json::from_str(r#"{"operators":3}"#).unwrap();
    assert_eq!(s.operators, 3);
    assert_eq!(s.grade, None);
}

#[test]
fn ownership_steps_once_per_day() {
    let pts = ownership_series(vec![SEP16 + 50, 0, SEP16 - DAY + 10, SEP16 + 10, -5]);
    assert_eq!(
        pts,
        vec![
            OwnershipPoint {
                ts: SEP16 - DAY,
                count: 1
            },
            OwnershipPoint {
                ts: SEP16,
                count: 3
            },
        ]
    );
    assert!(ownership_series(Vec::new()).is_empty());
}

#[tokio::test]
async fn reads_are_gated_on_profile_and_both_tabs() {
    let (ctx, db) = common::context().await;
    let owner = db.user(7001, Server::EN).await;
    let visitor = common::user(&ctx, db.user(7002, Server::EN).await, Role::User).await;

    assert!(matches!(
        overview(&ctx, &Subject::Anonymous, None).await,
        Err(AppError::Unauthorized)
    ));
    let own = overview(&ctx, &common::user(&ctx, owner, Role::User).await, None)
        .await
        .unwrap();
    assert!(own.snapshots.is_empty() && own.ownership.is_empty());

    assert!(matches!(
        overview(&ctx, &visitor, Some("7001")).await,
        Err(AppError::Forbidden)
    ));
    assert!(matches!(
        overview(&ctx, &Subject::Anonymous, Some("7001")).await,
        Err(AppError::Forbidden)
    ));
    assert!(matches!(
        overview(&ctx, &visitor, Some("9999")).await,
        Err(AppError::NotFound(_))
    ));

    let mut conn = ctx.db.conn().await.unwrap();
    users::set_base_facts(&mut conn, owner, &json!({}))
        .await
        .unwrap();
    let patch = json!({ "tabs": [{ "id": "roster", "visible": false }] });
    users::update_settings(
        &mut conn,
        owner,
        &SettingsUpdate {
            public_profile: Some(true),
            store_gacha: None,
            share_stats: None,
            public_gacha: None,
            layout: LayoutUpdate::Merge {
                patch: &patch,
                removed: &[],
            },
        },
    )
    .await
    .unwrap();
    drop(conn);
    assert!(matches!(
        overview(&ctx, &visitor, Some("7001")).await,
        Err(AppError::Forbidden)
    ));
    assert!(
        overview(
            &ctx,
            &common::user(&ctx, owner, Role::User).await,
            Some("7001")
        )
        .await
        .is_ok()
    );

    let mut conn = ctx.db.conn().await.unwrap();
    let patch = json!({ "tabs": [{ "id": "roster", "visible": true }, { "id": "lookback", "visible": false }] });
    users::update_settings(
        &mut conn,
        owner,
        &SettingsUpdate {
            public_profile: None,
            store_gacha: None,
            share_stats: None,
            public_gacha: None,
            layout: LayoutUpdate::Merge {
                patch: &patch,
                removed: &[],
            },
        },
    )
    .await
    .unwrap();
    drop(conn);
    assert!(matches!(
        overview(&ctx, &visitor, Some("7001")).await,
        Err(AppError::Forbidden)
    ));

    let mut conn = ctx.db.conn().await.unwrap();
    users::update_settings(
        &mut conn,
        owner,
        &SettingsUpdate {
            public_profile: None,
            store_gacha: None,
            share_stats: None,
            public_gacha: None,
            layout: LayoutUpdate::Clear,
        },
    )
    .await
    .unwrap();
    drop(conn);
    assert!(overview(&ctx, &visitor, Some("7001")).await.is_ok());
    assert!(matches!(
        snapshot(&ctx, &visitor, Some("7001"), SEP16).await,
        Err(AppError::NotFound(_))
    ));
    db.close().await;
}

#[tokio::test]
async fn record_writes_one_snapshot_a_period() {
    let Some((ctx, db, _)) = common::context_with_data().await else {
        return;
    };
    let user = db.user(7101, Server::EN).await;
    assert!(record(&ctx, user, Server::EN).await.unwrap());
    assert!(!record(&ctx, user, Server::EN).await.unwrap());
    let owner = common::user(&ctx, user, Role::User).await;
    let timeline = overview(&ctx, &owner, None).await.unwrap();
    assert_eq!(timeline.snapshots.len(), 1);
    let id = timeline.snapshots[0].id;
    let build = snapshot(&ctx, &owner, None, id).await.unwrap();
    assert_eq!(build.summary.operators, 0);
    assert!(build.operators.is_empty());
    db.close().await;
}
