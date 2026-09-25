/**
 * A story's social card, as data.
 *
 * The text is built twice, in two places that share nothing but the library
 * index: on the page, by the story route's `head()`, to hash the card into its
 * `og:image` URL; and on the server, by the `story` OG handler, when that URL
 * is requested and the PNG is not cached. Both go through
 * {@link buildStoryOgData} so the hash and the picture can never disagree about
 * what the card says.
 *
 * It reads the library INDEX only, never the script: the index already carries
 * the name, the operation code, the phase, the word count and the cutscene
 * flag, and it is the document the page has loaded anyway.
 */

import { DEFAULT_GAMEDATA_SERVER, type GamedataServer, isGamedataServer } from "#/lib/api/gamedata";
import { DEFAULT_LOCALE, formatMessage, type MessageValues, sourceMessage } from "#/lib/i18n";
import { fullMessageKey } from "#/lib/i18n/messages";
import type { IMetaSource } from "#/lib/meta";
import { DEFAULT_WPM, humanTime, minutesFor } from "#/lib/story/reading";
import { RARITY_HEX } from "#/lib/utils";
import type { StoryCategory } from "#/types/generated/StoryCategory";
import type { StoryEntry } from "#/types/generated/StoryEntry";
import type { StoryIndex } from "#/types/generated/StoryIndex";
import { messages } from "./story.messages";

/** Which picture the card's visual side draws, in the order it is tried. */
export type StoryOgArtKind = "banner" | "cover" | "operator" | "none";

export interface IStoryOgStat {
    label: string;
    value: string;
}

export interface IStoryOgData {
    /** The story's own name, the card's focal line. */
    name: string;
    /** The chapter or event the story belongs to; the OPERATOR's name on a record. */
    groupName: string;
    /** `Main story`, `Side stories`, ... the library's category label, resolved in the card's locale. */
    categoryLabel: string;
    /** `EPISODE 14` on a mainline chapter, absent elsewhere. */
    chapterTag?: string;
    /** The operation code (`0-1`, `PA-ST-1`), absent on records and interludes. */
    code?: string;
    /** `Before Operation`, `After Operation`, `Interlude`, in the game server's own language. */
    phase?: string;
    /** Set when the story plays a `[Video]`; the resolved chip text. */
    cutsceneLabel?: string;
    /** Set when the game lists the story but ships no script; the resolved chip text. */
    noScriptLabel?: string;
    stats: IStoryOgStat[];
    accent: string;
    /** The big faint mark drawn when there is no art: the chapter tag, the operation code's prefix, or the group id. */
    watermark: string;
    artKind: StoryOgArtKind;
    /** Backend-relative asset path of the chosen art, hashed so a new plate re-renders the card. */
    artPath?: string;
    /** Crop of a cover-fit plate. Mainline key visuals are posters with the title typeset low, so they bias up. */
    artPosition: string;
    /** The gamedata server the index came from: it picks the asset tree and is part of the cache key. */
    server: GamedataServer;
    /** The art as a data URI, filled by the handler only. Not hashed: `artPath` is. */
    artURL?: string;
}

/** One colour per shelf, drawn as the code pill and the bottom strip. A record takes its operator's rarity colour instead. */
const CATEGORY_ACCENT: Record<StoryCategory, string> = {
    main: "#ec6f5d",
    side: "#5aa9d9",
    vignette: "#9b73d4",
    is: "#5dbf86",
    reclamation: "#e0834a",
    record: "#d8b54a",
    sideContent: "#8a8a8a",
};

/**
 * One card message: the locale's translation, then the extracted English, then
 * the English written in `story.messages.ts`. The last step is what keeps a
 * card rendered before `i18n:extract` has run from printing a raw key, and it
 * resolves to the same words the extracted catalog will, so the hash does not
 * move when the catalog catches up.
 */
function cardMessage(key: keyof typeof messages, source: IMetaSource | null | undefined, values?: MessageValues): string {
    const full = fullMessageKey("meta", key);
    const message = source?.messages[full] ?? sourceMessage(full) ?? messages[key].text;
    return formatMessage(message, source?.locale ?? DEFAULT_LOCALE, values);
}

/** The library's category label, `story.category.*`, which the reader and the library already translate. */
function categoryLabel(category: StoryCategory, source: IMetaSource | null | undefined): string {
    const full = `story.category.${category}`;
    return source?.messages[full] ?? sourceMessage(full) ?? category;
}

/**
 * The card's id in the OG route. The default server's stories keep their bare
 * id, so an English card URL is just `/api/og/story/{storyId}`; any other
 * server is prefixed (`cn:act51side_...`), because a story id alone does not
 * say whose index it came from and the CN and EN names differ.
 */
export function storyOgId(storyId: string, server: string = DEFAULT_GAMEDATA_SERVER): string {
    return !isGamedataServer(server) || server === DEFAULT_GAMEDATA_SERVER ? storyId : `${server}:${storyId}`;
}

/** The inverse of {@link storyOgId}. `explicit` is false for a bare id, which the handler may resolve on a second server. */
export function parseStoryOgId(id: string): { server: GamedataServer; storyId: string; explicit: boolean } {
    const at = id.indexOf(":");
    if (at > 0) {
        const prefix = id.slice(0, at);
        if (isGamedataServer(prefix)) return { server: prefix, storyId: id.slice(at + 1), explicit: true };
    }
    return { server: DEFAULT_GAMEDATA_SERVER, storyId: id, explicit: false };
}

function byOrder(a: StoryEntry, b: StoryEntry): number {
    return a.sort - b.sort;
}

/** `PA-ST-1` -> `PA`, `14-22` -> `14`. Empty when there is no hyphenated code. */
function codePrefix(code: string | undefined): string {
    const trimmed = code?.trim();
    if (!trimmed?.includes("-")) return "";
    return trimmed.split("-")[0]?.trim().toUpperCase() ?? "";
}

export interface IBuildStoryOgArgs {
    server?: string;
    /** The locale's catalog, `match.context.i18n` on the page and `metaSourceForLocale` in the handler. */
    source?: IMetaSource | null;
    wpm?: number;
}

/**
 * The card for one story, or `null` when the index does not list it (the page
 * then keeps the generic card).
 *
 * THE ART is the group's key visual first and its cover second, the order
 * `plateSource` in the library uses, because the banner is the authored plate
 * and a `background` cover is only the chapter's first backdrop (`bg_black.png`
 * on four mainline chapters). A RECORD draws its operator instead: its group
 * carries no banner and its cover is a derived backdrop, while the operator is
 * who the story is about.
 */
export function buildStoryOgData(index: StoryIndex, storyId: string, args: IBuildStoryOgArgs = {}): IStoryOgData | null {
    const server: GamedataServer = isGamedataServer(args.server) ? args.server : DEFAULT_GAMEDATA_SERVER;
    const source = args.source;
    const group = index.groups.find((g) => g.stories.some((s) => s.id === storyId));
    if (!group) return null;

    const record = group.category === "record" ? index.records.find((r) => r.stories.some((s) => s.id === storyId)) : undefined;
    const siblings = [...(record?.stories ?? group.stories)].sort(byOrder);
    const at = siblings.findIndex((s) => s.id === storyId);
    const entry = siblings[at] ?? group.stories.find((s) => s.id === storyId);
    if (!entry) return null;

    const code = entry.code?.trim() || undefined;
    const phase = entry.avgTag?.trim() || undefined;
    const chapterTag = group.category === "main" ? group.zone?.nameThird?.trim() || (typeof group.chapterNumber === "number" ? `EPISODE ${String(group.chapterNumber).padStart(2, "0")}` : undefined) : undefined;

    const stats: IStoryOgStat[] = [];
    if (siblings.length > 1 && at >= 0) stats.push({ label: cardMessage("og.story.position", source), value: `${at + 1} / ${siblings.length}` });
    if (entry.hasScript && entry.wordCount > 0) {
        stats.push({ label: cardMessage("og.story.words", source), value: entry.wordCount.toLocaleString("en-US") });
        stats.push({ label: cardMessage("og.story.reading", source), value: `~${humanTime(minutesFor(entry.wordCount, args.wpm ?? DEFAULT_WPM))}` });
    }

    let artKind: StoryOgArtKind = "none";
    let artPath: string | undefined;
    if (record) {
        artKind = "operator";
        artPath = `/textures/chararts/${record.charId}/${record.charId}_1.png`;
    } else if (group.bannerUrl) {
        artKind = "banner";
        artPath = group.bannerUrl;
    } else if (group.coverUrl) {
        artKind = "cover";
        artPath = group.coverUrl;
    }

    return {
        name: entry.name.trim() || storyId,
        groupName: record?.name ?? group.name,
        categoryLabel: categoryLabel(group.category, source),
        chapterTag,
        code,
        phase,
        cutsceneLabel: entry.hasVideo ? cardMessage("og.story.cutscene", source) : undefined,
        noScriptLabel: entry.hasScript ? undefined : cardMessage("og.story.noScript", source),
        stats,
        accent: record ? (RARITY_HEX[record.rarity] ?? CATEGORY_ACCENT.record) : CATEGORY_ACCENT[group.category],
        watermark: chapterTag?.replace(/^EPISODE\s*/i, "EP ") || codePrefix(code) || group.id.toUpperCase(),
        artKind,
        artPath,
        artPosition: group.category === "main" ? "50% 20%" : "50% 50%",
        server,
    };
}

/** Everything the card draws, minus the data URI. The handler and the page both hash through this. */
export function storyOgHashParts(data: IStoryOgData): unknown[] {
    return [
        data.name,
        data.groupName,
        data.categoryLabel,
        data.chapterTag ?? "",
        data.code ?? "",
        data.phase ?? "",
        data.cutsceneLabel ?? "",
        data.noScriptLabel ?? "",
        data.stats.map((s) => `${s.label}=${s.value}`).join("|"),
        data.accent,
        data.watermark,
        data.artKind,
        data.artPath ?? "",
        data.artPosition,
        data.server,
    ];
}
