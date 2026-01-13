import type { NextApiRequest, NextApiResponse } from "next";
import { backendFetch } from "~/lib/backend-fetch";
import { buildSearchParamsFromQuery } from "~/lib/search-utils";
import type { SearchResponse } from "~/types/api";

export default async function handler(req: NextApiRequest, res: NextApiResponse) {
    if (req.method !== "GET") {
        return res.status(405).json({ error: "Method not allowed" });
    }

    const searchParams = buildSearchParamsFromQuery(req.query);

    try {
        const response = await backendFetch(`/search?${searchParams.toString()}`);

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
