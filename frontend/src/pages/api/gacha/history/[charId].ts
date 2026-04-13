import type { NextApiRequest, NextApiResponse } from "next";

export default function handler(_req: NextApiRequest, res: NextApiResponse) {
    return res.status(410).json({ error: "Gone. This endpoint has been removed in v3." });
}
