import type { NextApiRequest, NextApiResponse } from "next";
import { getSessionFromCookie, getSiteToken } from "~/lib/auth";
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

export default async function handler(req: NextApiRequest, res: NextApiResponse<ApiResponse>) {
    if (req.method !== "GET") {
        return res.status(405).json({ success: false, error: "Method not allowed" });
    }

    try {
        const session = getSessionFromCookie(req);

        if (!session) {
            return res.status(401).json({
                success: false,
                error: "Not authenticated",
            });
        }

        const { yssid, yssid_sig } = session;

        if (!yssid || !yssid_sig) {
            return res.status(400).json({
                success: false,
                error: "Gacha data not available. Please log in again to sync your Yostar account.",
            });
        }

        // Call backend /gacha endpoint
        // Include token for automatic gacha record storage (opt-out model)
        const token = getSiteToken(req);

        const params = new URLSearchParams();
        params.set("yssid", yssid);
        params.set("yssid_sig", yssid_sig);
        if (token) {
            params.set("token", token);
        }

        const gachaResponse = await backendFetch(`/gacha?${params.toString()}`);

        if (!gachaResponse.ok) {
            const errorText = await gachaResponse.text();
            console.error(`Backend gacha fetch failed: ${gachaResponse.status} - ${errorText}`);

            return res.status(500).json({
                success: false,
                error: "Failed to fetch gacha records",
            });
        }

        const gachaData: GachaRecords = await gachaResponse.json();

        return res.status(200).json({
            success: true,
            data: gachaData,
        });
    } catch (error) {
        console.error("Error fetching gacha records:", error);
        return res.status(500).json({
            success: false,
            error: "An internal server error occurred",
        });
    }
}
