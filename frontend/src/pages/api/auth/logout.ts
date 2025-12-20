import { serialize } from "cookie";
import type { NextApiRequest, NextApiResponse } from "next";
import { env } from "~/env";

interface ApiResponse {
    success: boolean;
}

export default function handler(req: NextApiRequest, res: NextApiResponse<ApiResponse>) {
    // Only allow POST
    if (req.method !== "POST") {
        res.setHeader("Allow", ["POST"]);
        return res.status(405).json({ success: false });
    }

    // Clear both session and indicator cookies
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

    return res.status(200).json({ success: true });
}
