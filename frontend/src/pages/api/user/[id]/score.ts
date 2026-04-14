import type { NextApiRequest, NextApiResponse } from "next";
import { backendFetch } from "~/lib/backend-fetch";
import type { UserProfile } from "~/types/api/impl/user";

/**
 * GET /api/user/{userId}/score
 * Returns score data for the Score tab.
 * v3: Extracts total_score and grade from UserProfile.
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

        const profile: UserProfile = await response.json();

        return res.status(200).json({
            total_score: profile.total_score ?? null,
            grade: profile.grade ?? null,
        });
    } catch (error) {
        console.error("User score API error:", error);
        return res.status(500).json({ error: "Internal server error" });
    }
}
