import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Download, Heart, Share2 } from "lucide-react";
import { useState } from "react";
import { toastManager } from "#/components/ui/toast";
import { useAuth } from "#/hooks/use-auth";
import { type ITierListDetail, myTierListFavoriteQueryOptions, toggleTierListFavoriteFn } from "#/lib/api/tier-lists";
import { cn, downloadBlob } from "#/lib/utils";

interface ITierListActionsProps {
    detail: ITierListDetail;
}

export function TierListActions({ detail }: ITierListActionsProps) {
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
            queryClient.setQueryData<{ favorited: boolean } | null>(favoriteKey, { favorited: !(previous?.favorited ?? false) });
            return { previous };
        },
        onError: (_err, _vars, ctx) => {
            if (ctx) queryClient.setQueryData(favoriteKey, ctx.previous);
            toastManager.add({
                id: `tl-fav-error-${Date.now()}`,
                title: "Couldn't update favorite",
                description: "Something went wrong. Try again in a moment.",
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
                title: "Sign in to favorite",
                description: "Connect your account to save lists for later.",
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
                title: "Couldn't download image",
                description: "Something went wrong while preparing the image. Try again in a moment.",
                type: "error",
            });
        } finally {
            setDownloading(false);
        }
    };

    const handleShare = async () => {
        if (typeof window === "undefined") return;
        const url = `${window.location.origin}/tier-lists/${detail.slug}`;
        const shareData = { title: detail.title, text: detail.description || `${detail.title} - tier list on myrtle.moe`, url };

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
                title: "Link copied",
                description: "Share it anywhere.",
                type: "success",
            });
        } catch {
            toastManager.add({
                id: `tl-share-error-${Date.now()}`,
                title: "Couldn't copy link",
                description: "Clipboard access was denied.",
                type: "error",
            });
        }
    };

    return (
        <div className="flex shrink-0 flex-wrap items-center gap-2">
            <button
                type="button"
                onClick={handleFavorite}
                disabled={favoriteMut.isPending}
                aria-pressed={favorited}
                aria-label={favorited ? "Remove from favorites" : "Add to favorites"}
                className={cn(
                    "inline-flex h-11 min-w-11 cursor-pointer items-center justify-center gap-1.5 rounded-lg border px-3 font-medium font-sans text-sm leading-none transition-colors disabled:opacity-60 sm:h-9 sm:min-w-0",
                    favorited ? "border-[color-mix(in_srgb,var(--primary)_50%,transparent)] bg-[color-mix(in_srgb,var(--primary)_12%,transparent)] text-primary hover:bg-[color-mix(in_srgb,var(--primary)_18%,transparent)]" : "border-border bg-popover text-foreground hover:bg-accent",
                )}
            >
                <Heart className={cn("h-4 w-4", favorited ? "fill-current" : "fill-none")} aria-hidden="true" />
                <span className="hidden sm:inline">{favorited ? "Favorited" : "Favorite"}</span>
            </button>

            <button
                type="button"
                onClick={handleDownload}
                disabled={downloading}
                aria-label="Download tier list as image"
                className="inline-flex h-11 min-w-11 cursor-pointer items-center justify-center gap-1.5 rounded-lg border border-border bg-popover px-3 font-medium font-sans text-foreground text-sm leading-none transition-colors hover:bg-accent disabled:cursor-wait disabled:opacity-60 sm:h-9 sm:min-w-0"
            >
                <Download className={cn("h-4 w-4", downloading && "animate-pulse")} aria-hidden="true" />
                <span className="hidden sm:inline">{downloading ? "Preparing…" : "Download"}</span>
            </button>

            <button
                type="button"
                onClick={handleShare}
                aria-label="Share this tier list"
                className="inline-flex h-11 min-w-11 cursor-pointer items-center justify-center gap-1.5 rounded-lg border border-border bg-popover px-3 font-medium font-sans text-foreground text-sm leading-none transition-colors hover:bg-accent sm:h-9 sm:min-w-0"
            >
                <Share2 className="h-4 w-4" aria-hidden="true" />
                <span className="hidden sm:inline">Share</span>
            </button>
        </div>
    );
}
