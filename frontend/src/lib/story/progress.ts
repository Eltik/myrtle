/**
 * Reading progress in localStorage under one key. Every field is COERCED on
 * read: a restored backup may be hand-edited, and a bad value must degrade to
 * "no progress" rather than throw inside a render.
 */

export const PROGRESS_KEY = "myrtle.story.progress";

export interface StoryPosition {
    halt: number;
    total: number;
    ts: number;
    choices: Record<number, string>;
}

/**
 * `v` is what the merge needs to tell a document's PROVENANCE apart, not a
 * format number: v1 and v2 have the same fields. In a v1 document `read` also
 * carries whatever the game's first import baked into it, and in a v2 document
 * `read` is the reader's own marks and nothing else. Everything this client
 * writes is v2; a v1 value only ever arrives from storage or from the account.
 */
export interface StoryProgress {
    v: 1 | 2;
    /**
     * Story id -> mark. A mark ABOVE 1 is the epoch ms the reader finished the
     * story HERE and is never withdrawn. A mark of exactly 1 is a legacy mark
     * of unknown provenance: the first import baked the game's whole union
     * into documents as `1`, indistinguishable from a reader's own, and a
     * corrected verdict can only withdraw what it can recognise, so a `1`
     * the game now refuses (`gameUnread`) is withdrawn on every merge,
     * whatever the document's version.
     */
    read: Record<string, number>;
    /**
     * Story id -> epoch ms the reader marked the story UNREAD by hand. This is
     * the one thing that outranks the game's verdict: a story the game says
     * was played stays read until the reader clears it here. Marking it read
     * again removes the entry. Absent on documents written before it existed.
     */
    unread?: Record<string, number>;
    pos: Record<string, StoryPosition>;
    last?: string;
}

export function emptyProgress(): StoryProgress {
    return { v: 2, read: {}, pos: {} };
}

function isRecord(v: unknown): v is Record<string, unknown> {
    return typeof v === "object" && v !== null && !Array.isArray(v);
}

function finiteInt(v: unknown, fallback: number): number {
    const n = typeof v === "number" ? v : typeof v === "string" ? Number(v) : Number.NaN;
    return Number.isFinite(n) ? Math.trunc(n) : fallback;
}

function coercePosition(v: unknown): StoryPosition | null {
    if (!isRecord(v)) return null;
    const halt = finiteInt(v.halt, -1);
    if (halt < 0) return null;
    const choices: Record<number, string> = {};
    if (isRecord(v.choices)) {
        for (const [k, val] of Object.entries(v.choices)) {
            const ordinal = finiteInt(k, -1);
            if (ordinal >= 0 && typeof val === "string") choices[ordinal] = val;
        }
    }
    return { halt, total: Math.max(0, finiteInt(v.total, 0)), ts: Math.max(0, finiteInt(v.ts, 0)), choices };
}

/**
 * Coerce anything into a valid `StoryProgress`; unknown or malformed input
 * yields an empty one.
 *
 * The incoming `v` is KEPT when it is 1, and anything else reads as 2. A v1
 * document is one the game's import may have written marks into, and the merge
 * withdraws those once; a document with no `v` at all is a hand-edited or
 * restored file whose marks nobody baked, so it is v2 and nothing is withdrawn
 * from it.
 */
export function coerceProgress(raw: unknown): StoryProgress {
    const out = emptyProgress();
    if (!isRecord(raw)) return out;
    if (raw.v === 1) out.v = 1;
    if (isRecord(raw.read)) {
        for (const [id, v] of Object.entries(raw.read)) {
            const mark = v === true || v === "1" ? 1 : typeof v === "number" ? v : Number.NaN;
            if (id !== "" && Number.isFinite(mark) && mark >= 1) out.read[id] = Math.floor(mark);
        }
    }
    if (isRecord(raw.unread)) {
        const unread: Record<string, number> = {};
        for (const [id, v] of Object.entries(raw.unread)) {
            if (id !== "" && typeof v === "number" && Number.isFinite(v) && v >= 1) unread[id] = Math.floor(v);
        }
        if (Object.keys(unread).length > 0) out.unread = unread;
    }
    if (isRecord(raw.pos)) {
        for (const [id, v] of Object.entries(raw.pos)) {
            const pos = coercePosition(v);
            if (id !== "" && pos) out.pos[id] = pos;
        }
    }
    if (typeof raw.last === "string" && raw.last !== "") out.last = raw.last;
    return out;
}

export function parseProgress(text: string): StoryProgress {
    try {
        return coerceProgress(JSON.parse(text));
    } catch {
        return emptyProgress();
    }
}

export function loadProgress(): StoryProgress {
    if (typeof window === "undefined") return emptyProgress();
    try {
        const raw = window.localStorage.getItem(PROGRESS_KEY);
        return raw == null ? emptyProgress() : parseProgress(raw);
    } catch {
        return emptyProgress();
    }
}

type ProgressListener = (p: StoryProgress) => void;

const writeListeners = new Set<ProgressListener>();

/**
 * Called after every `saveProgress`, with the document just written.
 *
 * This is the seam the account sync hangs off (`lib/story/sync.ts`). The reader
 * writes progress from four places and none of them knows an account exists;
 * routing those writes through a subscription keeps it that way, and keeps the
 * sync out of the render path entirely. A listener that throws is swallowed for
 * the same reason the storage write is: progress is a convenience, never a
 * failure.
 */
export function onProgressWritten(fn: ProgressListener): () => void {
    writeListeners.add(fn);
    return () => writeListeners.delete(fn);
}

/**
 * Called when ANOTHER TAB writes the document, with what it wrote.
 *
 * `saveProgress` notifies only its own tab: the reader open in one tab and
 * the library in another used to disagree until a reload. The browser's
 * `storage` event is the bridge, and it is a SEPARATE seam from
 * `onProgressWritten` on purpose: the writing tab already pushed that
 * document to the account, so the sync must not treat the echo as a write of
 * its own. A malformed value is coerced like any stored document.
 */
export function onProgressStorage(fn: ProgressListener): () => void {
    if (typeof window === "undefined") return () => {};
    const handler = (e: StorageEvent) => {
        if (e.key !== PROGRESS_KEY || e.newValue === null) return;
        fn(parseProgress(e.newValue));
    };
    window.addEventListener("storage", handler);
    return () => window.removeEventListener("storage", handler);
}

export function saveProgress(p: StoryProgress): void {
    if (typeof window === "undefined") return;
    try {
        window.localStorage.setItem(PROGRESS_KEY, JSON.stringify(p));
    } catch {
        // Storage full or blocked: progress is a convenience, never a failure.
    }
    for (const fn of writeListeners) {
        try {
            fn(p);
        } catch {
            // A broken listener must not cost the reader its saved position.
        }
    }
}

/**
 * Whether a story counts as read: the ONE predicate the library decides that
 * with.
 *
 * Two sources answer it and they are kept apart on purpose. `progress.read`
 * holds the marks this reader made, and `gameRead` holds the verdict the
 * Arknights client sends, which arrives with every pull and is never written
 * into the document. Keeping the game's half outside the document is what lets
 * the backend correct it: a mark baked into `read` survives every union merge,
 * and 94 stories that the game only UNLOCKED were baked that way once.
 */
export function isStoryRead(progress: StoryProgress, gameRead: ReadonlySet<string>, id: string): boolean {
    if (progress.read[id] !== undefined) return true;
    return gameRead.has(id) && progress.unread?.[id] === undefined;
}

/** Where a story's read state came from, for a control that can flip it. */
export type ReadSource = "own" | "game" | "cleared" | "none";

/**
 * `own`: the reader marked it here. `game`: only the game's verdict says so.
 * `cleared`: the game says read and the reader overrode it. `none`: nothing.
 */
export function readSourceOf(progress: StoryProgress, gameRead: ReadonlySet<string>, id: string): ReadSource {
    if (progress.read[id] !== undefined) return "own";
    if (!gameRead.has(id)) return "none";
    return progress.unread?.[id] === undefined ? "game" : "cleared";
}

export function withPosition(p: StoryProgress, storyId: string, pos: Omit<StoryPosition, "ts">, now: number = Date.now()): StoryProgress {
    return { ...p, pos: { ...p.pos, [storyId]: { ...pos, ts: now } }, last: storyId };
}

/**
 * Mark a story read. The document's `v` is carried through UNCHANGED: a v1
 * document that the reader adds a mark to is still one the game's import wrote
 * into, and upgrading it here would leave those baked marks beyond withdrawal.
 */
export function withRead(p: StoryProgress, storyId: string, now: number = Date.now()): StoryProgress {
    if (p.read[storyId] !== undefined && p.unread?.[storyId] === undefined) return p;
    // Never 1: that value is the legacy, withdrawable mark.
    const out: StoryProgress = { ...p, read: { ...p.read, [storyId]: Math.max(2, Math.floor(now)) } };
    if (p.unread?.[storyId] !== undefined) {
        const { [storyId]: _, ...rest } = p.unread;
        if (Object.keys(rest).length > 0) out.unread = rest;
        else delete out.unread;
    }
    return out;
}

/**
 * Mark a story UNREAD by hand: the reader's own mark goes, and an `unread`
 * entry outranks the game's verdict until the story is marked read again.
 * Idempotent on a story that is already neither marked nor game-read.
 */
export function withUnread(p: StoryProgress, storyId: string, now: number = Date.now()): StoryProgress {
    const { [storyId]: _dropped, ...read } = p.read;
    const unread = { ...(p.unread ?? {}), [storyId]: Math.max(1, Math.floor(now)) };
    return { ...p, read, unread };
}

export function withoutPosition(p: StoryProgress, storyId: string): StoryProgress {
    if (!(storyId in p.pos)) return p;
    const pos = { ...p.pos };
    delete pos[storyId];
    return { ...p, pos };
}
