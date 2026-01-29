import type { NextApiRequest, NextApiResponse } from "next";
import { backendFetch } from "~/lib/backend-fetch";
import type { StoredUser } from "~/types/api/impl/user";

/**
 * GET /api/user/{userId}/characters
 * Returns character data for the Characters tab
 * Used for client-side data fetching to reduce SSR payload
 */
export default async function handler(req: NextApiRequest, res: NextApiResponse) {
    if (req.method !== "GET") {
        return res.status(405).json({ error: "Method not allowed" });
    }

    const { id } = req.query;

    if (!id || typeof id !== "string") {
        return res.status(400).json({ error: "User ID is required" });
    }

    try {
        const response = await backendFetch(`/get-user?uid=${id}`);

        if (!response.ok) {
            if (response.status === 404) {
                return res.status(404).json({ error: "User not found" });
            }
            return res.status(response.status).json({ error: "Failed to fetch user data" });
        }

        const userData: StoredUser = await response.json();

        if (!userData?.data?.troop?.chars) {
            return res.status(404).json({ error: "Character data not found" });
        }

        // Return only the character data needed for the Characters tab
        return res.status(200).json({
            characters: userData.data.troop.chars,
        });
    } catch (error) {
        console.error("User characters API error:", error);
        return res.status(500).json({ error: "Internal server error" });
    }
}
