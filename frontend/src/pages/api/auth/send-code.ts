import type { NextApiRequest, NextApiResponse } from "next";
import { z } from "zod";
import { backendFetch } from "~/lib/backend-fetch";

const AKServerSchema = z.enum(["en", "jp", "kr", "cn", "bili", "tw"]);

const SendCodeSchema = z.object({
    email: z.email("Invalid email format").min(1, "Email is required").max(254, "Email too long"),
    server: AKServerSchema.default("en"),
});

type SendCodeInput = z.infer<typeof SendCodeSchema>;

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
    if (req.method !== "POST") {
        res.setHeader("Allow", ["POST"]);
        return res.status(405).json({
            success: false,
            error: `Method ${req.method} not allowed`,
        });
    }

    try {
        const parseResult = SendCodeSchema.safeParse(req.body);

        if (!parseResult.success) {
            return res.status(400).json({
                success: false,
                error: "Validation failed",
                details: parseResult.error.issues,
            });
        }

        const { email, server }: SendCodeInput = parseResult.data;

        const params = new URLSearchParams();
        params.set("email", email);
        params.set("server", server);

        const sendCodeResponse = await backendFetch(`/send-code?${params.toString()}`, {
            method: "POST",
        });

        if (!sendCodeResponse.ok) {
            const errorText = await sendCodeResponse.text();
            console.error(`Backend send-code failed: ${sendCodeResponse.status} - ${errorText}.`);

            return res.status(400).json({
                success: false,
                error: "Failed to send verification code",
            });
        }

        return res.status(200).json({ success: true });
    } catch (error) {
        console.error("Send-code handler error:", error);
        return res.status(500).json({
            success: false,
            error: "An internal server error occurred",
        });
    }
}
