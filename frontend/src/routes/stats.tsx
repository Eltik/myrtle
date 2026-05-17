import { createFileRoute } from "@tanstack/react-router";
import { StatsPage } from "#/components/stats/StatsPage";
import { statsQueryOptions } from "#/lib/api/stats";
import { defaultOgURL } from "#/lib/og";
import { seo } from "#/lib/seo";

export const Route = createFileRoute("/stats")({
    component: RouteComponent,
    loader: ({ context }) => context.queryClient.ensureQueryData(statsQueryOptions()),
    head: () => {
        const { meta, links } = seo({
            title: "Stats",
            description: "A live count of every operator, skill, module, tier list, and roster indexed on myrtle.moe.",
            path: "/stats",
            image: defaultOgURL("stats"),
        });
        return {
            meta: [{ charSet: "utf-8" }, { name: "viewport", content: "width=device-width, initial-scale=1" }, ...meta],
            links,
        };
    },
});

function RouteComponent() {
    return <StatsPage />;
}
