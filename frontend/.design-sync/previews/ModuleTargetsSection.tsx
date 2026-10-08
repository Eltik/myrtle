import { ModuleTargetsSection } from "frontend";
import { useState } from "react";
import type { IOperatorListItem } from "../../src/types/operators";

// The plan dialog's collapsible "Modules" block: one row per plannable module
// (the ORIGINAL badge is skipped) with its icon, name and type tag, then the
// stage buttons "-" (not planned) and 1-3. Every stage shares the module's one
// unlock requirement, so below it all three fade out with an "unlocks at"
// tooltip; a stage below the roster reads "Already reached".
//
// Eyjafjalla and Texas as `/api/operators/{id}` serves them (camelized by the
// API layer), trimmed to the fields the section reads.

const EYJAFJALLA = {"id": "char_180_amgoat", "name": "Eyjafjalla", "rarity": "TIER_6", "profession": "CASTER", "subProfessionId": "corecaster", "position": "RANGED", "server": "en", "portrait": "/portraits/char_180_amgoat_2.png", "skin": null, "audio": [], "phases": [{"maxLevel": 50}, {"maxLevel": 80}, {"maxLevel": 90}], "allSkillLevelUp": [{"unlockCond": {"phase": "PHASE_0", "level": 1}}, {"unlockCond": {"phase": "PHASE_0", "level": 1}}, {"unlockCond": {"phase": "PHASE_0", "level": 1}}, {"unlockCond": {"phase": "PHASE_1", "level": 1}}, {"unlockCond": {"phase": "PHASE_1", "level": 1}}, {"unlockCond": {"phase": "PHASE_1", "level": 1}}], "skills": [{"skillId": "skchr_amgoat_1", "levelUpCostCond": [{"unlockCond": {"phase": "PHASE_2", "level": 1}}, {"unlockCond": {"phase": "PHASE_2", "level": 1}}, {"unlockCond": {"phase": "PHASE_2", "level": 1}}], "static": {"skillId": "skchr_amgoat_1", "iconId": null, "image": "/textures/spritepack/skill_icons_0/skill_icon_skchr_amgoat_1.png", "levels": [{"name": "Duetto", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 10, "spCost": 45}}, {"name": "Duetto", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 10, "spCost": 44}}, {"name": "Duetto", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 10, "spCost": 43}}, {"name": "Duetto", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 15, "spCost": 42}}, {"name": "Duetto", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 15, "spCost": 41}}, {"name": "Duetto", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 15, "spCost": 40}}, {"name": "Duetto", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 20, "spCost": 39}}, {"name": "Duetto", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 20, "spCost": 38}}, {"name": "Duetto", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 20, "spCost": 37}}, {"name": "Duetto", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 25, "spCost": 35}}]}}, {"skillId": "skchr_amgoat_2", "levelUpCostCond": [{"unlockCond": {"phase": "PHASE_2", "level": 1}}, {"unlockCond": {"phase": "PHASE_2", "level": 1}}, {"unlockCond": {"phase": "PHASE_2", "level": 1}}], "static": {"skillId": "skchr_amgoat_2", "iconId": null, "image": "/textures/spritepack/skill_icons_0/skill_icon_skchr_amgoat_2.png", "levels": [{"name": "Ignition", "skillType": "AUTO", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 0, "spCost": 7}}, {"name": "Ignition", "skillType": "AUTO", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 0, "spCost": 7}}, {"name": "Ignition", "skillType": "AUTO", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 0, "spCost": 7}}, {"name": "Ignition", "skillType": "AUTO", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 0, "spCost": 7}}, {"name": "Ignition", "skillType": "AUTO", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 0, "spCost": 7}}, {"name": "Ignition", "skillType": "AUTO", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 0, "spCost": 7}}, {"name": "Ignition", "skillType": "AUTO", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 0, "spCost": 6}}, {"name": "Ignition", "skillType": "AUTO", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 0, "spCost": 6}}, {"name": "Ignition", "skillType": "AUTO", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 0, "spCost": 6}}, {"name": "Ignition", "skillType": "AUTO", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 0, "spCost": 5}}]}}, {"skillId": "skchr_amgoat_3", "levelUpCostCond": [{"unlockCond": {"phase": "PHASE_2", "level": 1}}, {"unlockCond": {"phase": "PHASE_2", "level": 1}}, {"unlockCond": {"phase": "PHASE_2", "level": 1}}], "static": {"skillId": "skchr_amgoat_3", "iconId": null, "image": "/textures/spritepack/skill_icons_0/skill_icon_skchr_amgoat_3.png", "levels": [{"name": "Volcano", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 30, "spCost": 80}}, {"name": "Volcano", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 31, "spCost": 80}}, {"name": "Volcano", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 32, "spCost": 80}}, {"name": "Volcano", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 33, "spCost": 80}}, {"name": "Volcano", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 34, "spCost": 80}}, {"name": "Volcano", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 35, "spCost": 80}}, {"name": "Volcano", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 40, "spCost": 80}}, {"name": "Volcano", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 45, "spCost": 80}}, {"name": "Volcano", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 50, "spCost": 80}}, {"name": "Volcano", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 55, "spCost": 80}}]}}], "modules": [{"uniEquipId": "uniequip_001_amgoat", "uniEquipName": "Eyjafjalla's Badge", "uniEquipIcon": "uniequip_001_amgoat", "typeName1": "ORIGINAL", "type": "INITIAL", "unlockEvolvePhase": "PHASE_0", "unlockLevel": 0}, {"uniEquipId": "uniequip_002_amgoat", "uniEquipName": "'Missed Sounds'", "uniEquipIcon": "uniequip_002_amgoat", "image": "/textures/spritepack/ui_equip_big_img_hub_0/uniequip_002_amgoat.png", "typeName1": "CCR", "typeName2": "X", "type": "ADVANCED", "unlockEvolvePhase": "PHASE_2", "unlockLevel": 60}, {"uniEquipId": "uniequip_003_amgoat", "uniEquipName": "Pet Contest First Prize", "uniEquipIcon": "uniequip_003_amgoat", "image": "/textures/spritepack/ui_equip_big_img_hub_21/uniequip_003_amgoat.png", "typeName1": "CCR", "typeName2": "Y", "type": "ADVANCED", "unlockEvolvePhase": "PHASE_2", "unlockLevel": 60}]} as unknown as IOperatorListItem;

const TEXAS = {"id": "char_102_texas", "name": "Texas", "rarity": "TIER_5", "profession": "PIONEER", "subProfessionId": "pioneer", "position": "MELEE", "server": "en", "portrait": "/portraits/char_102_texas_2.png", "skin": null, "audio": [], "phases": [{"maxLevel": 50}, {"maxLevel": 70}, {"maxLevel": 80}], "allSkillLevelUp": [{"unlockCond": {"phase": "PHASE_0", "level": 1}}, {"unlockCond": {"phase": "PHASE_0", "level": 1}}, {"unlockCond": {"phase": "PHASE_0", "level": 1}}, {"unlockCond": {"phase": "PHASE_1", "level": 1}}, {"unlockCond": {"phase": "PHASE_1", "level": 1}}, {"unlockCond": {"phase": "PHASE_1", "level": 1}}], "skills": [{"skillId": "skcom_charge_cost[3]", "levelUpCostCond": [{"unlockCond": {"phase": "PHASE_2", "level": 1}}, {"unlockCond": {"phase": "PHASE_2", "level": 1}}, {"unlockCond": {"phase": "PHASE_2", "level": 1}}], "static": {"skillId": "skcom_charge_cost[3]", "iconId": null, "image": "/textures/spritepack/skill_icons_1/skill_icon_skcom_charge_cost[3].png", "levels": [{"name": "Charge γ", "skillType": "AUTO", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 10, "spCost": 44}}, {"name": "Charge γ", "skillType": "AUTO", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 10, "spCost": 43}}, {"name": "Charge γ", "skillType": "AUTO", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 10, "spCost": 42}}, {"name": "Charge γ", "skillType": "AUTO", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 12, "spCost": 41}}, {"name": "Charge γ", "skillType": "AUTO", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 12, "spCost": 40}}, {"name": "Charge γ", "skillType": "AUTO", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 12, "spCost": 39}}, {"name": "Charge γ", "skillType": "AUTO", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 14, "spCost": 38}}, {"name": "Charge γ", "skillType": "AUTO", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 16, "spCost": 37}}, {"name": "Charge γ", "skillType": "AUTO", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 18, "spCost": 36}}, {"name": "Charge γ", "skillType": "AUTO", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 20, "spCost": 35}}]}}, {"skillId": "skchr_texas_2", "levelUpCostCond": [{"unlockCond": {"phase": "PHASE_2", "level": 1}}, {"unlockCond": {"phase": "PHASE_2", "level": 1}}, {"unlockCond": {"phase": "PHASE_2", "level": 1}}], "static": {"skillId": "skchr_texas_2", "iconId": null, "image": "/textures/spritepack/skill_icons_1/skill_icon_skchr_texas_2.png", "levels": [{"name": "Sword Rain", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 20, "spCost": 40}}, {"name": "Sword Rain", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 21, "spCost": 40}}, {"name": "Sword Rain", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 22, "spCost": 40}}, {"name": "Sword Rain", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 23, "spCost": 40}}, {"name": "Sword Rain", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 24, "spCost": 40}}, {"name": "Sword Rain", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 25, "spCost": 40}}, {"name": "Sword Rain", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 26, "spCost": 40}}, {"name": "Sword Rain", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 27, "spCost": 40}}, {"name": "Sword Rain", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 28, "spCost": 40}}, {"name": "Sword Rain", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 30, "spCost": 40}}]}}], "modules": [{"uniEquipId": "uniequip_001_texas", "uniEquipName": "Texas's Badge", "uniEquipIcon": "uniequip_001_texas", "typeName1": "ORIGINAL", "type": "INITIAL", "unlockEvolvePhase": "PHASE_0", "unlockLevel": 0}, {"uniEquipId": "uniequip_002_texas", "uniEquipName": "Private Field Supply Package", "uniEquipIcon": "uniequip_002_texas", "image": "/textures/spritepack/ui_equip_big_img_hub_9/uniequip_002_texas.png", "typeName1": "SOL", "typeName2": "Y", "type": "ADVANCED", "unlockEvolvePhase": "PHASE_2", "unlockLevel": 50}]} as unknown as IOperatorListItem;

type Floor = { elite: number; level: number; skills: Record<number, number>; modules: Record<string, number> };

/** Rarity caps as `getMaxLevel` in planTargets.ts has them. */
function maxLevelFor(rarity: number, elite: number): number {
    if (rarity === 3) return elite === 0 ? 40 : 55;
    if (rarity === 4) return elite === 0 ? 45 : elite === 1 ? 60 : 70;
    if (rarity === 5) return elite === 0 ? 50 : elite === 1 ? 70 : 80;
    return elite === 0 ? 50 : elite === 1 ? 80 : 90;
}

const NO_FLOOR: Floor = { elite: 0, level: 1, skills: {}, modules: {} };

/**
 * The slice of `usePlanTargets()` the target sections read, held in local
 * state so the steps stay clickable. The real hook needs the dialog's queries.
 */
function useTargets({ rarity, elite: e0, level: l0, skills = {}, modules = {}, floor = NO_FLOOR }: { rarity: number; elite: number; level: number; skills?: Record<number, number>; modules?: Record<string, number>; floor?: Floor }) {
    const [elite, setElite] = useState(e0);
    const [level, setLevel] = useState(l0);
    const [skillTargets, setSkillTargets] = useState(skills);
    const [moduleTargets, setModuleTargets] = useState(modules);
    const maxLevel = maxLevelFor(rarity, elite);
    const minLevel = elite === floor.elite ? floor.level : 1;
    return {
        elite,
        level,
        maxElite: rarity <= 2 ? 0 : rarity === 3 ? 1 : 2,
        maxLevel,
        minLevel,
        floor,
        skillTargets,
        moduleTargets,
        displayOnProfile: false,
        selectedGroups: [] as string[],
        setDisplayOnProfile: () => undefined,
        setSelectedGroups: () => undefined,
        changeElite: (next: number) => {
            setElite(next);
            setLevel((prev) => Math.min(prev, maxLevelFor(rarity, next)));
        },
        changeLevel: (value: number) => setLevel(Math.min(Math.max(minLevel, Number.isNaN(value) ? minLevel : value), maxLevel)),
        changeSkillTarget: (idx: number, value: number) => setSkillTargets((prev) => ({ ...prev, [idx]: value })),
        changeModuleTarget: (id: string, stage: number) => setModuleTargets((prev) => ({ ...prev, [id]: stage })),
    };
}

function Stage({ operator, ...props }: Parameters<typeof useTargets>[0] & { operator: IOperatorListItem }) {
    const targets = useTargets(props);
    const [open, setOpen] = useState(true);
    return (
        <div className="max-w-3xl">
            <ModuleTargetsSection operator={operator} targets={targets} open={open} onOpenChange={setOpen} />
        </div>
    );
}

/** Two modules at E2 Lv90: CCR-X planned to stage 3, CCR-Y not planned. */
export const TwoModules = () => <Stage operator={EYJAFJALLA} rarity={6} elite={2} level={90} modules={{ uniequip_002_amgoat: 3, uniequip_003_amgoat: 0 }} />;

/** An E1 target: both modules unlock at E2 Lv60, so only "-" stays selectable. */
export const LockedBelowE2 = () => <Stage operator={EYJAFJALLA} rarity={6} elite={1} level={80} modules={{ uniequip_002_amgoat: 0, uniequip_003_amgoat: 0 }} />;

/** CCR-X owned at stage 2: stage 1 fades out as already reached. */
export const WithRosterFloor = () => <Stage operator={EYJAFJALLA} rarity={6} elite={2} level={90} modules={{ uniequip_002_amgoat: 3, uniequip_003_amgoat: 1 }} floor={{ elite: 2, level: 90, skills: {}, modules: { uniequip_002_amgoat: 2 } }} />;

/** A 5★ with a single module (SOL-Y), unlocked at E2 Lv50. */
export const SingleModule = () => <Stage operator={TEXAS} rarity={5} elite={2} level={80} modules={{ uniequip_002_texas: 2 }} />;
