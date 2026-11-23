use myrtle_db::repos::release::{self, OverrideInput, PlanRow};
use myrtle_engine_release::kinds::{KIND_ACTIVITY, KIND_POOL, KIND_RETRO, KIND_REVIEW, KIND_SKIN};
use myrtle_engine_release::{
    OverrideRow, PutOverride, PutReleasePlan, ReleaseOverride, ReleasePlan,
};

use crate::policy::{Action, Resource, Subject};
use crate::{AppContext, AppError};

const KINDS: &[&str] = &[KIND_ACTIVITY, KIND_POOL, KIND_SKIN, KIND_RETRO, KIND_REVIEW];

#[must_use]
pub fn override_row(r: &release::OverrideRow) -> OverrideRow {
    OverrideRow {
        kind: r.kind.clone(),
        cn_id: r.cn_id.clone(),
        en_id: r.en_id.clone(),
        en_name: r.en_name.clone(),
        en_start: r.en_start,
        en_end: r.en_end,
        featured_chars: r.featured_chars.clone(),
        source: r.source.clone(),
        note: r.note.clone(),
    }
}

#[must_use]
pub fn wire_override(r: release::OverrideRow) -> ReleaseOverride {
    ReleaseOverride {
        kind: r.kind,
        cn_id: r.cn_id,
        en_id: r.en_id,
        en_name: r.en_name,
        en_start: r.en_start,
        en_end: r.en_end,
        featured_chars: r.featured_chars,
        source: r.source,
        note: r.note,
        updated_at: r.updated_at.timestamp(),
    }
}

fn require_editor(ctx: &AppContext, subject: &Subject) -> Result<(), AppError> {
    ctx.policy
        .require(subject, Action::EditReleaseData, &Resource::ReleaseData)
}

pub async fn list_overrides(
    ctx: &AppContext,
    subject: &Subject,
) -> Result<Vec<ReleaseOverride>, AppError> {
    require_editor(ctx, subject)?;
    let mut conn = ctx.db.conn().await?;
    Ok(release::list_overrides(&mut conn)
        .await?
        .into_iter()
        .map(wire_override)
        .collect())
}

pub async fn put_override(
    ctx: &AppContext,
    subject: &Subject,
    body: PutOverride,
) -> Result<ReleaseOverride, AppError> {
    require_editor(ctx, subject)?;
    if !KINDS.contains(&body.kind.as_str()) {
        return Err(AppError::bad_request(format!(
            "kind must be one of {}",
            KINDS.join(", ")
        )));
    }
    let cn_id = body.cn_id.trim();
    if cn_id.is_empty() {
        return Err(AppError::bad_request("cn_id is required"));
    }
    let mut conn = ctx.db.conn().await?;
    let row = release::put_override(
        &mut conn,
        &OverrideInput {
            kind: &body.kind,
            cn_id,
            en_id: body.en_id.as_deref(),
            en_name: body.en_name.as_deref().filter(|n| !n.trim().is_empty()),
            en_start: body.en_start,
            en_end: body.en_end,
            featured_chars: body.featured_chars.as_deref().filter(|f| !f.is_empty()),
            source: body.source.as_deref().unwrap_or("manual"),
            note: body.note.as_deref().unwrap_or_default(),
            updated_by: subject.user_id(),
        },
    )
    .await?;
    drop(conn);
    super::revise(ctx).await;
    Ok(wire_override(row))
}

pub async fn delete_override(
    ctx: &AppContext,
    subject: &Subject,
    kind: &str,
    cn_id: &str,
) -> Result<(), AppError> {
    require_editor(ctx, subject)?;
    let mut conn = ctx.db.conn().await?;
    if !release::delete_override(&mut conn, kind, cn_id).await? {
        return Err(AppError::NotFound(None));
    }
    drop(conn);
    super::revise(ctx).await;
    Ok(())
}

fn wire_plan(r: PlanRow) -> ReleasePlan {
    ReleasePlan {
        initial: r.initial,
        initial_manual: r.initial_manual,
        picks: serde_json::from_value(r.picks).unwrap_or_default(),
        stages: serde_json::from_value(r.stages).unwrap_or_default(),
        updated_at: r.updated_at.timestamp(),
    }
}

pub async fn get_plan(
    ctx: &AppContext,
    subject: &Subject,
) -> Result<Option<ReleasePlan>, AppError> {
    let user = subject.user_id().ok_or(AppError::Unauthorized)?;
    let mut conn = ctx.db.conn().await?;
    Ok(release::get_plan(&mut conn, user).await?.map(wire_plan))
}

pub async fn put_plan(
    ctx: &AppContext,
    subject: &Subject,
    body: PutReleasePlan,
) -> Result<ReleasePlan, AppError> {
    let user = subject.user_id().ok_or(AppError::Unauthorized)?;
    let picks = serde_json::to_value(&body.picks).unwrap_or_default();
    let stages = serde_json::to_value(&body.stages).unwrap_or_default();
    let mut conn = ctx.db.conn().await?;
    let row = release::put_plan(
        &mut conn,
        user,
        body.initial.max(0),
        body.initial_manual,
        &picks,
        &stages,
    )
    .await?;
    Ok(wire_plan(row))
}
