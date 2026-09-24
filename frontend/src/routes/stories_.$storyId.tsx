import { useQuery } from "@tanstack/react-query";
import { createFileRoute } from "@tanstack/react-router";
import type { messages as readerMessages } from "#/components/story/reader/reader.messages";
import { StoryReader } from "#/components/story/reader/StoryReader";
import { storyIndexQueryOptions, storyQueryOptions } from "#/lib/api/story";
import { useGamedataServer, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { metaT } from "#/lib/meta";
import { defaultOgURL } from "#/lib/og";
import { seo } from "#/lib/seo";
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

export const Route = createFileRoute("/stories_/$storyId")({
    component: RouteComponent,
    errorComponent: RootErrorComponent,
    // `?halt=N` resumes at halt N (0-based); absent means the title card, or the
    // resume prompt. `?ratio=` overrides the Playback speed for a debugging run
    // and `?legacyclamp=1` restores the pre-parity 1.5 s clamps. The capture
    // pass adds four: `?mask=1` paints the client's black outside the 16:9
    // canvas box instead of the extend fill, `?canvas=stretch` restores the
    // pre-capture mapping where the canvas width was the stage width,
    // `?plate=0` draws every body at the 1024-at-203 slot template instead of
    // its own wire plate, and `?firstright=1` puts the legacy `[character]`
    // pair back with the first name on the right. `?video=0` is the kill
    // switch for the cutscene layer: every `[Video]` is skipped and the halt
    // count is the one the reader had before cutscenes shipped. Each is read as MISSING
    // rather than falsy: `Number(null)` is 0 and finite, which would pin the
    // ratio at zero and make every scene instant.
    validateSearch: (search: Record<string, unknown>): { halt?: number; ratio?: number; legacyclamp?: boolean; mask?: boolean; canvas?: "stretch"; plate?: boolean; firstright?: boolean; video?: boolean } => {
        const out: { halt?: number; ratio?: number; legacyclamp?: boolean; mask?: boolean; canvas?: "stretch"; plate?: boolean; firstright?: boolean; video?: boolean } = {};
        const asNumber = (v: unknown): number => (typeof v === "number" ? v : typeof v === "string" && v.trim() !== "" ? Number(v) : Number.NaN);
        // The router's own search parser turns `?mask=1` into the NUMBER 1, so
        // a string compare alone reads every switch as absent.
        const asFlag = (v: unknown): boolean => v === true || v === 1 || v === "1" || v === "true";
        const asOff = (v: unknown): boolean => v === false || v === 0 || v === "0" || v === "false";
        const halt = asNumber(search.halt);
        if (Number.isFinite(halt) && halt >= 0) out.halt = Math.floor(halt);
        const ratio = asNumber(search.ratio);
        if (Number.isFinite(ratio) && ratio >= 0 && ratio <= 20) out.ratio = ratio;
        if (asFlag(search.legacyclamp)) out.legacyclamp = true;
        if (asFlag(search.mask)) out.mask = true;
        if (String(search.canvas ?? "").toLowerCase() === "stretch") out.canvas = "stretch";
        // `?plate=0` is an OFF switch, so only an explicit zero counts.
        if (asOff(search.plate)) out.plate = false;
        if (asFlag(search.firstright)) out.firstright = true;
        // `?video=0` is an OFF switch, so only an explicit zero counts.
        if (asOff(search.video)) out.video = false;
        return out;
    },
    loader: async ({ context, params }) => {
        const server = context.i18n.gamedataServer;
        const [index, script] = await Promise.all([context.queryClient.ensureQueryData(storyIndexQueryOptions(server)), context.queryClient.ensureQueryData(storyQueryOptions(params.storyId, server))]);
        const placement = placeStory(index, params.storyId);
        return { name: script?.name ?? placement.entry?.name ?? params.storyId, groupName: placement.groupName, hasScript: script !== null };
    },
    head: ({ loaderData, match, params }) => {
        const t = metaT(match.context.i18n);
        const name = loaderData ? (loaderData.groupName ? `${loaderData.groupName} · ${loaderData.name}` : loaderData.name) : params.storyId;
        const { meta, links } = seo({
            title: t("story.title", { name }),
            description: t("story.description", { name }),
            path: `/stories/${params.storyId}`,
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
    const { storyId } = Route.useParams();
    const { halt, ratio, legacyclamp, mask, canvas, plate, firstright, video } = Route.useSearch();
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
    return (
        <StoryReader
            key={storyId}
            script={script}
            entry={placement.entry}
            groupName={placement.groupName}
            category={placement.category}
            previous={placement.previous}
            next={placement.next}
            initialHalt={halt}
            ratioOverride={ratio}
            legacyClamp={legacyclamp}
            mask={mask}
            canvasMode={canvas === "stretch" ? "stretch" : "box"}
            plateFromWire={plate}
            firstNameRight={firstright}
            video={video}
        />
    );
}
