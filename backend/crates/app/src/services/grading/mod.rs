mod pass;

use myrtle_db::repos::scores::ScoreWrite;
use myrtle_db::repos::{progress, roster, scores, users};
use myrtle_engine_base::score::grade_base;
use myrtle_engine_grade::operators::ScoreModel;
use myrtle_engine_grade::sandbox::grade_sandbox;
use myrtle_engine_grade::total::{
    SECTION_WEIGHT_BASE, SECTION_WEIGHT_MEDAL, SECTION_WEIGHT_OPERATOR, SECTION_WEIGHT_ROGUELIKE,
    SECTION_WEIGHT_SANDBOX, SECTION_WEIGHT_STAGE, SECTION_WEIGHT_TOTAL, score_to_grade,
};
use myrtle_engine_grade::{BaseScore, GradeInputs, StageClears, UserGrade, calculate_user_grade};
use myrtle_jobs::{JobKind, JobSpec};
use myrtle_types::UserId;
use serde_json::json;
use std::sync::Arc;

use crate::snapshots::Snapshot;
use crate::{AppContext, AppError};
pub use pass::*;

fn snapshot_of(ctx: &AppContext) -> Result<Arc<Snapshot>, AppError> {
    ctx.snapshots.or_default(ctx.snapshots.default_server())
}

#[must_use]
pub fn score_model(ctx: &AppContext) -> ScoreModel {
    let switches = &ctx.config.switches;
    ScoreModel {
        invested_only: switches.grade_invested_only.enabled,
        legacy_rarity_weights: switches.grade_legacy_rarity_weights.enabled,
        module_per_slot: switches.grade_module_per_slot.enabled,
    }
}

pub async fn load_inputs(
    ctx: &AppContext,
    user: UserId,
    snapshot: &Arc<Snapshot>,
) -> Result<GradeInputs, AppError> {
    let mut conn = ctx.db.conn().await?;
    let conn = &mut *conn;
    let user_roster = roster::roster(conn, user).await?;
    let supports = roster::supports(conn, user).await?;
    let building = progress::building(conn, user).await?;
    let roguelike = progress::roguelike(conn, user).await?;
    let medals = progress::medals(conn, user).await?;
    let clears = progress::stage_clears(conn, user).await?;
    let known_stages = progress::known_stage_ids_for_server(conn, user).await?;
    let sandbox = progress::sandbox(conn, user).await?;

    let data = Arc::clone(snapshot);
    let (user_roster, base) = ctx
        .cpu
        .offload("grade_base", move || {
            let base = grade_base(&user_roster, building.as_ref(), &data.base);
            (user_roster, base)
        })
        .await?;

    Ok(GradeInputs {
        roster: user_roster,
        supports: supports.into_iter().map(|s| s.operator_id).collect(),
        base: BaseScore {
            score: base.score,
            utilization: base.utilization,
            infrastructure: base.infrastructure,
        },
        roguelike: roguelike
            .into_iter()
            .map(|row| (row.theme_id, row.progress))
            .collect(),
        medals: medals
            .into_iter()
            .map(|row| (row.medal_id, row.val, row.first_ts, row.reach_ts))
            .collect(),
        stage_clears: StageClears {
            clears: clears.clears,
            last_synced_ts: clears.last_synced_ts,
        },
        known_stages,
        sandbox,
        now_unix: chrono::Utc::now().timestamp(),
        score_model: score_model(ctx),
    })
}

pub async fn compute_grade(ctx: &AppContext, user: UserId) -> Result<UserGrade, AppError> {
    let snapshot = snapshot_of(ctx)?;
    let inputs = load_inputs(ctx, user, &snapshot).await?;
    ctx.cpu
        .offload("user_grade", move || {
            calculate_user_grade(&snapshot.grade, &inputs)
        })
        .await
}

pub async fn regrade(ctx: &AppContext, user: UserId) -> Result<UserGrade, AppError> {
    let grade = compute_grade(ctx, user).await?;
    let write = ScoreWrite {
        total_score: grade.total_score,
        operator_score: grade.operator_grade,
        stage_score: grade.stage_grade,
        roguelike_score: grade.roguelike_grade,
        sandbox_score: grade.sandbox_grade,
        medal_score: grade.medal_grade,
        base_score: grade.base_grade,
        base_utilization: Some(grade.base_utilization),
        base_infrastructure: Some(grade.base_infrastructure),
        skin_score: 0.0,
        grade: Some(&grade.overall),
    };
    scores::upsert(&mut *ctx.db.conn().await?, user, &write).await?;
    Ok(grade)
}

#[derive(Debug, Clone, PartialEq)]
pub struct SandboxRescore {
    pub sandbox: f64,
    pub total: f64,
    pub grade: String,
}

pub async fn compute_sandbox(
    ctx: &AppContext,
    user: UserId,
) -> Result<Option<SandboxRescore>, AppError> {
    let snapshot = snapshot_of(ctx)?;
    let mut conn = ctx.db.conn().await?;
    let Some(stored) = scores::stored_sections(&mut conn, user).await? else {
        return Ok(None);
    };
    let progress = progress::sandbox(&mut conn, user).await?;
    let sandbox = grade_sandbox(&snapshot.grade, progress.as_ref());
    let total = SECTION_WEIGHT_SANDBOX.mul_add(
        sandbox,
        SECTION_WEIGHT_STAGE.mul_add(
            stored.stage_score,
            SECTION_WEIGHT_MEDAL.mul_add(
                stored.medal_score,
                SECTION_WEIGHT_ROGUELIKE.mul_add(
                    stored.roguelike_score,
                    SECTION_WEIGHT_BASE.mul_add(
                        stored.base_score,
                        SECTION_WEIGHT_OPERATOR * stored.operator_score,
                    ),
                ),
            ),
        ),
    ) / SECTION_WEIGHT_TOTAL;
    Ok(Some(SandboxRescore {
        sandbox,
        total,
        grade: score_to_grade(total),
    }))
}

pub async fn rescore_sandbox(ctx: &AppContext, user: UserId) -> Result<Option<String>, AppError> {
    let Some(r) = compute_sandbox(ctx, user).await? else {
        return Ok(None);
    };
    scores::write_sandbox(
        &mut *ctx.db.conn().await?,
        user,
        r.sandbox,
        r.total,
        &r.grade,
    )
    .await?;
    Ok(Some(r.grade))
}

pub async fn enqueue_regrade(ctx: &AppContext, user: UserId) -> Result<(), AppError> {
    ctx.queue
        .enqueue(JobSpec {
            payload: json!({ "user_id": user.0 }),
            dedupe_key: Some(user.0.to_string()),
            ..JobSpec::new(JobKind::RegradeUser)
        })
        .await?;
    Ok(())
}

pub async fn enqueue_regrade_all(ctx: &AppContext) -> Result<u64, AppError> {
    let page_size = ctx.config.jobs.regrade.page_size.max(1);
    let mut after = None;
    let mut queued = 0;
    loop {
        let page = users::keys_page(&mut *ctx.db.conn().await?, None, after, page_size).await?;
        let Some(last) = page.last() else {
            return Ok(queued);
        };
        after = Some(UserId(last.id));
        for key in &page {
            enqueue_regrade(ctx, UserId(key.id)).await?;
            queued += 1;
        }
    }
}
