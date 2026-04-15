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

        // Step 1: Trigger sync from game server. v3 /refresh returns the sync
        // result (not the user profile), so we need to follow up with /get-user.
        const refreshResponse = await backendFetch("/refresh", {
            method: "POST",
            bearerToken: token,
        });

        if (!refreshResponse.ok) {
            const errorText = await refreshResponse.text();
            console.error(`Backend refresh failed: ${refreshResponse.status} - ${errorText}`);
            return res.status(500).json({
                success: false,
                message: "Failed to refresh profile from game servers",
            });
        }

        // Step 2: Verify token to get uid, then fetch the freshly-synced profile.
        const verifyResponse = await backendFetch("/auth/verify", { bearerToken: token });
        if (!verifyResponse.ok) {
            return res.status(401).json({ success: false, message: "Invalid token" });
        }
        const verifyData = (await verifyResponse.json()) as { valid: boolean; uid?: string };
        if (!verifyData.valid || !verifyData.uid) {
            return res.status(401).json({ success: false, message: "Invalid token" });
        }

        const profileResponse = await backendFetch(`/get-user?uid=${encodeURIComponent(verifyData.uid)}`);
        if (!profileResponse.ok) {
            return res.status(500).json({ success: false, message: "Failed to fetch updated profile" });
        }

        const user: UserProfile = await profileResponse.json();

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
