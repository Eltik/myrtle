import type { GetServerSideProps, NextPage } from "next";
import Head from "next/head";
import { LeaderboardPage as LeaderboardContent } from "~/components/users/leaderboard";
import type { LeaderboardResponse } from "~/types/api";

interface Props {
    data: LeaderboardResponse;
}

const LeaderboardPage: NextPage<Props> = ({ data }) => {
    return (
        <>
            <Head>
                <title>Leaderboard - myrtle.moe</title>
                <meta content="View the top Arknights players ranked by collection, progress, and achievements." name="description" />
            </Head>
            <LeaderboardContent initialData={data} />
        </>
    );
};

export const getServerSideProps: GetServerSideProps<Props> = async (context) => {
    const { sort_by, order, server, limit, offset } = context.query;

    const { env } = await import("~/env");
    const backendUrl = new URL("/leaderboard", env.BACKEND_URL);

    if (sort_by && typeof sort_by === "string") {
        backendUrl.searchParams.set("sort_by", sort_by);
    }
    if (order && typeof order === "string") {
        backendUrl.searchParams.set("order", order);
    }
    if (server && typeof server === "string") {
        backendUrl.searchParams.set("server", server);
    }
    if (limit && typeof limit === "string") {
        backendUrl.searchParams.set("limit", limit);
    }
    if (offset && typeof offset === "string") {
        backendUrl.searchParams.set("offset", offset);
    }

    try {
        const response = await fetch(backendUrl.toString(), {
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
