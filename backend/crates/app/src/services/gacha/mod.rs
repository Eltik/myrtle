pub mod pools;
pub mod rarity;
pub mod shops;
pub mod sidecar;
pub mod stats;

use std::collections::HashMap;

use myrtle_db::repos::gacha::{self, BatchOutcome, GachaRecordRow, HistoryFilter};
use myrtle_gamedata_raw::tables::operator::Operator;
use myrtle_hypergryph::login::yostar::AccountPortalSession;
use myrtle_types::UserId;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::policy::Subject;
use crate::privacy;
use crate::services::accounts::sessions::portal_session_key;
use crate::{AppContext, AppError};

pub use pools::refresh_pool_details;
pub use rarity::reconcile_rarities;
pub use shops::refresh_event_shops;
pub use stats::{enhanced_stats, flush_shared_stats, global_stats, per_banner_stats};

const PAGE_SIZE: usize = 50;
const MAX_PAGES: usize = 400;
const MAX_POOL_FILTER: usize = 64;
const MAX_POOL_ID_LEN: usize = 64;

pub const STORAGE_OFF_MESSAGE: &str = "Gacha history storage is turned off. Turn on \"Store gacha history\" in Settings > Privacy to sync your pulls.";

#[derive(Debug, Deserialize)]
pub struct GachaApiResponse {
    pub data: Option<GachaApiData>,
}

#[derive(Debug, Deserialize)]
pub struct GachaApiData {
    pub rows: Vec<GachaApiItem>,
    pub count: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GachaApiItem {
    pub char_id: String,
    pub char_name: Option<String>,
    pub star: String,
    pub pool_id: String,
    pub pool_name: Option<String>,
    pub type_name: Option<String>,
    pub at: i64,
}

impl GachaApiItem {
    #[must_use]
    pub fn gacha_type(&self) -> &'static str {
        let id = self.pool_id.as_str();
        [
            ("LIMITED_", "limited"),
            ("LINKAGE_", "linkage"),
            ("CLASSIC_", "classic"),
            ("SINGLE_", "single"),
            ("BOOT_", "boot"),
        ]
        .into_iter()
        .find_map(|(prefix, kind)| id.starts_with(prefix).then_some(kind))
        .unwrap_or("normal")
    }
}

#[derive(Debug, Serialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct FetchResult {
    pub total_fetched: usize,
    pub new_records: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct GachaStats {
    pub user_id: Uuid,
    #[ts(type = "number | null")]
    pub total_pulls: i64,
    #[ts(type = "number | null")]
    pub six_star_count: i64,
    #[ts(type = "number | null")]
    pub five_star_count: i64,
    #[ts(type = "number | null")]
    pub four_star_count: i64,
    #[ts(type = "number | null")]
    pub first_pull: i64,
    #[ts(type = "number | null")]
    pub last_pull: i64,
}

#[derive(Debug, Serialize, Clone, ts_rs::TS)]
#[ts(export)]
pub struct GachaRecordEntryDto {
    pub id: String,
    #[serde(rename = "charId")]
    pub char_id: String,
    #[serde(rename = "charName")]
    pub char_name: String,
    pub rarity: i16,
    #[serde(rename = "poolId")]
    pub pool_id: String,
    #[serde(rename = "poolName")]
    pub pool_name: String,
    #[serde(rename = "gachaType")]
    pub gacha_type: String,
    #[serde(rename = "pullTimestamp")]
    #[ts(type = "number")]
    pub pull_timestamp: i64,
    #[serde(rename = "pullTimestampStr")]
    pub pull_timestamp_str: Option<String>,
}

impl From<GachaRecordRow> for GachaRecordEntryDto {
    fn from(r: GachaRecordRow) -> Self {
        Self {
            id: r.id.to_string(),
            char_id: r.char_id,
            char_name: String::new(),
            rarity: r.rarity,
            pool_id: r.pool_id,
            pool_name: r.pool_name.unwrap_or_default(),
            gacha_type: r.gacha_type.unwrap_or_else(|| "normal".to_owned()),
            pull_timestamp: r.pull_timestamp,
            pull_timestamp_str: None,
        }
    }
}

#[derive(Debug, Serialize, Clone, ts_rs::TS)]
#[ts(export)]
pub struct GachaItemDto {
    #[serde(rename = "charId")]
    pub char_id: String,
    #[serde(rename = "charName")]
    pub char_name: String,
    pub star: String,
    pub color: String,
    #[serde(rename = "poolId")]
    pub pool_id: String,
    #[serde(rename = "poolName")]
    pub pool_name: String,
    #[serde(rename = "typeName")]
    pub type_name: String,
    #[ts(type = "number")]
    pub at: i64,
    #[serde(rename = "atStr")]
    pub at_str: String,
}

impl From<GachaRecordRow> for GachaItemDto {
    fn from(r: GachaRecordRow) -> Self {
        Self {
            char_id: r.char_id,
            char_name: String::new(),
            star: r.rarity.to_string(),
            color: String::new(),
            pool_id: r.pool_id,
            pool_name: r.pool_name.unwrap_or_default(),
            type_name: r.gacha_type.unwrap_or_else(|| "normal".to_owned()),
            at: r.pull_timestamp,
            at_str: String::new(),
        }
    }
}

#[derive(Debug, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct GachaTypeRecordsDto {
    pub gacha_type: &'static str,
    pub records: Vec<GachaItemDto>,
    pub total: usize,
}

#[derive(Debug, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct GachaRecordsDto {
    pub limited: GachaTypeRecordsDto,
    pub regular: GachaTypeRecordsDto,
    pub special: GachaTypeRecordsDto,
}

#[derive(Debug, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct GachaPaginationInfoDto {
    pub limit: u32,
    pub offset: u32,
    #[ts(type = "number")]
    pub total: i64,
    #[serde(rename = "hasMore")]
    pub has_more: bool,
}

#[derive(Debug, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct HistoryFiltersAppliedDto {
    pub rarity: Option<i16>,
    #[serde(rename = "gachaType")]
    pub gacha_type: Option<String>,
    #[serde(rename = "charId")]
    pub char_id: Option<String>,
    #[serde(rename = "dateRange")]
    pub date_range: Option<DateRangeDto>,
}

#[derive(Debug, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct DateRangeDto {
    #[ts(type = "number | null")]
    pub from: Option<i64>,
    #[ts(type = "number | null")]
    pub to: Option<i64>,
}

#[derive(Debug, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct GachaHistoryEnvelopeDto {
    pub records: Vec<GachaRecordEntryDto>,
    pub pagination: GachaPaginationInfoDto,
    #[serde(rename = "filtersApplied")]
    pub filters_applied: HistoryFiltersAppliedDto,
}

#[derive(Debug, Default)]
pub struct HistoryQuery {
    pub rarity: Option<i16>,
    pub gacha_type: Option<String>,
    pub char_id: Option<String>,
    pub from: Option<i64>,
    pub to: Option<i64>,
    pub newest_first: bool,
    pub limit: u32,
    pub offset: u32,
}

#[derive(Debug, Serialize, ts_rs::TS)]
#[ts(export)]
pub struct GachaSettingsDto {
    pub user_id: String,
    pub store_records: bool,
    pub share_anonymous_stats: bool,
    #[ts(type = "number")]
    pub total_pulls: i64,
    #[ts(type = "number")]
    pub six_star_count: i64,
    #[ts(type = "number")]
    pub five_star_count: i64,
    pub last_sync_at: Option<String>,
}

fn require_user(subject: &Subject) -> Result<UserId, AppError> {
    subject.user_id().ok_or(AppError::Unauthorized)
}

pub async fn fetch_and_store(ctx: &AppContext, subject: &Subject) -> Result<FetchResult, AppError> {
    let user = require_user(subject)?;
    let uid = subject.uid().ok_or(AppError::Unauthorized)?;
    if !gacha::store_gacha_enabled(&mut *ctx.db.conn().await?, user).await? {
        return Err(AppError::Conflict(STORAGE_OFF_MESSAGE.into()));
    }
    let portal: AccountPortalSession = ctx
        .cache
        .get(&portal_session_key(uid))
        .await
        .ok_or_else(|| AppError::bad_request("no portal session - login again"))
        .and_then(|json| {
            serde_json::from_str(&json).map_err(|_| AppError::bad_request("invalid portal session"))
        })?;

    let items = fetch_history(ctx, &portal).await?;
    if items.is_empty() {
        return Ok(FetchResult {
            total_fetched: 0,
            new_records: 0,
        });
    }
    let snapshot = ctx.snapshots.get(ctx.snapshots.default_server());
    let (records, missing) = pull_records(&items, snapshot.as_ref().map(|s| &s.model.operators));
    if !missing.is_empty() {
        tracing::warn!(
            count = missing.len(),
            char_ids = ?missing,
            "char_id(s) missing from game data; rarity falls back to the API star until the next reconcile",
        );
    }
    let mut conn = ctx.db.conn().await?;
    match gacha::insert_batch(&mut conn, user, &records).await? {
        BatchOutcome::Stored => Ok(FetchResult {
            total_fetched: items.len(),
            new_records: items.len(),
        }),
        BatchOutcome::StorageOff => Err(AppError::Conflict(STORAGE_OFF_MESSAGE.into())),
    }
}

async fn fetch_history(
    ctx: &AppContext,
    portal: &AccountPortalSession,
) -> Result<Vec<GachaApiItem>, AppError> {
    let base = ctx.upstream.config().hosts.account_portal.clone();
    let cookie = format!("YSSID={}; YSSID.sig={}", portal.yssid, portal.yssid_sig);
    let mut items = Vec::new();
    for index in 1..=MAX_PAGES {
        let page: GachaApiResponse = ctx
            .upstream
            .client()
            .get(format!(
                "{base}/api/game/gachas?key=ark&index={index}&size={PAGE_SIZE}"
            ))
            .header("Cookie", &cookie)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(AppError::internal)?
            .json()
            .await
            .map_err(AppError::internal)?;
        let Some(data) = page.data else { break };
        let full = data.rows.len() >= PAGE_SIZE;
        items.extend(data.rows);
        if !full {
            break;
        }
        if index == MAX_PAGES {
            tracing::warn!(pages = MAX_PAGES, "gacha history page cap reached");
        }
    }
    Ok(items)
}

#[must_use]
pub fn pull_records(
    items: &[GachaApiItem],
    operators: Option<&HashMap<String, Operator>>,
) -> (Value, Vec<String>) {
    let mut missing = Vec::new();
    let mut batch_counters: HashMap<(i64, &str), i16> = HashMap::new();
    let rows = items
        .iter()
        .map(|item| {
            let counter = batch_counters
                .entry((item.at, item.pool_id.as_str()))
                .or_insert(0);
            let batch_index = *counter;
            *counter += 1;
            let rarity = operators
                .and_then(|ops| ops.get(&item.char_id))
                .map_or_else(
                    || {
                        missing.push(item.char_id.clone());
                        item.star.parse().unwrap_or(3_i16)
                    },
                    |op| op.rarity.to_star_int(),
                );
            json!({
                "char_id": item.char_id,
                "pool_id": item.pool_id,
                "rarity": rarity,
                "pull_timestamp": item.at,
                "pool_name": item.pool_name,
                "gacha_type": item.gacha_type(),
                "batch_index": batch_index,
            })
        })
        .collect();
    missing.sort_unstable();
    missing.dedup();
    (Value::Array(rows), missing)
}

pub async fn history(
    ctx: &AppContext,
    subject: &Subject,
    query: HistoryQuery,
) -> Result<GachaHistoryEnvelopeDto, AppError> {
    let user = require_user(subject)?;
    let gacha_types = query.gacha_type.as_deref().map(gacha_types_of_group);
    let filter = HistoryFilter {
        rarity: query.rarity,
        gacha_types: gacha_types.as_deref(),
        char_id: query.char_id.as_deref(),
        from_ts: query.from,
        to_ts: query.to,
    };
    let mut conn = ctx.db.conn().await?;
    let (rows, total) = gacha::history_filtered(
        &mut conn,
        user,
        &filter,
        query.newest_first,
        i64::from(query.limit),
        i64::from(query.offset),
    )
    .await?;
    let records: Vec<GachaRecordEntryDto> = rows.into_iter().map(Into::into).collect();
    let has_more = i64::from(query.offset) + (records.len() as i64) < total;
    let date_range = (query.from.is_some() || query.to.is_some()).then_some(DateRangeDto {
        from: query.from,
        to: query.to,
    });
    Ok(GachaHistoryEnvelopeDto {
        records,
        pagination: GachaPaginationInfoDto {
            limit: query.limit,
            offset: query.offset,
            total,
            has_more,
        },
        filters_applied: HistoryFiltersAppliedDto {
            rarity: query.rarity,
            gacha_type: query.gacha_type,
            char_id: query.char_id,
            date_range,
        },
    })
}

#[must_use]
pub fn gacha_types_of_group(group: &str) -> Vec<String> {
    let types: &[&str] = match group {
        "limited" => &["limited", "linkage"],
        "regular" => &["normal", "classic"],
        "special" => &["single", "boot"],
        _ => &[],
    };
    types.iter().map(|&t| t.to_owned()).collect()
}

#[must_use]
pub fn classify_gacha_group(gacha_type: &str) -> &'static str {
    match gacha_type {
        "limited" | "linkage" => "limited",
        "single" | "boot" => "special",
        _ => "regular",
    }
}

pub async fn history_for_char(
    ctx: &AppContext,
    subject: &Subject,
    char_id: &str,
) -> Result<Vec<GachaRecordEntryDto>, AppError> {
    let user = require_user(subject)?;
    let rows = gacha::list_for_char(&mut *ctx.db.conn().await?, user, char_id).await?;
    Ok(rows.into_iter().map(Into::into).collect())
}

pub async fn stored_records(
    ctx: &AppContext,
    subject: &Subject,
) -> Result<GachaRecordsDto, AppError> {
    let user = require_user(subject)?;
    let rows = gacha::list_in_pools(&mut *ctx.db.conn().await?, user, None).await?;
    Ok(group_records(rows))
}

pub async fn public_records(
    ctx: &AppContext,
    subject: &Subject,
    uid: &str,
    pools: Option<&str>,
) -> Result<GachaRecordsDto, AppError> {
    let (profile, own) = privacy::resolve_public_profile(ctx, subject, uid).await?;
    let owner = UserId(profile.id);
    let mut conn = ctx.db.conn().await?;
    if !own && !gacha::public_history_enabled(&mut conn, owner).await? {
        return Err(AppError::Forbidden);
    }
    let pools = parse_pool_filter(pools)?;
    let rows = gacha::list_in_pools(&mut conn, owner, pools.as_deref()).await?;
    Ok(group_records(rows))
}

pub fn parse_pool_filter(raw: Option<&str>) -> Result<Option<Vec<String>>, AppError> {
    let mut pools: Vec<String> = Vec::new();
    for id in raw.unwrap_or("").split(',').map(str::trim) {
        if id.is_empty() {
            continue;
        }
        if id.len() > MAX_POOL_ID_LEN || !id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
        {
            return Err(AppError::BadRequest(format!(
                "`pools` holds an invalid banner id: {id:?}"
            )));
        }
        if !pools.iter().any(|p| p == id) {
            pools.push(id.to_owned());
        }
    }
    if pools.len() > MAX_POOL_FILTER {
        return Err(AppError::BadRequest(format!(
            "`pools` names {} banners; at most {MAX_POOL_FILTER} are allowed",
            pools.len()
        )));
    }
    Ok((!pools.is_empty()).then_some(pools))
}

#[must_use]
pub fn group_records(rows: Vec<GachaRecordRow>) -> GachaRecordsDto {
    let (mut limited, mut regular, mut special) = (Vec::new(), Vec::new(), Vec::new());
    for row in rows {
        let group = classify_gacha_group(row.gacha_type.as_deref().unwrap_or("normal"));
        let item = GachaItemDto::from(row);
        match group {
            "limited" => limited.push(item),
            "special" => special.push(item),
            _ => regular.push(item),
        }
    }
    let tab = |gacha_type, records: Vec<GachaItemDto>| GachaTypeRecordsDto {
        gacha_type,
        total: records.len(),
        records,
    };
    GachaRecordsDto {
        limited: tab("limited", limited),
        regular: tab("regular", regular),
        special: tab("special", special),
    }
}

pub async fn user_stats(ctx: &AppContext, subject: &Subject) -> Result<GachaStats, AppError> {
    let user = require_user(subject)?;
    let row = gacha::stats(&mut *ctx.db.conn().await?, user)
        .await?
        .ok_or(AppError::NotFound(None))?;
    Ok(GachaStats {
        user_id: row.user_id,
        total_pulls: row.total_pulls,
        six_star_count: row.six_star_count,
        five_star_count: row.five_star_count,
        four_star_count: row.four_star_count,
        first_pull: row.first_pull,
        last_pull: row.last_pull,
    })
}

pub async fn settings(ctx: &AppContext, subject: &Subject) -> Result<GachaSettingsDto, AppError> {
    let user = require_user(subject)?;
    let mut conn = ctx.db.conn().await?;
    let flags = gacha::flags(&mut conn, user).await?;
    settings_dto(&mut conn, user, flags.store_gacha, flags.share_stats).await
}

pub async fn update_settings(
    ctx: &AppContext,
    subject: &Subject,
    store_records: Option<bool>,
    share_anonymous_stats: Option<bool>,
) -> Result<GachaSettingsDto, AppError> {
    let user = require_user(subject)?;
    let mut conn = ctx.db.conn().await?;
    let before = gacha::flags(&mut conn, user).await?;
    let after = gacha::update_flags(&mut conn, user, store_records, share_anonymous_stats).await?;
    if before.share_stats != after.share_stats {
        flush_shared_stats(ctx).await;
    }
    settings_dto(&mut conn, user, after.store_gacha, after.share_stats).await
}

async fn settings_dto(
    conn: &mut myrtle_db::Conn,
    user: UserId,
    store_records: bool,
    share_anonymous_stats: bool,
) -> Result<GachaSettingsDto, AppError> {
    let stats = gacha::stats(conn, user).await?;
    Ok(GachaSettingsDto {
        user_id: user.0.to_string(),
        store_records,
        share_anonymous_stats,
        total_pulls: stats.as_ref().map_or(0, |s| s.total_pulls),
        six_star_count: stats.as_ref().map_or(0, |s| s.six_star_count),
        five_star_count: stats.as_ref().map_or(0, |s| s.five_star_count),
        last_sync_at: None,
    })
}
