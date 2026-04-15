import type { GetServerSideProps, NextPage } from "next";
import { SEO } from "~/components/seo";
import { LeaderboardPage as LeaderboardContent } from "~/components/users/leaderboard";
import type { LeaderboardResponse } from "~/types/api";

interface Props {
    data: LeaderboardResponse;
}

const LeaderboardPage: NextPage<Props> = ({ data }) => {
    return (
        <>
            <SEO
                description="View the top Arknights players ranked by collection, progress, and achievements. Compare your account across operators, stages, roguelike, sandbox, medals, and base scores."
                keywords={["leaderboard", "top players", "player rankings", "account score", "Arknights rankings"]}
                path="/users/leaderboard"
                title="Leaderboard"
            />
            <LeaderboardContent initialData={data} />
        </>
    );
};

export const getServerSideProps: GetServerSideProps<Props> = async (context) => {
    const { sort, server, limit, offset } = context.query;

    const { backendFetch } = await import("~/lib/backend-fetch");

    const params = new URLSearchParams();
    if (sort && typeof sort === "string") params.set("sort", sort);
    if (server && typeof server === "string") params.set("server", server);
    if (limit && typeof limit === "string") params.set("limit", limit);
    if (offset && typeof offset === "string") params.set("offset", offset);

    const queryString = params.toString();
    const path = queryString ? `/leaderboard?${queryString}` : "/leaderboard";

    try {
        const response = await backendFetch(path);

        if (!response.ok) {
            console.error(`Leaderboard fetch failed: ${response.status}`);
            return { notFound: true };
        }

        const data = (await response.json()) as LeaderboardResponse;

        if (!data.entries) {
            return { notFound: true };
        }

        return {
            props: {
                data,
            },
        };
    } catch (error) {
        console.error("Failed to fetch leaderboard:", error);
        return { notFound: true };
    }
};

export default LeaderboardPage;
