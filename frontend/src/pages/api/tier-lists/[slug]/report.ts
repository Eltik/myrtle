import type { NextApiRequest, NextApiResponse } from "next";
import { getSiteToken } from "~/lib/auth";
import { backendFetch } from "~/lib/backend-fetch";
import type { ReportReason, TierListReport } from "~/types/api/impl/tier-list";

interface CreateReportResponse {
    success: boolean;
    report: TierListReport;
}

interface ApiSuccessResponse {
    success: true;
    report: TierListReport;
}

interface ApiErrorResponse {
    success: false;
    error: string;
}

type ApiResponse = ApiSuccessResponse | ApiErrorResponse;

export default async function handler(req: NextApiRequest, res: NextApiResponse<ApiResponse>) {
    if (req.method !== "POST") {
        res.setHeader("Allow", ["POST"]);
        return res.status(405).json({
            success: false,
            error: `Method ${req.method} not allowed`,
        });
    }

    const { slug } = req.query;

    if (!slug || typeof slug !== "string") {
        return res.status(400).json({
            success: false,
            error: "Missing tier list slug",
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

        const { reason, description } = req.body as { reason: ReportReason; description?: string };

        if (!reason) {
            return res.status(400).json({
                success: false,
                error: "Report reason is required",
            });
        }

        const validReasons: ReportReason[] = ["inappropriate_content", "spam", "harassment", "other"];
        if (!validReasons.includes(reason)) {
            return res.status(400).json({
                success: false,
                error: "Invalid report reason",
            });
        }

        const response = await backendFetch(`/tier-lists/${encodeURIComponent(slug)}/report`, {
            method: "POST",
            headers: {
                Authorization: `Bearer ${siteToken}`,
            },
            body: JSON.stringify({
                reason,
                description: description || null,
            }),
        });

        if (!response.ok) {
            const errorData = await response.json().catch(() => ({}));
            console.error(`Backend POST /tier-lists/${slug}/report failed: ${response.status}`, errorData);
            return res.status(response.status).json({
                success: false,
                error: errorData.error || "Failed to submit report",
            });
        }

        const data: CreateReportResponse = await response.json();
        return res.status(201).json({
            success: true,
            report: data.report,
        });
    } catch (error) {
        console.error("Report tier list handler error:", error);
        return res.status(500).json({
            success: false,
            error: "An internal server error occurred",
        });
    }
}
