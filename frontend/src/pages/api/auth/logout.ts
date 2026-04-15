import type { NextApiRequest, NextApiResponse } from "next";
import { clearAuthCookies } from "~/lib/auth";

interface ApiResponse {
    success: boolean;
}

export default function handler(req: NextApiRequest, res: NextApiResponse<ApiResponse>) {
    if (req.method !== "POST") {
        res.setHeader("Allow", ["POST"]);
        return res.status(405).json({ success: false });
    }

    clearAuthCookies(res);

    return res.status(200).json({ success: true });
}
