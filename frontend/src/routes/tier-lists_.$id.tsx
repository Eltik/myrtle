import { createFileRoute, notFound } from "@tanstack/react-router";
import { TierListDetail } from "#/components/tier-lists/detail/TierListDetail";
import { myTierListFavoriteQueryOptions, tierListDetailQueryOptions } from "#/lib/api/tier-lists";
import { defaultOgURL } from "#/lib/og";
import { seo } from "#/lib/seo";

export const Route = createFileRoute("/tier-lists_/$id")({
    component: RouteComponent,
    loader: async ({ context, params }) => {
        const detail = await context.queryClient.ensureQueryData(tierListDetailQueryOptions(params.id));
        if (!detail) throw notFound();
        if (context.user) {
            void context.queryClient.prefetchQuery(myTierListFavoriteQueryOptions(params.id, true));
        }
        return detail;
    },
    head: ({ loaderData, params }) => {
        if (!loaderData) {
            return seo({ title: "Tier List", path: `/tier-lists/${params.id}` });
        }
        const description = loaderData.description || `${loaderData.listType === "official" ? "Official" : "Community"} tier list with ${loaderData.tiers.length} tiers on myrtle.moe.`;
        return seo({
            title: loaderData.title,
            description,
            path: `/tier-lists/${loaderData.slug}`,
            image: defaultOgURL("tier-lists"),
            type: "article",
        });
    },
});

function RouteComponent() {
    return <TierListDetail />;
}
