import { createFileRoute } from "@tanstack/react-router";
import Home from "#/components/home/Home";
import { statsQueryOptions } from "#/lib/api/stats";
import { homeTierListsQueryOptions } from "#/lib/api/tier-lists";
import { defaultOgURL } from "#/lib/og";
import { seo } from "#/lib/seo";

export const Route = createFileRoute("/")({
    component: Home,
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
