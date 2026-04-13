import type { NextApiRequest, NextApiResponse } from "next";
import { getToken } from "~/lib/auth";
import { backendFetch } from "~/lib/backend-fetch";

export default async function handler(req: NextApiRequest, res: NextApiResponse) {
    if (req.method !== "GET") {
        return res.status(405).json({ error: "Method not allowed" });
    }

    const token = getToken(req);

    if (!token) {
        return res.status(401).json({ error: "Not authenticated" });
    }

    try {
        const { rarity, limit, offset } = req.query;

        const params = new URLSearchParams();
        if (rarity && typeof rarity === "string") params.set("rarity", rarity);
        if (limit && typeof limit === "string") params.set("limit", limit);
        if (offset && typeof offset === "string") params.set("offset", offset);

        const response = await backendFetch(`/gacha/history?${params.toString()}`, {
            headers: {
                Authorization: `Bearer ${token}`,
            },
        });

        if (!response.ok) {
            const errorText = await response.text();
            console.error(`Backend gacha history fetch failed: ${response.status} - ${errorText}`);
            return res.status(response.status).json({ error: "Failed to fetch gacha history" });
        }

        const data = await response.json();
        return res.status(200).json(data);
    } catch (error) {
        console.error("Error fetching gacha history:", error);
        return res.status(500).json({ error: "An internal server error occurred" });
    }
}
