import { createFileRoute } from "@tanstack/react-router";
import { validateMusicSearch } from "#/components/gallery/music/impl/route";
import { MusicGallery } from "#/components/gallery/music/MusicGallery";
import { musicIndexQueryOptions, sirenAlbumQueryOptions, sirenCatalogQueryOptions } from "#/lib/api/music";
import { metaT } from "#/lib/meta";
import { defaultOgURL, ogURL } from "#/lib/og";
import { buildMusicOgData, isMusicAlbumId } from "#/lib/og/impl/music";
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
    loaderDeps: ({ search }) => ({ album: search.album }),
    // One album record (a few KB) is read on the server too when the link opens
    // one: the social card of that album hashes it.
    loader: async ({ context: { queryClient, i18n }, deps }) => {
        const album = deps.album && isMusicAlbumId(deps.album) ? queryClient.ensureQueryData(sirenAlbumQueryOptions(deps.album)).catch(() => null) : null;
        if (typeof window !== "undefined") {
            void queryClient.prefetchQuery(sirenCatalogQueryOptions());
            void queryClient.prefetchQuery(musicIndexQueryOptions(i18n.gamedataServer));
        }
        return { album: await album };
    },
    head: ({ loaderData, match }) => {
        const t = metaT(match.context.i18n);
        // An album link unfurls as the album. A playlist is private to its owner and
        // an artist has no record of its own, so both keep the music preset.
        const album = loaderData?.album;
        const image = album ? ogURL("music", album.cid, buildMusicOgData(album)) : defaultOgURL("music", match.context.i18n);
        const { meta, links } = seo({
            title: t("music.title"),
            description: t("music.description"),
            path: "/gallery/music",
            image,
            locale: match.context.i18n?.locale,
        });
        return {
            meta: [{ charSet: "utf-8" }, { name: "viewport", content: "width=device-width, initial-scale=1" }, ...meta],
            links,
        };
    },
});
