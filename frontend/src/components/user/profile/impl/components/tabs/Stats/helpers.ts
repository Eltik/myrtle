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

interface IUserStats {
    professions: IProfessionStats[];
    eliteBreakdown: { e0: number; e1: number; e2: number; total: number };
    masteries: {
        m3Count: number;
        m6Count: number;
        m9Count: number;
        totalMasteryLevels: number;
        maxPossibleMasteryLevels: number;
    };
    modules: { unlocked: number; atMax: number; totalAvailable: number };
    skins: { totalOwned: number; totalAvailable: number; percentage: number };
    totalOwned: number;
    totalAvailable: number;
    collectionPercentage: number;
}

export function computeUserStats(roster: IRosterEntry[], operatorsStatic: IOperatorListItem[], skins: Record<string, ISkin> | undefined): IUserStats {
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
    const userSkinIds = new Set<string>();

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

        let skillsAtM3 = 0;
        for (const mastery of entry.masteries ?? []) {
            totalMasteryLevels += mastery.mastery;
            if (mastery.mastery === 3) skillsAtM3++;
        }
        if (skillsAtM3 >= 1) m3Count++;
        if (skillsAtM3 >= 2) m6Count++;
        if (skillsAtM3 >= 3) m9Count++;

        if (entry.elite === 2) {
            maxPossibleMasteryLevels += (entry.masteries?.length ?? 0) * 3;
        }

        for (const mod of entry.modules ?? []) {
            if (EXCLUDED_MODULE_KEYS.some((key) => mod.id.startsWith(key))) continue;

            totalModulesAvailable++;
            if (!mod.locked) {
                modulesUnlocked++;
                if (mod.level === 3) modulesAtMax++;
            }
        }

        if (entry.skin_id?.includes("@")) {
            userSkinIds.add(entry.skin_id);
        }
    }

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
    const userSkinsOwned = userSkinIds.size;

    return {
        professions,
        eliteBreakdown: { e0, e1, e2, total: totalOwned },
        masteries: { m3Count, m6Count, m9Count, totalMasteryLevels, maxPossibleMasteryLevels },
        modules: { unlocked: modulesUnlocked, atMax: modulesAtMax, totalAvailable: totalModulesAvailable },
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
