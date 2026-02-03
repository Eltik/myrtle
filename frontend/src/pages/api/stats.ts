import type { NextApiRequest, NextApiResponse } from "next";
import { backendFetch } from "~/lib/backend-fetch";
import type { StatsResponse } from "~/types/api";

export default async function handler(req: NextApiRequest, res: NextApiResponse) {
    if (req.method !== "GET") {
        return res.status(405).json({ error: "Method not allowed" });
    }

    try {
        const response = await backendFetch("/stats");

        if (!response.ok) {
            return res.status(response.status).json({ error: "Failed to fetch stats" });
        }

        const data = (await response.json()) as StatsResponse;

        // Cache for 5 minutes, allow stale-while-revalidate for 10 minutes
        res.setHeader("Cache-Control", "public, s-maxage=300, stale-while-revalidate=600");

        return res.status(200).json(data);
    } catch (error) {
        console.error("Stats API error:", error);
        return res.status(500).json({ error: "Internal server error" });
    }
}
