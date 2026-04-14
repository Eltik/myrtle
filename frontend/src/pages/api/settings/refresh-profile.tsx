import type { NextApiRequest, NextApiResponse } from "next";
import { getToken } from "~/lib/auth";
import { backendFetch } from "~/lib/backend-fetch";
import type { UserProfile } from "~/types/api/impl/user";

interface RefreshProfileResponse {
    success: boolean;
    message?: string;
    user?: UserProfile;
}

export default async function handler(req: NextApiRequest, res: NextApiResponse<RefreshProfileResponse>) {
    if (req.method !== "POST") {
        return res.status(405).json({ success: false, message: "Method not allowed" });
    }

    try {
        const token = getToken(req);

        if (!token) {
            return res.status(401).json({
                success: false,
                message: "Not authenticated",
            });
        }

        // Call backend /refresh with bearer token to fetch fresh data from game servers
        const refreshResponse = await backendFetch("/refresh", {
            method: "POST",
            headers: {
                Authorization: `Bearer ${token}`,
            },
        });

        if (!refreshResponse.ok) {
            const errorText = await refreshResponse.text();
            console.error(`Backend refresh failed: ${refreshResponse.status} - ${errorText}`);

            return res.status(500).json({
                success: false,
                message: "Failed to refresh profile from game servers",
            });
        }

        const user: UserProfile = await refreshResponse.json();

        return res.status(200).json({
            success: true,
            message: "Profile refreshed successfully",
            user,
        });
    } catch (error) {
        console.error("Error refreshing profile:", error);
        return res.status(500).json({ success: false, message: "Failed to refresh profile" });
    }
}
