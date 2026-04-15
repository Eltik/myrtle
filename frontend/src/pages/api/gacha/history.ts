import type { NextApiRequest, NextApiResponse } from "next";
import { getToken } from "~/lib/auth";
import { backendFetch } from "~/lib/backend-fetch";
import type { GachaHistoryResponse } from "~/types/api";

interface SuccessResponse {
    success: true;
    data: GachaHistoryResponse;
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

    const token = getToken(req);
    if (!token) {
        return res.status(401).json({ success: false, error: "Not authenticated" });
    }

    try {
        const { rarity, limit, offset, gachaType, charId, from, to, order } = req.query;

        const params = new URLSearchParams();
        const setIfString = (k: string, v: unknown) => {
            if (typeof v === "string" && v.length > 0) params.set(k, v);
        };
        setIfString("rarity", rarity);
        setIfString("limit", limit);
        setIfString("offset", offset);
        setIfString("gachaType", gachaType);
        setIfString("charId", charId);
        setIfString("from", from);
        setIfString("to", to);
        setIfString("order", order);

        const response = await backendFetch(`/gacha/history?${params.toString()}`, {
            headers: { Authorization: `Bearer ${token}` },
        });

        if (!response.ok) {
            const errorText = await response.text();
            console.error(`Backend gacha history fetch failed: ${response.status} - ${errorText}`);
            return res.status(response.status).json({
                success: false,
                error: "Failed to fetch gacha history",
            });
        }

        const data: GachaHistoryResponse = await response.json();
        return res.status(200).json({ success: true, data });
    } catch (error) {
        console.error("Error fetching gacha history:", error);
        return res.status(500).json({ success: false, error: "An internal server error occurred" });
    }
}
