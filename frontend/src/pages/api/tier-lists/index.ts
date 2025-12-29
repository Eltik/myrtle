import type { NextApiRequest, NextApiResponse } from "next";
import { env } from "~/env";
import { getSiteToken } from "~/lib/auth";

interface TierListFromBackend {
    id: string;
    name: string;
    slug: string;
    description: string | null;
    is_active: boolean;
    created_at: string;
    updated_at: string;
}

interface ListResponse {
    tier_lists: TierListFromBackend[];
}

interface CreateResponse {
    success: boolean;
    tier_list: TierListFromBackend;
}

interface ApiSuccessResponse {
    success: true;
    tier_lists?: TierListFromBackend[];
    tier_list?: TierListFromBackend;
}

interface ApiErrorResponse {
    success: false;
    error: string;
}

type ApiResponse = ApiSuccessResponse | ApiErrorResponse;

export default async function handler(req: NextApiRequest, res: NextApiResponse<ApiResponse>) {
    const backendUrl = env.BACKEND_URL;

    try {
        if (req.method === "GET") {
            // GET /tier-lists - List all tier lists (public)
            const response = await fetch(`${backendUrl}/tier-lists`, {
                method: "GET",
                headers: {
                    "Content-Type": "application/json",
                },
            });

            if (!response.ok) {
                const errorText = await response.text();
                console.error(`Backend GET /tier-lists failed: ${response.status} - ${errorText}`);
                return res.status(response.status).json({
                    success: false,
                    error: "Failed to fetch tier lists",
                });
            }

            const data: ListResponse = await response.json();
            return res.status(200).json({
                success: true,
                tier_lists: data.tier_lists,
            });
        }

        if (req.method === "POST") {
            // POST /tier-lists - Create a new tier list (requires TierListAdmin)
            const siteToken = getSiteToken(req);

            if (!siteToken) {
                return res.status(401).json({
                    success: false,
                    error: "Not authenticated",
                });
            }

            const { name, slug, description, is_active } = req.body;

            if (!name || !slug) {
                return res.status(400).json({
                    success: false,
                    error: "Name and slug are required",
                });
            }

            const response = await fetch(`${backendUrl}/tier-lists`, {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                    Authorization: `Bearer ${siteToken}`,
                },
                body: JSON.stringify({
                    name,
                    slug,
                    description: description || null,
                    is_active: is_active ?? false,
                }),
            });

            if (!response.ok) {
                const errorData = await response.json().catch(() => ({}));
                console.error(`Backend POST /tier-lists failed: ${response.status}`, errorData);
                return res.status(response.status).json({
                    success: false,
                    error: errorData.error || "Failed to create tier list",
                });
            }

            const data: CreateResponse = await response.json();
            return res.status(201).json({
                success: true,
                tier_list: data.tier_list,
            });
        }

        res.setHeader("Allow", ["GET", "POST"]);
        return res.status(405).json({
            success: false,
            error: `Method ${req.method} not allowed`,
        });
    } catch (error) {
        console.error("Tier lists handler error:", error);
        return res.status(500).json({
            success: false,
            error: "An internal server error occurred",
        });
    }
}
