import { CLASS_SORT_ORDER, CLASSES } from "#/components/operators/list/impl/constants";
import type { ISkin } from "#/lib/api/skins";
import type { IRosterEntry } from "#/lib/api/user";
import { formatProfession, formatSubProfession } from "#/lib/utils";
import type { IOperatorListItem } from "#/types/operators";

interface IProfessionStats {
    profession: string;
    displayName: string;
    owned: number;
    total: number;
    percentage: number;
    subProfessions: {
        subProfessionId: string;
        displayName: string;
        owned: number;
        total: number;
        percentage: number;
    }[];
}

export interface IOperatorGapItem {
    id: string;
    operatorId: string;
    name: string;
    charId: string;
    rarity: number;
    sub?: string;
}

export interface IMasteryGapDetails {
    pendingM3: IOperatorGapItem[];
    pendingM6: IOperatorGapItem[];
    pendingM9: IOperatorGapItem[];
}

export interface IModuleGapDetails {
    locked: IOperatorGapItem[];
    belowMax: IOperatorGapItem[];
}

interface IUserStats {
    professions: IProfessionStats[];
    eliteBreakdown: { e0: number; e1: number; e2: number; total: number };
    masteries: {
        m3Count: number;
        m6Count: number;
        m9Count: number;
        totalMasteryLevels: number;
        maxPossibleMasteryLevels: number;
        e2Count: number;
        details: IMasteryGapDetails;
    };
    modules: {
        unlocked: number;
        atMax: number;
        totalAvailable: number;
        details: IModuleGapDetails;
    };
    skins: { totalOwned: number; totalAvailable: number; percentage: number };
    totalOwned: number;
    totalAvailable: number;
    collectionPercentage: number;
}

export function computeUserStats(roster: IRosterEntry[], operatorsStatic: IOperatorListItem[], skins: Record<string, ISkin> | undefined, nonDefaultSkinCount: number | null): IUserStats {
    const EXCLUDED_PROFESSIONS = new Set(["TOKEN", "TRAP"]);
    const EXCLUDED_MODULE_KEYS = ["uniequip_000", "uniequip_001"];
    const pct = (owned: number, total: number) => (total > 0 ? (owned / total) * 100 : 0);

    const operators = new Map<string, IOperatorListItem>();
    const totalByProfession: Record<string, number> = {};
    const totalBySubProfession: Record<string, Record<string, number>> = {};
    let totalAvailable = 0;

    for (const op of operatorsStatic) {
        if (!op.id || op.isNotObtainable || EXCLUDED_PROFESSIONS.has(op.profession)) continue;

        operators.set(op.id, op);
        totalAvailable++;
        totalByProfession[op.profession] = (totalByProfession[op.profession] ?? 0) + 1;

        if (op.subProfessionId) {
            totalBySubProfession[op.profession] ??= {};
            const subTotals = totalBySubProfession[op.profession];
            subTotals[op.subProfessionId] = (subTotals[op.subProfessionId] ?? 0) + 1;
        }
    }

    const ownedByProfession: Record<string, number> = {};
    const ownedBySubProfession: Record<string, Record<string, number>> = {};

    let totalOwned = 0;
    let e0 = 0,
        e1 = 0,
        e2 = 0;
    let m3Count = 0,
        m6Count = 0,
        m9Count = 0;
    let totalMasteryLevels = 0;
    let maxPossibleMasteryLevels = 0;
    let modulesUnlocked = 0;
    let modulesAtMax = 0;
    let totalModulesAvailable = 0;

    const pendingM3: IOperatorGapItem[] = [];
    const pendingM6: IOperatorGapItem[] = [];
    const pendingM9: IOperatorGapItem[] = [];
    const lockedModules: IOperatorGapItem[] = [];
    const belowMaxModules: IOperatorGapItem[] = [];

    for (const entry of roster) {
        const staticOp = operators.get(entry.operator_id);
        if (!staticOp) continue;

        totalOwned++;
        ownedByProfession[staticOp.profession] = (ownedByProfession[staticOp.profession] ?? 0) + 1;

        if (staticOp.subProfessionId) {
            ownedBySubProfession[staticOp.profession] ??= {};
            const subOwned = ownedBySubProfession[staticOp.profession];
            subOwned[staticOp.subProfessionId] = (subOwned[staticOp.subProfessionId] ?? 0) + 1;
        }

        if (entry.elite === 2) e2++;
        else if (entry.elite === 1) e1++;
        else e0++;

        const masteries = entry.masteries ?? [];
        let skillsAtM3 = 0;
        for (const mastery of masteries) {
            totalMasteryLevels += mastery.mastery;
            if (mastery.mastery === 3) skillsAtM3++;
        }
        if (skillsAtM3 >= 1) m3Count++;
        if (skillsAtM3 >= 2) m6Count++;
        if (skillsAtM3 >= 3) m9Count++;

        if (entry.elite === 2) {
            maxPossibleMasteryLevels += masteries.length * 3;

            if (masteries.length > 0) {
                const rarityNum = parseRarity(staticOp.rarity);
                const masterySub = formatMasterySummary(masteries);
                const baseGap: IOperatorGapItem = {
                    id: entry.operator_id,
                    operatorId: entry.operator_id,
                    name: staticOp.name,
                    charId: entry.operator_id,
                    rarity: rarityNum,
                    sub: masterySub,
                };
                if (skillsAtM3 === 0) pendingM3.push(baseGap);
                else if (skillsAtM3 === 1) pendingM6.push(baseGap);
                else if (skillsAtM3 === 2) pendingM9.push(baseGap);
            }
        }

        for (const mod of entry.modules ?? []) {
            if (EXCLUDED_MODULE_KEYS.some((key) => mod.id.startsWith(key))) continue;

            totalModulesAvailable++;
            const isLocked = mod.locked;
            const isBelowMax = !mod.locked && mod.level < 3;

            if (!mod.locked) {
                modulesUnlocked++;
                if (mod.level === 3) modulesAtMax++;
            }

            if (isLocked || isBelowMax) {
                const staticMod = staticOp.modules?.find((m) => m.uniEquipId === mod.id);
                const moduleLabel = staticMod ? `${staticMod.typeName1}${staticMod.typeName2 ? `-${staticMod.typeName2}` : ""}` : "Module";
                const item: IOperatorGapItem = {
                    id: `${entry.operator_id}:${mod.id}`,
                    operatorId: entry.operator_id,
                    name: staticOp.name,
                    charId: entry.operator_id,
                    rarity: parseRarity(staticOp.rarity),
                    sub: isLocked ? `${moduleLabel} • Locked` : `${moduleLabel} • Lv ${mod.level}/3`,
                };
                if (isLocked) lockedModules.push(item);
                else belowMaxModules.push(item);
            }
        }
    }

    const byRarityDesc = (a: IOperatorGapItem, b: IOperatorGapItem) => b.rarity - a.rarity || a.name.localeCompare(b.name);
    pendingM3.sort(byRarityDesc);
    pendingM6.sort(byRarityDesc);
    pendingM9.sort(byRarityDesc);
    lockedModules.sort(byRarityDesc);
    belowMaxModules.sort(byRarityDesc);

    const professions: IProfessionStats[] = ([...CLASSES] as string[])
        .map((prof) => {
            const owned = ownedByProfession[prof] ?? 0;
            const total = totalByProfession[prof] ?? 0;
            const subTotals = totalBySubProfession[prof] ?? {};
            const subOwned = ownedBySubProfession[prof] ?? {};

            const subProfessionIds = new Set([...Object.keys(subTotals), ...Object.keys(subOwned)]);
            const subProfessions = Array.from(subProfessionIds)
                .map((subId) => {
                    const so = subOwned[subId] ?? 0;
                    const st = subTotals[subId] ?? 0;
                    return {
                        subProfessionId: subId,
                        displayName: formatSubProfession(subId),
                        owned: so,
                        total: st,
                        percentage: pct(so, st),
                    };
                })
                .sort((a, b) => a.displayName.localeCompare(b.displayName));

            return {
                profession: prof,
                displayName: formatProfession(prof),
                owned,
                total,
                percentage: pct(owned, total),
                subProfessions,
            };
        })
        .sort((a, b) => (CLASS_SORT_ORDER[a.profession] ?? 99) - (CLASS_SORT_ORDER[b.profession] ?? 99));

    let totalSkinsAvailable = 0;
    if (skins) {
        for (const skin of Object.values(skins)) {
            if (skin.skinId?.includes("@")) totalSkinsAvailable++;
        }
    }
    const userSkinsOwned = nonDefaultSkinCount ?? 0;

    return {
        professions,
        eliteBreakdown: { e0, e1, e2, total: totalOwned },
        masteries: {
            m3Count,
            m6Count,
            m9Count,
            totalMasteryLevels,
            maxPossibleMasteryLevels,
            e2Count: e2,
            details: { pendingM3, pendingM6, pendingM9 },
        },
        modules: {
            unlocked: modulesUnlocked,
            atMax: modulesAtMax,
            totalAvailable: totalModulesAvailable,
            details: { locked: lockedModules, belowMax: belowMaxModules },
        },
        skins: {
            totalOwned: userSkinsOwned,
            totalAvailable: totalSkinsAvailable,
            percentage: pct(userSkinsOwned, totalSkinsAvailable),
        },
        totalOwned,
        totalAvailable,
        collectionPercentage: pct(totalOwned, totalAvailable),
    };
}

const RARITY_TIER_MAP: Record<string, number> = {
    TIER_1: 1,
    TIER_2: 2,
    TIER_3: 3,
    TIER_4: 4,
    TIER_5: 5,
    TIER_6: 6,
};

function parseRarity(rarity: string): number {
    return RARITY_TIER_MAP[rarity] ?? 0;
}

function formatMasterySummary(masteries: IRosterEntry["masteries"]): string {
    if (!masteries || masteries.length === 0) return "";
    const sorted = [...masteries].sort((a, b) => a.index - b.index);
    return sorted.map((m) => `M${m.mastery}`).join(" · ");
}
