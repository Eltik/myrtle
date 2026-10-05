import { createFileRoute } from "@tanstack/react-router";
import { isLibraryTab, type LibraryTab, StoryLibrary } from "#/components/story/library/StoryLibrary";
import { storyIndexQueryOptions, storySpriteQueryOptions } from "#/lib/api/story";
import { metaT } from "#/lib/meta";
import { defaultOgURL } from "#/lib/og";
import { seo } from "#/lib/seo";

export const Route = createFileRoute("/stories")({
    component: RouteComponent,
    errorComponent: RootErrorComponent,
    // `?tab=` picks a library tab (absent is Browse) and `?sprite=<base>` opens
    // one character's sheet over the Characters tab. Both are read as MISSING
    // rather than falsy, and an unknown tab value is dropped.
    validateSearch: (search: Record<string, unknown>): { tab?: LibraryTab; sprite?: string } => {
        const out: { tab?: LibraryTab; sprite?: string } = {};
        if (isLibraryTab(search.tab) && search.tab !== "browse") out.tab = search.tab;
        if (typeof search.sprite === "string" && search.sprite.trim() !== "") out.sprite = search.sprite.trim();
        return out;
    },
    loaderDeps: ({ search }) => ({ sprite: search.sprite }),
    loader: async ({ context, deps }) => {
        const server = context.i18n.gamedataServer;
        // The one sheet a sprite link opens is prefetched beside the index; a
        // backend that predates the route answers null and the sheet says so.
        await Promise.all([context.queryClient.ensureQueryData(storyIndexQueryOptions(server)), deps.sprite ? context.queryClient.ensureQueryData(storySpriteQueryOptions(deps.sprite, server)).catch(() => null) : null]);
    },
    head: ({ match }) => {
        const t = metaT(match.context.i18n);
        const { meta, links } = seo({
            title: t("stories.title"),
            description: t("stories.description"),
            path: "/stories",
            image: defaultOgURL("stages", match.context.i18n),
            locale: match.context.i18n?.locale,
        });
        return {
            meta: [{ charSet: "utf-8" }, { name: "viewport", content: "width=device-width, initial-scale=1" }, ...meta],
            links,
        };
    },
});

function RootErrorComponent({ error }: { error: unknown }) {
    console.error("Router error:", error);
    return (
        <div style={{ padding: 20 }}>
            <h1>Something went wrong</h1>
            <pre style={{ color: "red" }}>{error instanceof Error ? error.message : JSON.stringify(error, null, 2)}</pre>
        </div>
    );
}

function RouteComponent() {
    return <StoryLibrary />;
}
