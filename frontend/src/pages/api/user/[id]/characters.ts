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
        const [rosterResponse, operatorsResponse] = await Promise.all([
            backendFetch(`/roster?uid=${id}`),
            backendFetch("/static/operators?limit=1000"),
        ]);

        if (!rosterResponse.ok) {
            if (rosterResponse.status === 404) {
                return res.status(404).json({ error: "User not found" });
            }
            return res.status(rosterResponse.status).json({ error: "Failed to fetch roster data" });
        }

        const roster: RosterEntry[] = await rosterResponse.json();

        // Build operator static data lookup map
        let staticMap: Record<string, CharacterStatic> = {};
        if (operatorsResponse.ok) {
            const operatorsData = await operatorsResponse.json();
            const operators: CharacterStatic[] = operatorsData.operators ?? [];
            staticMap = Object.fromEntries(operators.map((op) => [op.id, op]));
        }

        // Enrich roster entries with static data
        const enriched: EnrichedRosterEntry[] = roster.map((entry) => ({
            ...entry,
            static: staticMap[entry.operator_id] ?? null,
        }));

        return res.status(200).json({
            characters: enriched,
        });
    } catch (error) {
        console.error("User characters API error:", error);
        return res.status(500).json({ error: "Internal server error" });
    }
}
