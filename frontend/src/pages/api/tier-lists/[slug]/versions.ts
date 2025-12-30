import type { NextApiRequest, NextApiResponse } from "next";
import { env } from "~/env";
import type { TierListVersionsResponse } from "~/types/api/impl/tier-list";

interface ApiErrorResponse {
    success: false;
    error: string;
}

type ApiResponse = TierListVersionsResponse | ApiErrorResponse;

export default async function handler(req: NextApiRequest, res: NextApiResponse<ApiResponse>) {
    const { slug } = req.query;
    const backendUrl = env.BACKEND_URL;

    if (!slug || typeof slug !== "string") {
        return res.status(400).json({
            success: false,
            error: "Slug is required",
        });
    }

    if (req.method !== "GET") {
        res.setHeader("Allow", ["GET"]);
        return res.status(405).json({
            success: false,
            error: `Method ${req.method} not allowed`,
        });
    }

    try {
        const limit = req.query.limit || "50";
        const response = await fetch(`${backendUrl}/tier-lists/${slug}/versions?limit=${limit}`, {
            method: "GET",
            headers: {
                "Content-Type": "application/json",
            },
        });

        if (!response.ok) {
            if (response.status === 404) {
                return res.status(404).json({
                    success: false,
                    error: "Tier list not found",
                });
            }
            const errorText = await response.text();
            console.error(`Backend GET /tier-lists/${slug}/versions failed: ${response.status} - ${errorText}`);
            return res.status(response.status).json({
                success: false,
                error: "Failed to fetch versions",
            });
        }

        const data = (await response.json()) as TierListVersionsResponse;
        return res.status(200).json(data);
    } catch (error) {
        console.error("Versions handler error:", error);
        return res.status(500).json({
            success: false,
            error: "An internal server error occurred",
        });
    }
}
