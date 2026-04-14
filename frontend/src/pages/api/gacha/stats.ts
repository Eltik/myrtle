import type { NextApiRequest, NextApiResponse } from "next";
import { getToken } from "~/lib/auth";
import { backendFetch } from "~/lib/backend-fetch";

/**
 * GET /api/gacha/stats
 * Two modes:
 * - Default (no params): Global stats from GET /gacha/global-stats (public, no auth)
 * - With ?user=true: User stats from GET /gacha/stats (auth required, Bearer token)
 */
export default async function handler(req: NextApiRequest, res: NextApiResponse) {
    if (req.method !== "GET") {
        return res.status(405).json({ error: "Method not allowed" });
    }

    try {
        const isUserStats = req.query.user === "true";

        if (isUserStats) {
            const token = getToken(req);

            if (!token) {
                return res.status(401).json({ error: "Not authenticated" });
            }

            const response = await backendFetch("/gacha/stats", {
                headers: {
                    Authorization: `Bearer ${token}`,
                },
            });

            if (!response.ok) {
                const errorText = await response.text();
                console.error(`Backend user gacha stats fetch failed: ${response.status} - ${errorText}`);
                return res.status(response.status).json({ error: "Failed to fetch user gacha statistics" });
            }

            const data = await response.json();
            return res.status(200).json(data);
        }

        // Global stats - no auth needed
        const response = await backendFetch("/gacha/global-stats");

        if (!response.ok) {
            const errorText = await response.text();
            console.error(`Backend gacha global stats fetch failed: ${response.status} - ${errorText}`);
            return res.status(response.status).json({ error: "Failed to fetch global gacha statistics" });
        }

        const data = await response.json();
        return res.status(200).json(data);
    } catch (error) {
        console.error("Error fetching gacha stats:", error);
        return res.status(500).json({ error: "An internal server error occurred" });
    }
}
