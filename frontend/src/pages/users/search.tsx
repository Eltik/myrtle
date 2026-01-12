import type { GetServerSideProps, NextPage } from "next";
import Head from "next/head";
import type { SearchQuery, SearchResponse } from "~/types/api";

interface Props {
    data: SearchResponse;
}

/**
 * Fetches search results from the API (client-side)
 */
export async function fetchSearchResults(params: SearchQuery): Promise<SearchResponse> {
    const searchParams = new URLSearchParams();

    // Text search params
    if (params.nickname) searchParams.set("nickname", params.nickname);
    if (params.uid) searchParams.set("uid", params.uid);
    if (params.resume) searchParams.set("resume", params.resume);

    // Exact match filters
    if (params.server) searchParams.set("server", params.server);
    if (params.grade) searchParams.set("grade", params.grade);
    if (params.secretary) searchParams.set("secretary", params.secretary);

    // Range query params
    if (params.level) searchParams.set("level", params.level);
    if (params.totalScore) searchParams.set("totalScore", params.totalScore);
    if (params.compositeScore) searchParams.set("compositeScore", params.compositeScore);
    if (params.operatorScore) searchParams.set("operatorScore", params.operatorScore);
    if (params.stageScore) searchParams.set("stageScore", params.stageScore);
    if (params.roguelikeScore) searchParams.set("roguelikeScore", params.roguelikeScore);
    if (params.sandboxScore) searchParams.set("sandboxScore", params.sandboxScore);
    if (params.medalScore) searchParams.set("medalScore", params.medalScore);
    if (params.baseScore) searchParams.set("baseScore", params.baseScore);

    // Query options
    if (params.logic) searchParams.set("logic", params.logic);
    if (params.sortBy) searchParams.set("sortBy", params.sortBy);
    if (params.order) searchParams.set("order", params.order);
    if (params.limit !== undefined) searchParams.set("limit", String(params.limit));
    if (params.offset !== undefined) searchParams.set("offset", String(params.offset));
    if (params.fields) searchParams.set("fields", params.fields);

    const response = await fetch(`/api/search?${searchParams.toString()}`);

    if (!response.ok) {
        throw new Error("Failed to fetch search results");
    }

    return response.json() as Promise<SearchResponse>;
}

const SearchPage: NextPage<Props> = ({ data }) => {
    // Skeleton: data is available via props for building UI on top
    return (
        <>
            <Head>
                <title>User Search - myrtle.moe</title>
                <meta content="Search for Arknights players by name, UID, score, and more." name="description" />
            </Head>
            <div>
                <p>Search page skeleton - {data.pagination.total} results available</p>
            </div>
        </>
    );
};

export const getServerSideProps: GetServerSideProps<Props> = async (context) => {
    const { nickname, uid, resume, server, grade, secretary, level, totalScore, compositeScore, operatorScore, stageScore, roguelikeScore, sandboxScore, medalScore, baseScore, logic, sortBy, order, limit, offset, fields } = context.query;

    const { env } = await import("~/env");
    const backendUrl = new URL("/search", env.BACKEND_URL);

    // Text search params
    if (nickname && typeof nickname === "string") {
        backendUrl.searchParams.set("nickname", nickname);
    }
    if (uid && typeof uid === "string") {
        backendUrl.searchParams.set("uid", uid);
    }
    if (resume && typeof resume === "string") {
        backendUrl.searchParams.set("resume", resume);
    }

    // Exact match filters
    if (server && typeof server === "string") {
        backendUrl.searchParams.set("server", server);
    }
    if (grade && typeof grade === "string") {
        backendUrl.searchParams.set("grade", grade);
    }
    if (secretary && typeof secretary === "string") {
        backendUrl.searchParams.set("secretary", secretary);
    }

    // Range query params
    if (level && typeof level === "string") {
        backendUrl.searchParams.set("level", level);
    }
    if (totalScore && typeof totalScore === "string") {
        backendUrl.searchParams.set("totalScore", totalScore);
    }
    if (compositeScore && typeof compositeScore === "string") {
        backendUrl.searchParams.set("compositeScore", compositeScore);
    }
    if (operatorScore && typeof operatorScore === "string") {
        backendUrl.searchParams.set("operatorScore", operatorScore);
    }
    if (stageScore && typeof stageScore === "string") {
        backendUrl.searchParams.set("stageScore", stageScore);
    }
    if (roguelikeScore && typeof roguelikeScore === "string") {
        backendUrl.searchParams.set("roguelikeScore", roguelikeScore);
    }
    if (sandboxScore && typeof sandboxScore === "string") {
        backendUrl.searchParams.set("sandboxScore", sandboxScore);
    }
    if (medalScore && typeof medalScore === "string") {
        backendUrl.searchParams.set("medalScore", medalScore);
    }
    if (baseScore && typeof baseScore === "string") {
        backendUrl.searchParams.set("baseScore", baseScore);
    }

    // Query options
    if (logic && typeof logic === "string") {
        backendUrl.searchParams.set("logic", logic);
    }
    if (sortBy && typeof sortBy === "string") {
        backendUrl.searchParams.set("sortBy", sortBy);
    }
    if (order && typeof order === "string") {
        backendUrl.searchParams.set("order", order);
    }
    if (limit && typeof limit === "string") {
        backendUrl.searchParams.set("limit", limit);
    }
    if (offset && typeof offset === "string") {
        backendUrl.searchParams.set("offset", offset);
    }
    if (fields && typeof fields === "string") {
        backendUrl.searchParams.set("fields", fields);
    }

    try {
        const response = await fetch(backendUrl.toString(), {
            method: "GET",
            headers: {
                "Content-Type": "application/json",
            },
        });

        if (!response.ok) {
            console.error(`Search fetch failed: ${response.status}`);
            return { notFound: true };
        }

        const data = (await response.json()) as SearchResponse;

        if (!data.results) {
            return { notFound: true };
        }

        return {
            props: {
                data,
            },
        };
    } catch (error) {
        console.error("Failed to fetch search results:", error);
        return { notFound: true };
    }
};

export default SearchPage;
