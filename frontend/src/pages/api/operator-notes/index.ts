import type { NextApiRequest, NextApiResponse } from "next";
import { backendFetch } from "~/lib/backend-fetch";
import type { OperatorNote } from "~/types/api/impl/operator-notes";

/**
 * GET /api/operator-notes
 * Returns all operator notes
 */
export default async function handler(req: NextApiRequest, res: NextApiResponse) {
    if (req.method !== "GET") {
        return res.status(405).json({ error: "Method not allowed" });
    }

    try {
        const response = await backendFetch("/operator-notes");

        if (!response.ok) {
            return res.status(response.status).json({ error: "Failed to fetch operator notes" });
        }

        const data: OperatorNote[] = await response.json();
        return res.status(200).json(data);
    } catch (error) {
        console.error("Operator notes list API error:", error);
        return res.status(500).json({ error: "Internal server error" });
    }
}
