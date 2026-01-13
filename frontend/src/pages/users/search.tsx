import type { GetServerSideProps, NextPage } from "next";
import { SEO } from "~/components/seo";
import { SearchPageContent } from "~/components/users/search";
import { buildSearchParamsFromQuery } from "~/lib/search-utils";
import type { SearchResponse } from "~/types/api";

interface Props {
    data: SearchResponse;
}

const SearchPageView: NextPage<Props> = ({ data }) => {
    return (
        <>
            <SEO description="Search for Arknights players by name, server, level, and more. Find and explore player profiles on myrtle.moe." keywords={["player search", "find players", "Arknights profiles", "search players"]} path="/users/search" title="Player Search" />
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

    const backendURL = new URL("/search", env.BACKEND_URL);
    backendURL.search = searchParams.toString();

    try {
        const response = await fetch(backendURL.toString(), {
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
