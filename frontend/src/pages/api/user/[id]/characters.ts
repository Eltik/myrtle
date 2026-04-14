import type { NextApiRequest, NextApiResponse } from "next";
import { backendFetch } from "~/lib/backend-fetch";
import type { CharacterStatic, EnrichedRosterEntry, RosterEntry } from "~/types/api/impl/user";

/**
 * GET /api/user/{userId}/characters
 * Returns roster data for the Characters tab.
 * v3: Fetches from /roster endpoint, returns EnrichedRosterEntry[] with static data merged in.
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
        // Fetch roster and static operator data in parallel
        // v3: /static/operators returns Record<string, Operator> keyed by operator ID
        const [rosterResponse, operatorsResponse] = await Promise.all([
            backendFetch(`/roster?uid=${encodeURIComponent(id)}`),
            backendFetch("/static/operators"),
        ]);

        if (!rosterResponse.ok) {
            if (rosterResponse.status === 404) {
                return res.status(404).json({ error: "User not found" });
            }
            return res.status(rosterResponse.status).json({ error: "Failed to fetch roster data" });
        }

        // v3 returns masteries/modules in the view's raw shape — map them to the
        // field names the frontend components expect.
        type V3RosterEntry = Omit<RosterEntry, "masteries" | "modules"> & {
            masteries?: Array<{ index: number; mastery: number }>;
            modules?: Array<{ id: string; level: number; locked?: boolean }>;
        };
        const roster = (await rosterResponse.json()) as V3RosterEntry[];

        // Build operator static data lookup map - v3 returns a direct map, not wrapped
        let staticMap: Record<string, CharacterStatic> = {};
        if (operatorsResponse.ok) {
            staticMap = (await operatorsResponse.json()) as Record<string, CharacterStatic>;
        }

        // Enrich roster entries with static data + normalize field names
        const enriched: EnrichedRosterEntry[] = roster.map((entry) => {
            const staticOp = staticMap[entry.operator_id] ?? null;

            // Map {index, mastery} → {skill_id, specialize_level}
            // using the operator's static skill list (ordered by index)
            const masteries = (entry.masteries ?? []).map((m) => ({
                skill_id: staticOp?.skills?.[m.index]?.skillId ?? "",
                specialize_level: m.mastery,
            }));

            // Map {id, level, locked} → {equip_id, level}
            // Arknights game quirk: locked (not-yet-unlocked) modules still report
            // `level: 1` in the equip record. Treat them as level 0 so the UI
            // doesn't incorrectly show them as unlocked.
            const modules = (entry.modules ?? [])
                .filter((m) => !m.locked)
                .map((m) => ({
                    equip_id: m.id,
                    level: m.level,
                }));

            return {
                ...entry,
                masteries,
                modules,
                static: staticOp,
            } as EnrichedRosterEntry;
        });

        return res.status(200).json({
            characters: enriched,
        });
    } catch (error) {
        console.error("User characters API error:", error);
        return res.status(500).json({ error: "Internal server error" });
    }
}
