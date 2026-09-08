import { useQuery } from "@tanstack/react-query";
import { useMemo } from "react";
import { operatorsListQueryOptions } from "#/lib/api/operators";
import type { IEnrichedSkill, IOperatorListItem, IOperatorModule } from "#/types/operators";
import type { IOperatorListEntry } from "./types";

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
export function useOperatorDetail(entry: IOperatorListEntry | undefined): IOperatorDetail {
    const { data: operators } = useQuery(operatorsListQueryOptions());
    return useMemo(
        () =>
            buildDetail(
                operators?.find((op) => op.id === entry?.id),
                entry,
            ),
        [operators, entry],
    );
}

function buildDetail(op: IOperatorListItem | undefined, entry: IOperatorListEntry | undefined): IOperatorDetail {
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
            const pick = resolveModule(op, moduleIndex, entry);
            // Prefer the in-game designator. `Mod N` is the engine's own index
            // and means nothing to a reader, so it is only a last resort.
            const label = moduleDesignator(pick) ?? `Mod ${moduleIndex}`;
            const name = pick?.uniEquipName?.trim();
            return name && name.length > 0 ? `${label} · ${name}` : label;
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
            return resolveModule(op, moduleIndex, entry);
        },
        maxLevelForPromotion(promotion) {
            const phase = op?.phases?.[promotion];
            return phase?.maxLevel ?? 90;
        },
        phaseCount: op?.phases?.length ?? 3,
    };
}

/** `SUM-X` and friends, or null for the badge and for malformed rows. */
export function moduleDesignator(mod: IOperatorModule | undefined): string | null {
    const t1 = mod?.typeName1?.trim();
    const t2 = mod?.typeName2?.trim();
    if (!t1 || !t2) return null;
    return `${t1}-${t2}`;
}

export function resolveModule(op: IOperatorListItem | undefined, moduleIndex: number, entry: IOperatorListEntry | undefined): IOperatorModule | undefined {
    if (moduleIndex <= 0) return undefined;

    // Join on identity: the DPS payload names the `uniEquipId` that each module
    // number resolves to, resolved server-side by the same code the engine
    // simulates with. Counting positions instead used to answer with whatever
    // the operator endpoint happened to list at that offset - a different list,
    // in a different order, from a different request.
    const slot = entry?.availableModules?.indexOf(moduleIndex) ?? -1;
    const id = slot >= 0 ? entry?.availableModuleIds?.[slot] : undefined;
    if (id) {
        const byId = (op?.modules ?? []).find((m) => m.uniEquipId === id);
        if (byId) return byId;
    }

    // Backend too old to send ids: fall back to the historical positional
    // reading rather than showing nothing.
    const optional = (op?.modules ?? []).filter((m) => m.type !== "INITIAL" && m.typeName1?.toUpperCase() !== "ORIGINAL");
    return optional[moduleIndex - 1];
}

/**
 * Compact module label for chart exports: the in-game designator when it can be
 * resolved, the engine's own index only when it cannot.
 */
export function moduleShortLabel(op: IOperatorListItem | undefined, entry: IOperatorListEntry | undefined, moduleIndex: number): string {
    if (moduleIndex <= 0) return "no module";
    return moduleDesignator(resolveModule(op, moduleIndex, entry)) ?? `Mod${moduleIndex}`;
}
