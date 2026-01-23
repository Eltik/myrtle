import type { NextApiRequest, NextApiResponse } from "next";
import { z } from "zod";
import { getSessionFromCookie, getSiteToken } from "~/lib/auth";
import { backendFetch } from "~/lib/backend-fetch";
import type { GachaRecordEntry } from "~/types/api";

// Schema for path parameter
const PathParamsSchema = z.object({
    charId: z.string().min(1),
});

interface SuccessResponse {
    success: true;
    data: GachaRecordEntry[];
}

interface ErrorResponse {
    success: false;
    error: string;
}

type ApiResponse = SuccessResponse | ErrorResponse;

/**
 * GET /api/gacha/history/[charId]
 * Fetches all pulls of a specific operator for the authenticated user.
 * Requires authentication.
 */
export default async function handler(req: NextApiRequest, res: NextApiResponse<ApiResponse>) {
    if (req.method !== "GET") {
        return res.status(405).json({ success: false, error: "Method not allowed" });
    }

    const token = getSiteToken(req);
    const session = getSessionFromCookie(req);

    if (!token || !session) {
        return res.status(401).json({
            success: false,
            error: "Not authenticated",
        });
    }

    try {
        // Validate path parameter
        const parseResult = PathParamsSchema.safeParse(req.query);

        if (!parseResult.success) {
            return res.status(400).json({
                success: false,
                error: "Invalid operator ID",
            });
        }

        const { charId } = parseResult.data;

        const response = await backendFetch(`/gacha/history/operator/${encodeURIComponent(charId)}?token=${encodeURIComponent(token)}`);

        if (!response.ok) {
            const errorText = await response.text();
            console.error(`Backend operator history fetch failed: ${response.status} - ${errorText}`);

            return res.status(500).json({
                success: false,
                error: "Failed to fetch operator history",
            });
        }

        const data: GachaRecordEntry[] = await response.json();

        return res.status(200).json({
            success: true,
            data,
        });
    } catch (error) {
        console.error("Error fetching operator history:", error);
        return res.status(500).json({
            success: false,
            error: "An internal server error occurred",
        });
    }
}
