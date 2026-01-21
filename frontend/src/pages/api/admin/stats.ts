import { parse } from "cookie";
import type { NextApiRequest, NextApiResponse } from "next";
import { backendFetch } from "~/lib/backend-fetch";
import { isAdminRole } from "~/lib/permissions";
import type { AdminStats } from "~/types/frontend/impl/admin";

interface VerifyResponse {
    valid: boolean;
    user_id?: string;
    uid?: string;
    role?: string;
}

interface ApiResponse {
    success: boolean;
    data?: AdminStats;
    error?: string;
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
        // Step 1: Extract token from cookies
        const cookies = parse(req.headers.cookie ?? "");
        const siteToken = cookies.site_token;

        if (!siteToken) {
            return res.status(401).json({
                success: false,
                error: "Not authenticated",
            });
        }

        // Step 2: Verify token and check role with backend
        const verifyResponse = await backendFetch("/auth/verify", {
            method: "POST",
            body: JSON.stringify({ token: siteToken }),
        });

        if (!verifyResponse.ok) {
            return res.status(401).json({
                success: false,
                error: "Token verification failed",
            });
        }

        const verifyData: VerifyResponse = await verifyResponse.json();

        if (!verifyData.valid) {
            return res.status(401).json({
                success: false,
                error: "Invalid token",
            });
        }

        // Step 3: Check admin role
        if (!isAdminRole(verifyData.role)) {
            return res.status(403).json({
                success: false,
                error: "Insufficient permissions",
            });
        }

        const userRole = verifyData.role;

        // Step 4: Fetch admin stats from backend (requires backend /admin/stats endpoint)
        const statsResponse = await backendFetch("/admin/stats", {
            method: "GET",
            headers: {
                Authorization: `Bearer ${siteToken}`,
            },
        });

        if (!statsResponse.ok) {
            // If backend endpoint doesn't exist yet or user lacks backend permission,
            // return placeholder data (tier_list_editor may not have backend /admin/stats access)
            // Note: Backend returns 400 for permission errors (BadRequest), so we handle that too
            if (statsResponse.status === 404 || statsResponse.status === 403 || statsResponse.status === 400) {
                // For tier_list_editor, fetch tier lists separately so they can still manage them
                let tierListsData: AdminStats["tierLists"] = {
                    total: 0,
                    active: 0,
                    totalVersions: 0,
                    totalPlacements: 0,
                    tierLists: [],
                };

                try {
                    const tierListsResponse = await backendFetch("/tier-lists", {
                        method: "GET",
                    });
                    if (tierListsResponse.ok) {
                        const data = await tierListsResponse.json();
                        const tierLists = data.tier_lists || [];
                        tierListsData = {
                            total: tierLists.length,
                            active: tierLists.filter((t: { is_active: boolean }) => t.is_active).length,
                            totalVersions: 0,
                            totalPlacements: 0,
                            tierLists: tierLists.map((t: { id: string; name: string; slug: string; is_active: boolean; created_at: string; updated_at: string }) => ({
                                id: t.id,
                                name: t.name,
                                slug: t.slug,
                                isActive: t.is_active,
                                tierCount: 0,
                                operatorCount: 0,
                                versionCount: 0,
                                createdAt: t.created_at,
                                updatedAt: t.updated_at,
                            })),
                        };
                    }
                } catch {
                    // If fetching tier lists fails, continue with empty list
                }

                return res.status(200).json({
                    success: true,
                    data: {
                        users: null,
                        tierLists: tierListsData,
                        recentActivity: [],
                    },
                });
            }

            return res.status(statsResponse.status).json({
                success: false,
                error: "Failed to fetch admin stats",
            });
        }

        const statsData: AdminStats = await statsResponse.json();

        // Filter out user data for non-super_admin roles
        if (userRole !== "super_admin") {
            return res.status(200).json({
                success: true,
                data: {
                    ...statsData,
                    users: null,
                } as unknown as AdminStats,
            });
        }

        return res.status(200).json({
            success: true,
            data: statsData,
        });
    } catch (error) {
        console.error("Admin stats handler error:", error);
        return res.status(500).json({
            success: false,
            error: "An internal server error occurred",
        });
    }
}
