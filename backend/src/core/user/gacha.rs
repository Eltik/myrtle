use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::core::authentication::constants::FetchError;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GachaType {
    Limited,
    Regular,
    Special,
}

impl GachaType {
    pub fn as_query_param(&self) -> &'static str {
        match self {
            GachaType::Limited => "Limited Headhunting",
            GachaType::Regular => "Regular Headhunting",
            GachaType::Special => "Special Headhunting",
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            GachaType::Limited => "limited",
            GachaType::Regular => "regular",
            GachaType::Special => "special",
        }
    }

    pub fn all() -> &'static [GachaType] {
        &[GachaType::Limited, GachaType::Regular, GachaType::Special]
    }

    /// Derive gacha type from pool_id prefix.
    /// Returns None if the prefix doesn't match any known type.
    pub fn from_pool_id(pool_id: &str) -> Option<Self> {
        if pool_id.starts_with("LIMITED_") {
            Some(GachaType::Limited)
        } else if pool_id.starts_with("NORM_")
            || pool_id.starts_with("CLASSIC_")
            || pool_id.starts_with("BOOT_")
        {
            Some(GachaType::Regular)
        } else if pool_id.starts_with("SINGLE_") || pool_id.starts_with("LINKAGE_") {
            Some(GachaType::Special)
        } else {
            None
        }
    }

    /// Get the pool_id prefixes that match this gacha type.
    pub fn pool_prefixes(&self) -> &'static [&'static str] {
        match self {
            GachaType::Limited => &["LIMITED_"],
            GachaType::Regular => &["NORM_", "CLASSIC_", "BOOT_"],
            GachaType::Special => &["SINGLE_", "LINKAGE_"],
        }
    }
}

impl FromStr for GachaType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "limited" => Ok(GachaType::Limited),
            "regular" => Ok(GachaType::Regular),
            "special" => Ok(GachaType::Special),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GachaItem {
    pub char_id: String,
    pub char_name: String,
    pub star: String,
    pub color: String,
    pub pool_id: String,
    pub pool_name: String,
    pub type_name: String,
    pub at: i64,
    pub at_str: String,
}

#[derive(Debug, Deserialize)]
struct GachaApiResponse {
    data: GachaRow,
}

#[derive(Debug, Deserialize)]
struct GachaRow {
    rows: Vec<GachaItem>,
    #[allow(dead_code)]
    count: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct GachaTypeRecords {
    pub gacha_type: GachaType,
    pub records: Vec<GachaItem>,
    pub total: i64,
}

#[derive(Debug, Serialize)]
pub struct GachaRecords {
    pub limited: GachaTypeRecords,
    pub regular: GachaTypeRecords,
    pub special: GachaTypeRecords,
}

const PAGE_SIZE: i32 = 50;

/// Fetch all gacha records from the API (the API ignores type filter)
async fn fetch_all_records(
    client: &Client,
    yostar_ssid: &str,
    yostar_sig: &str,
) -> Result<Vec<GachaItem>, FetchError> {
    let mut index = 1;
    let mut all: Vec<GachaItem> = Vec::new();
    let cookie = format!("YSSID={}; YSSID.sig={}", yostar_ssid, yostar_sig);

    loop {
        let url = format!(
            "https://account.yo-star.com/api/game/gachas?key=ark&index={}&size={}",
            index, PAGE_SIZE,
        );

        let response = client
            .get(&url)
            .header("Cookie", &cookie)
            .send()
            .await
            .map_err(FetchError::RequestFailed)?;

        let data: GachaApiResponse = response.json().await.map_err(FetchError::RequestFailed)?;

        if data.data.rows.is_empty() {
            break;
        }

        all.extend(data.data.rows);
        index += 1;
    }

    Ok(all)
}

/// Filter records by gacha type based on poolId prefix
fn filter_by_type(records: &[GachaItem], gacha_type: GachaType) -> Vec<GachaItem> {
    let prefixes = gacha_type.pool_prefixes();
    records
        .iter()
        .filter(|r| prefixes.iter().any(|p| r.pool_id.starts_with(p)))
        .cloned()
        .collect()
}

/// Get records that don't match any known gacha type prefix.
/// Useful for debugging and identifying new banner types.
#[allow(dead_code)]
pub fn get_uncategorized_records(records: &[GachaItem]) -> Vec<GachaItem> {
    records
        .iter()
        .filter(|r| GachaType::from_pool_id(&r.pool_id).is_none())
        .cloned()
        .collect()
}

/// Fetch all gacha records and categorize by type
pub async fn get_gacha(
    client: &Client,
    yostar_ssid: &str,
    yostar_sig: &str,
) -> Result<GachaRecords, FetchError> {
    // Fetch all records once (API ignores type parameter)
    let all_records = fetch_all_records(client, yostar_ssid, yostar_sig).await?;

    // Filter records by poolId prefix
    let limited_records = filter_by_type(&all_records, GachaType::Limited);
    let regular_records = filter_by_type(&all_records, GachaType::Regular);
    let special_records = filter_by_type(&all_records, GachaType::Special);

    Ok(GachaRecords {
        limited: GachaTypeRecords {
            gacha_type: GachaType::Limited,
            total: limited_records.len() as i64,
            records: limited_records,
        },
        regular: GachaTypeRecords {
            gacha_type: GachaType::Regular,
            total: regular_records.len() as i64,
            records: regular_records,
        },
        special: GachaTypeRecords {
            gacha_type: GachaType::Special,
            total: special_records.len() as i64,
            records: special_records,
        },
    })
}

/// Fetch gacha records for a specific type only
pub async fn get_gacha_by_type(
    client: &Client,
    yostar_ssid: &str,
    yostar_sig: &str,
    gacha_type: GachaType,
) -> Result<GachaTypeRecords, FetchError> {
    let all_records = fetch_all_records(client, yostar_ssid, yostar_sig).await?;
    let filtered = filter_by_type(&all_records, gacha_type);

    Ok(GachaTypeRecords {
        gacha_type,
        total: filtered.len() as i64,
        records: filtered,
    })
}
