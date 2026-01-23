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
    let prefix = match gacha_type {
        GachaType::Limited => "LIMITED_",
        GachaType::Regular => "CLASSIC_", // Standard/Regular banner
        GachaType::Special => "SINGLE_",  // Single-target banners
    };
    records
        .iter()
        .filter(|r| r.pool_id.starts_with(prefix))
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
