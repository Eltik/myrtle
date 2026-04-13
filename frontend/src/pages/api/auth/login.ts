import type { NextApiRequest, NextApiResponse } from "next";
import { z } from "zod";
import { AKServerSchema, setAuthCookies } from "~/lib/auth";
import { backendFetch } from "~/lib/backend-fetch";
import type { UserProfile } from "~/types/api/impl/user";

const LoginSchema = z.object({
    email: z.string().min(1, "Email is required").max(254, "Email too long").email("Invalid email format"),
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

interface LoginResponse {
    token: string;
    uid: string;
    server: string;
}

interface SuccessResponse {
    success: true;
    user: UserProfile;
}

interface ErrorResponse {
    success: false;
    error: string;
    details?: z.ZodIssue[];
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
        const loginResponse = await backendFetch("/login", {
            method: "POST",
            body: JSON.stringify({ email, code, server }),
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

        const loginData: LoginResponse = await loginResponse.json();

        if (!loginData.token || !loginData.uid) {
            console.error("Invalid backend response structure:", loginData);
            return res.status(500).json({
                success: false,
                error: "Authentication service error",
            });
        }

        // Step 2: Set auth cookies with JWT token
        setAuthCookies(res, loginData.token);

        // Step 3: Fire and forget refresh to trigger game data sync
        backendFetch("/refresh", {
            method: "POST",
            bearerToken: loginData.token,
        }).catch((err) => {
            console.error("Background refresh failed:", err);
        });

        // Step 4: Fetch user profile
        const userResponse = await backendFetch(`/get-user?uid=${encodeURIComponent(loginData.uid)}`);

        if (!userResponse.ok) {
            const errorText = await userResponse.text();
            console.error(`Backend get-user failed: ${userResponse.status} - ${errorText}`);

            return res.status(500).json({
                success: false,
                error: "Failed to fetch user data",
            });
        }

        const user: UserProfile = await userResponse.json();

        return res.status(200).json({ success: true, user });
    } catch (error) {
        console.error("Login handler error:", error);
        return res.status(500).json({
            success: false,
            error: "An internal server error occurred",
        });
    }
}
