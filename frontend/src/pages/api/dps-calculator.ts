import type { NextApiRequest, NextApiResponse } from "next";
import { env } from "~/env";
import type { DpsCalculateRequest, DpsCalculateResponse, DpsListOperatorsResponse } from "~/types/api/impl/dps-calculator";

/**
 * DPS Calculator API Route
 *
 * Proxies requests to the backend DPS calculator endpoints.
 *
 * Methods:
 * - POST: Calculate DPS for an operator
 * - GET: List all operators with DPS calculators
 */

// Determine if we're in development mode
const isDevelopment = env.NODE_ENV === "development";

/**
 * Fetch data from the backend
 */
async function fetchFromBackend<T>(endpoint: string, options?: RequestInit): Promise<T> {
    const url = `${env.BACKEND_URL}${endpoint}`;

    if (isDevelopment) {
        console.log(`[DPS Calculator] Fetching ${options?.method ?? "GET"} ${url}`);
    }

    const response = await fetch(url, {
        ...options,
        headers: {
            "Content-Type": "application/json",
            "Accept-Encoding": "gzip, deflate",
            ...options?.headers,
        },
    });

    if (!response.ok) {
        const errorBody = await response.text();
        console.error(`[DPS Calculator] Backend error ${response.status}: ${errorBody}`);
        throw new Error(`Backend request failed with status ${response.status}`);
    }

    return response.json() as Promise<T>;
}

export default async function handler(req: NextApiRequest, res: NextApiResponse) {
    try {
        switch (req.method) {
            case "GET": {
                // GET /api/dps-calculator - List all operators with DPS calculators
                const operators = await fetchFromBackend<DpsListOperatorsResponse>("/dps-calculator/operators");
                return res.status(200).json(operators);
            }

            case "POST": {
                // POST /api/dps-calculator - Calculate DPS for an operator
                const body = req.body as DpsCalculateRequest;

                if (!body.operatorId) {
                    return res.status(400).json({ error: "Missing required field: operatorId" });
                }

                const result = await fetchFromBackend<DpsCalculateResponse>("/dps-calculator", {
                    method: "POST",
                    body: JSON.stringify(body),
                });

                return res.status(200).json(result);
            }

            default: {
                res.setHeader("Allow", ["GET", "POST"]);
                return res.status(405).json({ error: `Method ${req.method} Not Allowed` });
            }
        }
    } catch (error: unknown) {
        console.error("[DPS Calculator] API error:", error);

        const errorMessage = error instanceof Error ? (isDevelopment ? error.message : "An error occurred while processing the request.") : "An unexpected error occurred.";

        return res.status(500).json({ error: errorMessage });
    }
}
