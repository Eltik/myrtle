use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::core::gamedata::types::skin::Skin;

const STORE: i32 = 18;
const EARLY_STORE: i32 = 15;
const DYNAMIC_ART: i32 = 3;
const OWN_VOICE: i32 = 3;
const SPECIAL_DYNAMIC_ART: i32 = 3;

/// The 15 tier: the store outfits sold at 15 instead of 18. No field in
/// `skin_table`, `shop_client_table` or `item_table` separates them (the
/// eighteenth pass in `docs/release/CN_EN_RELEASE_MAPPING.md`: brand, rarity,
/// voice, chibi and prefab all mix, and "released before 2023-09" is wrong 119
/// times against 58), and the tier stopped after 2023-08, so this is a CLOSED
/// list, not a derivation: the 58 outfits the Terra wiki's brand pages list as
/// "Outfit Store (15)", read 2026-09-19, matched by operator and outfit name
/// (Provence's "HD008" is `HD08`, Blacknight's "Summer Flower" is "Summer
/// Flowers"). All 58 are plain store outfits with `GetTime` from 2020-02-05 to
/// 2023-08-22. A new outfit never joins this list.
const EARLY: &[&str] = &[
    "char_101_sora@summer#1",
    "char_1027_greyy2@snow#5",
    "char_102_texas@epoque#7",
    "char_102_texas@winter#1",
    "char_103_angel@wild#1",
    "char_107_liskam@nian#2",
    "char_108_silent@sweep#1",
    "char_115_headbr@it#1",
    "char_118_yuki@boc#2",
    "char_130_doberm@epoque#7",
    "char_134_ifrit@summer#1",
    "char_137_brownb@kitchen#1",
    "char_143_ghost@winter#1",
    "char_145_prove@summer#3",
    "char_149_scave@striker#2",
    "char_150_snakek@wild#1",
    "char_158_milu@snow#2",
    "char_173_slchan@wild#1",
    "char_174_slbell@snow#1",
    "char_181_flower@epoque#9",
    "char_185_frncat@wild#7",
    "char_187_ccheal@epoque#2",
    "char_196_sunbr@summer#1",
    "char_198_blackd@as#1",
    "char_198_blackd@winter#1",
    "char_201_moeshd@summer#4",
    "char_204_platnm@summer#3",
    "char_214_kafka@snow#3",
    "char_218_cuttle@epoque#12",
    "char_226_hmau@nian#4",
    "char_235_jesica@sweep#1",
    "char_235_jesica@wild#2",
    "char_236_rope@summer#2",
    "char_237_gravel@winter#2",
    "char_241_panda@marthe#1",
    "char_241_panda@nian#7",
    "char_261_sddrag@ambienceSynesthesia#2",
    "char_272_strong@summer#6",
    "char_274_astesi@epoque#5",
    "char_274_astesi@shining#1",
    "char_283_midn@boc#1",
    "char_290_vigna@as#1",
    "char_294_ayer@boc#3",
    "char_326_glacus@ghost#1",
    "char_333_sidero@summer#6",
    "char_345_folnic@wild#4",
    "char_365_aprl@wild#3",
    "char_367_swllow@boc#1",
    "char_373_lionhd@snow#3",
    "char_383_snsant@witch#2",
    "char_4016_kazema@witch#3",
    "char_402_tuye@epoque#14",
    "char_415_flint@boc#3",
    "char_421_crow@summer#9",
    "char_422_aurora@boc#4",
    "char_459_tachak@rainbow6#1",
    "char_476_blkngt@summer#8",
    "char_478_kirara@game#2",
];

const STORE_APPROACHES: &[&str] = &["采购中心", "Store"];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS, utoipa::ToSchema)]
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

fn base_price(skin: &Skin) -> i32 {
    if EARLY.contains(&skin.skin_id.as_str()) {
        EARLY_STORE
    } else {
        STORE
    }
}

fn feature_price(skin: &Skin) -> i32 {
    i32::from(skin.dyn_illust_id.is_some()) * DYNAMIC_ART
        + i32::from(own_voice(skin)) * OWN_VOICE
        + i32::from(skin.sp_dyn_illust_id.is_some()) * SPECIAL_DYNAMIC_ART
}

pub fn store_price(skin: &Skin) -> i32 {
    if !sold_in_store(skin) {
        return 0;
    }
    base_price(skin) + feature_price(skin)
}

/// A store outfit at its base tier: no dynamic art, own voice or special
/// variant. The Fashion Review stocks these and only these.
pub fn plain_store(skin: &Skin) -> bool {
    sold_in_store(skin) && feature_price(skin) == 0
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
        let early = Skin {
            skin_id: "char_103_angel@wild#1".into(),
            ..skin("Store", false, None, false)
        };
        assert_eq!(store_price(&early), 15, "the 15 tier is a closed list");
        assert!(plain_store(&early));
        assert!(!plain_store(&skin("采购中心", true, None, false)));
        assert!(!plain_store(&skin("活动获得", false, None, false)));
        let event = skin_price(&skin("活动获得", true, None, false), "Event Reward".into());
        assert_eq!(
            (event.price, event.store, event.obtain.as_str()),
            (0, false, "Event Reward")
        );
    }

    #[test]
    fn the_early_list_is_58_distinct_ids() {
        let mut ids: Vec<&str> = EARLY.to_vec();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), 58);
        assert_eq!(EARLY.len(), 58);
    }
}
