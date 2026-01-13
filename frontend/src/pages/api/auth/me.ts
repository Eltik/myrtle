import type { NextApiRequest, NextApiResponse } from "next";
import { env } from "~/env";
import { clearAuthCookies, getSessionFromCookie } from "~/lib/auth";
import type { User } from "~/types/api";

interface SuccessResponse {
    success: true;
    user: User;
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
        const session = getSessionFromCookie(req);

        if (!session) {
            return res.status(401).json({
                success: false,
                error: "Not authenticated",
            });
        }

        const { uid } = session;

        // Build backend URL to get user data (no refresh)
        const backendURL = new URL("/get-user", env.BACKEND_URL);
        backendURL.searchParams.set("uid", uid);

        const userResponse = await fetch(backendURL.toString(), {
            method: "GET",
            headers: {
                "Content-Type": "application/json",
            },
        });

        if (!userResponse.ok) {
            const errorText = await userResponse.text();
            console.error(`Backend get-user failed: ${userResponse.status} - ${errorText}`);

            // If not found or unauthorized, clear the session cookies
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

        // Backend returns database User model: { id, uid, server, data, created_at }
        // The actual user data is nested in the 'data' field
        const dbUser = await userResponse.json();
        const user: User = dbUser.data;

        if (!user || !user.status) {
            clearAuthCookies(res);
            return res.status(401).json({
                success: false,
                error: "Invalid user data",
            });
        }

        return res.status(200).json({
            success: true,
            user,
        });
    } catch (error) {
        console.error("Me handler error:", error);
        return res.status(500).json({
            success: false,
            error: "An internal server error occurred",
        });
    }
}
