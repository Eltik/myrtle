use crate::core::local::{
    asset_mapping::AssetMappings,
    types::{
        module::{BattleEquip, Module, RawModules},
        operator::OperatorModule,
    },
};

pub fn get_operator_modules(
    char_id: &str,
    modules: &RawModules,
    battle_equip: &BattleEquip,
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

            // Get module image path using asset mappings
            // Note: uniequip_001_xxx modules return None as they are "original" placeholders
            let module = Module {
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
                item_cost: None, // TODO: Parse from raw_module.item_cost which is Vec<serde_json::Value>
                module_type: raw_module.module_type.clone(),
                uni_equip_get_time: raw_module.uni_equip_get_time,
                char_equip_order: raw_module.char_equip_order,
            };

            OperatorModule {
                module,
                data: module_data,
            }
        })
        .collect()
}
