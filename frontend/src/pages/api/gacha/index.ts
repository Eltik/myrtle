import type { NextApiRequest, NextApiResponse } from "next";
import { getToken } from "~/lib/auth";
import { backendFetch } from "~/lib/backend-fetch";

export default async function handler(req: NextApiRequest, res: NextApiResponse) {
    if (req.method !== "POST") {
        return res.status(405).json({ error: "Method not allowed" });
    }

    try {
        const token = getToken(req);

        if (!token) {
            return res.status(401).json({ error: "Not authenticated" });
        }

        const response = await backendFetch("/gacha/fetch", {
            method: "POST",
            headers: {
                Authorization: `Bearer ${token}`,
            },
        });

        if (!response.ok) {
            const errorText = await response.text();
            console.error(`Backend gacha fetch failed: ${response.status} - ${errorText}`);
            return res.status(response.status).json({ error: "Failed to fetch gacha records" });
        }

        const data = await response.json();
        return res.status(200).json(data);
    } catch (error) {
        console.error("Error fetching gacha records:", error);
        return res.status(500).json({ error: "An internal server error occurred" });
    }
}
