//! Event token shops, as the game server serves them.
//!
//! The client has no shop table, only the shop's id
//! (`ActivityBasicInfo.template_shop_id`): it asks `templateShop/getGoodList`
//! `{shopId}` when the shop opens and the server answers with the goods, their
//! token prices and stock, and `allPriceDict`, whose `maxPrice` is the token
//! cost of clearing the whole shop. The server answers for closed shops too
//! (Ato, closed 2026-02, still returns 37 goods and `maxPrice` 8854 on EN)
//! and rejects an id it does not know (`invalid shop id`, 5681).
//! [`super::super::event_shop_job`] fetches each server's shops with its
//! service account and keeps them in a sidecar next to the extract, like the
//! gacha pool details.

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const EVENT_SHOP_FILE_VERSION: u32 = 2;
pub const EVENT_SHOP_REL_PATH: &str = "derived/event-shops.json";

pub fn event_shop_path(assets_dir: &Path) -> PathBuf {
    match std::env::var("EVENT_SHOP_DIR") {
        Ok(dir) if !dir.trim().is_empty() => {
            let server_key = assets_dir
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("default");
            PathBuf::from(dir).join(format!("{server_key}.json"))
        }
        _ => assets_dir.join(EVENT_SHOP_REL_PATH),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventShopGood {
    pub good_id: String,
    pub item_id: String,
    pub item_type: String,
    pub count: i32,
    pub price: i32,
    /// Purchase limit; `-1` is unlimited stock, which `max_price` leaves out.
    pub avail_count: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventShopData {
    pub shop_id: String,
    pub shop_name: String,
    pub token_id: String,
    pub start_time: i64,
    pub end_time: i64,
    /// Token cost of every limited good at full stock, the server's own figure.
    pub max_price: i32,
    pub goods: Vec<EventShopGood>,
}

impl EventShopData {
    /// Parse a `templateShop/getGoodList` response body.
    pub fn from_response(value: &Value) -> Option<Self> {
        let data = value.get("data")?;
        let shop_id = data.get("shopId")?.as_str()?.to_string();
        let mut goods: Vec<EventShopGood> = data
            .get("shopGroup")
            .and_then(Value::as_object)
            .into_iter()
            .flat_map(|groups| groups.values())
            .filter_map(|g| g.get("shopGood").and_then(Value::as_object))
            .flat_map(|goods| goods.values())
            // The first slot is an empty placeholder (`act54side_1`: no item,
            // price 0) on every shop seen.
            .filter(|g| g["item"]["id"].as_str().is_some_and(|id| !id.is_empty()))
            .map(|g| EventShopGood {
                good_id: g["goodId"].as_str().unwrap_or("").to_string(),
                item_id: g["item"]["id"].as_str().unwrap_or("").to_string(),
                item_type: g["item"]["type"].as_str().unwrap_or("").to_string(),
                count: g["item"]["count"].as_i64().unwrap_or(0) as i32,
                price: g["price"].as_i64().unwrap_or(0) as i32,
                avail_count: g["availCount"].as_i64().unwrap_or(0) as i32,
            })
            .collect();
        goods.sort_by(|a, b| a.good_id.cmp(&b.good_id));
        let max_price = data
            .get("allPriceDict")
            .and_then(Value::as_array)
            .and_then(|a| a.iter().filter_map(|p| p["maxPrice"].as_i64()).max())
            .unwrap_or_else(|| {
                goods
                    .iter()
                    .filter(|g| g.avail_count > 0)
                    .map(|g| i64::from(g.price) * i64::from(g.avail_count))
                    .sum()
            }) as i32;
        Some(Self {
            shop_id,
            shop_name: data["shopName"].as_str().unwrap_or("").to_string(),
            token_id: data["price"]["id"].as_str().unwrap_or("").to_string(),
            start_time: data["startTime"].as_i64().unwrap_or(0),
            end_time: data["endTime"].as_i64().unwrap_or(0),
            max_price,
            goods,
        })
    }
}

/// The sidecar, keyed by activity id: one entry per shop the server has
/// answered, plus the activities whose listed shop it refused and when, so a
/// refusal is not retried on every run.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventShopFile {
    #[serde(default)]
    pub version: u32,
    #[serde(default)]
    pub fetched_at: i64,
    #[serde(default)]
    pub server: String,
    #[serde(default)]
    pub shops: HashMap<String, EventShopData>,
    #[serde(default)]
    pub absent: HashMap<String, i64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn response_parses_goods_and_the_buyout_total() {
        let body = json!({"data": {
            "shopId": "shop_act44side", "shopName": "Vintage Bazaar",
            "price": {"id": "act44side_token_ticket", "count": 0, "type": "ACTIVITY_ITEM"},
            "startTime": 100, "endTime": 200,
            "allPriceDict": [{"startTime": 100, "maxPrice": 8854}],
            "shopGroup": {"g1": {"shopGood": {
                "act44side_2": {"goodId": "act44side_2", "goodType": "NORMAL", "item": {"id": "char_x@y#1", "count": 1, "type": "CHAR_SKIN"}, "price": 500, "availCount": 1},
                "act44side_35": {"goodId": "act44side_35", "goodType": "NORMAL", "item": {"id": "4001", "count": 100, "type": "GOLD"}, "price": 1, "availCount": -1},
                "act44side_1": {"goodId": "act44side_1", "goodType": "NORMAL", "item": {"id": "", "count": 0, "type": ""}, "price": 0, "availCount": 0}
            }}}
        }});
        let shop = EventShopData::from_response(&body).unwrap();
        assert_eq!(shop.max_price, 8854);
        assert_eq!(shop.goods.len(), 2);
        assert_eq!(shop.goods[0].item_id, "char_x@y#1");
        assert_eq!(shop.token_id, "act44side_token_ticket");
    }

    #[test]
    fn buyout_falls_back_to_limited_goods_when_the_dict_is_missing() {
        let body = json!({"data": {"shopId": "shop_a", "shopGroup": {"g": {"shopGood": {
            "a_1": {"goodId": "a_1", "item": {"id": "i", "count": 1, "type": "MATERIAL"}, "price": 10, "availCount": 3},
            "a_2": {"goodId": "a_2", "item": {"id": "j", "count": 1, "type": "MATERIAL"}, "price": 5, "availCount": -1}
        }}}}});
        assert_eq!(EventShopData::from_response(&body).unwrap().max_price, 30);
    }
}
