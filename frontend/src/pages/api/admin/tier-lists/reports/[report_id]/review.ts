import type { NextApiRequest, NextApiResponse } from "next";
import { getSiteToken } from "~/lib/auth";
import { backendFetch } from "~/lib/backend-fetch";
import { hasMinRole, isAdminRole } from "~/lib/permissions";
import type { TierListReport } from "~/types/api/impl/tier-list";

interface ReviewResponse {
    success: boolean;
    report: TierListReport;
}

interface VerifyResponse {
    valid: boolean;
    role?: string;
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

async function verifyUserRole(token: string): Promise<{ valid: boolean; role: string | null }> {
    try {
        const response = await backendFetch("/auth/verify", {
            method: "POST",
            body: JSON.stringify({ token }),
        });
        if (!response.ok) return { valid: false, role: null };
        const data: VerifyResponse = await response.json();
        return { valid: data.valid, role: data.role || null };
    } catch {
        return { valid: false, role: null };
    }
}

export default async function handler(req: NextApiRequest, res: NextApiResponse<ApiResponse>) {
    if (req.method !== "POST") {
        res.setHeader("Allow", ["POST"]);
        return res.status(405).json({
            success: false,
            error: `Method ${req.method} not allowed`,
        });
    }

    const { report_id } = req.query;

    if (!report_id || typeof report_id !== "string") {
        return res.status(400).json({
            success: false,
            error: "Missing report ID",
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

        // Verify user has admin role (tier_list_admin or higher)
        const { valid, role } = await verifyUserRole(siteToken);
        if (!valid || !isAdminRole(role ?? undefined) || !hasMinRole(role as "tier_list_editor" | "tier_list_admin" | "super_admin", "tier_list_admin")) {
            return res.status(403).json({
                success: false,
                error: "You don't have permission to review reports",
            });
        }

        const { action, action_taken } = req.body as { action: "approve" | "dismiss"; action_taken?: string };

        if (!action || !["approve", "dismiss"].includes(action)) {
            return res.status(400).json({
                success: false,
                error: "Invalid action. Must be 'approve' or 'dismiss'",
            });
        }

        const response = await backendFetch(`/admin/tier-lists/reports/${encodeURIComponent(report_id)}/review`, {
            method: "POST",
            headers: {
                Authorization: `Bearer ${siteToken}`,
            },
            body: JSON.stringify({
                action,
                action_taken: action_taken || null,
            }),
        });

        if (!response.ok) {
            const errorData = await response.json().catch(() => ({}));
            console.error(`Backend POST /admin/tier-lists/reports/${report_id}/review failed: ${response.status}`, errorData);
            return res.status(response.status).json({
                success: false,
                error: errorData.error || "Failed to review report",
            });
        }

        const data: ReviewResponse = await response.json();
        return res.status(200).json({
            success: true,
            report: data.report,
        });
    } catch (error) {
        console.error("Review report handler error:", error);
        return res.status(500).json({
            success: false,
            error: "An internal server error occurred",
        });
    }
}
