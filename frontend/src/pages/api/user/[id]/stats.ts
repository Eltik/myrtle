import type { NextApiRequest, NextApiResponse } from "next";
import { CLASS_DISPLAY, CLASS_SORT_ORDER, CLASSES } from "~/components/collection/operators/list/constants";
import { backendFetch } from "~/lib/backend-fetch";
import type { ProfessionStat, UserStatsResponse } from "~/types/api/impl/stats";
import type { StoredUser } from "~/types/api/impl/user";

const EXCLUDED_PROFESSIONS = new Set(["TOKEN", "TRAP"]);

interface StaticOperator {
    id: string;
    profession: string;
    isNotObtainable?: boolean;
}

/**
 * GET /api/user/{userId}/stats
 * Returns computed account stats for the Stats tab.
 * Computes all stats server-side to avoid sending multi-MB character data to the client.
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
        // Parallel fetch: user data + static operators for per-profession totals
        const [userResponse, operatorsResponse] = await Promise.all([backendFetch(`/get-user?uid=${id}`), backendFetch("/static/operators?limit=1000&fields=id,profession,isNotObtainable")]);

        if (!userResponse.ok) {
            if (userResponse.status === 404) {
                return res.status(404).json({ error: "User not found" });
            }
            return res.status(userResponse.status).json({ error: "Failed to fetch user data" });
        }

        const userData: StoredUser = await userResponse.json();
        const chars = userData?.data?.troop?.chars;

        if (!chars) {
            return res.status(404).json({ error: "Character data not found" });
        }

        // Build per-profession totals from static operators
        const totalByProfession: Record<string, number> = {};
        let totalAvailable = 0;

        if (operatorsResponse.ok) {
            const operatorsJson = (await operatorsResponse.json()) as { operators?: StaticOperator[] };
            const allOperators = operatorsJson.operators ?? [];

            for (const op of allOperators) {
                if (!op.profession || EXCLUDED_PROFESSIONS.has(op.profession)) continue;
                if (op.isNotObtainable) continue;
                totalByProfession[op.profession] = (totalByProfession[op.profession] ?? 0) + 1;
                totalAvailable++;
            }
        }

        // Process user's characters
        const ownedByProfession: Record<string, number> = {};
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

        for (const char of Object.values(chars)) {
            const profession = char.static?.profession;
            if (!profession || EXCLUDED_PROFESSIONS.has(profession)) continue;

            totalOwned++;
            ownedByProfession[profession] = (ownedByProfession[profession] ?? 0) + 1;

            // Elite breakdown
            if (char.evolvePhase === 2) e2++;
            else if (char.evolvePhase === 1) e1++;
            else e0++;

            // Mastery stats: M3 = at least 1, M6 = exactly 2, M9 = exactly 3 skills at M3
            const skills = char.skills ?? [];
            let skillsAtM3 = 0;
            for (const skill of skills) {
                totalMasteryLevels += skill.specializeLevel;
                if (skill.specializeLevel === 3) skillsAtM3++;
            }
            if (skillsAtM3 >= 1) m3Count++;
            if (skillsAtM3 === 2) m6Count++;
            if (skillsAtM3 === 3) m9Count++;

            // Max possible mastery: only E2 operators can have masteries
            if (char.evolvePhase === 2) {
                maxPossibleMasteryLevels += skills.length * 3;
            }

            // Module stats — only count non-default (ADVANCED) modules
            const staticModules = char.static?.modules ?? [];
            for (const mod of staticModules) {
                // Skip default modules: check both type field and ID pattern as safeguard
                if (mod.type === "INITIAL" || mod.type === "ORIGINAL") continue;
                if (mod.uniEquipId?.includes("_001_")) continue;
                totalModulesAvailable++;

                const equipData = char.equip?.[mod.uniEquipId];
                if (equipData && equipData.level > 0 && equipData.locked !== 1) {
                    modulesUnlocked++;
                    if (equipData.level === 3) modulesAtMax++;
                }
            }
        }

        // Build per-profession stats sorted by CLASS_SORT_ORDER
        const professions: ProfessionStat[] = ([...CLASSES] as string[])
            .map((prof) => {
                const owned = ownedByProfession[prof] ?? 0;
                const total = totalByProfession[prof] ?? 0;
                return {
                    profession: prof,
                    displayName: CLASS_DISPLAY[prof] ?? prof,
                    owned,
                    total,
                    percentage: total > 0 ? (owned / total) * 100 : 0,
                };
            })
            .sort((a, b) => (CLASS_SORT_ORDER[a.profession] ?? 99) - (CLASS_SORT_ORDER[b.profession] ?? 99));

        // Skin stats from score breakdown
        const breakdown = userData.score?.breakdown;

        const stats: UserStatsResponse = {
            professions,
            eliteBreakdown: { e0, e1, e2, total: totalOwned },
            masteries: { m3Count, m6Count, m9Count, totalMasteryLevels, maxPossibleMasteryLevels },
            modules: { unlocked: modulesUnlocked, atMax: modulesAtMax, totalAvailable: totalModulesAvailable },
            skins: {
                totalOwned: breakdown?.totalSkinsOwned ?? 0,
                fullCollectionCount: breakdown?.fullSkinCollectionCount ?? 0,
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
