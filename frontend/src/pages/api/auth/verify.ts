import type { NextApiRequest, NextApiResponse } from "next";
import { getToken } from "~/lib/auth";
import { backendFetch } from "~/lib/backend-fetch";

interface VerifyData {
    valid: boolean;
    userId?: string;
    uid?: string;
    server?: string;
    role?: string;
}

interface SuccessResponse {
    success: true;
    data: VerifyData;
}

interface ErrorResponse {
    success: false;
    error: string;
}

type ApiResponse = SuccessResponse | ErrorResponse;

export default async function handler(req: NextApiRequest, res: NextApiResponse<ApiResponse>) {
    if (req.method !== "POST") {
        res.setHeader("Allow", ["POST"]);
        return res.status(405).json({
            success: false,
            error: `Method ${req.method} not allowed`,
        });
    }

    try {
        const token = getToken(req);

        if (!token) {
            return res.status(401).json({
                success: false,
                error: "No token found",
            });
        }

        const verifyResponse = await backendFetch("/auth/verify", {
            bearerToken: token,
        });

        if (!verifyResponse.ok) {
            return res.status(401).json({
                success: false,
                error: "Token verification failed",
            });
        }

        const data: VerifyData = await verifyResponse.json();

        if (!data.valid) {
            return res.status(401).json({
                success: false,
                error: "Invalid token",
            });
        }

        return res.status(200).json({
            success: true,
            data,
        });
    } catch (error) {
        console.error("Verify handler error:", error);
        return res.status(500).json({
            success: false,
            error: "An internal server error occurred",
        });
    }
}
