import type { NextApiRequest, NextApiResponse } from "next";
import { backendFetch } from "~/lib/backend-fetch";

export default async function handler(req: NextApiRequest, res: NextApiResponse) {
    try {
        switch (req.method) {
            case "GET": {
                const response = await backendFetch("/dps/operators");

                if (!response.ok) {
                    return res.status(response.status).json({ error: "Failed to fetch DPS operators" });
                }

                const data = await response.json();
                return res.status(200).json(data);
            }

            case "POST": {
                const response = await backendFetch("/dps/calculate", {
                    method: "POST",
                    body: JSON.stringify(req.body),
                });

                if (!response.ok) {
                    return res.status(response.status).json({ error: "Failed to calculate DPS" });
                }

                const data = await response.json();
                return res.status(200).json(data);
            }

            default: {
                res.setHeader("Allow", ["GET", "POST"]);
                return res.status(405).json({ error: `Method ${req.method} Not Allowed` });
            }
        }
    } catch (error) {
        console.error("[DPS Calculator] API error:", error);
        return res.status(500).json({ error: "An error occurred while processing the request." });
    }
}
