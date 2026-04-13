import type { NextApiRequest, NextApiResponse } from "next";
import { clearAuthCookies, getToken } from "~/lib/auth";
import { backendFetch } from "~/lib/backend-fetch";
import type { UserProfile } from "~/types/api/impl/user";

interface ErrorResponse {
    success: false;
    error: string;
}

type ApiResponse = UserProfile | ErrorResponse;

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
                error: "Not authenticated",
            });
        }

        // Step 1: Verify the token to get uid
        const verifyResponse = await backendFetch("/auth/verify", {
            bearerToken: token,
        });

        if (!verifyResponse.ok) {
            clearAuthCookies(res);
            return res.status(401).json({
                success: false,
                error: "Invalid token",
            });
        }

        const verifyData: { valid: boolean; uid?: string } = await verifyResponse.json();

        if (!verifyData.valid || !verifyData.uid) {
            clearAuthCookies(res);
            return res.status(401).json({
                success: false,
                error: "Invalid token",
            });
        }

        // Step 2: Fetch user profile
        const userResponse = await backendFetch(`/get-user?uid=${encodeURIComponent(verifyData.uid)}`);

        if (!userResponse.ok) {
            const errorText = await userResponse.text();
            console.error(`Backend get-user failed: ${userResponse.status} - ${errorText}`);

            if (userResponse.status === 404 || userResponse.status === 401) {
                clearAuthCookies(res);
                return res.status(401).json({
                    success: false,
                    error: "User not found",
                });
            }

            return res.status(400).json({
                success: false,
                error: "Failed to get user data",
            });
        }

        const user: UserProfile = await userResponse.json();

        return res.status(200).json(user);
    } catch (error) {
        console.error("Me handler error:", error);
        return res.status(500).json({
            success: false,
            error: "An internal server error occurred",
        });
    }
}
