import type { NextApiRequest, NextApiResponse } from "next";
import { env } from "~/env";
import type { SearchResponse } from "~/types/api";

export default async function handler(req: NextApiRequest, res: NextApiResponse) {
    if (req.method !== "GET") {
        return res.status(405).json({ error: "Method not allowed" });
    }

    const { nickname, uid, resume, server, grade, secretary, level, totalScore, compositeScore, operatorScore, stageScore, roguelikeScore, sandboxScore, medalScore, baseScore, logic, sortBy, order, limit, offset, fields } = req.query;

    const backendUrl = new URL("/search", env.BACKEND_URL);

    // Text search params
    if (nickname && typeof nickname === "string") {
        backendUrl.searchParams.set("nickname", nickname);
    }
    if (uid && typeof uid === "string") {
        backendUrl.searchParams.set("uid", uid);
    }
    if (resume && typeof resume === "string") {
        backendUrl.searchParams.set("resume", resume);
    }

    // Exact match filters
    if (server && typeof server === "string") {
        backendUrl.searchParams.set("server", server);
    }
    if (grade && typeof grade === "string") {
        backendUrl.searchParams.set("grade", grade);
    }
    if (secretary && typeof secretary === "string") {
        backendUrl.searchParams.set("secretary", secretary);
    }

    // Range query params
    if (level && typeof level === "string") {
        backendUrl.searchParams.set("level", level);
    }
    if (totalScore && typeof totalScore === "string") {
        backendUrl.searchParams.set("totalScore", totalScore);
    }
    if (compositeScore && typeof compositeScore === "string") {
        backendUrl.searchParams.set("compositeScore", compositeScore);
    }
    if (operatorScore && typeof operatorScore === "string") {
        backendUrl.searchParams.set("operatorScore", operatorScore);
    }
    if (stageScore && typeof stageScore === "string") {
        backendUrl.searchParams.set("stageScore", stageScore);
    }
    if (roguelikeScore && typeof roguelikeScore === "string") {
        backendUrl.searchParams.set("roguelikeScore", roguelikeScore);
    }
    if (sandboxScore && typeof sandboxScore === "string") {
        backendUrl.searchParams.set("sandboxScore", sandboxScore);
    }
    if (medalScore && typeof medalScore === "string") {
        backendUrl.searchParams.set("medalScore", medalScore);
    }
    if (baseScore && typeof baseScore === "string") {
        backendUrl.searchParams.set("baseScore", baseScore);
    }

    // Query options
    if (logic && typeof logic === "string") {
        backendUrl.searchParams.set("logic", logic);
    }
    if (sortBy && typeof sortBy === "string") {
        backendUrl.searchParams.set("sortBy", sortBy);
    }
    if (order && typeof order === "string") {
        backendUrl.searchParams.set("order", order);
    }
    if (limit && typeof limit === "string") {
        backendUrl.searchParams.set("limit", limit);
    }
    if (offset && typeof offset === "string") {
        backendUrl.searchParams.set("offset", offset);
    }
    if (fields && typeof fields === "string") {
        backendUrl.searchParams.set("fields", fields);
    }

    try {
        const response = await fetch(backendUrl.toString(), {
            method: "GET",
            headers: {
                "Content-Type": "application/json",
            },
        });

        if (!response.ok) {
            return res.status(response.status).json({ error: "Failed to fetch search results" });
        }

        const data = (await response.json()) as SearchResponse;
        return res.status(200).json(data);
    } catch (error) {
        console.error("Search API error:", error);
        return res.status(500).json({ error: "Internal server error" });
    }
}
