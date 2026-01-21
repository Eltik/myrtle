use reqwest::Client;
use serde::{Deserialize, Serialize};

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
            GachaType::Limited => "Limited+Headhunting",
            GachaType::Regular => "Regular+Headhunting",
            GachaType::Special => "Special+Headhunting",
        }
    }

    pub fn all() -> &'static [GachaType] {
        &[GachaType::Limited, GachaType::Regular, GachaType::Special]
    }
}

#[derive(Debug, Deserialize, Serialize)]
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

async fn fetch_gacha_type(
    client: &Client,
    yostar_ssid: &str,
    yostar_sig: &str,
    gacha_type: GachaType,
) -> Result<GachaTypeRecords, FetchError> {
    let mut index = 1;
    let mut all: Vec<GachaItem> = Vec::new();
    let mut total: Option<i64> = None;
    let cookie = format!("YSSID={}; YSSID.sig={}", yostar_ssid, yostar_sig);

    loop {
        let url = format!(
            "https://account.yo-star.com/api/game/gachas?key=ark&index={}&size={}&type={}",
            index,
            PAGE_SIZE,
            gacha_type.as_query_param()
        );

        let response = client
            .get(&url)
            .header("Cookie", &cookie)
            .send()
            .await
            .map_err(FetchError::RequestFailed)?;

        let data: GachaApiResponse = response.json().await.map_err(FetchError::RequestFailed)?;

        if total.is_none() {
            total = data.data.count;
        }

        if data.data.rows.is_empty() {
            break;
        }

        all.extend(data.data.rows);
        index += 1;
    }

    Ok(GachaTypeRecords {
        gacha_type,
        records: all,
        total: total.unwrap_or(0),
    })
}

/// Fetch all gacha records (limited, regular, special) in parallel
pub async fn get_gacha(
    client: &Client,
    yostar_ssid: &str,
    yostar_sig: &str,
) -> Result<GachaRecords, FetchError> {
    // Fetch all three types in parallel
    let (limited, regular, special) = tokio::join!(
        fetch_gacha_type(client, yostar_ssid, yostar_sig, GachaType::Limited),
        fetch_gacha_type(client, yostar_ssid, yostar_sig, GachaType::Regular),
        fetch_gacha_type(client, yostar_ssid, yostar_sig, GachaType::Special),
    );

    Ok(GachaRecords {
        limited: limited?,
        regular: regular?,
        special: special?,
    })
}

/// Fetch gacha records for a specific type only
pub async fn get_gacha_by_type(
    client: &Client,
    yostar_ssid: &str,
    yostar_sig: &str,
    gacha_type: GachaType,
) -> Result<GachaTypeRecords, FetchError> {
    fetch_gacha_type(client, yostar_ssid, yostar_sig, gacha_type).await
}
