import type { NextApiRequest, NextApiResponse } from "next";
import { getToken } from "~/lib/auth";
import { backendFetch } from "~/lib/backend-fetch";
import type { GachaRecordEntry } from "~/types/api";

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
 */
export default async function handler(req: NextApiRequest, res: NextApiResponse<ApiResponse>) {
    if (req.method !== "GET") {
        return res.status(405).json({ success: false, error: "Method not allowed" });
    }

    const token = getToken(req);
    if (!token) {
        return res.status(401).json({ success: false, error: "Not authenticated" });
    }

    const charIdParam = req.query.charId;
    const charId = Array.isArray(charIdParam) ? charIdParam[0] : charIdParam;
    if (typeof charId !== "string" || charId.length === 0) {
        return res.status(400).json({ success: false, error: "Invalid operator ID" });
    }

    try {
        const response = await backendFetch(`/gacha/history/${encodeURIComponent(charId)}`, {
            headers: { Authorization: `Bearer ${token}` },
        });

        if (!response.ok) {
            const errorText = await response.text();
            console.error(`Backend operator history fetch failed: ${response.status} - ${errorText}`);
            return res.status(response.status).json({
                success: false,
                error: "Failed to fetch operator history",
            });
        }

        const data: GachaRecordEntry[] = await response.json();
        return res.status(200).json({ success: true, data });
    } catch (error) {
        console.error("Error fetching operator history:", error);
        return res.status(500).json({ success: false, error: "An internal server error occurred" });
    }
}
