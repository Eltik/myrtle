import type { NextApiRequest, NextApiResponse } from "next";
import { backendFetch } from "~/lib/backend-fetch";
import type { StoredUser } from "~/types/api/impl/user";

/**
 * GET /api/user/{userId}/score
 * Returns score data for the Score tab
 * Used for client-side data fetching to reduce SSR payload
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
        const response = await backendFetch(`/get-user?uid=${id}`);

        if (!response.ok) {
            if (response.status === 404) {
                return res.status(404).json({ error: "User not found" });
            }
            return res.status(response.status).json({ error: "Failed to fetch user data" });
        }

        const userData: StoredUser = await response.json();

        // Return score data (can be null if not calculated)
        return res.status(200).json({
            score: userData.score ?? null,
        });
    } catch (error) {
        console.error("User score API error:", error);
        return res.status(500).json({ error: "Internal server error" });
    }
}
