import type { GetServerSideProps, NextPage } from "next";
import { SEO } from "~/components/seo";
import { SearchPageContent } from "~/components/users/search";
import type { SearchApiResponse, SearchResponse } from "~/types/api";

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
    const { env } = await import("~/env");

    const q = context.query.q as string | undefined;
    const limit = Number(context.query.limit) || 24;
    const offset = Number(context.query.offset) || 0;

    const backendURL = new URL("/search", env.BACKEND_URL);
    if (q) {
        backendURL.searchParams.set("q", q);
    }
    backendURL.searchParams.set("limit", String(limit));
    backendURL.searchParams.set("offset", String(offset));

    try {
        const response = await fetch(backendURL.toString(), {
            method: "GET",
            headers: {
                "Content-Type": "application/json",
            },
        });

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
