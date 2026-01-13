import type { NextApiRequest, NextApiResponse } from "next";
import { env } from "~/env";
import { getSiteToken } from "~/lib/auth";
import type { TierListVersionSummary } from "~/types/api/impl/tier-list";

interface PublishSuccessResponse {
    success: true;
    version: TierListVersionSummary;
}

interface PublishErrorResponse {
    success: false;
    error: string;
}

type ApiResponse = PublishSuccessResponse | PublishErrorResponse;

export default async function handler(req: NextApiRequest, res: NextApiResponse<ApiResponse>) {
    const { slug } = req.query;
    const backendURL = env.BACKEND_URL;

    if (!slug || typeof slug !== "string") {
        return res.status(400).json({
            success: false,
            error: "Slug is required",
        });
    }

    if (req.method !== "POST") {
        res.setHeader("Allow", ["POST"]);
        return res.status(405).json({
            success: false,
            error: `Method ${req.method} not allowed`,
        });
    }

    // Require authentication
    const siteToken = getSiteToken(req);
    if (!siteToken) {
        return res.status(401).json({
            success: false,
            error: "Not authenticated",
        });
    }

    try {
        const { changelog, change_summary } = req.body;

        // Validate changelog is provided
        if (!changelog || typeof changelog !== "string" || changelog.trim() === "") {
            return res.status(400).json({
                success: false,
                error: "Changelog is required",
            });
        }

        const response = await fetch(`${backendURL}/tier-lists/${slug}/publish`, {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
                Authorization: `Bearer ${siteToken}`,
            },
            body: JSON.stringify({
                changelog: changelog.trim(),
                change_summary: change_summary?.trim() || null,
            }),
        });

        if (!response.ok) {
            if (response.status === 401 || response.status === 403) {
                return res.status(response.status).json({
                    success: false,
                    error: "You don't have permission to publish versions",
                });
            }
            if (response.status === 404) {
                return res.status(404).json({
                    success: false,
                    error: "Tier list not found",
                });
            }
            const errorData = await response.json().catch(() => ({}));
            console.error(`Backend POST /tier-lists/${slug}/publish failed: ${response.status}`, errorData);
            return res.status(response.status).json({
                success: false,
                error: errorData.error || "Failed to publish version",
            });
        }

        const data = await response.json();
        return res.status(200).json({
            success: true,
            version: data.version,
        });
    } catch (error) {
        console.error("Publish handler error:", error);
        return res.status(500).json({
            success: false,
            error: "An internal server error occurred",
        });
    }
}
