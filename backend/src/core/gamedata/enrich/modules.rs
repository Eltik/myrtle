use std::collections::HashMap;

use crate::core::gamedata::{
    assets::AssetIndex,
    types::{
        material::Materials,
        module::{BattleEquip, Module, ModuleItemCost, RawModule, RawModules},
        operator::OperatorModule,
    },
};

use super::resolve_item_icon;

pub fn get_operator_modules(
    char_id: &str,
    modules: &RawModules,
    battle_equip: &BattleEquip,
    materials: &Materials,
    assets: &AssetIndex,
) -> Vec<OperatorModule> {
    modules
        .equip_dict
        .values()
        .filter(|m| m.char_id == char_id)
        .map(|raw| {
            let data = battle_equip
                .get(&raw.uni_equip_id)
                .cloned()
                .unwrap_or_default();

            let module = enrich_module(raw, materials, assets);
            OperatorModule { module, data }
        })
        .collect()
}

fn enrich_module(raw: &RawModule, materials: &Materials, assets: &AssetIndex) -> Module {
    Module {
        id: Some(raw.uni_equip_id.clone()),
        uni_equip_id: raw.uni_equip_id.clone(),
        uni_equip_name: raw.uni_equip_name.clone(),
        uni_equip_icon: raw.uni_equip_icon.clone(),
        image: assets
            .module_big_path(&raw.uni_equip_icon)
            .map(str::to_owned),
        uni_equip_desc: raw.uni_equip_desc.clone(),
        type_icon: raw.type_icon.clone(),
        type_name1: raw.type_name1.clone(),
        type_name2: raw.type_name2.clone(),
        equip_shining_color: raw.equip_shining_color.clone(),
        show_evolve_phase: raw.show_evolve_phase.clone(),
        unlock_evolve_phase: raw.unlock_evolve_phase.clone(),
        char_id: raw.char_id.clone(),
        tmpl_id: raw.tmpl_id.clone(),
        show_level: raw.show_level,
        unlock_level: raw.unlock_level,
        unlock_favor_point: raw.unlock_favor_point,
        mission_list: raw.mission_list.clone(),
        item_cost: convert_item_costs(&raw.item_cost, materials, assets),
        module_type: raw.module_type.clone(),
        uni_equip_get_time: raw.uni_equip_get_time,
        char_equip_order: raw.char_equip_order,
    }
}

fn convert_item_costs(
    raw: &Option<HashMap<i32, Vec<crate::core::gamedata::types::module::RawModuleItemCost>>>,
    materials: &Materials,
    assets: &AssetIndex,
) -> Option<HashMap<String, Vec<ModuleItemCost>>> {
    raw.as_ref().map(|costs| {
        costs
            .iter()
            .map(|(stage, items)| {
                let converted = items
                    .iter()
                    .map(|item| {
                        let (icon_id, image) = resolve_item_icon(&item.id, materials, assets);
                        ModuleItemCost {
                            id: item.id.clone(),
                            count: item.count,
                            item_type: item.item_type.clone(),
                            icon_id,
                            image,
                        }
                    })
                    .collect();
                (stage.to_string(), converted)
            })
            .collect()
    })
}
