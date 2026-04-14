import type { NextApiRequest, NextApiResponse } from "next";
import { backendFetch } from "~/lib/backend-fetch";

export default async function handler(req: NextApiRequest, res: NextApiResponse) {
    if (req.method !== "GET") {
        return res.status(405).json({ error: "Method not allowed" });
    }

    const { q, limit, offset } = req.query;

    // v3 search requires a non-empty `q`. When the client sends no query
    // (e.g., user cleared the search box), fall back to the leaderboard
    // to show a default list of players ranked by total_score.
    if (!q || typeof q !== "string" || !q.trim()) {
        try {
            const lbParams = new URLSearchParams();
            if (limit && typeof limit === "string") lbParams.set("limit", limit);
            if (offset && typeof offset === "string") lbParams.set("offset", offset);

            const lbResponse = await backendFetch(`/leaderboard?${lbParams.toString()}`);
            if (!lbResponse.ok) {
                return res.status(200).json([]);
            }

            const lbData = (await lbResponse.json()) as {
                entries: Array<{
                    id: string;
                    uid: string;
                    nickname: string | null;
                    level: number | null;
                    avatar_id: string | null;
                    secretary: string | null;
                    secretary_skin_id: string | null;
                    server: string;
                    total_score: number | null;
                    grade: string | null;
                }>;
                total: number;
            };

            // Adapt LeaderboardEntry -> UserProfile shape expected by the search UI
            const results = (lbData.entries ?? []).map((e) => ({
                id: e.id,
                uid: e.uid,
                nickname: e.nickname,
                level: e.level,
                avatar_id: e.avatar_id,
                secretary: e.secretary,
                secretary_skin_id: e.secretary_skin_id,
                resume_id: null,
                role: "user",
                server: e.server,
                total_score: e.total_score,
                grade: e.grade,
                public_profile: null,
                store_gacha: null,
                share_stats: null,
                exp: null,
                orundum: null,
                lmd: null,
                sanity: null,
                max_sanity: null,
                gacha_tickets: null,
                ten_pull_tickets: null,
                monthly_sub_end: null,
                register_ts: null,
                last_online_ts: null,
                resume: null,
                friend_num_limit: null,
                operator_count: null,
                item_count: null,
                skin_count: null,
            }));

            return res.status(200).json(results);
        } catch (error) {
            console.error("Default leaderboard fetch error:", error);
            return res.status(200).json([]);
        }
    }

    const params = new URLSearchParams();
    params.set("q", q);
    if (limit && typeof limit === "string") params.set("limit", limit);
    if (offset && typeof offset === "string") params.set("offset", offset);

    try {
        const response = await backendFetch(`/search?${params.toString()}`);

        if (!response.ok) {
            return res.status(response.status).json({ error: "Failed to fetch search results" });
        }

        const data = await response.json();
        return res.status(200).json(data);
    } catch (error) {
        console.error("Search API error:", error);
        return res.status(500).json({ error: "Internal server error" });
    }
}
