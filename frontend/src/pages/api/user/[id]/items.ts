import type { NextApiRequest, NextApiResponse } from "next";

/**
 * GET /api/user/{userId}/items
 * Items endpoint is not available in v3 backend.
 * Returns a stub response indicating unavailability.
 */
export default async function handler(req: NextApiRequest, res: NextApiResponse) {
    if (req.method !== "GET") {
        return res.status(405).json({ error: "Method not allowed" });
    }

    const { id } = req.query;

    if (!id || typeof id !== "string") {
        return res.status(400).json({ error: "User ID is required" });
    }

    return res.status(200).json({ inventory: null, unavailable: true });
}
