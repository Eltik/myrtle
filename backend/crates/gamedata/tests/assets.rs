use myrtle_gamedata::assets::*;

// Ported from main: the index is now built from a path list, not a walk.
#[test]
fn item_icons_include_activity_common_assets_without_overriding_item_packs() {
    let paths: Vec<String> = [
        "textures/arts/ui_item_icons_0/MTL_SL_IRON3.png",
        "textures/arts/ui_item_icons_0/SHARED_ICON.png",
        "textures/activity/commonassets/act12side_token_key01.png",
        "textures/activity/commonassets/SHARED_ICON.png",
        "textures/ui/commonassets/not_an_item.png",
    ]
    .map(str::to_owned)
    .into();

    let idx = AssetIndex::from_paths(&paths);
    assert_eq!(
        idx.path(AssetKind::ItemIcon, "MTL_SL_IRON3"),
        Some("/textures/arts/ui_item_icons_0/MTL_SL_IRON3.png")
    );
    assert_eq!(
        idx.path(AssetKind::ItemIcon, "act12side_token_key01"),
        Some("/textures/activity/commonassets/act12side_token_key01.png")
    );
    assert_eq!(
        idx.path(AssetKind::ItemIcon, "SHARED_ICON"),
        Some("/textures/arts/ui_item_icons_0/SHARED_ICON.png")
    );
    assert_eq!(idx.path(AssetKind::ItemIcon, "not_an_item"), None);
}
