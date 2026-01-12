import type { GetServerSideProps, NextPage } from "next";
import Head from "next/head";
import { SearchPage } from "~/components/users/search";
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
            <div>
                <SearchPage initialData={data} />
            </div>
        </>
    );
};

export const getServerSideProps: GetServerSideProps<Props> = async (context) => {
    const { env } = await import("~/env");

    const searchParams = buildSearchParamsFromQuery(context.query);
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

export default SearchPageView;
