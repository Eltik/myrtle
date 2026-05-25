import { useQuery } from "@tanstack/react-query";
import { useMemo } from "react";
import { operatorsListQueryOptions } from "#/lib/api/operators";
import type { IEnrichedSkill, IOperatorListItem, IOperatorModule } from "#/types/operators";

export interface IOperatorDetail {
    raw: IOperatorListItem | undefined;
    /** Display name of the skill at the given 1-indexed position. Falls back to `S{n}` if unknown. */
    skillName: (skillIndex: number) => string;
    /** Display name of the module at the given 1-indexed position. Returns "No module" for index 0. */
    moduleName: (moduleIndex: number) => string;
    /** Short description of a potential rank (potential 1 = no buff). */
    potentialLabel: (potential: number) => string;
    /** Skill object for the given 1-indexed position. */
    skillAt: (skillIndex: number) => IEnrichedSkill | undefined;
    /** Module object for the given 1-indexed position. */
    moduleAt: (moduleIndex: number) => IOperatorModule | undefined;
    /** Maximum operator level for the given promotion phase. Defaults to 90. */
    maxLevelForPromotion: (promotion: number) => number;
    /** Number of promotion phases this operator supports. */
    phaseCount: number;
}

/**
 * Look up enrichment data (skill names, module names, potential text) for the
 * operator that the DPS calculator already knows by id. Backed by the global
 * operator list query so all instances share a single fetch.
 */
export function useOperatorDetail(operatorId: string): IOperatorDetail {
    const { data: operators } = useQuery(operatorsListQueryOptions());
    return useMemo(() => buildDetail(operators?.find((op) => op.id === operatorId)), [operators, operatorId]);
}

function buildDetail(op: IOperatorListItem | undefined): IOperatorDetail {
    return {
        raw: op,
        skillName(skillIndex) {
            if (skillIndex <= 0) return "Basic attack";
            const fallback = `S${skillIndex}`;
            const skill = op?.skills?.[skillIndex - 1];
            const name = skill?.static?.levels?.[0]?.name?.trim();
            return name && name.length > 0 ? `${fallback} · ${name}` : fallback;
        },
        moduleName(moduleIndex) {
            if (moduleIndex <= 0) return "No module";
            const fallback = `Mod ${moduleIndex}`;
            const pick = pickModule(op, moduleIndex);
            const name = pick?.uniEquipName?.trim();
            return name && name.length > 0 ? `${fallback} · ${name}` : fallback;
        },
        potentialLabel(potential) {
            const fallback = `P${potential}`;
            if (potential <= 1) return `${fallback} · Base`;
            const rank = op?.potentialRanks?.[potential - 2];
            const desc = rank?.description?.trim();
            return desc && desc.length > 0 ? `${fallback} · ${desc}` : fallback;
        },
        skillAt(skillIndex) {
            if (skillIndex <= 0) return undefined;
            return op?.skills?.[skillIndex - 1];
        },
        moduleAt(moduleIndex) {
            if (moduleIndex <= 0) return undefined;
            return pickModule(op, moduleIndex);
        },
        maxLevelForPromotion(promotion) {
            const phase = op?.phases?.[promotion];
            return phase?.maxLevel ?? 90;
        },
        phaseCount: op?.phases?.length ?? 3,
    };
}

function pickModule(op: IOperatorListItem | undefined, moduleIndex: number): IOperatorModule | undefined {
    // `op.modules` includes the default "Operator's Badge" at index 0 (with
    // type === "INITIAL" / typeName1 === "ORIGINAL"). The DPS engine's
    // `availableModules` is 1-indexed against the FILTERED list (excluding
    // the badge), so to map e.g. moduleIndex 1 → first selectable module we
    // strip the badge first.
    if (moduleIndex <= 0) return undefined;
    const optional = (op?.modules ?? []).filter((m) => m.type !== "INITIAL" && m.typeName1?.toUpperCase() !== "ORIGINAL");
    return optional[moduleIndex - 1];
}
