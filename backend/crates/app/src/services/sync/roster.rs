use std::collections::HashMap;

use myrtle_db::repos::{items, progress, roster};
use myrtle_types::roster::{RosterEntry, SupportUnit};
use myrtle_types::stage_clear::StageClear;
use serde::Serialize;

use crate::privacy::layout::{ProfileHeaderField, ProfileTabId};
use crate::privacy::{resolve_user_id, resolve_user_id_as_viewer};
use crate::{AppContext, AppError, Subject};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct OwnedSkin {
    pub skin_id: String,
    pub obtained_at: Option<i64>,
}

#[derive(Debug, Clone, Copy, Serialize, ts_rs::TS)]
#[ts(export, rename = "StageClearDto")]
#[serde(rename_all = "camelCase")]
pub struct StageClearEntry {
    pub state: i16,
    pub state_max: i16,
    pub inferred: bool,
    pub complete_times: i32,
    pub practice_times: i32,
}

impl From<StageClear> for StageClearEntry {
    fn from(clear: StageClear) -> Self {
        Self {
            state: clear.state,
            state_max: clear.state_max,
            inferred: clear.inferred,
            complete_times: clear.complete_times,
            practice_times: clear.practice_times,
        }
    }
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct ItemEntry {
    pub item_id: String,
    pub quantity: i32,
}

pub async fn operators(
    ctx: &AppContext,
    subject: &Subject,
    uid: Option<&str>,
) -> Result<Vec<RosterEntry>, AppError> {
    let user = resolve_user_id(ctx, subject, uid, &[ProfileTabId::Roster]).await?;
    Ok(roster::roster(&mut *ctx.db.conn().await?, user).await?)
}

pub async fn operator(
    ctx: &AppContext,
    subject: &Subject,
    uid: Option<&str>,
    operator_id: &str,
) -> Result<RosterEntry, AppError> {
    let user = resolve_user_id(ctx, subject, uid, &[ProfileTabId::Roster]).await?;
    roster::operator(&mut *ctx.db.conn().await?, user, operator_id)
        .await?
        .ok_or(AppError::NotFound(None))
}

pub async fn supports(
    ctx: &AppContext,
    subject: &Subject,
    uid: Option<&str>,
) -> Result<Vec<SupportUnit>, AppError> {
    let user = resolve_user_id(ctx, subject, uid, &[]).await?;
    Ok(roster::supports(&mut *ctx.db.conn().await?, user).await?)
}

pub async fn owned_skins(
    ctx: &AppContext,
    subject: &Subject,
    uid: Option<&str>,
) -> Result<Vec<OwnedSkin>, AppError> {
    let tabs = [ProfileTabId::Stats, ProfileTabId::Roster];
    let user = resolve_user_id(ctx, subject, uid, &tabs).await?;
    let rows = roster::owned_skins(&mut *ctx.db.conn().await?, user).await?;
    Ok(rows
        .into_iter()
        .map(|r| OwnedSkin {
            skin_id: r.skin_id,
            obtained_at: r.obtained_at,
        })
        .collect())
}

pub async fn stage_clears(
    ctx: &AppContext,
    subject: &Subject,
    uid: Option<&str>,
) -> Result<HashMap<String, StageClearEntry>, AppError> {
    let user = resolve_user_id(ctx, subject, uid, &[]).await?;
    let clears = progress::stage_clears(&mut *ctx.db.conn().await?, user).await?;
    Ok(clears
        .clears
        .into_iter()
        .map(|(id, clear)| (id, clear.into()))
        .collect())
}

pub async fn inventory(
    ctx: &AppContext,
    subject: &Subject,
    uid: Option<&str>,
) -> Result<Vec<ItemEntry>, AppError> {
    let (user, visitor_layout) =
        resolve_user_id_as_viewer(ctx, subject, uid, ProfileTabId::Inventory).await?;
    let rows = items::inventory(&mut *ctx.db.conn().await?, user).await?;
    Ok(rows
        .into_iter()
        .filter(|r| {
            let field = currency_header_field(&r.item_id);
            visitor_layout
                .as_ref()
                .is_none_or(|layout| field.is_none_or(|f| layout.header_shows(f)))
        })
        .map(|r| ItemEntry {
            item_id: r.item_id,
            quantity: r.quantity,
        })
        .collect())
}

#[must_use]
pub fn currency_header_field(item_id: &str) -> Option<ProfileHeaderField> {
    match item_id {
        "4001" => Some(ProfileHeaderField::Lmd),
        "4002" => Some(ProfileHeaderField::Originite),
        "4003" => Some(ProfileHeaderField::Orundum),
        "7003" => Some(ProfileHeaderField::HeadhuntingPermits),
        "7004" => Some(ProfileHeaderField::TenPullPermits),
        _ => None,
    }
}
