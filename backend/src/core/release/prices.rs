use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::core::gamedata::types::skin::Skin;

pub const STORE: i32 = 18;
const DYNAMIC_ART: i32 = 3;
const OWN_VOICE: i32 = 3;
const SPECIAL_DYNAMIC_ART: i32 = 3;

const STORE_APPROACHES: &[&str] = &["采购中心", "Store"];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SkinPrice {
    pub price: i32,
    pub store: bool,
    pub obtain: String,
}

pub fn sold_in_store(skin: &Skin) -> bool {
    skin.display_skin
        .obtain_approach
        .as_deref()
        .is_some_and(|a| STORE_APPROACHES.iter().any(|s| a.starts_with(s)))
}

pub fn own_voice(skin: &Skin) -> bool {
    skin.voice_id
        .as_deref()
        .is_some_and(|v| v == skin.skin_id.replace('@', "_"))
}

pub fn store_price(skin: &Skin) -> i32 {
    if !sold_in_store(skin) {
        return 0;
    }
    STORE
        + i32::from(skin.dyn_illust_id.is_some()) * DYNAMIC_ART
        + i32::from(own_voice(skin)) * OWN_VOICE
        + i32::from(skin.sp_dyn_illust_id.is_some()) * SPECIAL_DYNAMIC_ART
}

pub fn skin_price(skin: &Skin, obtain: String) -> SkinPrice {
    SkinPrice {
        price: store_price(skin),
        store: sold_in_store(skin),
        obtain,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::gamedata::types::skin::DisplaySkin;

    fn skin(approach: &str, dynamic: bool, voice: Option<&str>, special: bool) -> Skin {
        Skin {
            skin_id: "char_1_test@brand#1".into(),
            dyn_illust_id: dynamic.then(|| "dyn".into()),
            voice_id: voice.map(str::to_string),
            sp_dyn_illust_id: special.then(|| "sp".into()),
            display_skin: DisplaySkin {
                obtain_approach: Some(approach.into()),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    fn price(approach: &str, dynamic: bool, voice: Option<&str>, special: bool) -> i32 {
        skin_price(&skin(approach, dynamic, voice, special), approach.into()).price
    }

    #[test]
    fn store_outfits_add_three_per_feature_and_the_rest_cost_nothing() {
        assert_eq!(price("采购中心", false, None, false), 18);
        assert_eq!(price("Store, Redeem Code", false, None, false), 18);
        assert_eq!(price("采购中心", true, None, false), 21);
        assert_eq!(
            price("采购中心", true, Some("char_1_test_brand#1"), false),
            24
        );
        assert_eq!(price("采购中心", true, None, true), 24);
        assert_eq!(
            price("采购中心", true, Some("char_1_test_brand#1"), true),
            27
        );
        assert_eq!(
            price("采购中心", false, Some("char_1_test#1"), false),
            18,
            "a borrowed voice set is not a feature"
        );
        let event = skin_price(&skin("活动获得", true, None, false), "Event Reward".into());
        assert_eq!(
            (event.price, event.store, event.obtain.as_str()),
            (0, false, "Event Reward")
        );
    }
}
