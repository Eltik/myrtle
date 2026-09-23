import * as React from "react";

import type { IOperatorPlanResponse } from "#/lib/api/planner";
import type { IRosterEntry } from "#/lib/api/user";
import { rarityToNumber } from "#/lib/utils";
import type { IOperatorListItem } from "#/types/operators";
import { clampModuleTargets, clampSkillTargets, getMaxLevel, initialTargets, type ModuleTargets, maxEliteFor, type SkillTargets, withSkillTarget } from "./planTargets";

interface IUsePlanTargetsArgs {
    open: boolean;
    /** The operator's full record, or null while none is picked or it is loading. */
    operator: IOperatorListItem | null;
    existingPlan: IOperatorPlanResponse | undefined;
    /** Null when the roster endpoint answers with no roster. */
    roster: IRosterEntry[] | null;
    /** False only while a signed-in player's roster is still loading. */
    hasRosterData: boolean;
}

/** What the last seeding wrote, so the clamp can tell when it has landed. */
interface ISeededState {
    operatorId: string;
    elite: number;
    level: number;
    /** True once the seeded promotion and level have rendered. */
    isFullyApplied: boolean;
}

/**
 * The plan dialog's target state: promotion, level, skill and module targets,
 * the profile flag and the plan's groups.
 *
 * Two effects keep it consistent. The first seeds the targets once per picked
 * operator (from the saved plan, the roster, or defaults). It waits for a
 * signed-in player's roster because it never re-seeds the same operator:
 * seeding before the roster lands would lock in the E0 defaults. The second
 * lowers skill and module targets whenever promotion or level drop below what
 * they need. It runs in the same commit as the seed, while promotion and level
 * still hold the stale pre-seed values, and clamping the seeded skills against
 * those would throw away masteries the plan or roster legitimately has; the
 * `isFullyApplied` flag holds it off until the seeded promotion and level have
 * rendered.
 */
export function usePlanTargets({ open, operator, existingPlan, roster, hasRosterData }: IUsePlanTargetsArgs) {
    const [elite, setElite] = React.useState<number>(0);
    const [level, setLevel] = React.useState<number>(1);
    const [skillTargets, setSkillTargets] = React.useState<SkillTargets>({});
    const [moduleTargets, setModuleTargets] = React.useState<ModuleTargets>({});
    const [displayOnProfile, setDisplayOnProfile] = React.useState<boolean>(false);
    const [selectedGroups, setSelectedGroups] = React.useState<string[]>([]);
    const seededRef = React.useRef<ISeededState | null>(null);

    const rarity = operator ? rarityToNumber(operator.rarity) : 6;
    const maxElite = maxEliteFor(rarity);
    const maxLevel = operator ? getMaxLevel(rarity, elite) : 90;

    React.useEffect(() => {
        if (open) return;
        setElite(0);
        setLevel(1);
        setSkillTargets({});
        setModuleTargets({});
        setDisplayOnProfile(false);
        seededRef.current = null;
        setSelectedGroups([]);
    }, [open]);

    React.useEffect(() => {
        if (!operator || !hasRosterData) return;
        if (seededRef.current?.operatorId === operator.id) return;

        const rosterEntry = roster?.find((entry) => entry.operator_id === operator.id);
        const seed = initialTargets(operator, existingPlan, rosterEntry);

        setDisplayOnProfile(seed.displayOnProfile);
        setSelectedGroups(seed.groups);
        seededRef.current = {
            operatorId: operator.id ?? "",
            elite: seed.elite,
            level: seed.level,
            isFullyApplied: false,
        };
        setElite(seed.elite);
        setLevel(seed.level);
        setSkillTargets(seed.skills);
        setModuleTargets(seed.modules);
    }, [operator, roster, hasRosterData, existingPlan]);

    React.useEffect(() => {
        if (!operator) return;
        const seeded = seededRef.current;
        if (!seeded || seeded.operatorId !== operator.id) return;
        if (!seeded.isFullyApplied) {
            if (elite !== seeded.elite || level !== seeded.level) return;
            seeded.isFullyApplied = true;
        }

        setSkillTargets((prev) => clampSkillTargets(prev, operator, elite, level));
        setModuleTargets((prev) => clampModuleTargets(prev, operator, elite, level));
    }, [elite, level, operator]);

    const changeElite = (newElite: number) => {
        setElite(newElite);
        const maxLevelForElite = getMaxLevel(rarity, newElite);
        setLevel((prev) => Math.min(prev, maxLevelForElite));
    };

    /** Takes raw slider or input values: NaN (a cleared input) reads as 1, and the result is clamped to 1..maxLevel. */
    const changeLevel = (value: number) => {
        const target = Number.isNaN(value) ? 1 : value;
        setLevel(Math.min(Math.max(1, target), maxLevel));
    };

    const changeSkillTarget = (skillIdx: number, value: number) => {
        setSkillTargets((prev) => withSkillTarget(prev, operator?.skills.length ?? 0, skillIdx, value));
    };

    const changeModuleTarget = (moduleId: string, stage: number) => {
        setModuleTargets((prev) => ({ ...prev, [moduleId]: stage }));
    };

    return {
        elite,
        level,
        maxElite,
        maxLevel,
        skillTargets,
        moduleTargets,
        displayOnProfile,
        selectedGroups,
        setDisplayOnProfile,
        setSelectedGroups,
        changeElite,
        changeLevel,
        changeSkillTarget,
        changeModuleTarget,
    };
}

export type IPlanTargets = ReturnType<typeof usePlanTargets>;
