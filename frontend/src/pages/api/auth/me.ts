import { parse, serialize } from "cookie";
import type { NextApiRequest, NextApiResponse } from "next";
import { z } from "zod";
import { env } from "~/env";
import type { User } from "~/types/api";

// Valid Arknights server regions
const AKServerSchema = z.enum(["en", "jp", "kr", "cn", "bili", "tw"]);

// Session cookie validation schema
const SessionSchema = z.object({
    uid: z.string().min(1, "UID is required"),
    secret: z.string().min(1, "Secret is required"),
    seqnum: z.number().int().min(0),
    server: AKServerSchema,
});

type SessionData = z.infer<typeof SessionSchema>;

// API response types
interface SuccessResponse {
    success: true;
    user: User;
}

interface ErrorResponse {
    success: false;
    error: string;
}

type ApiResponse = SuccessResponse | ErrorResponse;

/**
 * Parse and validate session from cookie
 */
function getSessionFromCookie(req: NextApiRequest): SessionData | null {
    const cookies = parse(req.headers.cookie ?? "");
    const sessionCookie = cookies.auth_session;

    if (!sessionCookie) {
        return null;
    }

    try {
        const sessionData = JSON.parse(sessionCookie);
        const parseResult = SessionSchema.safeParse(sessionData);
        return parseResult.success ? parseResult.data : null;
    } catch {
        return null;
    }
}

/**
 * Clear auth cookies helper
 */
function clearAuthCookies(res: NextApiResponse) {
    res.setHeader("Set-Cookie", [
        serialize("auth_session", "", {
            httpOnly: true,
            secure: env.NODE_ENV === "production",
            sameSite: "strict",
            path: "/",
            maxAge: 0,
        }),
        serialize("auth_indicator", "", {
            httpOnly: false,
            secure: env.NODE_ENV === "production",
            sameSite: "strict",
            path: "/",
            maxAge: 0,
        }),
    ]);
}

export default async function handler(req: NextApiRequest, res: NextApiResponse<ApiResponse>) {
    // Only allow POST
    if (req.method !== "POST") {
        res.setHeader("Allow", ["POST"]);
        return res.status(405).json({
            success: false,
            error: `Method ${req.method} not allowed`,
        });
    }

    try {
        // Get session from cookie
        const session = getSessionFromCookie(req);

        if (!session) {
            return res.status(401).json({
                success: false,
                error: "Not authenticated",
            });
        }

        const { uid } = session;

        // Build backend URL to get user data (no refresh)
        const backendUrl = new URL("/get-user", env.BACKEND_URL);
        backendUrl.searchParams.set("uid", uid);

        // Call backend to get user data
        const userResponse = await fetch(backendUrl.toString(), {
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
