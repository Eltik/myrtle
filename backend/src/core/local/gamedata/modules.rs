use std::collections::HashMap;

use crate::core::local::{
    asset_mapping::AssetMappings,
    types::{
        material::Materials,
        module::{BattleEquip, Module, ModuleItemCost, RawModule, RawModules},
        operator::OperatorModule,
    },
};

/// Convert raw module item costs to processed module item costs
/// Raw keys are i32 (1, 2, 3), output keys are String ("1", "2", "3")
/// Looks up the material's iconId and uses asset mappings to find the correct image path
fn convert_item_costs(
    raw_item_cost: &Option<HashMap<i32, Vec<crate::core::local::types::module::RawModuleItemCost>>>,
    materials: &Materials,
    asset_mappings: &AssetMappings,
) -> Option<HashMap<String, Vec<ModuleItemCost>>> {
    raw_item_cost.as_ref().map(|costs| {
        costs
            .iter()
            .map(|(stage, items)| {
                let converted_items: Vec<ModuleItemCost> = items
                    .iter()
                    .map(|raw| {
                        // Look up the material to get its icon_id
                        let icon_id = materials
                            .items
                            .get(&raw.id)
                            .map(|m| m.icon_id.clone())
                            .unwrap_or_else(|| raw.id.clone());
                        // Use asset mappings to get correct path
                        let image_path = asset_mappings.get_item_icon_path(&icon_id);
                        ModuleItemCost {
                            id: raw.id.clone(),
                            count: raw.count,
                            item_type: raw.item_type.clone(),
                            icon_id: Some(icon_id),
                            image: Some(image_path),
                        }
                    })
                    .collect();
                (stage.to_string(), converted_items)
            })
            .collect()
    })
}

fn convert_raw_module_to_module(
    raw_module: &RawModule,
    materials: &Materials,
    asset_mappings: &AssetMappings,
) -> Module {
    Module {
        id: Some(raw_module.uni_equip_id.clone()),
        uni_equip_id: raw_module.uni_equip_id.clone(),
        uni_equip_name: raw_module.uni_equip_name.clone(),
        uni_equip_icon: raw_module.uni_equip_icon.clone(),
        image: asset_mappings.get_module_big_path(&raw_module.uni_equip_icon),
        uni_equip_desc: raw_module.uni_equip_desc.clone(),
        type_icon: raw_module.type_icon.clone(),
        type_name1: raw_module.type_name1.clone(),
        type_name2: raw_module.type_name2.clone(),
        equip_shining_color: raw_module.equip_shining_color.clone(),
        show_evolve_phase: raw_module.show_evolve_phase.clone(),
        unlock_evolve_phase: raw_module.unlock_evolve_phase.clone(),
        char_id: raw_module.char_id.clone(),
        tmpl_id: raw_module.tmpl_id.clone(),
        show_level: raw_module.show_level,
        unlock_level: raw_module.unlock_level,
        unlock_favor_point: raw_module.unlock_favor_point,
        mission_list: raw_module.mission_list.clone(),
        item_cost: convert_item_costs(&raw_module.item_cost, materials, asset_mappings),
        module_type: raw_module.module_type.clone(),
        uni_equip_get_time: raw_module.uni_equip_get_time,
        char_equip_order: raw_module.char_equip_order,
    }
}

pub fn get_operator_modules(
    char_id: &str,
    modules: &RawModules,
    battle_equip: &BattleEquip,
    materials: &Materials,
    asset_mappings: &AssetMappings,
) -> Vec<OperatorModule> {
    modules
        .equip_dict
        .values()
        .filter(|m| m.char_id == char_id)
        .map(|raw_module| {
            let module_data = battle_equip
                .get(&raw_module.uni_equip_id)
                .cloned()
                .unwrap_or_default();

            let module = convert_raw_module_to_module(raw_module, materials, asset_mappings);

            OperatorModule {
                module,
                data: module_data,
            }
        })
        .collect()
}
