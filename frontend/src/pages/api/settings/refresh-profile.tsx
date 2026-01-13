import type { NextApiRequest, NextApiResponse } from "next";
import { env } from "~/env";
import { getSessionFromCookie, setAuthCookies } from "~/lib/auth";
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
        const refreshURL = new URL("/refresh", env.BACKEND_URL);
        refreshURL.searchParams.set("uid", uid);
        refreshURL.searchParams.set("secret", secret);
        refreshURL.searchParams.set("seqnum", seqnum.toString());
        refreshURL.searchParams.set("server", server);

        const refreshResponse = await fetch(refreshURL.toString(), {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
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
