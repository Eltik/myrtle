import type { NextApiRequest, NextApiResponse } from "next";
import { env } from "~/env";
import type { LeaderboardResponse } from "~/types/api";

export default async function handler(req: NextApiRequest, res: NextApiResponse) {
    if (req.method !== "GET") {
        return res.status(405).json({ error: "Method not allowed" });
    }

    const { sort_by, order, server, limit, offset } = req.query;

    const backendURL = new URL("/leaderboard", env.BACKEND_URL);

    if (sort_by && typeof sort_by === "string") {
        backendURL.searchParams.set("sort_by", sort_by);
    }
    if (order && typeof order === "string") {
        backendURL.searchParams.set("order", order);
    }
    if (server && typeof server === "string") {
        backendURL.searchParams.set("server", server);
    }
    if (limit && typeof limit === "string") {
        backendURL.searchParams.set("limit", limit);
    }
    if (offset && typeof offset === "string") {
        backendURL.searchParams.set("offset", offset);
    }

    try {
        const response = await fetch(backendURL.toString(), {
            method: "GET",
            headers: {
                "Content-Type": "application/json",
            },
        });

        if (!response.ok) {
            return res.status(response.status).json({ error: "Failed to fetch leaderboard" });
        }

        const data = (await response.json()) as LeaderboardResponse;
        return res.status(200).json(data);
    } catch (error) {
        console.error("Leaderboard API error:", error);
        return res.status(500).json({ error: "Internal server error" });
    }
}
