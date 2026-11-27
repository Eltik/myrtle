#![allow(clippy::unwrap_used, clippy::float_cmp)]

mod common;

use std::time::Duration;

use myrtle_app::AppError;
use myrtle_app::cache::key_hash;
use myrtle_app::services::catalog::dps::{
    self, BatchRequest, BatchResponse, CalculateRequest, HpsBatchResponse, RankRequest,
    RetentionStatus, SupportersQuery, Sweep, SweepAxis, enemy_roster,
};
use myrtle_engine_dps::supporters::SupportersResponse;
use myrtle_engine_dps::{DpsResult, ENGINE_VERSION, HpsResult};

const CEOBE: &str = "char_2013_cerber";
const PTILOPSIS: &str = "char_128_plosis";

fn calc(operator_id: &str, skill: i32) -> CalculateRequest {
    serde_json::from_value(serde_json::json!({
        "operatorId": operator_id,
        "skillIndex": skill,
        "defense": 300.0,
        "res": 20.0,
    }))
    .unwrap()
}

#[tokio::test]
async fn identity_is_the_operator_table_digest_whatever_the_version() {
    let Some((ctx, db, snapshot)) = common::context_with_data().await else {
        return;
    };
    let value = serde_json::to_value(&snapshot.model.operators).unwrap();
    let expected = key_hash(&serde_json::to_vec(&value).unwrap());
    assert_eq!(dps::data(&ctx).await.unwrap().identity(), expected);
    db.close().await;
}

#[tokio::test]
async fn calculations_are_finite_and_cached() {
    let Some((ctx, db, _)) = common::context_with_data().await else {
        return;
    };
    let identity = dps::data(&ctx).await.unwrap().identity().to_owned();
    assert_eq!(identity.len(), 16);
    assert_eq!(dps::data(&ctx).await.unwrap().identity(), identity);

    let req = calc(CEOBE, 3);
    let first = dps::calculate(&ctx, req.clone()).await.unwrap();
    let result: DpsResult = serde_json::from_str(&first.body).unwrap();
    for v in [result.skill_dps, result.total_damage, result.average_dps] {
        assert!(v.is_finite() && v > 0.0, "{v}");
    }
    let key = format!(
        "dps:calc:dps3:{ENGINE_VERSION}:{identity}:{}",
        key_hash(serde_json::to_string(&req).unwrap().as_bytes())
    );
    assert!(ctx.cache.get(&key).await.is_some(), "stored under {key}");
    let second = dps::calculate(&ctx, req).await.unwrap();
    assert_eq!(second.etag, first.etag);

    let unknown = dps::calculate(&ctx, calc("char_nope", 1)).await;
    assert!(matches!(unknown, Err(AppError::NotFound(None))));

    let mut out_of_range = calc(CEOBE, 3);
    out_of_range.level = Some(91);
    assert!(matches!(
        dps::calculate(&ctx, out_of_range).await,
        Err(AppError::BadRequest(_))
    ));

    let hps = dps::calculate_hps(&ctx, calc(PTILOPSIS, 2)).await.unwrap();
    let healed: HpsResult = serde_json::from_str(&hps.body).unwrap();
    for v in [healed.skill_hps, healed.avg_hps] {
        assert!(v.is_finite() && v > 0.0, "{v}");
    }
    let again = dps::calculate_hps(&ctx, calc(PTILOPSIS, 2)).await.unwrap();
    assert_eq!(again.etag, hps.etag);
    db.close().await;
}

#[tokio::test]
async fn muelsyse_defaults_to_an_ela_clone() {
    let Some((ctx, db, _)) = common::context_with_data().await else {
        return;
    };
    let data = dps::data(&ctx).await.unwrap();
    let result = dps::calculate_in(&data, calc("char_249_mlyss", 3)).unwrap();
    assert!(result.skill_dps.is_finite() && result.skill_dps > 0.0);
    let mut bad_clone = calc("char_249_mlyss", 3);
    bad_clone.clone_operator_id = Some("char_nope".into());
    assert!(matches!(
        dps::calculate_in(&data, bad_clone),
        Err(AppError::BadRequest(_))
    ));
    db.close().await;
}

#[tokio::test]
async fn batches_match_single_calls_and_isolate_failures() {
    let Some((ctx, db, _)) = common::context_with_data().await else {
        return;
    };
    let data = dps::data(&ctx).await.unwrap();
    let body = BatchRequest {
        requests: vec![calc(CEOBE, 3), calc("char_nope", 1)],
        sweep: None,
    };
    let cached = dps::batch(&ctx, body.clone()).await.unwrap();
    let batch: BatchResponse = serde_json::from_str(&cached.body).unwrap();
    let single = dps::calculate_in(&data, calc(CEOBE, 3)).unwrap();
    assert_eq!(
        batch.results[0].result.as_ref().unwrap().skill_dps,
        single.skill_dps
    );
    assert_eq!(
        batch.results[1].error.as_deref(),
        Some("unknown operator char_nope")
    );
    assert_eq!(dps::batch(&ctx, body).await.unwrap().etag, cached.etag);

    let swept = BatchRequest {
        requests: vec![calc(CEOBE, 3)],
        sweep: Some(Sweep {
            axis: SweepAxis::Res,
            min: 0.0,
            max: 50.0,
            steps: 3,
        }),
    };
    let swept: BatchResponse =
        serde_json::from_str(&dps::batch(&ctx, swept.clone()).await.unwrap().body).unwrap();
    let points = swept.results[0].points.as_ref().unwrap();
    assert_eq!(
        points.iter().map(|p| p.x).collect::<Vec<_>>(),
        [0.0, 25.0, 50.0]
    );
    assert!(
        points[0].result.skill_dps > points[2].result.skill_dps,
        "RES cuts arts damage"
    );

    let heals = BatchRequest {
        requests: vec![calc(PTILOPSIS, 2), calc(CEOBE, 3)],
        sweep: None,
    };
    let heals: HpsBatchResponse =
        serde_json::from_str(&dps::batch_hps(&ctx, heals).await.unwrap().body).unwrap();
    assert!(heals.results[0].result.as_ref().unwrap().skill_hps > 0.0);
    assert!(
        heals.results[1].error.is_some(),
        "Ceobe has no heal formula"
    );

    let swept_heals = BatchRequest {
        requests: vec![calc(PTILOPSIS, 2)],
        sweep: Some(Sweep {
            axis: SweepAxis::Def,
            min: 0.0,
            max: 10.0,
            steps: 2,
        }),
    };
    assert!(matches!(
        dps::batch_hps(&ctx, swept_heals).await,
        Err(AppError::BadRequest(_))
    ));
    db.close().await;
}

#[tokio::test]
async fn operator_lists_are_cached_with_an_etag() {
    let Some((ctx, db, _)) = common::context_with_data().await else {
        return;
    };
    let identity = dps::data(&ctx).await.unwrap().identity().to_owned();
    let first = dps::list_operators(&ctx).await.unwrap();
    let entries: Vec<serde_json::Value> = serde_json::from_str(&first.body).unwrap();
    let ceobe = entries.iter().find(|e| e["id"] == CEOBE).unwrap();
    assert!(ceobe["availableSkills"].as_array().unwrap().len() >= 3);
    assert_eq!(
        ceobe["availableModules"].as_array().unwrap().len(),
        ceobe["availableModuleIds"].as_array().unwrap().len()
    );
    let key = format!("dps:list:operators:{ENGINE_VERSION}:{identity}");
    assert!(ctx.cache.get(&key).await.is_some());
    assert_eq!(dps::list_operators(&ctx).await.unwrap().etag, first.etag);

    let healers: Vec<serde_json::Value> =
        serde_json::from_str(&dps::list_healers(&ctx).await.unwrap().body).unwrap();
    assert!(healers.iter().any(|e| e["id"] == PTILOPSIS));
    db.close().await;
}

#[tokio::test]
async fn a_rank_page_is_sorted_cached_and_pending_retention() {
    let Some((ctx, db, _)) = common::context_with_data().await else {
        return;
    };
    let identity = dps::data(&ctx).await.unwrap().identity().to_owned();
    let req = RankRequest {
        limit: Some(10),
        ..RankRequest::default()
    };
    let page = dps::rank(&ctx, req.clone()).await.unwrap();
    assert_eq!(page.rows.len(), 10);
    assert!(page.computed > 100, "{}", page.computed);
    assert_eq!(page.matched, page.computed);
    assert_eq!(page.data, identity);
    assert_eq!(page.retention_status, RetentionStatus::Pending);
    assert!(page.rows.iter().all(|r| r.retention.is_none()));
    for w in page.rows.windows(2) {
        assert!(w[0].skill_dps >= w[1].skill_dps);
    }

    let context = r#"{"defense":null,"res":null,"buffs":null,"shred":null,"targets":null,"spBoost":null,"allCond":null,"zones":null,"secondary":null,"cycleModel":null,"preset":"p1Max"}"#;
    let key = format!(
        "dps:rank:{ENGINE_VERSION}:{identity}:{}",
        key_hash(context.as_bytes())
    );
    assert!(ctx.cache.get(&key).await.is_some(), "stored under {key}");

    let snipers = RankRequest {
        filters: Some(
            serde_json::from_str(r#"{"professions":["sniper"],"rarities":[6]}"#).unwrap(),
        ),
        ..req
    };
    let filtered = dps::rank(&ctx, snipers).await.unwrap();
    assert_eq!(
        filtered.computed, page.computed,
        "filters reuse the cached rows"
    );
    assert!(filtered.matched < page.computed);
    assert!(
        filtered
            .rows
            .iter()
            .all(|r| r.profession == "SNIPER" && r.rarity == 6)
    );
    db.close().await;
}

#[tokio::test]
async fn supporters_resolve_and_validate() {
    let Some((ctx, db, _)) = common::context_with_data().await else {
        return;
    };
    let cached = dps::supporters(&ctx, SupportersQuery::default())
        .await
        .unwrap();
    let response: SupportersResponse = serde_json::from_str(&cached.body).unwrap();
    assert!(!response.entries.is_empty());
    let bad = SupportersQuery {
        skill_level: Some(11),
        ..SupportersQuery::default()
    };
    match dps::supporters(&ctx, bad).await {
        Err(AppError::BadRequest(m)) => {
            assert_eq!(m, "skillLevel must be between 1 and 10 (got 11)");
        }
        other => panic!("{other:?}"),
    }
    db.close().await;
}

#[tokio::test]
async fn roster_is_the_stage_appearances() {
    let Some((_, db, snapshot)) = common::context_with_data().await else {
        return;
    };
    let model = &snapshot.model;
    let (roster, pairs) = enemy_roster(model);
    let total: u64 = pairs.iter().map(|p| p.stages).sum();
    assert_eq!(total as usize, roster.appearances);
    assert_eq!(pairs.len(), roster.distinct_pairs);
    assert_eq!(
        pairs.iter().map(|p| p.spawns).sum::<u64>(),
        roster.total_spawns
    );
    let kept: usize = model
        .enemy_stage_index
        .iter()
        .filter(|(id, _)| {
            model
                .enemies
                .enemy_data
                .get(*id)
                .and_then(|e| e.stats.as_ref())
                .is_some_and(|s| !s.levels.is_empty())
        })
        .map(|(_, refs)| refs.len())
        .sum();
    assert_eq!(kept, roster.appearances);
    for w in pairs.windows(2) {
        assert!((w[0].def, w[0].res) < (w[1].def, w[1].res));
    }
    if roster.appearances > roster.no_level_file {
        assert!(roster.nonzero_level > 0);
        assert!(roster.overridden > 0);
    }
    db.close().await;
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "computes the full retention table, about 30 s in a debug build"]
async fn a_dropped_retention_request_still_finishes_the_table() {
    let Some((ctx, db, _)) = common::context_with_data().await else {
        return;
    };
    let data = dps::data(&ctx).await.unwrap();
    let dropped = tokio::time::timeout(Duration::from_millis(50), dps::retention(&ctx)).await;
    assert!(dropped.is_err(), "the request was dropped mid-computation");
    for _ in 0..600 {
        if data.retention_table().is_some() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    assert!(data.retention_table().is_some());
    db.close().await;
}

#[tokio::test]
#[ignore = "computes the full retention table, about 30 s in a debug build"]
async fn retention_scores_every_row_with_damage() {
    let Some((ctx, db, _)) = common::context_with_data().await else {
        return;
    };
    let response = dps::retention(&ctx).await.unwrap();
    assert_eq!(response.status, RetentionStatus::Ready);
    let null = response
        .rows
        .iter()
        .filter(|r| r.retention.is_none())
        .count();
    let zero_base = response.rows.iter().filter(|r| r.base_dps == 0.0).count();
    assert_eq!(null, zero_base, "a non-zero row failed a pair");

    let page = dps::rank(&ctx, RankRequest::default()).await.unwrap();
    assert_eq!(page.retention_status, RetentionStatus::Ready);
    assert!(page.rows.iter().any(|r| r.retention.is_some()));
    db.close().await;
}
