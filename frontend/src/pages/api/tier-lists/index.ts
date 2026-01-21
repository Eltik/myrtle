import type { NextApiRequest, NextApiResponse } from "next";
import { getSiteToken } from "~/lib/auth";
import { backendFetch } from "~/lib/backend-fetch";
import { isAdminRole } from "~/lib/permissions";
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

interface VerifyResponse {
    valid: boolean;
    role?: string;
}

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
    try {
        if (req.method === "GET") {
            // GET /tier-lists - List all tier lists (public)
            // Support type filter: ?type=official|community|all (default: all)
            const typeFilter = req.query.type as string | undefined;
            const queryParams = typeFilter ? `?type=${encodeURIComponent(typeFilter)}` : "";
            const response = await backendFetch(`/tier-lists${queryParams}`);

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
            // POST /tier-lists - Create a new tier list
            // - Any authenticated user can create community tier lists
            // - Only admins can create official tier lists
            const siteToken = getSiteToken(req);

            if (!siteToken) {
                return res.status(401).json({
                    success: false,
                    error: "Not authenticated",
                });
            }

            // Server-side role verification
            const { valid, role } = await verifyUserRole(siteToken);
            if (!valid) {
                return res.status(401).json({
                    success: false,
                    error: "Invalid or expired session",
                });
            }

            const { name, slug, description, is_active, tier_list_type } = req.body;

            if (!name || !slug) {
                return res.status(400).json({
                    success: false,
                    error: "Name and slug are required",
                });
            }

            // Determine tier list type based on user role
            // Non-admins can only create community tier lists
            const isAdmin = isAdminRole(role ?? undefined);
            const effectiveType: TierListType = isAdmin && tier_list_type === "official" ? "official" : "community";

            // Only admins can create active official tier lists
            const effectiveActive = effectiveType === "official" ? (is_active ?? false) : true;

            const response = await backendFetch("/tier-lists", {
                method: "POST",
                headers: {
                    Authorization: `Bearer ${siteToken}`,
                },
                body: JSON.stringify({
                    name,
                    slug,
                    description: description || null,
                    is_active: effectiveActive,
                    tier_list_type: effectiveType,
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
