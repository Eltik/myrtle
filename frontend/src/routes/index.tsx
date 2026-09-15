import { createFileRoute } from "@tanstack/react-router";
import Home from "#/components/home/Home";
import { statsQueryOptions } from "#/lib/api/stats";
import { homeTierListsQueryOptions } from "#/lib/api/tier-lists";
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
    loader: ({ context }) => Promise.all([context.queryClient.ensureQueryData(statsQueryOptions()), context.queryClient.ensureQueryData(homeTierListsQueryOptions(context.i18n.gamedataServer))]),
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
