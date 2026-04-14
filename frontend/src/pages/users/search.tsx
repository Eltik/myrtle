import type { GetServerSideProps, NextPage } from "next";
import { SEO } from "~/components/seo";
import { SearchPageContent } from "~/components/users/search";
import type { LeaderboardEntry, SearchApiResponse, SearchResponse, UserProfile } from "~/types/api";

interface Props {
    data: SearchResponse;
}

const EMPTY_RESPONSE: SearchResponse = {
    results: [],
    pagination: {
        limit: 24,
        offset: 0,
        total: 0,
        hasMore: false,
    },
};

const SearchPageView: NextPage<Props> = ({ data }) => {
    return (
        <>
            <SEO description="Search for Arknights players by name or UID. Find and explore player profiles on myrtle.moe." keywords={["player search", "find players", "Arknights profiles", "search players"]} path="/users/search" title="Player Search" />
            <SearchPageContent initialData={data} />
        </>
    );
};

export const getServerSideProps: GetServerSideProps<Props> = async (context) => {
    const { backendFetch } = await import("~/lib/backend-fetch");

    const q = context.query.q as string | undefined;
    const limit = Number(context.query.limit) || 24;
    const offset = Number(context.query.offset) || 0;

    // v3 requires a non-empty `q` on /search. When the user hasn't entered a query,
    // fall back to the leaderboard endpoint to show a default list of players
    // (ranked by total_score). LeaderboardEntry is a superset of the fields
    // UserProfile exposes, so it renders the same way in the search UI.
    if (!q || !q.trim()) {
        try {
            const lbParams = new URLSearchParams();
            lbParams.set("limit", String(limit));
            lbParams.set("offset", String(offset));

            const lbResponse = await backendFetch(`/leaderboard?${lbParams.toString()}`);

            if (!lbResponse.ok) {
                return { props: { data: EMPTY_RESPONSE } };
            }

            const lbData = (await lbResponse.json()) as { entries: LeaderboardEntry[]; total: number };

            // Adapt LeaderboardEntry -> UserProfile (fill missing fields with nulls)
            const results: UserProfile[] = (lbData.entries ?? []).map((e) => ({
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

            return {
                props: {
                    data: {
                        results,
                        pagination: {
                            limit,
                            offset,
                            total: lbData.total ?? results.length,
                            hasMore: offset + results.length < (lbData.total ?? 0),
                        },
                    },
                },
            };
        } catch (error) {
            console.error("Failed to fetch default leaderboard for search:", error);
            return { props: { data: EMPTY_RESPONSE } };
        }
    }

    const params = new URLSearchParams();
    params.set("q", q);
    params.set("limit", String(limit));
    params.set("offset", String(offset));

    try {
        const response = await backendFetch(`/search?${params.toString()}`);

        if (!response.ok) {
            console.error(`Search fetch failed: ${response.status}`);
            return { props: { data: EMPTY_RESPONSE } };
        }

        const results = (await response.json()) as SearchApiResponse;

        // Wrap the raw array response with pagination info for the component
        const data: SearchResponse = {
            results: Array.isArray(results) ? results : [],
            pagination: {
                limit,
                offset,
                total: Array.isArray(results) ? (results.length < limit ? offset + results.length : offset + limit + 1) : 0,
                hasMore: Array.isArray(results) && results.length >= limit,
            },
        };

        return { props: { data } };
    } catch (error) {
        console.error("Failed to fetch search results:", error);
        return { props: { data: EMPTY_RESPONSE } };
    }
};

export default SearchPageView;
