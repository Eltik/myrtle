import { createFileRoute } from "@tanstack/react-router";
import { StatsPage } from "#/components/stats/StatsPage";
import { statsQueryOptions } from "#/lib/api/stats";
import { metaT } from "#/lib/meta";
import { defaultOgURL } from "#/lib/og";
import { seo } from "#/lib/seo";

export const Route = createFileRoute("/stats")({
    component: RouteComponent,
    loader: ({ context }) => context.queryClient.ensureQueryData(statsQueryOptions()),
    head: ({ match }) => {
        const t = metaT(match.context.i18n);
        const { meta, links } = seo({
            title: t("stats.title"),
            description: t("stats.description"),
            path: "/stats",
            image: defaultOgURL("stats", match.context.i18n),
            locale: match.context.i18n?.locale,
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
