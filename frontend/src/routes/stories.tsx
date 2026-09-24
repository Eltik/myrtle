import { createFileRoute } from "@tanstack/react-router";
import { StoryLibrary } from "#/components/story/library/StoryLibrary";
import { storyIndexQueryOptions } from "#/lib/api/story";
import { metaT } from "#/lib/meta";
import { defaultOgURL } from "#/lib/og";
import { seo } from "#/lib/seo";

export const Route = createFileRoute("/stories")({
    component: RouteComponent,
    errorComponent: RootErrorComponent,
    loader: async ({ context }) => {
        await context.queryClient.ensureQueryData(storyIndexQueryOptions(context.i18n.gamedataServer));
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
