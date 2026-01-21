import type { NextApiRequest, NextApiResponse } from "next";
import { getSiteToken } from "~/lib/auth";
import { backendFetch } from "~/lib/backend-fetch";
import type { TierListType } from "~/types/api/impl/tier-list";

interface TierListFromBackend {
    id: string;
    name: string;
    slug: string;
    description: string | null;
    is_active: boolean;
    tier_list_type: TierListType;
    created_by: string | null;
    created_at: string;
    updated_at: string;
}

interface ListResponse {
    tier_lists: TierListFromBackend[];
    count: number;
}

interface ApiSuccessResponse {
    success: true;
    tier_lists: TierListFromBackend[];
    count: number;
}

interface ApiErrorResponse {
    success: false;
    error: string;
}

type ApiResponse = ApiSuccessResponse | ApiErrorResponse;

export default async function handler(req: NextApiRequest, res: NextApiResponse<ApiResponse>) {
    if (req.method !== "GET") {
        res.setHeader("Allow", ["GET"]);
        return res.status(405).json({
            success: false,
            error: `Method ${req.method} not allowed`,
        });
    }

    try {
        const siteToken = getSiteToken(req);

        if (!siteToken) {
            return res.status(401).json({
                success: false,
                error: "Not authenticated",
            });
        }

        const response = await backendFetch("/tier-lists/mine", {
            headers: {
                Authorization: `Bearer ${siteToken}`,
            },
        });

        if (!response.ok) {
            const errorData = await response.json().catch(() => ({}));
            console.error(`Backend GET /tier-lists/mine failed: ${response.status}`, errorData);
            return res.status(response.status).json({
                success: false,
                error: errorData.error || "Failed to fetch your tier lists",
            });
        }

        const data: ListResponse = await response.json();
        return res.status(200).json({
            success: true,
            tier_lists: data.tier_lists,
            count: data.count,
        });
    } catch (error) {
        console.error("My tier lists handler error:", error);
        return res.status(500).json({
            success: false,
            error: "An internal server error occurred",
        });
    }
}
