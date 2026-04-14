import type { NextApiRequest, NextApiResponse } from "next";
import { backendFetch } from "~/lib/backend-fetch";
import type { RosterEntry, UnownedOperator } from "~/types/api/impl/user";

const EXCLUDED_PROFESSIONS = new Set(["TOKEN", "TRAP"]);

interface StaticOperator {
    id: string;
    name: string;
    rarity: string;
    profession: string;
    subProfessionId: string;
    portrait: string;
    position: string;
    isNotObtainable?: boolean;
}

/**
 * GET /api/user/{userId}/unowned
 * Returns operators the user does not own.
 * v3: Fetches from /roster endpoint for owned operator IDs.
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
        const [rosterResponse, operatorsResponse] = await Promise.all([backendFetch(`/roster?uid=${id}`), backendFetch("/static/operators?limit=1000&fields=id,name,rarity,profession,subProfessionId,portrait,position,isNotObtainable")]);

        if (!rosterResponse.ok) {
            if (rosterResponse.status === 404) {
                return res.status(404).json({ error: "User not found" });
            }
            return res.status(rosterResponse.status).json({ error: "Failed to fetch roster data" });
        }

        const roster: RosterEntry[] = await rosterResponse.json();

        // Build set of owned operator IDs
        const ownedIds = new Set<string>();
        for (const entry of roster) {
            ownedIds.add(entry.operator_id);
        }

        const unowned: UnownedOperator[] = [];

        if (operatorsResponse.ok) {
            const operatorsJson = (await operatorsResponse.json()) as { operators?: StaticOperator[] };
            const allOperators = operatorsJson.operators ?? [];

            for (const op of allOperators) {
                if (!op.profession || EXCLUDED_PROFESSIONS.has(op.profession)) continue;
                if (op.isNotObtainable) continue;
                if (ownedIds.has(op.id)) continue;

                unowned.push({
                    charId: op.id,
                    name: op.name,
                    rarity: op.rarity,
                    profession: op.profession,
                    subProfessionId: op.subProfessionId,
                    portrait: op.portrait,
                    position: op.position,
                    isOwned: false,
                });
            }
        }

        return res.status(200).json({ unowned });
    } catch (error) {
        console.error("User unowned API error:", error);
        return res.status(500).json({ error: "Internal server error" });
    }
}
