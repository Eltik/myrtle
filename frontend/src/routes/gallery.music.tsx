import { createFileRoute } from "@tanstack/react-router";
import { validateMusicSearch } from "#/components/gallery/music/impl/route";
import { MusicGallery } from "#/components/gallery/music/MusicGallery";
import { musicIndexQueryOptions, sirenCatalogQueryOptions } from "#/lib/api/music";
import { metaT } from "#/lib/meta";
import { defaultOgURL } from "#/lib/og";
import { seo } from "#/lib/seo";

/**
 * `/gallery/music`: Monster Siren Records and the game's soundtrack. Both
 * catalogues are fetched on the client; see `MusicGallery`. The search params
 * (`?tab`, `?view`, `?playlist`, `?artist`, `?album`) are the page's state.
 *
 * The loader starts both reads in the browser without waiting on them: on a
 * link hover (the router preloads on intent) or alongside the route's chunk on
 * a click, instead of after the page mounts. The server skips them, so the
 * 1,161,730-byte index never lands in the HTML.
 */
export const Route = createFileRoute("/gallery/music")({
    component: MusicGallery,
    validateSearch: validateMusicSearch,
    loader: ({ context: { queryClient, i18n } }) => {
        if (typeof window === "undefined") return;
        void queryClient.prefetchQuery(sirenCatalogQueryOptions());
        void queryClient.prefetchQuery(musicIndexQueryOptions(i18n.gamedataServer));
    },
    head: ({ match }) => {
        const t = metaT(match.context.i18n);
        const { meta, links } = seo({
            title: t("music.title"),
            description: t("music.description"),
            path: "/gallery/music",
            image: defaultOgURL("stages", match.context.i18n),
            locale: match.context.i18n?.locale,
        });
        return {
            meta: [{ charSet: "utf-8" }, { name: "viewport", content: "width=device-width, initial-scale=1" }, ...meta],
            links,
        };
    },
});
