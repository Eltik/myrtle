//! Types for the `gacha/getPoolDetail` game-server response, and the on-disk
//! sidecar that caches it.
//!
//! `gacha_table.json` only carries a *subset* of each banner's featured
//! operators — the base64-BSON blobs in `LimitParam` / `DynMeta` (see
//! [`super::super::enrich::gacha`]). The client fills the rest in from
//! a second request, `POST {gs}/gacha/getPoolDetail` with `{"poolId": "..."}`.
//!
//! The two sources are **complementary, not redundant**, and which one carries
//! a banner's rate-ups depends on its rule type:
//!
//! | rule type        | `upCharInfo.perCharList` | authoritative source            |
//! |------------------|--------------------------|---------------------------------|
//! | `LIMITED`        | populated                | this API (+ `weightUpCharInfoList`) |
//! | `CLASSIC_DOUBLE` | populated                | this API                        |
//! | `FESCLASSIC`     | **empty**                | static `DynMeta.rarityPickCharDict` |
//! | `SPECIAL`        | **empty**                | static, + `availCharInfo` 6★ (pickup candidates) |
//! | `CLASSIC_ATTAIN` | **empty**                | static `DynMeta.attainRare6CharList` |
//!
//! The reason is visible in the payload: the `gachaObjList` entries that name a
//! character block (`FES_CLASSIC_UP_CHAR`, `ATTAIN_CHAR`,
//! `SPECIAL_PICKUP_SELECT_CHAR`) all carry `param: null`. `gachaObjList` is a
//! *layout instruction* naming which source fills which block; the client joins
//! the two itself, and so does [`enrich_banners`].
//!
//! [`enrich_banners`]: super::super::enrich::gacha::enrich_banners

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// `rarityRank` is 0-indexed in this API: 5 is a 6★, 4 is a 5★.
pub const RARITY_RANK_6: i32 = 5;
/// See [`RARITY_RANK_6`].
pub const RARITY_RANK_5: i32 = 4;

/// Bumped when the sidecar layout changes incompatibly; a mismatch makes the
/// loader discard the file rather than deserialize a stale shape.
pub const POOL_DETAIL_FILE_VERSION: u32 = 1;

/// Default sidecar location, relative to a server's assets dir.
///
/// Under `derived/` rather than in `gamedata/excel/`: that tree is owned by the
/// asset pipeline's unpacker, and anything left there is at the mercy of the
/// next extract.
pub const POOL_DETAIL_REL_PATH: &str = "derived/gacha_pool_details.json";

/// Resolve where a server's pool-detail sidecar lives.
///
/// Defaults to `{assets_dir}/derived/gacha_pool_details.json`. Set
/// `GACHA_DETAIL_DIR` to relocate it, which is required whenever the assets
/// tree is mounted read-only (the container deployment mounts it `:ro`), since
/// the refresh job has to write this file.
///
/// In the override case the file is named after the final component of
/// `assets_dir`, which is the server code by construction — assets dirs are
/// built as `{ASSETS_DIR}/{server}`. Loader and job both call this with the same
/// `assets_dir`, so they cannot disagree.
pub fn pool_detail_path(assets_dir: &Path) -> PathBuf {
    match std::env::var("GACHA_DETAIL_DIR") {
        Ok(dir) if !dir.trim().is_empty() => {
            let server_key = assets_dir
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("default");
            PathBuf::from(dir).join(format!("{server_key}.json"))
        }
        _ => assets_dir.join(POOL_DETAIL_REL_PATH),
    }
}

/// One `gacha/getPoolDetail` response body.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GachaPoolDetail {
    #[serde(default)]
    pub detail_info: DetailInfo,
    #[serde(default)]
    pub has_rate_up: bool,
    #[serde(default)]
    pub gacha_obj_group_type: i32,
}

impl GachaPoolDetail {
    /// True when the banner lets the player choose their own rate-up (the
    /// `SPECIAL` pickup pools). Detected from the layout discriminator rather
    /// than from `gacha_rule_type`, so a new rule type Yostar invents with the
    /// same UI still resolves correctly.
    pub fn is_pickup(&self) -> bool {
        self.detail_info.gacha_obj_list.as_ref().is_some_and(|l| {
            l.iter()
                .any(|o| o.gacha_object.starts_with("SPECIAL_PICKUP"))
        })
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DetailInfo {
    /// The rate-ups proper. Empty on FESCLASSIC / SPECIAL / `CLASSIC_ATTAIN`.
    #[serde(default)]
    pub up_char_info: Option<UpCharInfo>,
    /// Soft rate-boosted units ("5x rate up"). A *different* thing from a
    /// rate-up — these keep the base 6★ pool rate but take a larger share of it.
    #[serde(default)]
    pub weight_up_char_info_list: Option<Vec<WeightUpChar>>,
    #[serde(default)]
    pub limited_char: Option<Vec<String>>,
    #[serde(default)]
    pub avail_char_info: Option<AvailCharInfo>,
    #[serde(default)]
    pub gacha_obj_list: Option<Vec<GachaObj>>,
    #[serde(default)]
    pub gacha_obj_groups: Option<Vec<GachaObjGroup>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpCharInfo {
    #[serde(default)]
    pub per_char_list: Vec<PerCharEntry>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerCharEntry {
    #[serde(default)]
    pub rarity_rank: i32,
    /// Share of this rarity's total rate taken by `char_id_list`, e.g. `0.35`.
    #[serde(default)]
    pub percent: f64,
    #[serde(default)]
    pub count: i32,
    #[serde(default)]
    pub char_id_list: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WeightUpChar {
    #[serde(default)]
    pub char_id: String,
    #[serde(default)]
    pub rarity_rank: i32,
    #[serde(default)]
    pub weight: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AvailCharInfo {
    #[serde(default)]
    pub per_avail_list: Vec<PerAvailEntry>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerAvailEntry {
    #[serde(default)]
    pub rarity_rank: i32,
    /// Headline rate for the whole rarity band, e.g. `0.02` for 6★.
    #[serde(default)]
    pub total_percent: f64,
    #[serde(default)]
    pub char_id_list: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GachaObj {
    /// Layout discriminator: `UP_CHAR`, `UP_CHAR_WITH_LIMIT`, `AVAIL_CHAR`,
    /// `FES_CLASSIC_UP_CHAR`, `ATTAIN_CHAR`, `SPECIAL_PICKUP_SELECT_CHAR`,
    /// `RATE_UP_6`, `TEXT`, `IMAGE`, …
    #[serde(default)]
    pub gacha_object: String,
    #[serde(default)]
    pub image_type: i32,
    /// Free text for `TEXT` blocks; `null` for character blocks (the client
    /// sources those from `detail_info` or the static tables).
    #[serde(default)]
    pub param: Option<String>,
    #[serde(rename = "type", default)]
    pub obj_type: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GachaObjGroup {
    #[serde(default)]
    pub group_type: i32,
    #[serde(default)]
    pub start_index: i32,
    #[serde(default)]
    pub end_index: i32,
}

/// Per-rarity headline rate for a banner, flattened for the API.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RarityRate {
    pub rarity_rank: i32,
    pub total_percent: f64,
    /// How many operators sit in this rarity band on this banner.
    pub pool_size: usize,
}

/// The on-disk sidecar: every pool detail fetched so far, keyed by pool id.
///
/// Ended banners are immutable, so entries for them are cached indefinitely;
/// only pools whose `end_time` is still in the future get re-fetched.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PoolDetailFile {
    #[serde(default)]
    pub version: u32,
    /// Unix seconds of the last successful refresh.
    #[serde(default)]
    pub fetched_at: i64,
    #[serde(default)]
    pub server: String,
    #[serde(default)]
    pub pools: HashMap<String, GachaPoolDetail>,
}
