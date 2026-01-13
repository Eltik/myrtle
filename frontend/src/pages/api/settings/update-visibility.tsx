import type { NextApiRequest, NextApiResponse } from "next";
import { z } from "zod";
import { env } from "~/env";
import { getSessionFromCookie, getSiteToken } from "~/lib/auth";

const UpdateVisibilitySchema = z.object({
    publicProfile: z.boolean(),
});

interface UpdateVisibilityResponse {
    success: boolean;
    message?: string;
    settings?: Record<string, unknown>;
}

export default async function handler(req: NextApiRequest, res: NextApiResponse<UpdateVisibilityResponse>) {
    if (req.method !== "POST") {
        return res.status(405).json({ success: false, message: "Method not allowed" });
    }

    try {
        const parseResult = UpdateVisibilitySchema.safeParse(req.body);

        if (!parseResult.success) {
            return res.status(400).json({
                success: false,
                message: "Invalid request body",
            });
        }

        const { publicProfile } = parseResult.data;

        const token = getSiteToken(req);
        const session = getSessionFromCookie(req);

        if (!token || !session) {
            return res.status(401).json({
                success: false,
                message: "Not authenticated",
            });
        }

        const settings = {
            publicProfile,
        };

        const updateURL = new URL("/auth/update-settings", env.BACKEND_URL);

        const updateResponse = await fetch(updateURL.toString(), {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify({
                token,
                settings,
            }),
        });

        if (!updateResponse.ok) {
            const errorText = await updateResponse.text();
            console.error(`Backend settings update failed: ${updateResponse.status} - ${errorText}`);

            return res.status(500).json({
                success: false,
                message: "Failed to update visibility settings",
            });
        }

        const responseData = await updateResponse.json();

        return res.status(200).json({
            success: true,
            message: `Profile visibility updated to ${publicProfile ? "public" : "private"}`,
            settings: responseData.settings,
        });
    } catch (error) {
        console.error("Error updating profile visibility:", error);
        return res.status(500).json({ success: false, message: "Failed to update profile visibility" });
    }
}
