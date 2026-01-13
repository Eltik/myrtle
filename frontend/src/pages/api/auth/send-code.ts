import type { NextApiRequest, NextApiResponse } from "next";
import { z } from "zod";
import { env } from "~/env";

// Valid Arknights server regions
const AKServerSchema = z.enum(["en", "jp", "kr", "cn", "bili", "tw"]);

// Send code request validation schema
const SendCodeSchema = z.object({
    email: z.email("Invalid email format").min(1, "Email is required").max(254, "Email too long"),
    server: AKServerSchema.default("en"),
});

type SendCodeInput = z.infer<typeof SendCodeSchema>;

// API response types
interface SuccessResponse {
    success: true;
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
        const parseResult = SendCodeSchema.safeParse(req.body);

        if (!parseResult.success) {
            return res.status(400).json({
                success: false,
                error: "Validation failed",
                details: parseResult.error.issues,
            });
        }

        const { email, server }: SendCodeInput = parseResult.data;

        // Build backend URL with properly encoded parameters
        const backendURL = new URL("/send-code", env.BACKEND_URL);
        backendURL.searchParams.set("email", email);
        backendURL.searchParams.set("server", server);

        // Call backend to send verification code
        const sendCodeResponse = await fetch(backendURL.toString(), {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
        });

        if (!sendCodeResponse.ok) {
            // Log the actual error for debugging, but don't expose to client
            const errorText = await sendCodeResponse.text();
            console.error(`Backend send-code failed: ${sendCodeResponse.status} - ${errorText}.`);

            // Return generic error to client (don't leak backend details)
            return res.status(400).json({
                success: false,
                error: "Failed to send verification code",
            });
        }

        // Don't expose backend response details - just confirm success
        return res.status(200).json({ success: true });
    } catch (error) {
        console.error("Send-code handler error:", error);
        return res.status(500).json({
            success: false,
            error: "An internal server error occurred",
        });
    }
}
