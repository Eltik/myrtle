import type { NextApiRequest, NextApiResponse } from "next";
import { getSessionFromCookie, setAuthCookies } from "~/lib/auth";
import { backendFetch } from "~/lib/backend-fetch";
import type { User } from "~/types/api";

interface BackendRefreshResponse {
    user: User;
    site_token: string;
}

interface RefreshProfileResponse {
    success: boolean;
    message?: string;
    user?: User;
}

export default async function handler(req: NextApiRequest, res: NextApiResponse<RefreshProfileResponse>) {
    if (req.method !== "POST") {
        return res.status(405).json({ success: false, message: "Method not allowed" });
    }

    try {
        const session = getSessionFromCookie(req);

        if (!session) {
            return res.status(401).json({
                success: false,
                message: "Not authenticated",
            });
        }

        const { uid, secret, seqnum, server } = session;

        // Call backend /refresh to fetch fresh data from game servers
        const params = new URLSearchParams();
        params.set("uid", uid);
        params.set("secret", secret);
        params.set("seqnum", seqnum.toString());
        params.set("server", server);

        const refreshResponse = await backendFetch(`/refresh?${params.toString()}`, {
            method: "POST",
        });

        if (!refreshResponse.ok) {
            const errorText = await refreshResponse.text();
            console.error(`Backend refresh failed: ${refreshResponse.status} - ${errorText}`);

            return res.status(500).json({
                success: false,
                message: "Failed to refresh profile from game servers",
            });
        }

        const refreshData: BackendRefreshResponse = await refreshResponse.json();

        // Update session cookies with incremented seqnum
        setAuthCookies(res, { uid, secret, seqnum: seqnum + 1, server }, refreshData.site_token);

        return res.status(200).json({
            success: true,
            message: "Profile refreshed successfully",
            user: refreshData.user,
        });
    } catch (error) {
        console.error("Error refreshing profile:", error);
        return res.status(500).json({ success: false, message: "Failed to refresh profile" });
    }
}
