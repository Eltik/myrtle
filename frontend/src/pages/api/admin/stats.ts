import { parse } from "cookie";
import type { NextApiRequest, NextApiResponse } from "next";
import { env } from "~/env";
import type { AdminStats } from "~/types/frontend/admin";

const ADMIN_ROLES = ["super_admin", "tier_list_admin"];

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
        const verifyUrl = new URL("/auth/verify", env.BACKEND_URL);
        const verifyResponse = await fetch(verifyUrl.toString(), {
            method: "POST",
            headers: { "Content-Type": "application/json" },
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
        if (!verifyData.role || !ADMIN_ROLES.includes(verifyData.role)) {
            return res.status(403).json({
                success: false,
                error: "Insufficient permissions",
            });
        }

        // Step 4: Fetch admin stats from backend (requires backend /admin/stats endpoint)
        const statsUrl = new URL("/admin/stats", env.BACKEND_URL);
        const statsResponse = await fetch(statsUrl.toString(), {
            method: "GET",
            headers: {
                "Content-Type": "application/json",
                Authorization: `Bearer ${siteToken}`,
            },
        });

        if (!statsResponse.ok) {
            // If backend endpoint doesn't exist yet, return empty placeholder data
            if (statsResponse.status === 404) {
                return res.status(200).json({
                    success: true,
                    data: {
                        users: {
                            total: 0,
                            byRole: {
                                user: 0,
                                tier_list_editor: 0,
                                tier_list_admin: 0,
                                super_admin: 0,
                            },
                            byServer: {},
                            recentUsers: [],
                        },
                        tierLists: {
                            total: 0,
                            active: 0,
                            totalVersions: 0,
                            totalPlacements: 0,
                            tierLists: [],
                        },
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
