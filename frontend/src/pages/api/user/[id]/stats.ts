import type { NextApiRequest, NextApiResponse } from "next";
import { CLASS_DISPLAY, CLASS_SORT_ORDER, CLASSES } from "~/components/collection/operators/list/constants";
import { backendFetch } from "~/lib/backend-fetch";
import { formatSubProfession } from "~/lib/utils";
import type { Skin } from "~/types/api/impl/skin";
import type { ProfessionStat, SubProfessionStat, UserStatsResponse } from "~/types/api/impl/stats";
import type { RosterEntry } from "~/types/api/impl/user";

const EXCLUDED_PROFESSIONS = new Set(["TOKEN", "TRAP"]);

interface StaticOperator {
    id: string;
    profession: string;
    subProfessionId?: string;
    isNotObtainable?: boolean;
    rarity?: string;
}

/**
 * GET /api/user/{userId}/stats
 * Returns computed account stats for the Stats tab.
 * v3: Fetches from /roster endpoint for character data.
 */
export default async function handler(req: NextApiRequest, res: NextApiResponse) {
    if (req.method !== "GET") {
        return res.status(405).json({ error: "Method not allowed" });
    }

    const { id } = req.query;

    if (!id || typeof id !== "string") {
        return res.status(400).json({ error: "User ID is required" });
    }

    try {
        // Parallel fetch: roster + static operators + static skins
        const [rosterResponse, operatorsResponse, skinsResponse] = await Promise.all([backendFetch(`/roster?uid=${id}`), backendFetch("/static/operators?limit=1000&fields=id,profession,subProfessionId,isNotObtainable,rarity"), backendFetch("/static/skins?limit=5000")]);

        if (!rosterResponse.ok) {
            if (rosterResponse.status === 404) {
                return res.status(404).json({ error: "User not found" });
            }
            return res.status(rosterResponse.status).json({ error: "Failed to fetch roster data" });
        }

        const roster: RosterEntry[] = await rosterResponse.json();

        // Build static operator lookup: id -> StaticOperator
        const staticOperatorMap = new Map<string, StaticOperator>();
        const totalByProfession: Record<string, number> = {};
        const totalBySubProfession: Record<string, Record<string, number>> = {};
        let totalAvailable = 0;

        if (operatorsResponse.ok) {
            const operatorsJson = (await operatorsResponse.json()) as { operators?: StaticOperator[] };
            const allOperators = operatorsJson.operators ?? [];

            for (const op of allOperators) {
                if (!op.profession || EXCLUDED_PROFESSIONS.has(op.profession)) continue;
                staticOperatorMap.set(op.id, op);

                if (op.isNotObtainable) continue;
                totalByProfession[op.profession] = (totalByProfession[op.profession] ?? 0) + 1;
                totalAvailable++;

                if (op.subProfessionId) {
                    let profSubs = totalBySubProfession[op.profession];
                    if (!profSubs) {
                        profSubs = {};
                        totalBySubProfession[op.profession] = profSubs;
                    }
                    profSubs[op.subProfessionId] = (profSubs[op.subProfessionId] ?? 0) + 1;
                }
            }
        }

        // Process user's roster
        const ownedByProfession: Record<string, number> = {};
        const ownedBySubProfession: Record<string, Record<string, number>> = {};
        let e0 = 0;
        let e1 = 0;
        let e2 = 0;
        let m3Count = 0;
        let m6Count = 0;
        let m9Count = 0;
        let totalMasteryLevels = 0;
        let maxPossibleMasteryLevels = 0;
        let modulesUnlocked = 0;
        let modulesAtMax = 0;
        let totalModulesAvailable = 0;
        let totalOwned = 0;

        for (const entry of roster) {
            const staticOp = staticOperatorMap.get(entry.operator_id);
            const profession = staticOp?.profession;
            if (!profession || EXCLUDED_PROFESSIONS.has(profession)) continue;

            totalOwned++;
            ownedByProfession[profession] = (ownedByProfession[profession] ?? 0) + 1;

            const subProfId = staticOp?.subProfessionId;
            if (subProfId) {
                let profSubs = ownedBySubProfession[profession];
                if (!profSubs) {
                    profSubs = {};
                    ownedBySubProfession[profession] = profSubs;
                }
                profSubs[subProfId] = (profSubs[subProfId] ?? 0) + 1;
            }

            // Elite breakdown
            if (entry.elite === 2) e2++;
            else if (entry.elite === 1) e1++;
            else e0++;

            // Mastery stats
            const masteries = entry.masteries ?? [];
            let skillsAtM3 = 0;
            for (const mastery of masteries) {
                totalMasteryLevels += mastery.specialize_level;
                if (mastery.specialize_level === 3) skillsAtM3++;
            }
            if (skillsAtM3 >= 1) m3Count++;
            if (skillsAtM3 >= 2) m6Count++;
            if (skillsAtM3 >= 3) m9Count++;

            // Max possible mastery: only E2 operators can have masteries
            if (entry.elite === 2) {
                maxPossibleMasteryLevels += masteries.length * 3;
            }

            // Module stats - exclude INITIAL modules (uniequip_001 pattern)
            const modules = entry.modules ?? [];
            for (const mod of modules) {
                // Skip default/initial modules
                if (mod.equip_id.includes("uniequip_001")) continue;

                totalModulesAvailable++;

                if (mod.level > 0) {
                    modulesUnlocked++;
                    if (mod.level === 3) modulesAtMax++;
                }
            }
        }

        // Build per-profession stats sorted by CLASS_SORT_ORDER
        const professions: ProfessionStat[] = ([...CLASSES] as string[])
            .map((prof) => {
                const owned = ownedByProfession[prof] ?? 0;
                const total = totalByProfession[prof] ?? 0;

                const subProfTotals = totalBySubProfession[prof] ?? {};
                const subProfOwned = ownedBySubProfession[prof] ?? {};
                const allSubProfIds = new Set([...Object.keys(subProfTotals), ...Object.keys(subProfOwned)]);

                const subProfessions: SubProfessionStat[] = Array.from(allSubProfIds)
                    .map((subId) => {
                        const subOwned = subProfOwned[subId] ?? 0;
                        const subTotal = subProfTotals[subId] ?? 0;
                        return {
                            subProfessionId: subId,
                            displayName: formatSubProfession(subId),
                            owned: subOwned,
                            total: subTotal,
                            percentage: subTotal > 0 ? (subOwned / subTotal) * 100 : 0,
                        };
                    })
                    .sort((a, b) => a.displayName.localeCompare(b.displayName));

                return {
                    profession: prof,
                    displayName: CLASS_DISPLAY[prof] ?? prof,
                    owned,
                    total,
                    percentage: total > 0 ? (owned / total) * 100 : 0,
                    subProfessions,
                };
            })
            .sort((a, b) => (CLASS_SORT_ORDER[a.profession] ?? 99) - (CLASS_SORT_ORDER[b.profession] ?? 99));

        // Skin stats
        let totalSkinsAvailable = 0;
        if (skinsResponse.ok) {
            const skinsJson = (await skinsResponse.json()) as { skins?: Skin[] };
            const allSkins = skinsJson.skins ?? [];
            for (const skin of allSkins) {
                if (!skin.skinId.includes("@")) continue;
                totalSkinsAvailable++;
            }
        }

        // Count non-default skins from roster entries
        const userSkinIds = new Set<string>();
        for (const entry of roster) {
            if (entry.skin_id && entry.skin_id.includes("@")) {
                userSkinIds.add(entry.skin_id);
            }
        }
        const userSkinsOwned = userSkinIds.size;

        const skinPercentage = totalSkinsAvailable > 0 ? (userSkinsOwned / totalSkinsAvailable) * 100 : 0;

        const stats: UserStatsResponse = {
            professions,
            eliteBreakdown: { e0, e1, e2, total: totalOwned },
            masteries: { m3Count, m6Count, m9Count, totalMasteryLevels, maxPossibleMasteryLevels },
            modules: { unlocked: modulesUnlocked, atMax: modulesAtMax, totalAvailable: totalModulesAvailable },
            skins: {
                totalOwned: userSkinsOwned,
                totalAvailable: totalSkinsAvailable,
                percentage: skinPercentage,
            },
            totalOwned,
            totalAvailable,
            collectionPercentage: totalAvailable > 0 ? (totalOwned / totalAvailable) * 100 : 0,
        };

        return res.status(200).json(stats);
    } catch (error) {
        console.error("User stats API error:", error);
        return res.status(500).json({ error: "Internal server error" });
    }
}
