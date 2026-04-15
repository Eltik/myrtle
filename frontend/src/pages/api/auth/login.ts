import type { NextApiRequest, NextApiResponse } from "next";
import { z } from "zod";
import { AKServerSchema, setAuthCookies } from "~/lib/auth";
import { backendFetch } from "~/lib/backend-fetch";
import type { UserProfile } from "~/types/api/impl/user";

const LoginSchema = z.object({
    email: z.string().min(1, "Email is required").max(254, "Email too long").email("Invalid email format"),
    // v3 backend expects `code` as a 6-digit string. Accept string or number
    // from the client and always forward as a zero-padded string.
    code: z
        .union([z.string(), z.number()])
        .transform((val) => {
            const str = typeof val === "number" ? String(val) : val.trim();
            if (!/^\d{1,6}$/.test(str)) throw new Error("Code must be a 6-digit number");
            return str.padStart(6, "0");
        }),
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

        // Step 3: Trigger game data sync and WAIT for it to complete.
        // Without awaiting, step 4 would fetch a stale or empty profile.
        console.log(`[login] Triggering /refresh for uid=${loginData.uid}`);
        const refreshResponse = await backendFetch("/refresh", {
            method: "POST",
            bearerToken: loginData.token,
        });

        if (!refreshResponse.ok) {
            const errText = await refreshResponse.text().catch(() => "");
            console.error(`[login] /refresh FAILED ${refreshResponse.status}: ${errText}`);
            // Don't silently succeed — if sync fails, the profile will be empty/stale.
            return res.status(502).json({
                success: false,
                error: `Sync failed (${refreshResponse.status}). Try logging in again.`,
            });
        }

        const refreshBody = await refreshResponse.text().catch(() => "");
        console.log(`[login] /refresh OK for uid=${loginData.uid}, response length=${refreshBody.length}`);

        // Step 4: Fetch user profile (now populated with fresh game data)
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
