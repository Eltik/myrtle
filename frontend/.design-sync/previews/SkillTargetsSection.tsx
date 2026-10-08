import { SkillTargetsSection } from "frontend";
import { useState } from "react";
import type { IOperatorListItem } from "../../src/types/operators";

// The plan dialog's collapsible "Skills" block: one row per skill with its
// icon, name, SP recovery, trigger and the SP numbers at the targeted level,
// then the step buttons: shared levels 1-7 and the three masteries. Steps the
// target promotion cannot reach are faded and explain why in a tooltip; steps
// below the roster read "Already reached".
//
// Młynar as `/api/operators/char_4064_mlynar` serves him (camelized by the API
// layer), trimmed to the fields the section reads.

const MLYNAR = {"id": "char_4064_mlynar", "name": "Młynar", "rarity": "TIER_6", "profession": "WARRIOR", "subProfessionId": "librator", "position": "MELEE", "server": "en", "portrait": "/portraits/char_4064_mlynar_2.png", "skin": null, "audio": [], "phases": [{"maxLevel": 50}, {"maxLevel": 80}, {"maxLevel": 90}], "allSkillLevelUp": [{"unlockCond": {"phase": "PHASE_0", "level": 1}}, {"unlockCond": {"phase": "PHASE_0", "level": 1}}, {"unlockCond": {"phase": "PHASE_0", "level": 1}}, {"unlockCond": {"phase": "PHASE_1", "level": 1}}, {"unlockCond": {"phase": "PHASE_1", "level": 1}}, {"unlockCond": {"phase": "PHASE_1", "level": 1}}], "skills": [{"skillId": "skchr_mlynar_1", "levelUpCostCond": [{"unlockCond": {"phase": "PHASE_2", "level": 1}}, {"unlockCond": {"phase": "PHASE_2", "level": 1}}, {"unlockCond": {"phase": "PHASE_2", "level": 1}}], "static": {"skillId": "skchr_mlynar_1", "iconId": null, "image": "/textures/spritepack/skill_icons_0/skill_icon_skchr_mlynar_1.png", "levels": [{"name": "Unvoiced Anger", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 10, "spCost": 40}}, {"name": "Unvoiced Anger", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 10, "spCost": 40}}, {"name": "Unvoiced Anger", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 10, "spCost": 40}}, {"name": "Unvoiced Anger", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 15, "spCost": 35}}, {"name": "Unvoiced Anger", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 15, "spCost": 35}}, {"name": "Unvoiced Anger", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 15, "spCost": 35}}, {"name": "Unvoiced Anger", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 15, "spCost": 30}}, {"name": "Unvoiced Anger", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 15, "spCost": 30}}, {"name": "Unvoiced Anger", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 15, "spCost": 30}}, {"name": "Unvoiced Anger", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 15, "spCost": 30}}]}}, {"skillId": "skchr_mlynar_2", "levelUpCostCond": [{"unlockCond": {"phase": "PHASE_2", "level": 1}}, {"unlockCond": {"phase": "PHASE_2", "level": 1}}, {"unlockCond": {"phase": "PHASE_2", "level": 1}}], "static": {"skillId": "skchr_mlynar_2", "iconId": null, "image": "/textures/spritepack/skill_icons_0/skill_icon_skchr_mlynar_2.png", "levels": [{"name": "Unresolved Sorrow", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 10, "spCost": 40}}, {"name": "Unresolved Sorrow", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 10, "spCost": 40}}, {"name": "Unresolved Sorrow", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 10, "spCost": 40}}, {"name": "Unresolved Sorrow", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 10, "spCost": 35}}, {"name": "Unresolved Sorrow", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 10, "spCost": 35}}, {"name": "Unresolved Sorrow", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 10, "spCost": 35}}, {"name": "Unresolved Sorrow", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 10, "spCost": 30}}, {"name": "Unresolved Sorrow", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 10, "spCost": 30}}, {"name": "Unresolved Sorrow", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 10, "spCost": 30}}, {"name": "Unresolved Sorrow", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 10, "spCost": 25}}]}}, {"skillId": "skchr_mlynar_3", "levelUpCostCond": [{"unlockCond": {"phase": "PHASE_2", "level": 1}}, {"unlockCond": {"phase": "PHASE_2", "level": 1}}, {"unlockCond": {"phase": "PHASE_2", "level": 1}}], "static": {"skillId": "skchr_mlynar_3", "iconId": null, "image": "/textures/spritepack/skill_icons_0/skill_icon_skchr_mlynar_3.png", "levels": [{"name": "Unbrilliant Glory", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 10, "spCost": 55}}, {"name": "Unbrilliant Glory", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 10, "spCost": 55}}, {"name": "Unbrilliant Glory", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 10, "spCost": 55}}, {"name": "Unbrilliant Glory", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 15, "spCost": 50}}, {"name": "Unbrilliant Glory", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 15, "spCost": 50}}, {"name": "Unbrilliant Glory", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 15, "spCost": 50}}, {"name": "Unbrilliant Glory", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 20, "spCost": 45}}, {"name": "Unbrilliant Glory", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 20, "spCost": 45}}, {"name": "Unbrilliant Glory", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 20, "spCost": 45}}, {"name": "Unbrilliant Glory", "skillType": "MANUAL", "spData": {"spType": "INCREASE_WITH_TIME", "initSp": 20, "spCost": 42}}]}}], "modules": [{"uniEquipId": "uniequip_001_mlynar", "uniEquipName": "Młynar's Badge", "uniEquipIcon": "uniequip_001_mlynar", "typeName1": "ORIGINAL", "type": "INITIAL", "unlockEvolvePhase": "PHASE_0", "unlockLevel": 0}, {"uniEquipId": "uniequip_002_mlynar", "uniEquipName": "'Man in Scabbard'", "uniEquipIcon": "uniequip_002_mlynar", "image": "/textures/spritepack/ui_equip_big_img_hub_19/uniequip_002_mlynar.png", "typeName1": "LIB", "typeName2": "X", "type": "ADVANCED", "unlockEvolvePhase": "PHASE_2", "unlockLevel": 60}]} as unknown as IOperatorListItem;

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

function Stage(props: Parameters<typeof useTargets>[0] & { initialOpen?: boolean }) {
    const targets = useTargets(props);
    const [open, setOpen] = useState(props.initialOpen ?? true);
    return (
        <div className="max-w-3xl">
            <SkillTargetsSection operator={MLYNAR} targets={targets} open={open} onOpenChange={setOpen} />
        </div>
    );
}

/** E2 target: S3 to M3, S1 to M1, S2 left at level 7. Every step is in reach. */
export const E2Masteries = () => <Stage rarity={6} elite={2} level={90} skills={{ 0: 8, 1: 7, 2: 10 }} />;

/** E1 target: levels 5-7 are reachable, every mastery is faded and locked behind Elite 2. */
export const E1MasteriesLocked = () => <Stage rarity={6} elite={1} level={80} skills={{ 0: 7, 1: 7, 2: 7 }} />;

/** Owned at SL7 with S3 at M2: levels 1-7 and M1 of S3 fade out as already reached. */
export const WithRosterFloor = () => <Stage rarity={6} elite={2} level={90} skills={{ 0: 7, 1: 7, 2: 10 }} floor={{ elite: 2, level: 60, skills: { 0: 7, 1: 7, 2: 9 }, modules: {} }} />;

/** Collapsed: just the section header. */
export const Collapsed = () => <Stage rarity={6} elite={2} level={90} skills={{ 0: 7, 1: 7, 2: 10 }} initialOpen={false} />;
