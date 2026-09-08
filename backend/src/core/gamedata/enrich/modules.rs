use std::collections::HashMap;

use crate::core::gamedata::{
    assets::AssetIndex,
    types::{
        material::Materials,
        module::{
            BattleEquip, Module, ModuleItemCost, Modules, RawModule, RawModuleItemCost, RawModules,
        },
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
    let mut out: Vec<OperatorModule> = modules
        .equip_dict
        .values()
        // For Amiya, every module's `char_id` is the base (`char_002_amiya`)
        // but branch modules have `tmpl_id` set to their form id. Branch
        // modules belong only to their tmpl; base modules (no tmpl_id) belong
        // only to the base form. For regular ops, tmpl_id is always None and
        // this reduces to a simple `char_id` match.
        .filter(|m| match &m.tmpl_id {
            Some(tmpl) => tmpl == char_id,
            None => m.char_id == char_id,
        })
        .map(|raw| {
            let data = battle_equip
                .get(&raw.uni_equip_id)
                .cloned()
                .unwrap_or_default();

            let module = enrich_module(raw, materials, assets);
            OperatorModule { module, data }
        })
        .collect();

    // `equip_dict` is a HashMap, so the iteration above is in a per-process
    // random order: without this sort the same operator's modules come back in
    // a different order after every restart, and anything downstream that reads
    // position - a default selection, a dropdown, a "Mod N" label - silently
    // means a different module each time.
    //
    // Sort by the uniequip number, NOT `char_equip_order`. The latter looks like
    // the display order and is not: on 17 operators it reads [0, 2, 1], putting
    // uniequip_003 ahead of _002, where the game lists them 001, 002, 003
    // (Skadi, Ash, Mostima, Irene, Archetto, Logos and eleven more). The
    // uniequip number reproduces the game's own `char_equip` array for every
    // operator that has modules, and it is also the order the DPS formulas index
    // by, so display and engine agree. `uni_equip_id` breaks any tie, making the
    // ordering total and reproducible.
    out.sort_by(|a, b| {
        uniequip_number(&a.module.uni_equip_id)
            .cmp(&uniequip_number(&b.module.uni_equip_id))
            .then_with(|| a.module.uni_equip_id.cmp(&b.module.uni_equip_id))
    });
    out
}

/// `uniequip_002_mgllan` -> 2. Unparseable ids sort last rather than colliding
/// at 0, so a malformed id cannot displace a real module.
fn uniequip_number(uni_equip_id: &str) -> i32 {
    uni_equip_id
        .split('_')
        .nth(1)
        .and_then(|n| n.parse::<i32>().ok())
        .unwrap_or(i32::MAX)
}

pub fn enrich_modules_global(
    raw: &RawModules,
    battle_equip: &BattleEquip,
    materials: &Materials,
    assets: &AssetIndex,
) -> Modules {
    let equip_dict = raw
        .equip_dict
        .iter()
        .map(|(k, v)| (k.clone(), enrich_module(v, materials, assets)))
        .collect();

    Modules {
        equip_dict,
        mission_list: raw.mission_list.clone(),
        sub_prof_dict: raw.sub_prof_dict.clone(),
        char_equip: raw.char_equip.clone(),
        equip_track_dict: HashMap::new(),
        battle_equip: battle_equip.clone(),
    }
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
    raw: &Option<HashMap<i32, Vec<RawModuleItemCost>>>,
    materials: &Materials,
    assets: &AssetIndex,
) -> Option<HashMap<String, Vec<ModuleItemCost>>> {
    raw.as_ref().map(|costs| {
        costs
            .iter()
            .map(|(stage, items)| {
                let mut converted: Vec<ModuleItemCost> = items
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

                converted.sort_by(|a, b| {
                    let a_prio = match a.id.as_str() {
                        "4001" => 0,
                        "5001" => 1,
                        _ => 2,
                    };
                    let b_prio = match b.id.as_str() {
                        "4001" => 0,
                        "5001" => 1,
                        _ => 2,
                    };
                    a_prio.cmp(&b_prio)
                });

                (stage.to_string(), converted)
            })
            .collect()
    })
}
