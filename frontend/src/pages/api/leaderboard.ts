import type { NextApiRequest, NextApiResponse } from "next";
import { backendFetch } from "~/lib/backend-fetch";

export default async function handler(req: NextApiRequest, res: NextApiResponse) {
    if (req.method !== "GET") {
        return res.status(405).json({ error: "Method not allowed" });
    }

    const { sort, server, limit, offset } = req.query;

    const params = new URLSearchParams();
    if (sort && typeof sort === "string") params.set("sort", sort);
    if (server && typeof server === "string") params.set("server", server);
    if (limit && typeof limit === "string") params.set("limit", limit);
    if (offset && typeof offset === "string") params.set("offset", offset);

    try {
        const response = await backendFetch(`/leaderboard?${params.toString()}`);

        if (!response.ok) {
            return res.status(response.status).json({ error: "Failed to fetch leaderboard" });
        }

        const data = await response.json();
        return res.status(200).json(data);
    } catch (error) {
        console.error("Leaderboard API error:", error);
        return res.status(500).json({ error: "Internal server error" });
    }
}
