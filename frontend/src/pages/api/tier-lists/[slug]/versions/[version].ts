import type { NextApiRequest, NextApiResponse } from "next";
import { env } from "~/env";
import type { TierListVersionDetail } from "~/types/api/impl/tier-list";

interface ApiErrorResponse {
    success: false;
    error: string;
}

type ApiResponse = TierListVersionDetail | ApiErrorResponse;

export default async function handler(req: NextApiRequest, res: NextApiResponse<ApiResponse>) {
    const { slug, version } = req.query;
    const backendURL = env.BACKEND_URL;

    if (!slug || typeof slug !== "string") {
        return res.status(400).json({
            success: false,
            error: "Slug is required",
        });
    }

    if (!version || typeof version !== "string") {
        return res.status(400).json({
            success: false,
            error: "Version is required",
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
        const response = await fetch(`${backendURL}/tier-lists/${slug}/versions/${version}`, {
            method: "GET",
            headers: {
                "Content-Type": "application/json",
            },
        });

        if (!response.ok) {
            if (response.status === 404) {
                return res.status(404).json({
                    success: false,
                    error: "Version not found",
                });
            }
            const errorText = await response.text();
            console.error(`Backend GET /tier-lists/${slug}/versions/${version} failed: ${response.status} - ${errorText}`);
            return res.status(response.status).json({
                success: false,
                error: "Failed to fetch version details",
            });
        }

        const data = (await response.json()) as TierListVersionDetail;
        return res.status(200).json(data);
    } catch (error) {
        console.error("Version detail handler error:", error);
        return res.status(500).json({
            success: false,
            error: "An internal server error occurred",
        });
    }
}
