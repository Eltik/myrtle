import { CompletedPlansDialog } from "frontend";
import { type ReactNode, useEffect } from "react";
import type { IOperatorListItem } from "../../src/types/operators";

// "Completed plans": opened from the planner's toolbar once a sync shows plans
// at their targets. Every met plan is listed with a checkbox (all checked on
// open) and its target summary; the destructive action counts the checked
// ones. `plans` doubles as the open flag (null = closed); an empty list is the
// "nothing reached yet" state with a lone Close.
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

/**
 * The open popup focuses its first tabbable control once its transition
 * lands, and the brand-red ring reads as an error on a checkbox (or on the
 * whole popup while every control is disabled). Drop it on a short timer.
 */
function NoRing({ children }: { children: ReactNode }) {
    useEffect(() => {
        const timers = [60, 180, 400].map((ms) => window.setTimeout(() => (document.activeElement as HTMLElement | null)?.blur(), ms));
        return () => {
            for (const t of timers) window.clearTimeout(t);
        };
    }, []);
    return <>{children}</>;
}

const MET = [
    { ...MLYNAR_PLAN, met: true },
    { ...EYJA_PLAN, met: true },
    { ...TEXAS_PLAN, met: true },
];

/** Three plans met: all checked, "Delete 3 plans". */
export const ThreeCompleted = () => (
    <NoRing>
        <CompletedPlansDialog plans={MET} onOpenChange={noop} onDelete={noop} onKeep={noop} isSubmitting={false} errorMessage={null} />
    </NoRing>
);

/** One plan met: singular title and button. */
export const OneCompleted = () => (
    <NoRing>
        <CompletedPlansDialog plans={MET.slice(0, 1)} onOpenChange={noop} onDelete={noop} onKeep={noop} isSubmitting={false} errorMessage={null} />
    </NoRing>
);

/** Nothing has reached its target: the copy says so and only Close remains. */
export const NoneYet = () => (
    <NoRing>
        <CompletedPlansDialog plans={[]} onOpenChange={noop} onDelete={noop} onKeep={noop} isSubmitting={false} errorMessage={null} />
    </NoRing>
);

/** The delete call failed: the inline alert sits above the footer. */
export const DeleteFailed = () => (
    <NoRing>
        <CompletedPlansDialog plans={MET.slice(0, 2)} onOpenChange={noop} onDelete={noop} onKeep={noop} isSubmitting={false} errorMessage="Failed to delete. Please try again." />
    </NoRing>
);

/** Deleting: the checkboxes and Keep lock, the destructive button spins. */
export const Deleting = () => (
    <NoRing>
        <CompletedPlansDialog plans={MET.slice(0, 2)} onOpenChange={noop} onDelete={noop} onKeep={noop} isSubmitting errorMessage={null} />
    </NoRing>
);
