import type { NextApiRequest, NextApiResponse } from "next";
import { backendFetch } from "~/lib/backend-fetch";

/**
 * GET /api/user/{userId}/score
 * Returns the per-category score breakdown from `user_scores`:
 *   { total_score, operator_score, stage_score, roguelike_score,
 *     sandbox_score, medal_score, base_score, skin_score, grade,
 *     calculated_at }
 * or null if the user has never been scored.
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
        const response = await backendFetch(`/get-user-score?uid=${id}`);

        if (!response.ok) {
            if (response.status === 404) {
                return res.status(404).json({ error: "User not found" });
            }
            return res.status(response.status).json({ error: "Failed to fetch user score" });
        }

        const data = await response.json();
        return res.status(200).json(data);
    } catch (error) {
        console.error("User score API error:", error);
        return res.status(500).json({ error: "Internal server error" });
    }
}
