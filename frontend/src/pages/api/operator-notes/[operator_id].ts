import type { NextApiRequest, NextApiResponse } from "next";
import { backendFetch } from "~/lib/backend-fetch";
import type { OperatorNote } from "~/types/api/impl/operator-notes";

/**
 * GET /api/operator-notes/{operator_id}
 * Returns operator notes for the given operator
 */
export default async function handler(req: NextApiRequest, res: NextApiResponse) {
    if (req.method !== "GET") {
        return res.status(405).json({ error: "Method not allowed" });
    }

    const { operator_id } = req.query;

    if (!operator_id || typeof operator_id !== "string") {
        return res.status(400).json({ error: "Operator ID is required" });
    }

    try {
        const response = await backendFetch(`/operator-notes/${operator_id}`);

        if (!response.ok) {
            if (response.status === 404) {
                return res.status(404).json({ error: "Operator notes not found" });
            }
            return res.status(response.status).json({ error: "Failed to fetch operator notes" });
        }

        const data: OperatorNote = await response.json();
        return res.status(200).json(data);
    } catch (error) {
        console.error("Operator notes API error:", error);
        return res.status(500).json({ error: "Internal server error" });
    }
}
