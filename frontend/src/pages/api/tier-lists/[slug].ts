import type { NextApiRequest, NextApiResponse } from "next";
import { getSiteToken } from "~/lib/auth";
import { backendFetch } from "~/lib/backend-fetch";
import type { AdminRole } from "~/lib/permissions";
import { canDeleteTierList, canToggleTierListActive, isAdminRole } from "~/lib/permissions";
import type { TierListResponse } from "~/types/api/impl/tier-list";

interface ApiSuccessResponse {
    success?: true;
    tier_list?: TierListResponse["tier_list"];
    tiers?: TierListResponse["tiers"];
    message?: string;
}

interface ApiErrorResponse {
    success: false;
    error: string;
}

type ApiResponse = ApiSuccessResponse | ApiErrorResponse | TierListResponse;

interface VerifyResponse {
    valid: boolean;
    role?: string;
}

async function verifyUserRole(token: string): Promise<{ valid: boolean; role: AdminRole | null }> {
    try {
        const response = await backendFetch("/auth/verify", {
            method: "POST",
            body: JSON.stringify({ token }),
        });
        if (!response.ok) return { valid: false, role: null };
        const data: VerifyResponse = await response.json();
        if (!data.valid || !data.role || !isAdminRole(data.role)) {
            return { valid: false, role: null };
        }
        return { valid: true, role: data.role as AdminRole };
    } catch {
        return { valid: false, role: null };
    }
}

export default async function handler(req: NextApiRequest, res: NextApiResponse<ApiResponse>) {
    const { slug } = req.query;

    if (!slug || typeof slug !== "string") {
        return res.status(400).json({
            success: false,
            error: "Slug is required",
        });
    }

    try {
        if (req.method === "GET") {
            // GET /tier-lists/{slug} - Get tier list with all tiers and placements (public)
            // Add cache-busting for admin requests (when _t parameter is present)
            const bustCache = req.query._t !== undefined;
            const response = await backendFetch(`/tier-lists/${slug}`, {
                method: "GET",
                headers: {
                    ...(bustCache && { "Cache-Control": "no-cache, no-store, must-revalidate" }),
                },
                cache: bustCache ? "no-store" : undefined,
            });

            if (!response.ok) {
                if (response.status === 404) {
                    return res.status(404).json({
                        success: false,
                        error: "Tier list not found",
                    });
                }
                const errorText = await response.text();
                console.error(`Backend GET /tier-lists/${slug} failed: ${response.status} - ${errorText}`);
                return res.status(response.status).json({
                    success: false,
                    error: "Failed to fetch tier list",
                });
            }

            const data = await response.json();

            // Transform backend response to match frontend TierListResponse format
            const tierListResponse: TierListResponse = {
                tier_list: {
                    id: data.id,
                    name: data.name,
                    slug: data.slug,
                    description: data.description,
                    is_active: data.is_active,
                    created_by: data.created_by,
                    created_at: data.created_at,
                    updated_at: data.updated_at,
                },
                tiers: (data.tiers || []).map((tier: { id: string; name: string; display_order: number; color: string | null; description: string | null; operators: Array<{ id: string; operator_id: string; sub_order: number; notes: string | null }> }) => ({
                    id: tier.id,
                    tier_list_id: data.id,
                    name: tier.name,
                    display_order: tier.display_order,
                    color: tier.color,
                    description: tier.description,
                    placements: (tier.operators || []).map((op: { id: string; operator_id: string; sub_order: number; notes: string | null }) => ({
                        id: op.id,
                        tier_id: tier.id,
                        operator_id: op.operator_id,
                        sub_order: op.sub_order,
                        notes: op.notes,
                        created_at: data.created_at,
                        updated_at: data.updated_at,
                    })),
                })),
            };

            return res.status(200).json(tierListResponse);
        }

        if (req.method === "PUT") {
            // PUT /tier-lists/{slug} - Update tier list (requires Admin permission)
            const siteToken = getSiteToken(req);

            if (!siteToken) {
                return res.status(401).json({
                    success: false,
                    error: "Not authenticated",
                });
            }

            const { name, description, is_active } = req.body;

            // Check if user can toggle active status - if not, strip is_active from request
            let allowedIsActive: boolean | undefined;
            if (is_active !== undefined) {
                const { valid, role } = await verifyUserRole(siteToken);
                if (valid && canToggleTierListActive(role)) {
                    allowedIsActive = is_active;
                }
                // If user doesn't have permission, we just don't send is_active to backend
            }

            const response = await backendFetch(`/tier-lists/${slug}`, {
                method: "PUT",
                headers: {
                    Authorization: `Bearer ${siteToken}`,
                },
                body: JSON.stringify({
                    name,
                    description,
                    ...(allowedIsActive !== undefined && { is_active: allowedIsActive }),
                }),
            });

            if (!response.ok) {
                const errorData = await response.json().catch(() => ({}));
                console.error(`Backend PUT /tier-lists/${slug} failed: ${response.status}`, errorData);
                return res.status(response.status).json({
                    success: false,
                    error: errorData.error || "Failed to update tier list",
                });
            }

            const data = await response.json();
            return res.status(200).json({
                success: true,
                tier_list: data.tier_list,
            });
        }

        if (req.method === "DELETE") {
            // DELETE /tier-lists/{slug} - Delete tier list (requires TierListAdmin or SuperAdmin)
            const siteToken = getSiteToken(req);

            if (!siteToken) {
                return res.status(401).json({
                    success: false,
                    error: "Not authenticated",
                });
            }

            // Server-side role verification
            const { valid, role } = await verifyUserRole(siteToken);
            if (!valid || !canDeleteTierList(role)) {
                return res.status(403).json({
                    success: false,
                    error: "You don't have permission to delete tier lists",
                });
            }

            const response = await backendFetch(`/tier-lists/${slug}`, {
                method: "DELETE",
                headers: {
                    Authorization: `Bearer ${siteToken}`,
                },
            });

            if (!response.ok) {
                const errorData = await response.json().catch(() => ({}));
                console.error(`Backend DELETE /tier-lists/${slug} failed: ${response.status}`, errorData);
                return res.status(response.status).json({
                    success: false,
                    error: errorData.error || "Failed to delete tier list",
                });
            }

            return res.status(200).json({
                success: true,
                message: "Tier list deleted successfully",
            });
        }

        res.setHeader("Allow", ["GET", "PUT", "DELETE"]);
        return res.status(405).json({
            success: false,
            error: `Method ${req.method} not allowed`,
        });
    } catch (error) {
        console.error("Tier list handler error:", error);
        return res.status(500).json({
            success: false,
            error: "An internal server error occurred",
        });
    }
}
