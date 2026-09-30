/**
 * A SCOPE OF STORIES AS ONE BOOK. `bookOf` resolves the scope against the
 * library index (one story, one group in `sort` order, or an explicit ordered
 * selection), walks each script with `sectionOf`, and groups the sections in
 * parts, one per group the scope touches.
 */
import type { StoryScript } from "#/types/generated/StoryScript";
import type { Book, BookOptions, BookScope, Part } from "./types";
import { sectionOf } from "./walk";

/** The slice of a library entry the book reads; `StoryEntry` and the library's `LibEntry` both fit. */
export interface BookEntry {
    id: string;
    name: string;
    code?: string;
    sort: number;
    avgTag?: string;
    hasScript: boolean;
    wordCount?: number;
}

/** The slice of a library group the book reads. */
export interface BookGroup {
    id: string;
    name: string;
    coverUrl?: string;
    bannerUrl?: string;
    titleImageUrl?: string;
    illustrationCount?: number;
    stories: readonly BookEntry[];
}

export interface BookIndex {
    groups: readonly BookGroup[];
}

export interface BookSource {
    index: BookIndex;
    /** Scripts by story id. A story in scope with no script here is left out of the book. */
    scripts: ReadonlyMap<string, StoryScript>;
    server: string;
}

export interface ScopedStory {
    group: BookGroup;
    entry: BookEntry;
}

function sorted(stories: readonly BookEntry[]): BookEntry[] {
    return [...stories].sort((a, b) => a.sort - b.sort);
}

/** The stories a scope covers, in book order, each with its group. Stories with no script are skipped. */
export function scopeStories(index: BookIndex, scope: BookScope): ScopedStory[] {
    const byId = new Map<string, ScopedStory>();
    for (const group of index.groups) for (const entry of group.stories) if (!byId.has(entry.id)) byId.set(entry.id, { group, entry });
    if (scope.kind === "group") {
        const group = index.groups.find((g) => g.id === scope.groupId);
        return group ? sorted(group.stories).flatMap((entry) => (entry.hasScript ? [{ group, entry }] : [])) : [];
    }
    if (scope.kind === "groups") {
        const byGroup = new Map(index.groups.map((g) => [g.id, g]));
        const seen = new Set<string>();
        return scope.ids.flatMap((groupId) => {
            const group = byGroup.get(groupId);
            if (!group) return [];
            return sorted(group.stories).flatMap((entry) => {
                // A story listed under two groups prints once, in the first.
                if (!entry.hasScript || seen.has(entry.id)) return [];
                seen.add(entry.id);
                return [{ group, entry }];
            });
        });
    }
    const ids = scope.kind === "story" ? [scope.storyId] : scope.ids;
    return ids.flatMap((id) => {
        const hit = byId.get(id);
        return hit?.entry.hasScript ? [hit] : [];
    });
}

/** FNV-1a over the ids, so a selection's identifier is the same every time it is exported. */
function hashIds(ids: readonly string[]): string {
    let h = 0x811c9dc5;
    for (const ch of ids.join(",")) {
        h ^= ch.charCodeAt(0);
        h = Math.imul(h, 0x01000193) >>> 0;
    }
    return h.toString(16).padStart(8, "0");
}

export function scopeId(scope: BookScope): string {
    if (scope.kind === "story") return scope.storyId;
    if (scope.kind === "group") return scope.groupId;
    if (scope.kind === "groups") return scope.id;
    return `selection-${hashIds(scope.ids)}`;
}

const LANGUAGES: Record<string, string> = { en: "en", jp: "ja", kr: "ko", cn: "zh-Hans", bili: "zh-Hans", tw: "zh-Hant" };

export function languageOf(server: string): string {
    return LANGUAGES[server] ?? "en";
}

/** `0-1 Isolated Island`, or the name alone when there is no code. */
export function storyLabel(entry: Pick<BookEntry, "code" | "name">): string {
    const code = entry.code?.trim();
    return code ? `${code} ${entry.name}` : entry.name;
}

export function bookTitle(stories: readonly ScopedStory[], scope: BookScope): string {
    const first = stories[0];
    if (!first) return "";
    if (scope.kind === "story") return `${first.group.name}: ${storyLabel(first.entry)}`;
    if (scope.kind === "groups") return scope.title;
    return first.group.name;
}

export function bookOf(source: BookSource, scope: BookScope, options: BookOptions): Book {
    const stories = scopeStories(source.index, scope).filter((s) => source.scripts.has(s.entry.id));
    const parts: Part[] = [];
    for (const { group, entry } of stories) {
        const script = source.scripts.get(entry.id);
        if (!script) continue;
        let part = parts.find((p) => p.id === group.id);
        if (!part) {
            part = { id: group.id, title: group.name, sections: [] };
            parts.push(part);
        }
        part.sections.push(sectionOf(script, entry, options));
    }
    const group = stories[0]?.group;
    const id = scopeId(scope);
    const description = parts
        .flatMap((p) => p.sections)
        .filter((s) => s.synopsis)
        .map((s) => `${storyLabel(s)}: ${s.synopsis}`)
        .join("\n\n");
    return {
        meta: {
            title: bookTitle(stories, scope),
            identifier: `urn:myrtle:story:${source.server}:${id}`,
            server: source.server,
            language: languageOf(source.server),
            scopeId: id,
            options,
            cover: { titleImageUrl: group?.titleImageUrl, bannerUrl: group?.bannerUrl, coverUrl: group?.coverUrl },
            partArt: Object.fromEntries(stories.map((s) => [s.group.id, s.group.bannerUrl ?? s.group.coverUrl])),
            description,
        },
        parts,
    };
}

/** Characters no file system takes. Control characters are handled by `controlFree`. */
const UNSAFE = /[\\/:*?"<>|]+/g;

function controlFree(value: string): string {
    return [...value].map((ch) => (ch.charCodeAt(0) < 0x20 || ch.charCodeAt(0) === 0x7f ? " " : ch)).join("");
}

/**
 * `<Group name> · <first code>–<last code>.<ext>`, sanitised. The range is
 * dropped when no story in the book has a code, and collapses to one code
 * when the book starts and ends on the same operation. A one-story book
 * names the story instead: `<Group name> · <code> <title>`.
 */
export function bookFileName(book: Book, ext: string): string {
    return fileNameOf(
        book.parts.map((p) => ({ title: p.title, entries: p.sections })),
        book.meta.title,
        ext,
    );
}

/** The same name before anything is fetched, from the scope alone: what a save picker proposes. */
export function scopeFileName(index: BookIndex, scope: BookScope, ext: string): string {
    const stories = scopeStories(index, scope);
    const parts: { title: string; entries: BookEntry[] }[] = [];
    for (const { group, entry } of stories) {
        const last = parts[parts.length - 1];
        if (last && last.title === group.name) last.entries.push(entry);
        else parts.push({ title: group.name, entries: [entry] });
    }
    return fileNameOf(parts, bookTitle(stories, scope), ext);
}

function fileNameOf(parts: readonly { title: string; entries: readonly Pick<BookEntry, "code" | "name">[] }[], title: string, ext: string): string {
    const entries = parts.flatMap((p) => p.entries);
    const codes = entries.map((s) => s.code?.trim()).filter((c): c is string => Boolean(c));
    // One group: its name. Several (an arc, a storyline, a range): the book's title.
    const head = parts.length > 1 ? title : (parts[0]?.title ?? title);
    let stem = head;
    if (entries.length === 1) stem = `${head} · ${storyLabel(entries[0])}`;
    else if (codes.length > 0) {
        const first = codes[0];
        const last = codes[codes.length - 1];
        stem = `${head} · ${first === last ? first : `${first}–${last}`}`;
    }
    const clean = controlFree(stem).replace(UNSAFE, " ").replace(/\s+/g, " ").trim().slice(0, 150) || "story";
    return `${clean}.${ext}`;
}
