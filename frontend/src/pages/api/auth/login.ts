import { serialize } from "cookie";
import type { NextApiRequest, NextApiResponse } from "next";
import { z } from "zod";
import { env } from "~/env";
import type { User } from "~/types/api";

// Valid Arknights server regions
const AKServerSchema = z.enum(["en", "jp", "kr", "cn", "bili", "tw"]);

// Login request validation schema
const LoginSchema = z.object({
    email: z.email("Invalid email format").min(1, "Email is required").max(254, "Email too long"),
    code: z
        .union([z.string(), z.number()])
        .transform((val) => {
            const num = typeof val === "string" ? Number.parseInt(val, 10) : val;
            if (Number.isNaN(num)) throw new Error("Code must be a valid number");
            return num;
        })
        .refine((val) => val >= 0 && val <= 999999, "Code must be a 6-digit number"),
    server: AKServerSchema.default("en"),
});

type LoginInput = z.infer<typeof LoginSchema>;

interface BackendRefreshResponse {
    user: User;
    site_token: string;
}

// Backend response type
interface BackendLoginResponse {
    uid: string;
    secret: string;
    seqnum: number;
}

// API response types
interface SuccessResponse {
    success: true;
    user: User;
}

interface ErrorResponse {
    success: false;
    error: string;
    details?: z.core.$ZodIssue[];
}

type ApiResponse = SuccessResponse | ErrorResponse;

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
        // Parse and validate input
        const parseResult = LoginSchema.safeParse(req.body);

        if (!parseResult.success) {
            return res.status(400).json({
                success: false,
                error: "Validation failed",
                details: parseResult.error.issues,
            });
        }

        const { email, code, server }: LoginInput = parseResult.data;

        // Step 1: Authenticate with backend
        const loginUrl = new URL("/login", env.BACKEND_URL);
        loginUrl.searchParams.set("email", email);
        loginUrl.searchParams.set("code", code.toString());
        loginUrl.searchParams.set("server", server);

        const loginResponse = await fetch(loginUrl.toString(), {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
        });

        if (!loginResponse.ok) {
            const errorText = await loginResponse.text();
            console.error(`Backend login failed: ${loginResponse.status} - ${errorText}`);

            const statusCode = loginResponse.status === 401 ? 401 : 400;
            return res.status(statusCode).json({
                success: false,
                error: "Invalid login credentials",
            });
        }

        const backendData: BackendLoginResponse = await loginResponse.json();

        if (!backendData.uid || !backendData.secret || typeof backendData.seqnum !== "number") {
            console.error("Invalid backend response structure:", backendData);
            return res.status(500).json({
                success: false,
                error: "Authentication service error",
            });
        }

        // Step 2: Refresh to fetch user data and store in database
        const refreshUrl = new URL("/refresh", env.BACKEND_URL);
        refreshUrl.searchParams.set("uid", backendData.uid);
        refreshUrl.searchParams.set("secret", backendData.secret);
        refreshUrl.searchParams.set("seqnum", backendData.seqnum.toString());
        refreshUrl.searchParams.set("server", server);

        const refreshResponse = await fetch(refreshUrl.toString(), {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
        });

        if (!refreshResponse.ok) {
            const errorText = await refreshResponse.text();
            console.error(`Backend refresh failed: ${refreshResponse.status} - ${errorText}`);

            return res.status(500).json({
                success: false,
                error: "Failed to fetch user data",
            });
        }

        const refreshData: BackendRefreshResponse = await refreshResponse.json();
        const user = refreshData.user;

        // Step 3: Set session cookies (seqnum incremented after refresh)
        const sessionData = JSON.stringify({
            uid: backendData.uid,
            secret: backendData.secret,
            seqnum: backendData.seqnum + 1,
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
            serialize("site_token", refreshData.site_token, {
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

        return res.status(200).json({ success: true, user });
    } catch (error) {
        console.error("Login handler error:", error);
        return res.status(500).json({
            success: false,
            error: "An internal server error occurred",
        });
    }
}
