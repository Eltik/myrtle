import type { NextApiRequest, NextApiResponse } from "next";
import { getSiteToken } from "~/lib/auth";
import { backendFetch } from "~/lib/backend-fetch";
import { isAdminRole } from "~/lib/permissions";
import type { TierListReport } from "~/types/api/impl/tier-list";

interface ListReportsResponse {
    reports: TierListReport[];
}

interface VerifyResponse {
    valid: boolean;
    role?: string;
}

interface ApiSuccessResponse {
    success: true;
    reports: TierListReport[];
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

        // Verify user has admin role (any admin role can view reports)
        const { valid, role } = await verifyUserRole(siteToken);
        if (!valid || !isAdminRole(role ?? undefined)) {
            return res.status(403).json({
                success: false,
                error: "You don't have permission to view reports",
            });
        }

        // Get status filter from query
        const status = req.query.status as string | undefined;
        const queryParams = status ? `?status=${encodeURIComponent(status)}` : "";

        const response = await backendFetch(`/admin/tier-lists/reports${queryParams}`, {
            headers: {
                Authorization: `Bearer ${siteToken}`,
            },
        });

        if (!response.ok) {
            const errorData = await response.json().catch(() => ({}));
            console.error(`Backend GET /admin/tier-lists/reports failed: ${response.status}`, errorData);
            return res.status(response.status).json({
                success: false,
                error: errorData.error || "Failed to fetch reports",
            });
        }

        const data: ListReportsResponse = await response.json();
        return res.status(200).json({
            success: true,
            reports: data.reports,
        });
    } catch (error) {
        console.error("Admin reports handler error:", error);
        return res.status(500).json({
            success: false,
            error: "An internal server error occurred",
        });
    }
}
