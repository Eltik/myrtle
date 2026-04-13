import { parse } from "cookie";
import type { NextApiRequest, NextApiResponse } from "next";
import { backendFetch } from "~/lib/backend-fetch";
import type { AdminRole } from "~/lib/permissions";
import { canManageUsers } from "~/lib/permissions";

interface VerifyResponse {
    valid: boolean;
    user_id?: string;
    uid?: string;
    role?: string;
}

export default async function handler(req: NextApiRequest, res: NextApiResponse) {
    if (req.method !== "GET") {
        res.setHeader("Allow", ["GET"]);
        return res.status(405).json({ success: false, error: `Method ${req.method} not allowed` });
    }

    try {
        const cookies = parse(req.headers.cookie ?? "");
        const siteToken = cookies.site_token;

        if (!siteToken) {
            return res.status(401).json({ success: false, error: "Not authenticated" });
        }

        const verifyResponse = await backendFetch("/auth/verify", {
            method: "POST",
            body: JSON.stringify({ token: siteToken }),
        });

        if (!verifyResponse.ok) {
            return res.status(401).json({ success: false, error: "Token verification failed" });
        }

        const verifyData: VerifyResponse = await verifyResponse.json();

        if (!verifyData.valid || !canManageUsers((verifyData.role ?? null) as AdminRole | null)) {
            return res.status(403).json({ success: false, error: "Insufficient permissions" });
        }

        const { page = "1", limit = "10000" } = req.query;

        // Try dedicated /admin/users endpoint first
        const usersResponse = await backendFetch(`/admin/users?page=${page}&limit=${limit}`, {
            method: "GET",
            headers: { Authorization: `Bearer ${siteToken}` },
        });

        if (usersResponse.ok) {
            const data = await usersResponse.json();
            return res.status(200).json({ success: true, data });
        }

        // Fall back to /admin/stats which always works for super_admin
        const statsResponse = await backendFetch("/admin/stats", {
            method: "GET",
            headers: { Authorization: `Bearer ${siteToken}` },
        });

        if (!statsResponse.ok) {
            return res.status(statsResponse.status).json({ success: false, error: "Failed to fetch users" });
        }

        const statsData = await statsResponse.json();
        const users = statsData?.users?.recentUsers ?? [];

        return res.status(200).json({ success: true, data: { users, total: statsData?.users?.total ?? users.length } });
    } catch (error) {
        console.error("Admin users handler error:", error);
        return res.status(500).json({ success: false, error: "An internal server error occurred" });
    }
}
