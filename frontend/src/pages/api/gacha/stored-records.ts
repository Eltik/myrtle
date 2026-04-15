import type { NextApiRequest, NextApiResponse } from "next";
import { getToken } from "~/lib/auth";
import { backendFetch } from "~/lib/backend-fetch";
import type { GachaRecords } from "~/types/api";

interface SuccessResponse {
    success: true;
    data: GachaRecords;
}

interface ErrorResponse {
    success: false;
    error: string;
}

type ApiResponse = SuccessResponse | ErrorResponse;

/**
 * GET /api/gacha/stored-records
 * Proxies the backend /gacha/stored-records endpoint which returns records
 * already grouped into {limited, regular, special} with totals.
 */
export default async function handler(req: NextApiRequest, res: NextApiResponse<ApiResponse>) {
    if (req.method !== "GET") {
        return res.status(405).json({ success: false, error: "Method not allowed" });
    }

    const token = getToken(req);
    if (!token) {
        return res.status(401).json({ success: false, error: "Not authenticated" });
    }

    try {
        const response = await backendFetch("/gacha/stored-records", {
            headers: { Authorization: `Bearer ${token}` },
        });

        if (!response.ok) {
            const errorText = await response.text();
            console.error(`Backend stored-records fetch failed: ${response.status} - ${errorText}`);
            return res.status(response.status).json({
                success: false,
                error: "Failed to fetch stored records",
            });
        }

        const data: GachaRecords = await response.json();
        return res.status(200).json({ success: true, data });
    } catch (error) {
        console.error("Error fetching stored records:", error);
        return res.status(500).json({ success: false, error: "An internal server error occurred" });
    }
}
