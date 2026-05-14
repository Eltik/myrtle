import { createFileRoute } from "@tanstack/react-router";
import { CommunityPage } from "#/components/gacha/community/CommunityPage";
import { bannersQueryOptions, gachaEnhancedStatsQueryOptions, perBannerStatsQueryOptions } from "#/lib/api/gacha";
import { operatorsIndexQueryOptions } from "#/lib/api/operators";
import { defaultOgURL } from "#/lib/og";
import { seo } from "#/lib/seo";

export const Route = createFileRoute("/gacha/community")({
    component: RouteComponent,
    loader: async ({ context: { queryClient } }) => {
        await Promise.all([queryClient.prefetchQuery(gachaEnhancedStatsQueryOptions({ topN: 20, includeTiming: true })), queryClient.prefetchQuery(operatorsIndexQueryOptions()), queryClient.prefetchQuery(bannersQueryOptions()), queryClient.prefetchQuery(perBannerStatsQueryOptions())]);
    },
    head: () => {
        const { meta, links } = seo({
            title: "Gacha · Community",
            description: "Community-wide gacha statistics - pull rates, most-pulled operators, and pull timing across opted-in doctors.",
            path: "/gacha/community",
            image: defaultOgURL("gacha-community"),
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
