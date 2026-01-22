import type { NextApiRequest, NextApiResponse } from "next";
import { z } from "zod";
import { getSessionFromCookie, getSiteToken } from "~/lib/auth";
import { backendFetch } from "~/lib/backend-fetch";

// Schema for updating gacha settings
const UpdateGachaSettingsSchema = z.object({
    storeRecords: z.boolean().optional(),
    shareAnonymousStats: z.boolean().optional(),
});

// Response types
interface GachaSettings {
    user_id: string;
    store_records: boolean;
    share_anonymous_stats: boolean;
    total_pulls: number;
    six_star_count: number;
    five_star_count: number;
    last_sync_at: string | null;
}

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
    const token = getSiteToken(req);
    const session = getSessionFromCookie(req);

    if (!token || !session) {
        return res.status(401).json({
            success: false,
            error: "Not authenticated",
        });
    }

    // GET - Fetch current gacha settings
    if (req.method === "GET") {
        try {
            const response = await backendFetch(`/gacha/settings?token=${encodeURIComponent(token)}`);

            if (!response.ok) {
                const errorText = await response.text();
                console.error(`Backend gacha settings fetch failed: ${response.status} - ${errorText}`);

                return res.status(500).json({
                    success: false,
                    error: "Failed to fetch gacha settings",
                });
            }

            const settings: GachaSettings = await response.json();

            return res.status(200).json({
                success: true,
                settings,
            });
        } catch (error) {
            console.error("Error fetching gacha settings:", error);
            return res.status(500).json({
                success: false,
                error: "An internal server error occurred",
            });
        }
    }

    // POST - Update gacha settings
    if (req.method === "POST") {
        try {
            const parseResult = UpdateGachaSettingsSchema.safeParse(req.body);

            if (!parseResult.success) {
                return res.status(400).json({
                    success: false,
                    error: "Invalid request body",
                });
            }

            const { storeRecords, shareAnonymousStats } = parseResult.data;

            const response = await backendFetch("/gacha/settings", {
                method: "POST",
                body: JSON.stringify({
                    token,
                    store_records: storeRecords,
                    share_anonymous_stats: shareAnonymousStats,
                }),
            });

            if (!response.ok) {
                const errorText = await response.text();
                console.error(`Backend gacha settings update failed: ${response.status} - ${errorText}`);

                return res.status(500).json({
                    success: false,
                    error: "Failed to update gacha settings",
                });
            }

            const settings: GachaSettings = await response.json();

            return res.status(200).json({
                success: true,
                settings,
            });
        } catch (error) {
            console.error("Error updating gacha settings:", error);
            return res.status(500).json({
                success: false,
                error: "An internal server error occurred",
            });
        }
    }

    return res.status(405).json({ success: false, error: "Method not allowed" });
}
