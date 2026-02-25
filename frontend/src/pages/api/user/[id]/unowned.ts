import type { NextApiRequest, NextApiResponse } from "next";
import { backendFetch } from "~/lib/backend-fetch";
import type { StoredUser, UnownedOperator } from "~/types/api/impl/user";

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
        const [userResponse, operatorsResponse] = await Promise.all([backendFetch(`/get-user?uid=${id}`), backendFetch("/static/operators?limit=1000&fields=id,name,rarity,profession,subProfessionId,portrait,position,isNotObtainable")]);

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

        // Build set of owned character IDs
        const ownedIds = new Set<string>();
        for (const char of Object.values(chars)) {
            ownedIds.add(char.charId);
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
