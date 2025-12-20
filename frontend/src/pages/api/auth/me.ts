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

        const { uid, secret, seqnum, server } = session;

        // Build backend URL with properly encoded parameters
        const backendUrl = new URL("/refresh", env.BACKEND_URL);
        backendUrl.searchParams.set("uid", uid);
        backendUrl.searchParams.set("secret", secret);
        backendUrl.searchParams.set("seqnum", seqnum.toString());
        backendUrl.searchParams.set("server", server);

        // Call backend refresh service
        const refreshResponse = await fetch(backendUrl.toString(), {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
        });

        if (!refreshResponse.ok) {
            const errorText = await refreshResponse.text();
            console.error(`Backend refresh failed: ${refreshResponse.status} - ${errorText}`);

            // If unauthorized, clear the session cookies
            if (refreshResponse.status === 401) {
                res.setHeader("Set-Cookie", [
                    serialize("auth_session", "", {
                        httpOnly: true,
                        secure: env.NODE_ENV === "production",
                        sameSite: "strict",
                        path: "/",
                        maxAge: 0, // Expire immediately
                    }),
                    serialize("auth_indicator", "", {
                        httpOnly: false,
                        secure: env.NODE_ENV === "production",
                        sameSite: "strict",
                        path: "/",
                        maxAge: 0, // Expire immediately
                    }),
                ]);
                return res.status(401).json({
                    success: false,
                    error: "Session expired",
                });
            }

            return res.status(400).json({
                success: false,
                error: "Failed to refresh session",
            });
        }

        const user: User = await refreshResponse.json();

        // Increment seqnum and update the session cookie
        const newSeqnum = seqnum + 1;
        const sessionData = JSON.stringify({
            uid,
            secret,
            seqnum: newSeqnum,
            server,
        });

        res.setHeader("Set-Cookie", [
            serialize("auth_session", sessionData, {
                httpOnly: true,
                secure: env.NODE_ENV === "production",
                sameSite: "strict",
                path: "/",
                maxAge: 60 * 60 * 24 * 7, // 1 week
            }),
            serialize("auth_indicator", "1", {
                httpOnly: false, // Client-side readable to check if session exists
                secure: env.NODE_ENV === "production",
                sameSite: "strict",
                path: "/",
                maxAge: 60 * 60 * 24 * 7, // 1 week
            }),
        ]);

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
