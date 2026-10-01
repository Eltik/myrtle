import { createFileRoute, notFound, stripSearchParams } from "@tanstack/react-router";
import { TierListDetail } from "#/components/tier-lists/detail/TierListDetail";
import { env } from "#/env";
import { DEFAULT_GAMEDATA_SERVER } from "#/lib/api/gamedata";
import { entityIconURL, isOperatorEntity } from "#/lib/api/tier-entities";
import { type ITierListDetail, myTierListFavoriteQueryOptions, tierListDetailQueryOptions, tierListVersionsQueryOptions } from "#/lib/api/tier-lists";
import { metaT } from "#/lib/meta";
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

/** `server` is the one the detail was resolved against, so every icon reads the same server's assets. */
function buildOgData(detail: ITierListDetail, server: string | undefined): ITierListOgData {
    const assetServer = server && server !== DEFAULT_GAMEDATA_SERVER ? server : undefined;
    const sortedTiers = [...detail.tiers].sort((a, b) => a.displayOrder - b.displayOrder);
    const tiers = sortedTiers.slice(0, 4).map((t, i) => {
        const fallback = FALLBACK_TIER_HEX[i % FALLBACK_TIER_HEX.length] as string;
        // Unresolved placements stay off the image; every resolved kind shows its own icon.
        const ops = [...t.entities].sort((a, b) => a.subOrder - b.subOrder).filter((e) => e.resolved);
        return {
            name: t.name,
            color: safeHex(t.color, fallback),
            operators: ops.slice(0, 5).map((op) => ({
                id: op.id,
                name: op.name,
                rarity: isOperatorEntity(op) ? op.rarity : 1,
                avatarURL: isOperatorEntity(op) ? getAvatarById(op.id, assetServer) : op.icon ? entityIconURL(op.icon, env.VITE_BACKEND_URL ?? "", assetServer) : undefined,
            })),
            operatorCount: t.entities.length,
        };
    });

    const totalOperators = detail.tiers.reduce((sum, t) => sum + t.entities.length, 0);
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

interface ITierListDetailSearch {
    v?: number;
}

const DETAIL_SEARCH_DEFAULTS = { v: undefined };

export const Route = createFileRoute("/tier-lists_/$id")({
    component: RouteComponent,
    validateSearch: (search: Record<string, unknown>): ITierListDetailSearch => {
        const raw = search.v;
        if (typeof raw === "number" && Number.isFinite(raw) && raw >= 1) return { v: Math.floor(raw) };
        if (typeof raw === "string" && raw.length > 0) {
            const parsed = Number.parseInt(raw, 10);
            if (Number.isFinite(parsed) && parsed >= 1) return { v: parsed };
        }
        return {};
    },
    search: { middlewares: [stripSearchParams(DETAIL_SEARCH_DEFAULTS)] },
    loader: async ({ context, params }) => {
        const detail = await context.queryClient.ensureQueryData(tierListDetailQueryOptions(params.id, context.i18n.gamedataServer));
        if (!detail) throw notFound();
        if (context.user) {
            void context.queryClient.prefetchQuery(myTierListFavoriteQueryOptions(params.id, true));
        }
        void context.queryClient.prefetchQuery(tierListVersionsQueryOptions(params.id, context.i18n.gamedataServer));
        warmOg("tier-list", detail.slug, buildOgData(detail, context.i18n.gamedataServer));
        return detail;
    },
    head: ({ loaderData, match, params }) => {
        const t = metaT(match.context.i18n);
        const locale = match.context.i18n?.locale;
        if (!loaderData) {
            return seo({ title: t("tierList.fallbackTitle"), path: `/tier-lists/${params.id}`, locale });
        }
        // One message with a `select`, not two halves joined: which of
        // "Official"/"Community" opens the sentence is a word choice inside it,
        // and in most languages it moves.
        const description = loaderData.description || t("tierList.descriptionFallback", { type: loaderData.listType, count: loaderData.tiers.length });
        return seo({
            title: loaderData.title,
            description,
            path: `/tier-lists/${loaderData.slug}`,
            image: ogURL("tier-list", loaderData.slug, buildOgData(loaderData, match.context.i18n?.gamedataServer)),
            type: "article",
            preloadImage: true,
            locale,
        });
    },
});

function RouteComponent() {
    return <TierListDetail />;
}
