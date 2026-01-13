import { parse } from "cookie";
import type { NextApiRequest, NextApiResponse } from "next";
import { backendFetch } from "~/lib/backend-fetch";

interface VerifyResponse {
    valid: boolean;
    user_id?: string;
    uid?: string;
    role?: string;
}

interface ApiResponse {
    success: boolean;
    data?: VerifyResponse;
    error?: string;
}

export default async function handler(req: NextApiRequest, res: NextApiResponse<ApiResponse>) {
    if (req.method !== "POST") {
        res.setHeader("Allow", ["POST"]);
        return res.status(405).json({
            success: false,
            error: `Method ${req.method} not allowed`,
        });
    }

    try {
        const cookies = parse(req.headers.cookie ?? "");
        const siteToken = cookies.site_token;

        if (!siteToken) {
            return res.status(401).json({
                success: false,
                error: "No token found",
            });
        }

        // Call backend to verify token
        const verifyResponse = await backendFetch("/auth/verify", {
            method: "POST",
            body: JSON.stringify({ token: siteToken }),
        });

        if (!verifyResponse.ok) {
            return res.status(401).json({
                success: false,
                error: "Token verification failed",
            });
        }

        const data: VerifyResponse = await verifyResponse.json();

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
