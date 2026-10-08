import { GroupList } from "frontend";
import { useState } from "react";
import type { IOperatorListItem } from "../../src/types/operators";

// The planner's Groups tab: a card per group (pinned first, a dashed divider,
// then the rest). Clicking a card toggles it in the group filter; its header
// carries pin, collapse, rename and delete, and its plans are listed inside as
// nested entries. Rename and delete are the page's (native prompt/confirm),
// so they are inert here.
//
// Operators are `/api/operators/{id}` as the API layer camelizes it, trimmed.

const MLYNAR = {"id": "char_4064_mlynar", "name": "Młynar", "rarity": "TIER_6", "profession": "WARRIOR", "subProfessionId": "librator", "position": "MELEE", "server": "en", "portrait": "/portraits/char_4064_mlynar_2.png", "skin": null, "audio": [], "phases": [{"maxLevel": 50}, {"maxLevel": 80}, {"maxLevel": 90}], "allSkillLevelUp": [{"unlockCond": {"phase": "PHASE_0", "level": 1}}, {"unlockCond": {"phase": "PHASE_0", "level": 1}}, {"unlockCond": {"phase": "PHASE_0", "level": 1}}, {"unlockCond": {"phase": "PHASE_1", "level": 1}}, {"unlockCond": {"phase": "PHASE_1", "level": 1}}, {"unlockCond": {"phase": "PHASE_1", "level": 1}}], "skills": [{"skillId": "skchr_mlynar_1", "levelUpCostCond": [{"unlockCond": {"phase": "PHASE_2", "level": 1}}, {"unlockCond": {"phase": "PHASE_2", "level": 1}}, {"unlockCond": {"phase": "PHASE_2", "level": 1}}], "static": {"skillId": "skchr_mlynar_1", "iconId": null, "image": "/textures/spritepack/skill_icons_0/skill_icon_skchr_mlynar_1.png", "levels": [{"name": "Unvoiced Anger", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 10, "spCost": 40}}, {"name": "Unvoiced Anger", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 10, "spCost": 40}}, {"name": "Unvoiced Anger", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 10, "spCost": 40}}, {"name": "Unvoiced Anger", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 15, "spCost": 35}}, {"name": "Unvoiced Anger", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 15, "spCost": 35}}, {"name": "Unvoiced Anger", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 15, "spCost": 35}}, {"name": "Unvoiced Anger", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 15, "spCost": 30}}, {"name": "Unvoiced Anger", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 15, "spCost": 30}}, {"name": "Unvoiced Anger", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 15, "spCost": 30}}, {"name": "Unvoiced Anger", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 15, "spCost": 30}}]}}, {"skillId": "skchr_mlynar_2", "levelUpCostCond": [{"unlockCond": {"phase": "PHASE_2", "level": 1}}, {"unlockCond": {"phase": "PHASE_2", "level": 1}}, {"unlockCond": {"phase": "PHASE_2", "level": 1}}], "static": {"skillId": "skchr_mlynar_2", "iconId": null, "image": "/textures/spritepack/skill_icons_0/skill_icon_skchr_mlynar_2.png", "levels": [{"name": "Unresolved Sorrow", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 10, "spCost": 40}}, {"name": "Unresolved Sorrow", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 10, "spCost": 40}}, {"name": "Unresolved Sorrow", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 10, "spCost": 40}}, {"name": "Unresolved Sorrow", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 10, "spCost": 35}}, {"name": "Unresolved Sorrow", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 10, "spCost": 35}}, {"name": "Unresolved Sorrow", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 10, "spCost": 35}}, {"name": "Unresolved Sorrow", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 10, "spCost": 30}}, {"name": "Unresolved Sorrow", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 10, "spCost": 30}}, {"name": "Unresolved Sorrow", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 10, "spCost": 30}}, {"name": "Unresolved Sorrow", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 10, "spCost": 25}}]}}, {"skillId": "skchr_mlynar_3", "levelUpCostCond": [{"unlockCond": {"phase": "PHASE_2", "level": 1}}, {"unlockCond": {"phase": "PHASE_2", "level": 1}}, {"unlockCond": {"phase": "PHASE_2", "level": 1}}], "static": {"skillId": "skchr_mlynar_3", "iconId": null, "image": "/textures/spritepack/skill_icons_0/skill_icon_skchr_mlynar_3.png", "levels": [{"name": "Unbrilliant Glory", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 10, "spCost": 55}}, {"name": "Unbrilliant Glory", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 10, "spCost": 55}}, {"name": "Unbrilliant Glory", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 10, "spCost": 55}}, {"name": "Unbrilliant Glory", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 15, "spCost": 50}}, {"name": "Unbrilliant Glory", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 15, "spCost": 50}}, {"name": "Unbrilliant Glory", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 15, "spCost": 50}}, {"name": "Unbrilliant Glory", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 20, "spCost": 45}}, {"name": "Unbrilliant Glory", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 20, "spCost": 45}}, {"name": "Unbrilliant Glory", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 20, "spCost": 45}}, {"name": "Unbrilliant Glory", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 20, "spCost": 42}}]}}], "modules": [{"uniEquipId": "uniequip_001_mlynar", "uniEquipName": "Młynar's Badge", "uniEquipIcon": "uniequip_001_mlynar", "typeName1": "ORIGINAL", "type": "INITIAL", "unlockEvolvePhase": "PHASE_0", "unlockLevel": 0}, {"uniEquipId": "uniequip_002_mlynar", "uniEquipName": "'Man in Scabbard'", "uniEquipIcon": "uniequip_002_mlynar", "image": "/textures/spritepack/ui_equip_big_img_hub_19/uniequip_002_mlynar.png", "typeName1": "LIB", "typeName2": "X", "type": "ADVANCED", "unlockEvolvePhase": "PHASE_2", "unlockLevel": 60}]} as unknown as IOperatorListItem;

const EYJAFJALLA = {"id": "char_180_amgoat", "name": "Eyjafjalla", "rarity": "TIER_6", "profession": "CASTER", "subProfessionId": "corecaster", "position": "RANGED", "server": "en", "portrait": "/portraits/char_180_amgoat_2.png", "skin": null, "audio": [], "phases": [{"maxLevel": 50}, {"maxLevel": 80}, {"maxLevel": 90}], "allSkillLevelUp": [{"unlockCond": {"phase": "PHASE_0", "level": 1}}, {"unlockCond": {"phase": "PHASE_0", "level": 1}}, {"unlockCond": {"phase": "PHASE_0", "level": 1}}, {"unlockCond": {"phase": "PHASE_1", "level": 1}}, {"unlockCond": {"phase": "PHASE_1", "level": 1}}, {"unlockCond": {"phase": "PHASE_1", "level": 1}}], "skills": [{"skillId": "skchr_amgoat_1", "levelUpCostCond": [{"unlockCond": {"phase": "PHASE_2", "level": 1}}, {"unlockCond": {"phase": "PHASE_2", "level": 1}}, {"unlockCond": {"phase": "PHASE_2", "level": 1}}], "static": {"skillId": "skchr_amgoat_1", "iconId": null, "image": "/textures/spritepack/skill_icons_0/skill_icon_skchr_amgoat_1.png", "levels": [{"name": "Duetto", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 10, "spCost": 45}}, {"name": "Duetto", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 10, "spCost": 44}}, {"name": "Duetto", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 10, "spCost": 43}}, {"name": "Duetto", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 15, "spCost": 42}}, {"name": "Duetto", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 15, "spCost": 41}}, {"name": "Duetto", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 15, "spCost": 40}}, {"name": "Duetto", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 20, "spCost": 39}}, {"name": "Duetto", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 20, "spCost": 38}}, {"name": "Duetto", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 20, "spCost": 37}}, {"name": "Duetto", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 25, "spCost": 35}}]}}, {"skillId": "skchr_amgoat_2", "levelUpCostCond": [{"unlockCond": {"phase": "PHASE_2", "level": 1}}, {"unlockCond": {"phase": "PHASE_2", "level": 1}}, {"unlockCond": {"phase": "PHASE_2", "level": 1}}], "static": {"skillId": "skchr_amgoat_2", "iconId": null, "image": "/textures/spritepack/skill_icons_0/skill_icon_skchr_amgoat_2.png", "levels": [{"name": "Ignition", "skillType": "AUTO", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 0, "spCost": 7}}, {"name": "Ignition", "skillType": "AUTO", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 0, "spCost": 7}}, {"name": "Ignition", "skillType": "AUTO", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 0, "spCost": 7}}, {"name": "Ignition", "skillType": "AUTO", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 0, "spCost": 7}}, {"name": "Ignition", "skillType": "AUTO", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 0, "spCost": 7}}, {"name": "Ignition", "skillType": "AUTO", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 0, "spCost": 7}}, {"name": "Ignition", "skillType": "AUTO", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 0, "spCost": 6}}, {"name": "Ignition", "skillType": "AUTO", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 0, "spCost": 6}}, {"name": "Ignition", "skillType": "AUTO", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 0, "spCost": 6}}, {"name": "Ignition", "skillType": "AUTO", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 0, "spCost": 5}}]}}, {"skillId": "skchr_amgoat_3", "levelUpCostCond": [{"unlockCond": {"phase": "PHASE_2", "level": 1}}, {"unlockCond": {"phase": "PHASE_2", "level": 1}}, {"unlockCond": {"phase": "PHASE_2", "level": 1}}], "static": {"skillId": "skchr_amgoat_3", "iconId": null, "image": "/textures/spritepack/skill_icons_0/skill_icon_skchr_amgoat_3.png", "levels": [{"name": "Volcano", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 30, "spCost": 80}}, {"name": "Volcano", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 31, "spCost": 80}}, {"name": "Volcano", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 32, "spCost": 80}}, {"name": "Volcano", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 33, "spCost": 80}}, {"name": "Volcano", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 34, "spCost": 80}}, {"name": "Volcano", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 35, "spCost": 80}}, {"name": "Volcano", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 40, "spCost": 80}}, {"name": "Volcano", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 45, "spCost": 80}}, {"name": "Volcano", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 50, "spCost": 80}}, {"name": "Volcano", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 55, "spCost": 80}}]}}], "modules": [{"uniEquipId": "uniequip_001_amgoat", "uniEquipName": "Eyjafjalla's Badge", "uniEquipIcon": "uniequip_001_amgoat", "typeName1": "ORIGINAL", "type": "INITIAL", "unlockEvolvePhase": "PHASE_0", "unlockLevel": 0}, {"uniEquipId": "uniequip_002_amgoat", "uniEquipName": "'Missed Sounds'", "uniEquipIcon": "uniequip_002_amgoat", "image": "/textures/spritepack/ui_equip_big_img_hub_0/uniequip_002_amgoat.png", "typeName1": "CCR", "typeName2": "X", "type": "ADVANCED", "unlockEvolvePhase": "PHASE_2", "unlockLevel": 60}, {"uniEquipId": "uniequip_003_amgoat", "uniEquipName": "Pet Contest First Prize", "uniEquipIcon": "uniequip_003_amgoat", "image": "/textures/spritepack/ui_equip_big_img_hub_21/uniequip_003_amgoat.png", "typeName1": "CCR", "typeName2": "Y", "type": "ADVANCED", "unlockEvolvePhase": "PHASE_2", "unlockLevel": 60}]} as unknown as IOperatorListItem;

const TEXAS = {"id": "char_102_texas", "name": "Texas", "rarity": "TIER_5", "profession": "PIONEER", "subProfessionId": "pioneer", "position": "MELEE", "server": "en", "portrait": "/portraits/char_102_texas_2.png", "skin": null, "audio": [], "phases": [{"maxLevel": 50}, {"maxLevel": 70}, {"maxLevel": 80}], "allSkillLevelUp": [{"unlockCond": {"phase": "PHASE_0", "level": 1}}, {"unlockCond": {"phase": "PHASE_0", "level": 1}}, {"unlockCond": {"phase": "PHASE_0", "level": 1}}, {"unlockCond": {"phase": "PHASE_1", "level": 1}}, {"unlockCond": {"phase": "PHASE_1", "level": 1}}, {"unlockCond": {"phase": "PHASE_1", "level": 1}}], "skills": [{"skillId": "skcom_charge_cost[3]", "levelUpCostCond": [{"unlockCond": {"phase": "PHASE_2", "level": 1}}, {"unlockCond": {"phase": "PHASE_2", "level": 1}}, {"unlockCond": {"phase": "PHASE_2", "level": 1}}], "static": {"skillId": "skcom_charge_cost[3]", "iconId": null, "image": "/textures/spritepack/skill_icons_1/skill_icon_skcom_charge_cost[3].png", "levels": [{"name": "Charge γ", "skillType": "AUTO", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 10, "spCost": 44}}, {"name": "Charge γ", "skillType": "AUTO", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 10, "spCost": 43}}, {"name": "Charge γ", "skillType": "AUTO", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 10, "spCost": 42}}, {"name": "Charge γ", "skillType": "AUTO", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 12, "spCost": 41}}, {"name": "Charge γ", "skillType": "AUTO", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 12, "spCost": 40}}, {"name": "Charge γ", "skillType": "AUTO", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 12, "spCost": 39}}, {"name": "Charge γ", "skillType": "AUTO", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 14, "spCost": 38}}, {"name": "Charge γ", "skillType": "AUTO", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 16, "spCost": 37}}, {"name": "Charge γ", "skillType": "AUTO", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 18, "spCost": 36}}, {"name": "Charge γ", "skillType": "AUTO", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 20, "spCost": 35}}]}}, {"skillId": "skchr_texas_2", "levelUpCostCond": [{"unlockCond": {"phase": "PHASE_2", "level": 1}}, {"unlockCond": {"phase": "PHASE_2", "level": 1}}, {"unlockCond": {"phase": "PHASE_2", "level": 1}}], "static": {"skillId": "skchr_texas_2", "iconId": null, "image": "/textures/spritepack/skill_icons_1/skill_icon_skchr_texas_2.png", "levels": [{"name": "Sword Rain", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 20, "spCost": 40}}, {"name": "Sword Rain", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 21, "spCost": 40}}, {"name": "Sword Rain", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 22, "spCost": 40}}, {"name": "Sword Rain", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 23, "spCost": 40}}, {"name": "Sword Rain", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 24, "spCost": 40}}, {"name": "Sword Rain", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 25, "spCost": 40}}, {"name": "Sword Rain", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 26, "spCost": 40}}, {"name": "Sword Rain", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 27, "spCost": 40}}, {"name": "Sword Rain", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 28, "spCost": 40}}, {"name": "Sword Rain", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 30, "spCost": 40}}]}}], "modules": [{"uniEquipId": "uniequip_001_texas", "uniEquipName": "Texas's Badge", "uniEquipIcon": "uniequip_001_texas", "typeName1": "ORIGINAL", "type": "INITIAL", "unlockEvolvePhase": "PHASE_0", "unlockLevel": 0}, {"uniEquipId": "uniequip_002_texas", "uniEquipName": "Private Field Supply Package", "uniEquipIcon": "uniequip_002_texas", "image": "/textures/spritepack/ui_equip_big_img_hub_9/uniequip_002_texas.png", "typeName1": "SOL", "typeName2": "Y", "type": "ADVANCED", "unlockEvolvePhase": "PHASE_2", "unlockLevel": 50}]} as unknown as IOperatorListItem;


const plan = (id: string, operator: IOperatorListItem, target: { elite: number; level: number; skillLevel: number; skills: [number, number][]; modules: [string, number][] }, groups: string[], met = false) => ({
    id,
    user_id: "1000048871",
    operator_id: operator.id,
    operator,
    groups,
    met,
    target_elite: target.elite,
    target_level: target.level,
    target_skill_level: target.skillLevel,
    target_skills: target.skills.map(([skill_index, mastery_level]) => ({ skill_index, mastery_level })),
    target_modules: target.modules.map(([module_id, module_stage]) => ({ module_id, module_stage })),
    display_on_profile: true,
    created_at: "2024-04-02T10:12:00Z",
    updated_at: "2024-05-01T08:40:00Z",
});

/** Młynar to E2 Lv90, S3 M3 and S1 M1, his LIB-X module to stage 3. */
const MLYNAR_PLAN = plan("plan-mlynar", MLYNAR, { elite: 2, level: 90, skillLevel: 7, skills: [[0, 1], [2, 3]], modules: [["uniequip_002_mlynar", 3]] }, ["IS5 core", "Annihilation team"]);
/** Eyjafjalla to E2 Lv90, S2 M3, CCR-X to stage 3. */
const EYJA_PLAN = plan("plan-amgoat", EYJAFJALLA, { elite: 2, level: 90, skillLevel: 7, skills: [[1, 3]], modules: [["uniequip_002_amgoat", 3]] }, ["IS5 core"]);
/** Texas to E2 Lv70, S2 M2; not on the roster yet, so she reads as E0 Lv1. */
const TEXAS_PLAN = plan("plan-texas", TEXAS, { elite: 2, level: 70, skillLevel: 7, skills: [[1, 2]], modules: [] }, []);

const rosterEntry = (operator_id: string, elite: number, level: number, skill_level: number, masteries: { index: number; mastery: number }[], modules: { id: string; level: number; locked: boolean }[]) => ({
    user_id: "1000048871",
    operator_id,
    elite,
    level,
    exp: 0,
    potential: 0,
    skill_level,
    favor_point: 12000,
    skin_id: null,
    default_skill: 2,
    voice_lan: "JP",
    current_equip: null,
    current_tmpl: null,
    obtained_at: 1680000000,
    masteries,
    modules,
});

/** The synced roster: Młynar E2 Lv60 with S3 at M1 and LIB-X stage 1; Eyjafjalla freshly E2. */
const ROSTER = [rosterEntry("char_4064_mlynar", 2, 60, 7, [{ index: 2, mastery: 1 }], [{ id: "uniequip_002_mlynar", level: 1, locked: false }]), rosterEntry("char_180_amgoat", 2, 1, 7, [], [])];

const noop = () => undefined;

/** The page-held selection and expand state, kept local so checkboxes and chevrons work. */
function useEntryContext(initialActive: string[], initialExpanded: string[]) {
    const [active, setActive] = useState(() => new Set(initialActive));
    const [expanded, setExpanded] = useState(() => new Set(initialExpanded));
    const flip = (set: Set<string>, id: string) => {
        const next = new Set(set);
        if (next.has(id)) next.delete(id);
        else next.add(id);
        return next;
    };
    return {
        roster: ROSTER,
        isActive: (p: { id: string }) => active.has(p.id),
        isExpanded: (p: { id: string }) => expanded.has(p.id),
        onToggleActive: (p: { id: string }) => setActive((prev) => flip(prev, p.id)),
        onToggleExpanded: (p: { id: string }) => setExpanded((prev) => flip(prev, p.id)),
        onEdit: noop,
        onDelete: noop,
        active,
        setActive,
    };
}


const group = (name: string, pinned: boolean) => ({ id: `grp-${name}`, user_id: "1000048871", name, pinned, created_at: "2024-03-20T09:00:00Z", updated_at: "2024-03-20T09:00:00Z" });
const PLANS = [MLYNAR_PLAN, EYJA_PLAN, TEXAS_PLAN];

function Stage({ groups, pinDividerAt, filter = [], collapsed = [], isLoading = false }: { groups: ReturnType<typeof group>[]; pinDividerAt: number; filter?: string[]; collapsed?: string[]; isLoading?: boolean }) {
    const ctx = useEntryContext(["plan-mlynar", "plan-amgoat"], []);
    const [groupFilter, setGroupFilter] = useState(() => new Set(filter));
    const [expandedGroups, setExpandedGroups] = useState<Record<string, boolean>>(() => Object.fromEntries(collapsed.map((name) => [name, false])));
    const selection = {
        groupFilter,
        isActive: ctx.isActive,
        setAllActive: noop,
        applyGroupFilter: noop,
        toggleGroupFilter: (_plans: unknown, key: string) =>
            setGroupFilter((prev) => {
                const next = new Set(prev);
                if (next.has(key)) next.delete(key);
                else next.add(key);
                return next;
            }),
    };
    return (
        <div className="max-w-xl">
            <GroupList plans={PLANS} groups={groups} pinDividerAt={pinDividerAt} isLoading={isLoading} selection={selection} entryContext={ctx} expandedGroups={expandedGroups} onExpandedGroupsChange={setExpandedGroups} onTogglePin={noop} onRename={noop} onDelete={noop} />
        </div>
    );
}

/** A pinned group, the divider, an expanded group and an empty collapsed one. */
export const Default = () => <Stage groups={[group("IS5 core", true), group("Annihilation team", false), group("Contingency Contract", false)]} pinDividerAt={1} collapsed={["IS5 core"]} />;

/** "IS5 core" selected as the filter: ringed in red, the other cards dim. */
export const GroupSelected = () => <Stage groups={[group("IS5 core", true), group("Annihilation team", false)]} pinDividerAt={1} filter={["IS5 core"]} collapsed={["Annihilation team"]} />;

/** A group with no plans in it. */
export const EmptyGroup = () => <Stage groups={[group("Contingency Contract", false)]} pinDividerAt={-1} />;

/** No groups yet. */
export const NoGroups = () => <Stage groups={[]} pinDividerAt={-1} />;

/** Loading: three skeleton rows. */
export const Loading = () => <Stage groups={[]} pinDividerAt={-1} isLoading />;
