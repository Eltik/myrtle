import type { GetServerSideProps, NextPage } from "next";
import Head from "next/head";
import { SearchPageContent } from "~/components/users/search";
import { buildSearchParamsFromQuery } from "~/lib/search-utils";
import type { SearchResponse } from "~/types/api";

interface Props {
    data: SearchResponse;
}

const SearchPageView: NextPage<Props> = ({ data }) => {
    return (
        <>
            <Head>
                <title>Player Search - myrtle.moe</title>
                <meta content="Search for Arknights players by name, server, level, and more." name="description" />
            </Head>
            <SearchPageContent initialData={data} />
        </>
    );
};

export const getServerSideProps: GetServerSideProps<Props> = async (context) => {
    const { env } = await import("~/env");

    const searchParams = buildSearchParamsFromQuery(context.query);

    // Set default limit for initial page load
    if (!searchParams.has("limit")) {
        searchParams.set("limit", "24");
    }

    // Request full data for hover card info (resume, registerTs, etc.)
    searchParams.set("fields", "data");

    const backendUrl = new URL("/search", env.BACKEND_URL);
    backendUrl.search = searchParams.toString();

    try {
        const response = await fetch(backendUrl.toString(), {
            method: "GET",
            headers: {
                "Content-Type": "application/json",
            },
        });

        if (!response.ok) {
            console.error(`Search fetch failed: ${response.status}`);
            // Return empty results instead of 404 for better UX
            return {
                props: {
                    data: {
                        results: [],
                        pagination: {
                            limit: 24,
                            offset: 0,
                            total: 0,
                            hasMore: false,
                        },
                        meta: {
                            queryLogic: "and",
                            sortBy: "updated_at",
                            order: "desc",
                            filtersApplied: [],
                            cached: false,
                        },
                    },
                },
            };
        }

        const data = (await response.json()) as SearchResponse;

        return {
            props: {
                data: data.results
                    ? data
                    : {
                          results: [],
                          pagination: {
                              limit: 24,
                              offset: 0,
                              total: 0,
                              hasMore: false,
                          },
                          meta: {
                              queryLogic: "and",
                              sortBy: "updated_at",
                              order: "desc",
                              filtersApplied: [],
                              cached: false,
                          },
                      },
            },
        };
    } catch (error) {
        console.error("Failed to fetch search results:", error);
        return {
            props: {
                data: {
                    results: [],
                    pagination: {
                        limit: 24,
                        offset: 0,
                        total: 0,
                        hasMore: false,
                    },
                    meta: {
                        queryLogic: "and",
                        sortBy: "updated_at",
                        order: "desc",
                        filtersApplied: [],
                        cached: false,
                    },
                },
            },
        };
    }
};

export default SearchPageView;
