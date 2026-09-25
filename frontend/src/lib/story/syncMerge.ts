/**
 * THE MERGE RULES AND THE CONFLICT PROMPT: what two progress documents add up
 * to, and the one case where that is not the reader's answer.
 *
 * Everything here is PURE. `sync.ts` owns the pull, the push, the debounce and
 * the store; this file owns the arithmetic those triggers run, which is what
 * lets the rules be tested without a network, a timer or a signed-in account.
 *
 * The shape of the feature is a MERGE, not an owner. Neither side is
 * authoritative: a phone read three chapters offline while a laptop read two,
 * and both are true. The union is therefore the default and the prompt is the
 * exception, which is why {@link conflictOf} spends most of its length ruling
 * conflicts OUT.
 */

import type { StoryPosition, StoryProgress } from "#/lib/story/progress";

/** One side of a conflict, counted for the comparison the reader is shown. */
export interface SyncSideSummary {
    /** Read marks the side carries. */
    marks: number;
    /** Stories with a saved position. */
    positions: number;
    /** The story the side calls the last one read, or null. */
    last: string | null;
    /** The newest stamp anywhere on the side (a read mark, a clear, or a position), 0 when it carries none. */
    newest: number;
}

/**
 * What each of the three outcomes costs, in the numbers the buttons quote.
 *
 * `drops` is what a REPLACEMENT loses, counted as ids present on the losing
 * side and absent from the winning one. A position that both sides carry with
 * different halts is not a drop: it is overwritten, and the reader keeps a
 * place in that story either way.
 */
export interface SyncConflictSummary {
    local: SyncSideSummary;
    remote: SyncSideSummary;
    /** What a merge would hold. */
    merged: { marks: number; positions: number };
    /** What "use account" takes off this browser. */
    dropsLocal: { marks: number; positions: number };
    /** What "keep browser" takes off the account. */
    dropsRemote: { marks: number; positions: number };
    /** The side with the newer stamp, or null when the two are level. */
    newer: "local" | "remote" | null;
}

/** The three ways out of a conflict. `merge` is what the sync does when nothing asks otherwise. */
export type SyncChoice = "merge" | "account" | "browser";

/**
 * What to do the NEXT time the two sides disagree, kept in localStorage under
 * {@link SYNC_POLICY_KEY}.
 *
 * `merge` is the DEFAULT, which is to say the sync keeps syncing by itself and
 * a reader who never opens this setting is never interrupted: the union loses
 * nothing, and losing nothing is why it can be applied without being asked.
 * `ask` is the only value that ever stops a pull, and a reader picks it to see
 * the comparison instead. `account` and `browser` are the two replacements
 * applied silently, which is what a reader who has already decided which
 * device is the real one wants.
 */
export type SyncPolicy = "ask" | SyncChoice;

export const SYNC_POLICY_KEY = "myrtle.story.sync.policy";

export const SYNC_POLICIES: readonly SyncPolicy[] = ["ask", "merge", "account", "browser"] as const;

/** What a browser that has never been told otherwise does: sync by itself, union both sides, ask nothing. */
export const DEFAULT_SYNC_POLICY: SyncPolicy = "merge";

/**
 * Merge two progress documents, withdrawing the game's mis-imported marks on
 * the way through.
 *
 * `read` is a UNION over the two documents and nothing else. The game's own
 * verdict is NOT written into it: `gameRead` is read beside the document at
 * every use (`isStoryRead`), because a mark that lands in `read` is
 * indistinguishable from one the reader made and no later merge can take it
 * back. The first import did bake its union in, on the server and in every
 * browser that pulled, and the ids it should never have marked are exactly
 * `gameUnread`: stories the game's Archive lists that nothing ever played.
 *
 * Withdrawing them is VERSIONED rather than heuristic, and it has to be.
 * Finishing a story deletes its position, so a story the reader read to the end
 * carries a read mark and no other evidence, byte for byte what a baked mark
 * looks like. The only thing that separates them is which client wrote the
 * document: a v1 document is one the old bake could have touched, so every
 * `gameUnread` id is dropped from its `read` once; a v2 document was written by
 * a client that never baked, so it is left alone and a reader who really did
 * mark one of those stories keeps it. The result is always v2, which is why
 * `canonical` carries `v`: the upgrade of a v1 document that loses no ids is
 * still a change, and it has to reach the account.
 *
 * `pos` is per story, and the entry with the larger `ts` wins. A tie keeps the
 * LOCAL entry, because a tie is the same write seen twice and the local copy is
 * the one already on screen. The winner carries the larger `reach` of the two
 * when both walked the same choices ({@link withReachOf}).
 *
 * `last` is the newer of the two, judged by the `ts` of the position each one
 * names in the MERGED `pos`. A `last` naming a story with no position scores 0
 * and loses to any that does; when only one side carries a `last`, it wins by
 * default.
 */
export function merge(local: StoryProgress, remote: StoryProgress, gameUnread: readonly string[] = []): StoryProgress {
    const read: Record<string, number> = { ...upgraded(remote, gameUnread), ...upgraded(local, gameUnread) };
    // A hand-cleared mark is a write like any other and the NEWER write wins:
    // a `read` stamp and an `unread` stamp on the same id are compared as
    // times, the loser is dropped, and a legacy `1` loses to any clear.
    const unread: Record<string, number> = { ...(remote.unread ?? {}), ...(local.unread ?? {}) };
    for (const [id, at] of Object.entries(unread)) {
        const mark = read[id];
        if (mark === undefined) continue;
        if (mark > at) delete unread[id];
        else delete read[id];
    }
    const pos: Record<string, StoryPosition> = { ...remote.pos };
    for (const [id, entry] of Object.entries(local.pos)) {
        const other = pos[id];
        if (!other) pos[id] = entry;
        else pos[id] = entry.ts >= other.ts ? withReachOf(entry, other) : withReachOf(other, entry);
    }
    const out: StoryProgress = { v: 2, read, pos };
    if (Object.keys(unread).length > 0) out.unread = unread;
    const last = pickLast(local.last, remote.last, pos);
    if (last !== undefined) out.last = last;
    return out;
}

/**
 * One side's read marks with the old import's mistakes dropped.
 *
 * The withdrawal is keyed on the MARK, not the document version: a legacy mark
 * (`1`) on an id the game now refuses can only have come from the first
 * import, and the version stamp proved too coarse (the verdict was corrected
 * twice in one day, and a document stamped v2 by the first correction still
 * held 212 marks the second one refused). A timestamped mark is the reader's
 * own and is never withdrawn, whatever the game says.
 */
function upgraded(side: StoryProgress, gameUnread: readonly string[]): Record<string, number> {
    if (gameUnread.length === 0) return side.read;
    const read = { ...side.read };
    for (const id of gameUnread) if (read[id] === 1) delete read[id];
    return read;
}

/**
 * The winning position of one story, carrying the LARGER reach of the two.
 *
 * `halt` is the winner's, by the `ts` rule above. The reach is the furthest
 * halt either side read, but only when the two walked the same path: their
 * choices agree on every decision both recorded. Then the loser's choices past
 * the winner's are carried too, because the loser's reach was read on them
 * and a walk out to it needs them. Two positions that took different options
 * are two paths, and the winner's reach stands alone.
 */
function withReachOf(winner: StoryPosition, loser: StoryPosition): StoryPosition {
    const agree = Object.entries(loser.choices).every(([k, v]) => {
        const mine = winner.choices[Number(k)];
        return mine === undefined || mine === v;
    });
    if (!agree) return winner;
    const reach = Math.max(winner.reach ?? winner.halt, loser.reach ?? loser.halt);
    if (reach <= (winner.reach ?? winner.halt)) return winner;
    return { ...winner, choices: { ...loser.choices, ...winner.choices }, reach };
}

function pickLast(mine: string | undefined, theirs: string | undefined, pos: Record<string, StoryPosition>): string | undefined {
    if (mine === undefined) return theirs;
    if (theirs === undefined || theirs === mine) return mine;
    return (pos[theirs]?.ts ?? 0) > (pos[mine]?.ts ?? 0) ? theirs : mine;
}

/**
 * A stable encoding of a document, so two of them can be compared by value.
 *
 * `JSON.stringify` will not do: key order follows insertion order, and a merge
 * inserts in a different order than a reader does, so two identical documents
 * would compare unequal and the sync would PUT on every trigger forever.
 *
 * The engine compares against it on every pull and push (`acknowledged`), which
 * is why it is exported rather than private to {@link sameProgress}.
 *
 * `v` is part of it. Two documents with the same marks but different versions
 * are NOT the same document: the v1 one still claims its marks came partly from
 * the game's first import, and the account has to be told that they no longer
 * do, or every later pull would re-upgrade it and push nothing.
 */
export function canonical(p: StoryProgress): string {
    const positions = Object.keys(p.pos)
        .sort()
        .map((id) => {
            const e = p.pos[id];
            const choices = Object.keys(e.choices)
                .map(Number)
                .sort((a, b) => a - b)
                .map((k) => [k, e.choices[k]]);
            // `reach` rides at the END and only when present, so a position
            // written before it existed encodes exactly as it always did.
            return e.reach === undefined ? [id, e.halt, e.total, e.ts, choices] : [id, e.halt, e.total, e.ts, choices, e.reach];
        });
    const cleared = Object.keys(p.unread ?? {}).sort();
    return JSON.stringify([p.v, Object.keys(p.read).sort(), positions, p.last ?? null, cleared]);
}

/** Whether two documents carry the same progress, whatever order their keys sit in. */
export function sameProgress(a: StoryProgress, b: StoryProgress): boolean {
    return canonical(a) === canonical(b);
}

// ---------------------------------------------------------------------------
// Conflicts: the two sides disagree and neither of them is obviously right.
// ---------------------------------------------------------------------------

/**
 * One side as the merge would read it, alone: stamped v2, with the old
 * import's refused marks withdrawn, and nothing of the other side in it.
 *
 * This is what makes the prompt HONEST. A v1 document and a v2 document with
 * the same marks are not the same document, and neither are two documents that
 * differ only in marks the backend has since refused; in both cases the reader
 * has nothing to decide, and comparing the upgraded forms is what says so. The
 * conflict state carries these forms rather than the raw ones, so every number
 * the buttons quote is a number of the document that choice actually writes.
 */
function upgradedDoc(side: StoryProgress, gameUnread: readonly string[]): StoryProgress {
    const out: StoryProgress = { v: 2, read: upgraded(side, gameUnread), pos: side.pos };
    if (side.unread !== undefined && Object.keys(side.unread).length > 0) out.unread = side.unread;
    if (side.last !== undefined) out.last = side.last;
    return out;
}

/** Whether a document says anything at all: one read mark or one saved position is enough. */
function hasContent(p: StoryProgress): boolean {
    return Object.keys(p.read).length > 0 || Object.keys(p.pos).length > 0;
}

/** The newest stamp on a side, over read marks, hand clears and positions alike. A legacy `1` mark carries no time and contributes none. */
function newestStamp(p: StoryProgress): number {
    let newest = 0;
    for (const mark of Object.values(p.read)) if (mark > 1 && mark > newest) newest = mark;
    for (const at of Object.values(p.unread ?? {})) if (at > newest) newest = at;
    for (const entry of Object.values(p.pos)) if (entry.ts > newest) newest = entry.ts;
    return newest;
}

function sideSummary(p: StoryProgress): SyncSideSummary {
    return { marks: Object.keys(p.read).length, positions: Object.keys(p.pos).length, last: p.last ?? null, newest: newestStamp(p) };
}

/** How many ids `mine` holds that `theirs` does not, over the read marks and the positions separately. */
function dropped(mine: StoryProgress, theirs: StoryProgress): { marks: number; positions: number } {
    const marks = Object.keys(mine.read).filter((id) => theirs.read[id] === undefined).length;
    const positions = Object.keys(mine.pos).filter((id) => theirs.pos[id] === undefined).length;
    return { marks, positions };
}

/**
 * Whether the two sides need the reader, and what the three outcomes cost when
 * they do. Null means MERGE, which is the answer in every case but one.
 *
 * Three things disqualify a conflict, and they are all the same thing said
 * three ways: there is nothing to decide. A side with no marks and no
 * positions cannot lose anything. Two sides that differ only by the v1 -> v2
 * upgrade or by a mark the backend has withdrawn are the same reading history
 * written twice. And a side the merge reproduces exactly is a subset of the
 * other, so merging is the strictly larger choice and no reader would pick
 * against it.
 */
export function conflictOf(local: StoryProgress, remote: StoryProgress, gameUnread: readonly string[]): { local: StoryProgress; remote: StoryProgress; summary: SyncConflictSummary } | null {
    if (!hasContent(local) || !hasContent(remote)) return null;
    if (sameProgress(local, remote)) return null;
    const mine = upgradedDoc(local, gameUnread);
    const theirs = upgradedDoc(remote, gameUnread);
    if (sameProgress(mine, theirs)) return null;
    const merged = merge(local, remote, gameUnread);
    if (sameProgress(merged, mine) || sameProgress(merged, theirs)) return null;
    const mineSummary = sideSummary(mine);
    const theirsSummary = sideSummary(theirs);
    const summary: SyncConflictSummary = {
        local: mineSummary,
        remote: theirsSummary,
        merged: { marks: Object.keys(merged.read).length, positions: Object.keys(merged.pos).length },
        dropsLocal: dropped(mine, theirs),
        dropsRemote: dropped(theirs, mine),
        newer: mineSummary.newest === theirsSummary.newest ? null : mineSummary.newest > theirsSummary.newest ? "local" : "remote",
    };
    return { local: mine, remote: theirs, summary };
}
