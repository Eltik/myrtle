import type { NextApiRequest, NextApiResponse } from "next";
import { getToken } from "~/lib/auth";
import { backendFetch } from "~/lib/backend-fetch";
import type { GachaSettings } from "~/types/api";

interface SuccessResponse {
    success: true;
    settings: GachaSettings;
}

interface ErrorResponse {
    success: false;
    error: string;
}

type ApiResponse = SuccessResponse | ErrorResponse;

export default async function handler(req: NextApiRequest, res: NextApiResponse<ApiResponse>) {
    const token = getToken(req);
    if (!token) {
        return res.status(401).json({ success: false, error: "Not authenticated" });
    }

    if (req.method === "GET") {
        try {
            const response = await backendFetch("/gacha/settings", {
                headers: { Authorization: `Bearer ${token}` },
            });

            if (!response.ok) {
                const errorText = await response.text();
                console.error(`Backend gacha settings fetch failed: ${response.status} - ${errorText}`);
                return res.status(500).json({ success: false, error: "Failed to fetch gacha settings" });
            }

            const settings: GachaSettings = await response.json();
            return res.status(200).json({ success: true, settings });
        } catch (error) {
            console.error("Error fetching gacha settings:", error);
            return res.status(500).json({ success: false, error: "An internal server error occurred" });
        }
    }

    if (req.method === "POST") {
        try {
            const body = req.body as { storeRecords?: boolean; shareAnonymousStats?: boolean };
            const payload: Record<string, unknown> = {};
            if (typeof body.storeRecords === "boolean") payload.store_records = body.storeRecords;
            if (typeof body.shareAnonymousStats === "boolean") payload.share_anonymous_stats = body.shareAnonymousStats;

            const response = await backendFetch("/gacha/settings", {
                method: "POST",
                headers: {
                    Authorization: `Bearer ${token}`,
                    "Content-Type": "application/json",
                },
                body: JSON.stringify(payload),
            });

            if (!response.ok) {
                const errorText = await response.text();
                console.error(`Backend gacha settings update failed: ${response.status} - ${errorText}`);
                return res.status(500).json({ success: false, error: "Failed to update gacha settings" });
            }

            const settings: GachaSettings = await response.json();
            return res.status(200).json({ success: true, settings });
        } catch (error) {
            console.error("Error updating gacha settings:", error);
            return res.status(500).json({ success: false, error: "An internal server error occurred" });
        }
    }

    return res.status(405).json({ success: false, error: "Method not allowed" });
}
