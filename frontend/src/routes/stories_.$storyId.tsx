import { useQuery } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import { BookView } from "#/components/story/export/BookView";
import type { messages as readerMessages } from "#/components/story/reader/reader.messages";
import { StoryReader } from "#/components/story/reader/StoryReader";
import { storyIndexQueryOptions, storyQueryOptions } from "#/lib/api/story";
import { useGamedataServer, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { metaT } from "#/lib/meta";
import { buildStoryOgData, defaultOgURL, localizedOgURL, storyOgId, warmOg } from "#/lib/og";
import { seo } from "#/lib/seo";
import type { BookGroup } from "#/lib/story/book/book";
import { useStoryProgressSync } from "#/lib/story/sync";
import type { StoryCategory } from "#/types/generated/StoryCategory";
import type { StoryEntry } from "#/types/generated/StoryEntry";
import type { StoryIndex } from "#/types/generated/StoryIndex";

interface Placement {
    entry: StoryEntry | null;
    groupName: string;
    category: StoryCategory;
    previous: StoryEntry | null;
    next: StoryEntry | null;
}

/** Where a story sits in the library: its group (or operator), and its neighbours by `sort`. */
export function placeStory(index: StoryIndex, storyId: string): Placement {
    for (const g of index.groups) {
        const at = g.stories.findIndex((s) => s.id === storyId);
        if (at < 0) continue;
        const sorted = [...g.stories].sort((a, b) => a.sort - b.sort);
        const i = sorted.findIndex((s) => s.id === storyId);
        if (g.category === "record") {
            const rec = index.records.find((r) => r.stories.some((s) => s.id === storyId));
            const rs = rec ? [...rec.stories].sort((a, b) => a.sort - b.sort) : sorted;
            const ri = rs.findIndex((s) => s.id === storyId);
            return { entry: sorted[i], groupName: rec?.name ?? g.name, category: "record", previous: rs[ri - 1] ?? null, next: rs[ri + 1] ?? null };
        }
        return { entry: sorted[i], groupName: g.name, category: g.category, previous: sorted[i - 1] ?? null, next: sorted[i + 1] ?? null };
    }
    return { entry: null, groupName: "", category: "sideContent", previous: null, next: null };
}

/**
 * The set of stories the reader's Export offers "the chapter" from: the
 * story's group, or for an operator record the operator's whole record set
 * (keyed by `charId`, which the illustrations route also takes).
 */
export function exportGroupOf(index: StoryIndex, storyId: string): BookGroup | null {
    const rec = index.records.find((r) => r.stories.some((s) => s.id === storyId));
    if (rec) return { id: rec.charId, name: rec.name, stories: rec.stories };
    return index.groups.find((g) => g.stories.some((s) => s.id === storyId)) ?? null;
}

export const Route = createFileRoute("/stories_/$storyId")({
    component: RouteComponent,
    errorComponent: RootErrorComponent,
    // `?halt=N` resumes at halt N (0-based); absent means the title card, or the
    // resume prompt. It is read as MISSING rather than falsy: `Number(null)` is
    // 0 and finite, which would pin every visit to halt 0.
    // `?view=book` shows the story as one printable document (`BookView`) instead of the stage.
    validateSearch: (search: Record<string, unknown>): { halt?: number; view?: "book" } => {
        const out: { halt?: number; view?: "book" } = {};
        if (search.view === "book") out.view = "book";
        const asNumber = (v: unknown): number => (typeof v === "number" ? v : typeof v === "string" && v.trim() !== "" ? Number(v) : Number.NaN);
        const halt = asNumber(search.halt);
        if (Number.isFinite(halt) && halt >= 0) out.halt = Math.floor(halt);
        return out;
    },
    loader: async ({ context, params }) => {
        const server = context.i18n.gamedataServer;
        const [index, script] = await Promise.all([context.queryClient.ensureQueryData(storyIndexQueryOptions(server)), context.queryClient.ensureQueryData(storyQueryOptions(params.storyId, server))]);
        const placement = placeStory(index, params.storyId);
        // The share card is built from the index alone, the same call the OG
        // handler makes, so the `?v=` hash here is the hash of what it draws.
        // A story the index does not list keeps the generic card.
        const og = buildStoryOgData(index, params.storyId, { server, source: context.i18n });
        if (og) warmOg("story", storyOgId(params.storyId, server), og, context.i18n?.locale);
        return { name: script?.name ?? placement.entry?.name ?? params.storyId, groupName: placement.groupName, hasScript: script !== null, og, server };
    },
    head: ({ loaderData, match, params }) => {
        const t = metaT(match.context.i18n);
        const name = loaderData ? (loaderData.groupName ? `${loaderData.groupName} · ${loaderData.name}` : loaderData.name) : params.storyId;
        const { meta, links } = seo({
            title: t("story.title", { name }),
            description: t("story.description", { name }),
            path: `/stories/${params.storyId}`,
            image: loaderData?.og ? localizedOgURL("story", storyOgId(params.storyId, loaderData.server), loaderData.og, match.context.i18n?.locale) : defaultOgURL("stages", match.context.i18n),
            type: loaderData?.og ? "article" : undefined,
            preloadImage: Boolean(loaderData?.og),
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
    const { storyId } = Route.useParams();
    const { halt, view } = Route.useSearch();
    const server = useGamedataServer();
    // Opening a story is a sync trigger, and the reader shows nothing for it:
    // the pull, the merge and the debounced push all run beside the reading.
    // The hook sits ABOVE the early returns below, because a loading story is
    // still a mount.
    useStoryProgressSync();
    const tr: TypedT<typeof readerMessages> = useT("story");
    const { data: index } = useQuery(storyIndexQueryOptions(server));
    const { data: script, isPending } = useQuery(storyQueryOptions(storyId, server));
    if (isPending || !index) return <p className="p-8 text-center text-muted-foreground">{tr("reader.loading")}</p>;
    const placement = placeStory(index, storyId);
    if (script === null || script === undefined) {
        return <p className="p-8 text-center text-muted-foreground">{placement.entry ? tr("reader.error.noScript") : tr("reader.error.notFound")}</p>;
    }
    const exportGroup = exportGroupOf(index, storyId);
    if (view === "book" && exportGroup) return <BookView script={script} group={exportGroup} server={server} />;
    return <StoryReader key={storyId} script={script} entry={placement.entry} groupName={placement.groupName} exportGroup={exportGroup} category={placement.category} previous={placement.previous} next={placement.next} initialHalt={halt} />;
}
