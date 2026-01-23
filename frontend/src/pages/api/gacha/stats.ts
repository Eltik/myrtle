import type { NextApiRequest, NextApiResponse } from "next";
import { backendFetch } from "~/lib/backend-fetch";
import type { GachaGlobalStats } from "~/types/api";

interface SuccessResponse {
    success: true;
    data: GachaGlobalStats;
}

interface ErrorResponse {
    success: false;
    error: string;
}

type ApiResponse = SuccessResponse | ErrorResponse;

/**
 * GET /api/gacha/stats
 * Fetches global anonymous pull rate statistics.
 * This is a public endpoint - no authentication required.
 */
export default async function handler(req: NextApiRequest, res: NextApiResponse<ApiResponse>) {
    if (req.method !== "GET") {
        return res.status(405).json({ success: false, error: "Method not allowed" });
    }

    try {
        const response = await backendFetch("/gacha/stats");

        if (!response.ok) {
            const errorText = await response.text();
            console.error(`Backend gacha stats fetch failed: ${response.status} - ${errorText}`);

            return res.status(500).json({
                success: false,
                error: "Failed to fetch global gacha statistics",
            });
        }

        const stats: GachaGlobalStats = await response.json();

        return res.status(200).json({
            success: true,
            data: stats,
        });
    } catch (error) {
        console.error("Error fetching gacha stats:", error);
        return res.status(500).json({
            success: false,
            error: "An internal server error occurred",
        });
    }
}
