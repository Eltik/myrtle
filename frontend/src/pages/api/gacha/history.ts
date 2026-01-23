import type { NextApiRequest, NextApiResponse } from "next";
import { z } from "zod";
import { getSessionFromCookie, getSiteToken } from "~/lib/auth";
import { backendFetch } from "~/lib/backend-fetch";
import type { GachaHistoryResponse } from "~/types/api";

// Schema for query parameters
const HistoryQuerySchema = z.object({
    limit: z.coerce.number().int().min(1).max(100).optional(),
    offset: z.coerce.number().int().min(0).optional(),
    rarity: z.coerce.number().int().min(1).max(6).optional(),
    gachaType: z.enum(["limited", "regular", "special"]).optional(),
    charId: z.string().optional(),
    from: z.coerce.number().int().optional(),
    to: z.coerce.number().int().optional(),
    order: z.enum(["asc", "desc"]).optional(),
});

interface SuccessResponse {
    success: true;
    data: GachaHistoryResponse;
}

interface ErrorResponse {
    success: false;
    error: string;
}

type ApiResponse = SuccessResponse | ErrorResponse;

/**
 * GET /api/gacha/history
 * Fetches user's pull history with pagination and filters.
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
        // Validate query parameters
        const parseResult = HistoryQuerySchema.safeParse(req.query);

        if (!parseResult.success) {
            return res.status(400).json({
                success: false,
                error: "Invalid query parameters",
            });
        }

        const { limit, offset, rarity, gachaType, charId, from, to, order } = parseResult.data;

        // Build query string
        const params = new URLSearchParams();
        params.set("token", token);
        if (limit !== undefined) params.set("limit", String(limit));
        if (offset !== undefined) params.set("offset", String(offset));
        if (rarity !== undefined) params.set("rarity", String(rarity));
        if (gachaType !== undefined) params.set("gacha_type", gachaType);
        if (charId !== undefined) params.set("char_id", charId);
        if (from !== undefined) params.set("from", String(from));
        if (to !== undefined) params.set("to", String(to));
        if (order !== undefined) params.set("order", order);

        const response = await backendFetch(`/gacha/history?${params.toString()}`);

        if (!response.ok) {
            const errorText = await response.text();
            console.error(`Backend gacha history fetch failed: ${response.status} - ${errorText}`);

            return res.status(500).json({
                success: false,
                error: "Failed to fetch gacha history",
            });
        }

        const data: GachaHistoryResponse = await response.json();

        return res.status(200).json({
            success: true,
            data,
        });
    } catch (error) {
        console.error("Error fetching gacha history:", error);
        return res.status(500).json({
            success: false,
            error: "An internal server error occurred",
        });
    }
}
