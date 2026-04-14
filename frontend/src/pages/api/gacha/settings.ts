import type { NextApiRequest, NextApiResponse } from "next";
import { getToken } from "~/lib/auth";
import { backendFetch } from "~/lib/backend-fetch";

export default async function handler(req: NextApiRequest, res: NextApiResponse) {
    const token = getToken(req);

    if (!token) {
        return res.status(401).json({ error: "Not authenticated" });
    }

    // GET - Fetch current settings from user profile
    if (req.method === "GET") {
        try {
            const response = await backendFetch("/auth/me", {
                headers: {
                    Authorization: `Bearer ${token}`,
                },
            });

            if (!response.ok) {
                return res.status(response.status).json({ error: "Failed to fetch settings" });
            }

            const data = await response.json();
            return res.status(200).json(data);
        } catch (error) {
            console.error("Error fetching settings:", error);
            return res.status(500).json({ error: "An internal server error occurred" });
        }
    }

    // POST - Update settings
    if (req.method === "POST") {
        try {
            const { public_profile, store_gacha, share_stats } = req.body;

            const response = await backendFetch("/auth/update-settings", {
                method: "POST",
                headers: {
                    Authorization: `Bearer ${token}`,
                },
                body: JSON.stringify({
                    public_profile,
                    store_gacha,
                    share_stats,
                }),
            });

            if (!response.ok) {
                return res.status(response.status).json({ error: "Failed to update settings" });
            }

            const data = await response.json();
            return res.status(200).json(data);
        } catch (error) {
            console.error("Error updating settings:", error);
            return res.status(500).json({ error: "An internal server error occurred" });
        }
    }

    return res.status(405).json({ error: "Method not allowed" });
}
