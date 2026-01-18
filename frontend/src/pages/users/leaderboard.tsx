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
    const { sort_by, order, server, limit, offset } = context.query;

    const { env } = await import("~/env");
    const backendURL = new URL("/leaderboard", env.BACKEND_URL);

    if (sort_by && typeof sort_by === "string") {
        backendURL.searchParams.set("sort_by", sort_by);
    }
    if (order && typeof order === "string") {
        backendURL.searchParams.set("order", order);
    }
    if (server && typeof server === "string") {
        backendURL.searchParams.set("server", server);
    }
    if (limit && typeof limit === "string") {
        backendURL.searchParams.set("limit", limit);
    }
    if (offset && typeof offset === "string") {
        backendURL.searchParams.set("offset", offset);
    }

    // Request minimal fields for initial table view (optimization)
    backendURL.searchParams.set("fields", "minimal");

    try {
        const response = await fetch(backendURL.toString(), {
            method: "GET",
            headers: {
                "Content-Type": "application/json",
            },
        });

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
