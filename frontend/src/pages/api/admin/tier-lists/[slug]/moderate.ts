import type { NextApiRequest, NextApiResponse } from "next";
import { getSiteToken } from "~/lib/auth";
import { backendFetch } from "~/lib/backend-fetch";
import { hasMinRole, isAdminRole } from "~/lib/permissions";

interface ModerateResponse {
    success: boolean;
    message: string;
}

interface VerifyResponse {
    valid: boolean;
    role?: string;
}

interface ApiSuccessResponse {
    success: true;
    message: string;
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

        // Verify user has admin role (tier_list_admin or higher)
        const { valid, role } = await verifyUserRole(siteToken);
        if (!valid || !isAdminRole(role ?? undefined) || !hasMinRole(role as "tier_list_editor" | "tier_list_admin" | "super_admin", "tier_list_admin")) {
            return res.status(403).json({
                success: false,
                error: "You don't have permission to moderate tier lists",
            });
        }

        const { reason } = req.body as { reason: string };

        if (!reason || !reason.trim()) {
            return res.status(400).json({
                success: false,
                error: "Moderation reason is required",
            });
        }

        const response = await backendFetch(`/admin/tier-lists/${encodeURIComponent(slug)}/moderate`, {
            method: "POST",
            headers: {
                Authorization: `Bearer ${siteToken}`,
            },
            body: JSON.stringify({
                reason: reason.trim(),
            }),
        });

        if (!response.ok) {
            const errorData = await response.json().catch(() => ({}));
            console.error(`Backend POST /admin/tier-lists/${slug}/moderate failed: ${response.status}`, errorData);
            return res.status(response.status).json({
                success: false,
                error: errorData.error || "Failed to moderate tier list",
            });
        }

        const data: ModerateResponse = await response.json();
        return res.status(200).json({
            success: true,
            message: data.message,
        });
    } catch (error) {
        console.error("Moderate tier list handler error:", error);
        return res.status(500).json({
            success: false,
            error: "An internal server error occurred",
        });
    }
}
