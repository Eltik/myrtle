import { createFileRoute } from "@tanstack/react-router";
import Home from "#/components/home/Home";
import { TOP_DOCTORS_QUERY } from "#/components/home/impl/TopDoctors";
import { liveFeedQueryOptions } from "#/lib/api/live";
import { leaderboardQueryOptions } from "#/lib/api/user";
import { metaT } from "#/lib/meta";
import { defaultOgURL } from "#/lib/og";
import { seo } from "#/lib/seo";

interface IHomeSearch {
    auth?: "1";
    next?: string;
}

export const Route = createFileRoute("/")({
    component: Home,
    validateSearch: (search: Record<string, unknown>): IHomeSearch => {
        const auth = search.auth === "1" ? "1" : undefined;
        const next = typeof search.next === "string" ? search.next : undefined;
        return { auth, next };
    },
    // `prefetchQuery` never throws: a backend hiccup leaves a section in its own
    // error state instead of failing the whole home page.
    loader: ({ context: { queryClient, i18n } }) => Promise.all([queryClient.prefetchQuery(liveFeedQueryOptions(i18n.gamedataServer)), queryClient.prefetchQuery(leaderboardQueryOptions(TOP_DOCTORS_QUERY))]),
    head: ({ match }) => {
        const t = metaT(match.context.i18n);
        const { meta, links } = seo({
            title: t("home.title"),
            description: t("home.description"),
            path: "/",
            image: defaultOgURL("home", match.context.i18n),
            locale: match.context.i18n?.locale,
        });
        return {
            meta: [{ charSet: "utf-8" }, { name: "viewport", content: "width=device-width, initial-scale=1" }, ...meta],
            links,
        };
    },
});
