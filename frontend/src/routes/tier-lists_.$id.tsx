import { createFileRoute, notFound } from "@tanstack/react-router";
import { TierListDetail } from "#/components/tier-lists/detail/TierListDetail";
import { type ITierListDetail, myTierListFavoriteQueryOptions, tierListDetailQueryOptions } from "#/lib/api/tier-lists";
import type { ITierListOgData } from "#/lib/og/impl/templates/TierList";
import { ogURL, warmOg } from "#/lib/og/impl/url";
import { seo } from "#/lib/seo";
import { getAvatarById } from "#/lib/utils";

const HEX_COLOR_RE = /^#([0-9a-fA-F]{6})$/;
const FALLBACK_TIER_HEX = ["#dc4d56", "#e0834a", "#d8b54a", "#5dbf86", "#5aa9d9", "#9b73d4", "#8a8a8a"];

function safeHex(color: string | null | undefined, fallback: string): string {
    if (!color) return fallback;
    const trimmed = color.trim();
    return HEX_COLOR_RE.test(trimmed) ? trimmed : fallback;
}

function buildOgData(detail: ITierListDetail): ITierListOgData {
    const sortedTiers = [...detail.tiers].sort((a, b) => a.displayOrder - b.displayOrder);
    const tiers = sortedTiers.slice(0, 4).map((t, i) => {
        const fallback = FALLBACK_TIER_HEX[i % FALLBACK_TIER_HEX.length] as string;
        const ops = [...t.operators].sort((a, b) => a.subOrder - b.subOrder);
        return {
            name: t.name,
            color: safeHex(t.color, fallback),
            operators: ops.slice(0, 5).map((op) => ({
                id: op.id,
                name: op.name,
                rarity: op.rarity,
                avatarURL: getAvatarById(op.id),
            })),
            operatorCount: t.operators.length,
        };
    });

    const totalOperators = detail.tiers.reduce((sum, t) => sum + t.operators.length, 0);
    const flairColor = safeHex(detail.flair?.color, "");
    const updatedDate = new Date(detail.updatedAt);
    const updatedRelative = Number.isNaN(updatedDate.getTime()) ? undefined : updatedDate.toLocaleDateString("en-US", { month: "short", year: "numeric" });

    return {
        title: detail.title,
        slug: detail.slug,
        description: detail.description || undefined,
        listType: detail.listType,
        flairLabel: detail.flair?.label,
        flairColor: flairColor || undefined,
        authorName: detail.author?.nickname?.trim() || (detail.listType === "official" ? "Myrtle" : "Community"),
        authorAvatarURL: detail.author?.avatarId ? getAvatarById(detail.author.avatarId) : undefined,
        views: detail.stats?.viewCount ?? 0,
        favorites: detail.stats?.favoriteCount ?? 0,
        isTrending: detail.stats?.isTrending ?? false,
        updatedRelative,
        totalOperators,
        tierCount: detail.tiers.length,
        tiers,
    };
}

export const Route = createFileRoute("/tier-lists_/$id")({
    component: RouteComponent,
    loader: async ({ context, params }) => {
        const detail = await context.queryClient.ensureQueryData(tierListDetailQueryOptions(params.id));
        if (!detail) throw notFound();
        if (context.user) {
            void context.queryClient.prefetchQuery(myTierListFavoriteQueryOptions(params.id, true));
        }
        warmOg("tier-list", detail.slug, buildOgData(detail));
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
            image: ogURL("tier-list", loaderData.slug, buildOgData(loaderData)),
            type: "article",
            preloadImage: true,
        });
    },
});

function RouteComponent() {
    return <TierListDetail />;
}
