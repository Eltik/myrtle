import type { NextApiRequest, NextApiResponse } from "next";
import { getToken } from "~/lib/auth";
import { backendFetch } from "~/lib/backend-fetch";

/**
 * PUT /api/operator-notes/{operator_id}/update
 * Updates operator notes (requires auth)
 * v3: calls PUT /operator-notes/{id} (no /update suffix)
 */
export default async function handler(req: NextApiRequest, res: NextApiResponse) {
    if (req.method !== "PUT") {
        return res.status(405).json({ error: "Method not allowed" });
    }

    const { operator_id } = req.query;

    if (!operator_id || typeof operator_id !== "string") {
        return res.status(400).json({ error: "Operator ID is required" });
    }

    const token = getToken(req);
    if (!token) {
        return res.status(401).json({ error: "Not authenticated" });
    }

    try {
        const response = await backendFetch(`/operator-notes/${operator_id}`, {
            method: "PUT",
            headers: {
                Authorization: `Bearer ${token}`,
            },
            body: JSON.stringify(req.body),
        });

        if (!response.ok) {
            const errorData = await response.json().catch(() => ({}));
            return res.status(response.status).json({
                error: (errorData as { error?: string }).error || "Failed to update operator notes",
            });
        }

        const data = await response.json();
        return res.status(200).json(data);
    } catch (error) {
        console.error("Operator notes update API error:", error);
        return res.status(500).json({ error: "Internal server error" });
    }
}
