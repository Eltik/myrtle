use myrtle_engine_release::SkinGroupArt;
use myrtle_engine_release::translate::{AutoName, AutoNameSource};
use myrtle_gamedata::assets::{AssetIndex, AssetKind};

use super::Planner;

const EN: &str = "en";

impl Planner {
    /// A route path for the first asset found: EN under its EN id, then CN, then EN under the CN id
    fn image(
        &self,
        route: &str,
        en_id: Option<&str>,
        cn_id: &str,
        has: impl Fn(&AssetIndex, &str) -> bool,
    ) -> Option<String> {
        let (cn, en) = (&self.cn().assets, &self.en().assets);
        if let Some(id) = en_id
            && has(en, id)
        {
            return Some(format!("/{EN}/{route}/{id}"));
        }
        if has(cn, cn_id) {
            return Some(format!("/cn/{route}/{cn_id}"));
        }
        if has(en, cn_id) {
            return Some(format!("/{EN}/{route}/{cn_id}"));
        }
        None
    }

    /// Both asset trees as `(server, index)`, the one the item ships on first
    fn servers(&self, on_en: bool) -> [(&str, &AssetIndex); 2] {
        let (cn, en) = (("cn", &self.cn().assets), (EN, &self.en().assets));
        if on_en { [en, cn] } else { [cn, en] }
    }

    pub(super) fn event_image(&self, act_id: &str) -> Option<String> {
        let has = |idx: &AssetIndex, id: &str| idx.event_banner_path(id).is_some();
        self.image("event-image", Some(act_id), act_id, has)
            .or_else(|| {
                let side = format!("{}side", act_id.strip_suffix("sre")?);
                self.image("event-image", Some(&side), &side, has)
            })
    }

    pub(super) fn banner_image(&self, en_pool: Option<&str>, cn_pool: &str) -> Option<String> {
        self.image("banner-image", en_pool, cn_pool, |idx, id| {
            idx.gacha_banner_path(id).is_some()
        })
    }

    pub(super) fn item_icon(&self, icon_id: &str, on_en: bool) -> Option<String> {
        self.image("item-icon", on_en.then_some(icon_id), icon_id, |idx, id| {
            idx.path(AssetKind::ItemIcon, id).is_some()
        })
    }

    pub(super) fn avatar(&self, avatar_id: &str, on_en: bool) -> Option<String> {
        self.image(
            "avatar",
            on_en.then_some(avatar_id),
            avatar_id,
            |idx, id| idx.path(AssetKind::Avatar, id).is_some(),
        )
    }

    /// Served straight from the texture tree: there is no furniture route
    pub(super) fn furniture_icon(&self, icon_id: &str, on_en: bool) -> Option<String> {
        self.servers(on_en).iter().find_map(|(server, idx)| {
            idx.path(AssetKind::FurnitureIcon, icon_id)
                .map(|rel| format!("/{server}/assets{rel}"))
        })
    }

    pub(super) fn skin_portrait(
        &self,
        char_id: &str,
        portrait_id: &str,
        on_en: bool,
    ) -> Option<String> {
        self.image(
            "skin-portrait",
            on_en.then_some(portrait_id),
            portrait_id,
            |idx, id| idx.path(AssetKind::SkinPortrait, id).is_some(),
        )
        .or_else(|| self.skin_art(char_id, portrait_id, on_en))
    }

    fn skin_art(&self, char_id: &str, portrait_id: &str, on_en: bool) -> Option<String> {
        let thumb = format!("/textures/skinpack/{char_id}/{portrait_id}b.png");
        let full = format!("/textures/skinpack/{char_id}/{portrait_id}.png");
        let pick = |idx: &AssetIndex| {
            let files = idx.skinpack_paths(char_id)?;
            files
                .iter()
                .find(|f| **f == thumb)
                .or_else(|| files.iter().find(|f| **f == full))
                .cloned()
        };
        self.servers(on_en)
            .iter()
            .find_map(|(server, idx)| pick(idx).map(|rel| format!("/{server}/assets{rel}")))
    }

    pub(super) fn event_name_by_id(&self, cn_id: &str) -> Option<AutoName> {
        let (cn, en) = (self.cn(), self.en());
        let by_side = cn_id
            .strip_suffix("sre")
            .map(|stem| format!("{stem}side"))
            .and_then(|side| en.activities.get(&side))
            .filter(|a| !a.name.trim().is_empty())
            .map(|a| a.name.clone());
        let by_retro = || {
            cn.retro_acts
                .values()
                .find(|r| r.linked_act_id.iter().any(|l| l == cn_id))
                .and_then(|r| en.retro_acts.get(&r.retro_id))
                .filter(|r| !r.name.trim().is_empty())
                .map(|r| r.name.clone())
        };
        by_side.or_else(by_retro).map(AutoName::memory)
    }

    /// A mainline episode's shipped Latin title, off its first zone; side stories carry none
    pub(super) fn event_zone_name(&self, cn_id: &str) -> Option<AutoName> {
        let zone = self.cn().zones.get(&format!("{cn_id}_zone1"))?;
        let text = zone.zone_name_first.as_deref()?.trim();
        (!text.is_empty() && text.is_ascii()).then(|| AutoName {
            text: text.to_string(),
            source: AutoNameSource::Appellation,
        })
    }

    pub(super) fn group_art(&self, group_id: &str) -> Option<SkinGroupArt> {
        let brand = self
            .cn()
            .skins
            .brand_list
            .get(self.brand_by_group.get(group_id)?)?;
        let en_brand = self.en().skins.brand_list.get(&brand.brand_id);
        let kv = brand
            .kv_img_id_list
            .iter()
            .find(|k| k.linked_skin_group_id == group_id)
            .or_else(|| brand.kv_img_id_list.first())
            .map(|k| k.kv_img_id.as_str());
        Some(SkinGroupArt {
            skin_group_id: group_id.to_string(),
            brand_id: brand.brand_id.clone(),
            brand_name: en_brand.map_or_else(|| brand.brand_name.clone(), |b| b.brand_name.clone()),
            kv_path: kv.and_then(|k| {
                self.image("brand-kv", Some(k), k, |idx, id| {
                    idx.brand_kv_path(id).is_some()
                })
            }),
            logo_path: self.image(
                "brand-logo",
                Some(&brand.brand_id),
                &brand.brand_id,
                |idx, id| idx.brand_logo_path(id).is_some(),
            ),
        })
    }
}
