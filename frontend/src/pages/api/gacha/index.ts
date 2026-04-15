import type { NextApiRequest, NextApiResponse } from "next";
import { getToken } from "~/lib/auth";
import { backendFetch } from "~/lib/backend-fetch";

/**
 * GET /api/gacha — trigger a live Yostar gacha fetch and return the newly-grouped records.
 *                  Returns {success, data: GachaRecords} on success, matching the old frontend shape.
 * POST /api/gacha — same, backwards-compatible raw fetch-result.
 */
export default async function handler(req: NextApiRequest, res: NextApiResponse) {
    if (req.method !== "GET" && req.method !== "POST") {
        return res.status(405).json({ success: false, error: "Method not allowed" });
    }

    try {
        const token = getToken(req);
        if (!token) {
            return res.status(401).json({ success: false, error: "Not authenticated" });
        }

        // Trigger a backend fetch. This pulls from Yostar and stores records.
        const fetchResponse = await backendFetch("/gacha/fetch", {
            method: "POST",
            headers: { Authorization: `Bearer ${token}` },
        });

        if (!fetchResponse.ok) {
            const errorText = await fetchResponse.text();
            console.error(`Backend gacha fetch failed: ${fetchResponse.status} - ${errorText}`);
            return res.status(fetchResponse.status).json({
                success: false,
                error: "Failed to fetch gacha records",
            });
        }

        if (req.method === "POST") {
            const data = await fetchResponse.json();
            return res.status(200).json(data);
        }

        // GET: after the sync, return the stored grouped records.
        const storedResponse = await backendFetch("/gacha/stored-records", {
            headers: { Authorization: `Bearer ${token}` },
        });

        if (!storedResponse.ok) {
            const errorText = await storedResponse.text();
            console.error(`Backend stored-records fetch failed: ${storedResponse.status} - ${errorText}`);
            return res.status(storedResponse.status).json({
                success: false,
                error: "Failed to fetch stored records",
            });
        }

        const data = await storedResponse.json();
        return res.status(200).json({ success: true, data });
    } catch (error) {
        console.error("Error fetching gacha records:", error);
        return res.status(500).json({ success: false, error: "An internal server error occurred" });
    }
}
