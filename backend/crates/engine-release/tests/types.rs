#![allow(clippy::unwrap_used, clippy::expect_used)]
use myrtle_engine_release::types::ShopGoodKind;

#[test]
fn shop_goods_are_grouped_by_item_type() {
    let cases = [
        ("CHAR_SKIN", ShopGoodKind::Outfit),
        ("FURN", ShopGoodKind::Furniture),
        ("HOME_THEME", ShopGoodKind::Furniture),
        ("MATERIAL", ShopGoodKind::Material),
        ("GOLD", ShopGoodKind::Currency),
        ("DIAMOND_SHD", ShopGoodKind::Currency),
        ("AP_SUPPLY", ShopGoodKind::Currency),
        ("CARD_EXP", ShopGoodKind::Exp),
        ("TKT_RECRUIT", ShopGoodKind::Ticket),
        ("TKT_GACHA_10", ShopGoodKind::Ticket),
        ("TKT", ShopGoodKind::Other),
        ("DIAMOND", ShopGoodKind::Other),
        ("char_skin", ShopGoodKind::Other),
        ("", ShopGoodKind::Other),
    ];
    for (item_type, kind) in cases {
        assert_eq!(ShopGoodKind::of(item_type), kind, "{item_type}");
    }
}
