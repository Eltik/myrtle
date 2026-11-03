import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Download, Heart, Share2 } from "lucide-react";
import { useState } from "react";
import { Button } from "#/components/ui/button";
import { toastManager } from "#/components/ui/toast";
import { Toggle } from "#/components/ui/toggle";
import { useAuth } from "#/hooks/use-auth";
import { type ITierListDetail, myTierListFavoriteQueryOptions, toggleTierListFavoriteFn } from "#/lib/api/tier-lists";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn, downloadBlob } from "#/lib/utils";
import type { messages } from "./TierListActions.messages";

interface ITierListActionsProps {
    detail: ITierListDetail;
}

export function TierListActions({ detail }: ITierListActionsProps) {
    const t: TypedT<typeof messages> = useT("tierLists");
    const queryClient = useQueryClient();
    const { user } = useAuth();
    const authed = Boolean(user);
    const [downloading, setDownloading] = useState(false);

    const favoriteKey = ["tier-lists", "favorite", detail.slug, authed ? "auth" : "anon"] as const;

    const { data: favoriteData } = useQuery(myTierListFavoriteQueryOptions(detail.slug, authed));
    const favorited = favoriteData?.favorited ?? false;

    const favoriteMut = useMutation({
        mutationFn: () => toggleTierListFavoriteFn({ data: detail.slug }),
        onMutate: async () => {
            await queryClient.cancelQueries({ queryKey: favoriteKey });
            const previous = queryClient.getQueryData<{ favorited: boolean } | null>(favoriteKey);
            queryClient.setQueryData<{ favorited: boolean } | null>(favoriteKey, () => ({ favorited: !(previous?.favorited ?? false) }));
            return { previous };
        },
        onError: (_err, _vars, ctx) => {
            if (ctx) queryClient.setQueryData(favoriteKey, ctx.previous);
            toastManager.add({
                id: `tl-fav-error-${Date.now()}`,
                title: t("detail.actions.favorite.errTitle"),
                description: t("detail.actions.favorite.errBody"),
                type: "error",
            });
        },
        onSuccess: (data) => {
            if (data) queryClient.setQueryData<{ favorited: boolean }>(favoriteKey, data);
            queryClient.invalidateQueries({ queryKey: ["tier-lists", "detail", detail.slug] });
            queryClient.invalidateQueries({ queryKey: ["tier-lists", "browse"] });
            queryClient.invalidateQueries({ queryKey: ["tier-lists", "home"] });
            queryClient.invalidateQueries({ queryKey: ["tier-lists", "favorites"] });
        },
    });

    const handleFavorite = () => {
        if (!user) {
            toastManager.add({
                id: `tl-fav-auth-${Date.now()}`,
                title: t("detail.actions.favorite.authTitle"),
                description: t("detail.actions.favorite.authBody"),
                type: "error",
            });
            return;
        }
        favoriteMut.mutate();
    };

    const handleDownload = async () => {
        if (downloading) return;
        setDownloading(true);
        try {
            const res = await fetch(`/api/tier-lists/${encodeURIComponent(detail.slug)}/image?v=${encodeURIComponent(detail.updatedAt)}`);
            if (!res.ok) throw new Error(`HTTP ${res.status}`);
            const blob = await res.blob();
            const baseSlug = detail.slug.replace(/[^a-z0-9_-]+/gi, "-").replace(/^-+|-+$/g, "");
            const filename = /tier[-_]?list$/i.test(baseSlug) ? `${baseSlug || "tier-list"}.png` : `${baseSlug || "tier-list"}-tier-list.png`;
            downloadBlob(blob, filename);
        } catch {
            toastManager.add({
                id: `tl-download-error-${Date.now()}`,
                title: t("detail.actions.download.errTitle"),
                description: t("detail.actions.download.errBody"),
                type: "error",
            });
        } finally {
            setDownloading(false);
        }
    };

    const handleShare = async () => {
        if (typeof window === "undefined") return;
        const url = `${window.location.origin}/tier-lists/${detail.slug}`;
        const shareData = { title: detail.title, text: detail.description || t("detail.actions.share.text", { title: detail.title }), url };

        if (typeof navigator !== "undefined" && typeof navigator.share === "function") {
            try {
                await navigator.share(shareData);
                return;
            } catch {
                // fall through to clipboard
            }
        }

        try {
            await navigator.clipboard.writeText(url);
            toastManager.add({
                id: `tl-share-${Date.now()}`,
                title: t("detail.actions.share.okTitle"),
                description: t("detail.actions.share.okBody"),
                type: "success",
            });
        } catch {
            toastManager.add({
                id: `tl-share-error-${Date.now()}`,
                title: t("detail.actions.share.errTitle"),
                description: t("detail.actions.share.errBody"),
                type: "error",
            });
        }
    };

    return (
        <div className="flex shrink-0 flex-wrap items-center gap-2">
            <Toggle variant="outline" size="lg" className="gap-1.5" pressed={favorited} onPressedChange={() => handleFavorite()} disabled={favoriteMut.isPending} aria-label={favorited ? t("detail.actions.favorite.remove") : t("detail.actions.favorite.add")}>
                <Heart className={cn("h-4 w-4", favorited ? "fill-current text-primary" : "fill-none")} aria-hidden="true" />
                <span className="hidden sm:inline">{favorited ? t("detail.actions.favorited") : t("detail.actions.favorite")}</span>
            </Toggle>

            <Button variant="outline" size="lg" className="gap-1.5 disabled:cursor-wait" onClick={handleDownload} disabled={downloading} aria-label={t("detail.actions.download.label")}>
                <Download className={cn("h-4 w-4", downloading && "animate-pulse")} aria-hidden="true" />
                <span className="hidden sm:inline">{downloading ? t("detail.actions.downloading") : t("detail.actions.download")}</span>
            </Button>

            <Button variant="outline" size="lg" className="gap-1.5" onClick={handleShare} aria-label={t("detail.actions.share.label")}>
                <Share2 className="h-4 w-4" aria-hidden="true" />
                <span className="hidden sm:inline">{t("detail.actions.share")}</span>
            </Button>
        </div>
    );
}
