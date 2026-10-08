import { PromotionLevelPanel } from "frontend";
import { useState } from "react";

// The plan dialog's promotion block: one Elite button per phase the rarity
// reaches (a 3★ stops at E1), and the level slider with its number input,
// capped at the chosen phase's max. Phases below the player's roster render
// as faded "Already reached" buttons, so a plan can never ask to go down.

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

/** A 6★ planned from scratch to E2 Lv90. */
export const SixStarToE2Max = () => {
    const targets = useTargets({ rarity: 6, elite: 2, level: 90 });
    return (
        <div className="max-w-2xl">
            <PromotionLevelPanel targets={targets} />
        </div>
    );
};

/** Already E1 Lv80 on the roster: E0 is faded out as reached; the target sits at E2 Lv60, where modules unlock. */
export const FloorAtE1 = () => {
    const targets = useTargets({ rarity: 6, elite: 2, level: 60, floor: { elite: 1, level: 80, skills: {}, modules: {} } });
    return (
        <div className="max-w-2xl">
            <PromotionLevelPanel targets={targets} />
        </div>
    );
};

/** A 5★ at E1: the slider caps at 70. */
export const FiveStarE1 = () => {
    const targets = useTargets({ rarity: 5, elite: 1, level: 55 });
    return (
        <div className="max-w-2xl">
            <PromotionLevelPanel targets={targets} />
        </div>
    );
};

/** A 3★ only reaches E1, so only two Elite buttons render (cap 55). */
export const ThreeStar = () => {
    const targets = useTargets({ rarity: 3, elite: 1, level: 55 });
    return (
        <div className="max-w-2xl">
            <PromotionLevelPanel targets={targets} />
        </div>
    );
};
