import type { NextApiRequest, NextApiResponse } from "next";
import { z } from "zod";
import { backendFetch } from "~/lib/backend-fetch";
import type { GachaEnhancedStats } from "~/types/api";

// Schema for query parameters
const EnhancedStatsQuerySchema = z.object({
    topN: z.coerce.number().int().min(1).max(50).optional(),
    includeTiming: z
        .string()
        .transform((val) => val === "true")
        .optional(),
});

interface SuccessResponse {
    success: true;
    data: GachaEnhancedStats;
}

interface ErrorResponse {
    success: false;
    error: string;
}

type ApiResponse = SuccessResponse | ErrorResponse;

/**
 * GET /api/gacha/stats/enhanced
 * Fetches comprehensive global statistics with caching.
 * This is a public endpoint - no authentication required.
 */
export default async function handler(req: NextApiRequest, res: NextApiResponse<ApiResponse>) {
    if (req.method !== "GET") {
        return res.status(405).json({ success: false, error: "Method not allowed" });
    }

    try {
        // Validate query parameters
        const parseResult = EnhancedStatsQuerySchema.safeParse(req.query);

        if (!parseResult.success) {
            return res.status(400).json({
                success: false,
                error: "Invalid query parameters",
            });
        }

        const { topN, includeTiming } = parseResult.data;

        // Build query string
        const params = new URLSearchParams();
        if (topN !== undefined) params.set("top_n", String(topN));
        if (includeTiming !== undefined) params.set("include_timing", String(includeTiming));

        const queryString = params.toString();
        const url = queryString ? `/gacha/stats/enhanced?${queryString}` : "/gacha/stats/enhanced";

        const response = await backendFetch(url);

        if (!response.ok) {
            const errorText = await response.text();
            console.error(`Backend enhanced stats fetch failed: ${response.status} - ${errorText}`);

            return res.status(500).json({
                success: false,
                error: "Failed to fetch enhanced gacha statistics",
            });
        }

        const data: GachaEnhancedStats = await response.json();

        return res.status(200).json({
            success: true,
            data,
        });
    } catch (error) {
        console.error("Error fetching enhanced gacha stats:", error);
        return res.status(500).json({
            success: false,
            error: "An internal server error occurred",
        });
    }
}
