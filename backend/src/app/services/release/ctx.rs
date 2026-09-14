use std::{collections::HashMap, sync::Arc};

use crate::{
    app::{error::ApiError, state::AppState},
    core::{
        gamedata::{
            assets::{AssetIndex, AssetKind},
            types::GameData,
        },
        hypergryph::constants::Server,
        release::SkinGroupArt,
        translate::{AutoName, AutoNameSource},
    },
};

pub struct Ctx {
    pub cn: Arc<GameData>,
    pub en: Arc<GameData>,
    cn_assets: Arc<AssetIndex>,
    en_assets: Arc<AssetIndex>,
    en_server: &'static str,
    brand_by_group: HashMap<String, String>,
}

impl Ctx {
    pub fn load(state: &AppState) -> Result<Self, ApiError> {
        let cn_sd = state
            .try_server_data(Server::CN)
            .ok_or(ApiError::NotFound)?;
        let en_sd = state.server_data(state.default_server);
        let cn = cn_sd.game_data.load_full();
        let mut brand_by_group: HashMap<String, String> = HashMap::new();
        for b in cn.skins.brand_list.values() {
            for g in &b.group_list {
                brand_by_group.insert(g.skin_group_id.clone(), b.brand_id.clone());
            }
            for k in &b.kv_img_id_list {
                brand_by_group
                    .entry(k.linked_skin_group_id.clone())
                    .or_insert_with(|| b.brand_id.clone());
            }
        }
        Ok(Self {
            cn,
            en: en_sd.game_data.load_full(),
            cn_assets: cn_sd.asset_index.load_full(),
            en_assets: en_sd.asset_index.load_full(),
            en_server: state.default_server.as_str(),
            brand_by_group,
        })
    }

    fn image(
        &self,
        route: &str,
        en_id: Option<&str>,
        cn_id: &str,
        has: impl Fn(&AssetIndex, &str) -> bool,
    ) -> Option<String> {
        if let Some(id) = en_id
            && has(&self.en_assets, id)
        {
            return Some(format!("/{}/{route}/{id}", self.en_server));
        }
        if has(&self.cn_assets, cn_id) {
            return Some(format!("/cn/{route}/{cn_id}"));
        }
        if has(&self.en_assets, cn_id) {
            return Some(format!("/{}/{route}/{cn_id}", self.en_server));
        }
        None
    }

    pub fn event_image(&self, act_id: &str) -> Option<String> {
        let has = |idx: &AssetIndex, id: &str| idx.event_banner_path(id).is_some();
        self.image("event-image", Some(act_id), act_id, has)
            .or_else(|| {
                let side = format!("{}side", act_id.strip_suffix("sre")?);
                self.image("event-image", Some(&side), &side, has)
            })
    }

    pub fn banner_image(&self, en_pool: Option<&str>, cn_pool: &str) -> Option<String> {
        self.image("banner-image", en_pool, cn_pool, |idx, id| {
            idx.gacha_banner_path(id).is_some()
        })
    }

    pub fn skin_portrait(&self, portrait_id: &str, on_en: bool) -> Option<String> {
        self.image(
            "skin-portrait",
            on_en.then_some(portrait_id),
            portrait_id,
            |idx, id| idx.path(AssetKind::SkinPortrait, id).is_some(),
        )
    }

    pub fn event_name_by_id(&self, cn_id: &str) -> Option<AutoName> {
        let by_side = cn_id
            .strip_suffix("sre")
            .map(|stem| format!("{stem}side"))
            .and_then(|side| self.en.activities.get(&side))
            .filter(|a| !a.name.trim().is_empty())
            .map(|a| a.name.clone());
        let by_retro = || {
            self.cn
                .retro_acts
                .values()
                .find(|r| r.linked_act_id.iter().any(|l| l == cn_id))
                .and_then(|r| self.en.retro_acts.get(&r.retro_id))
                .filter(|r| !r.name.trim().is_empty())
                .map(|r| r.name.clone())
        };
        by_side.or_else(by_retro).map(|text| AutoName {
            text,
            source: AutoNameSource::Memory,
        })
    }

    pub fn group_art(&self, group_id: &str) -> Option<SkinGroupArt> {
        let brand = self
            .cn
            .skins
            .brand_list
            .get(self.brand_by_group.get(group_id)?)?;
        let en_brand = self.en.skins.brand_list.get(&brand.brand_id);
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
