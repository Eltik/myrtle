import { createFileRoute } from "@tanstack/react-router";
import Home from "#/components/home/Home";
import { statsQueryOptions } from "#/lib/api/stats";
import { homeTierListsQueryOptions } from "#/lib/api/tier-lists";
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
    loader: ({ context }) => Promise.all([context.queryClient.ensureQueryData(statsQueryOptions()), context.queryClient.ensureQueryData(homeTierListsQueryOptions())]),
    head: () => {
        const { meta, links } = seo({
            title: "Myrtle",
            description: "Track your roster, scout community pulls, and explore every operator.",
            path: "/",
            image: defaultOgURL("home"),
        });
        return {
            meta: [{ charSet: "utf-8" }, { name: "viewport", content: "width=device-width, initial-scale=1" }, ...meta],
            links,
        };
    },
});
