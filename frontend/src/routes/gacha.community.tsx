import { createFileRoute } from "@tanstack/react-router";
import { CommunityPage } from "#/components/gacha/community/CommunityPage";
import { bannersQueryOptions, gachaEnhancedStatsQueryOptions, perBannerStatsQueryOptions } from "#/lib/api/gacha";
import { operatorsIndexQueryOptions } from "#/lib/api/operators";
import { metaT } from "#/lib/meta";
import { defaultOgURL } from "#/lib/og";
import { seo } from "#/lib/seo";

export const Route = createFileRoute("/gacha/community")({
    component: RouteComponent,
    loader: async ({ context: { queryClient, i18n } }) => {
        await Promise.all([
            queryClient.prefetchQuery(gachaEnhancedStatsQueryOptions({ topN: 20, includeTiming: true })),
            queryClient.prefetchQuery(operatorsIndexQueryOptions(i18n.gamedataServer)),
            queryClient.prefetchQuery(bannersQueryOptions(i18n.gamedataServer)),
            queryClient.prefetchQuery(perBannerStatsQueryOptions()),
        ]);
    },
    head: ({ match }) => {
        const t = metaT(match.context.i18n);
        const { meta, links } = seo({
            title: t("gachaCommunity.title"),
            description: t("gachaCommunity.description"),
            path: "/gacha/community",
            image: defaultOgURL("gacha-community", match.context.i18n),
            locale: match.context.i18n?.locale,
        });
        return {
            meta: [{ charSet: "utf-8" }, { name: "viewport", content: "width=device-width, initial-scale=1" }, ...meta],
            links,
        };
    },
});

function RouteComponent() {
    return <CommunityPage />;
}
