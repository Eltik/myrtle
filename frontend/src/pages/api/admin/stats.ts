import type { NextApiRequest, NextApiResponse } from "next";
import { getToken } from "~/lib/auth";
import { backendFetch } from "~/lib/backend-fetch";

export default async function handler(req: NextApiRequest, res: NextApiResponse) {
    if (req.method !== "GET") {
        res.setHeader("Allow", ["GET"]);
        return res.status(405).json({ error: `Method ${req.method} not allowed` });
    }

    try {
        const token = getToken(req);

        if (!token) {
            return res.status(401).json({ error: "Not authenticated" });
        }

        const response = await backendFetch("/admin/stats", {
            method: "GET",
            headers: {
                Authorization: `Bearer ${token}`,
            },
        });

        if (!response.ok) {
            return res.status(response.status).json({ error: "Failed to fetch admin stats" });
        }

        const data = await response.json();
        return res.status(200).json(data);
    } catch (error) {
        console.error("Admin stats handler error:", error);
        return res.status(500).json({ error: "An internal server error occurred" });
    }
}
