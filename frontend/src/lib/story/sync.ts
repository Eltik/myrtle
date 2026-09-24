import { useStore } from "@tanstack/react-store";
import { useEffect, useSyncExternalStore } from "react";
import { getStoryProgressFn, importStoryProgressFn, putStoryProgressFn, type SyncFailure } from "#/lib/api/storyProgress";
import { authStore } from "#/lib/auth/store";
import { coerceProgress, emptyProgress, loadProgress, onProgressWritten, type StoryProgress, saveProgress } from "#/lib/story/progress";
import { canonical, conflictOf, DEFAULT_SYNC_POLICY, merge, SYNC_POLICIES, SYNC_POLICY_KEY, type SyncChoice, type SyncConflictSummary, type SyncPolicy } from "./syncMerge";

/**
 * Reading progress on the account, on top of the localStorage document.
 *
 * The shape of the feature is a MERGE, not an owner. Neither side is
 * authoritative: a phone read three chapters offline while a laptop read two,
 * and both are true. So every trigger pulls the remote document, merges it
 * against the local one by the rules below, and writes the result back to
 * whichever side does not already hold it.
 *
 * Nothing here is allowed to block reading. Every failure - offline, signed
 * out, a backend that does not carry the route yet - leaves the local document
 * untouched, shows a state in the library's Progress tab, and retries on the
 * next trigger.
 */

/** How long a local write waits before it is pushed, in milliseconds. */
export const PUSH_DEBOUNCE_MS = 2000;

export type SyncState =
    /** Signed out: nothing is synced and nothing is attempted. */
    | { kind: "off" }
    /** In agreement with the account as of `at` (epoch ms), or never synced yet when null. */
    | { kind: "idle"; at: number | null }
    | { kind: "syncing" }
    /**
     * Both sides carry progress, they disagree, and a merge is no longer the
     * obvious answer. Nothing has been written on either side: `local` and
     * `remote` are the two candidate documents, upgraded but not merged, and
     * the reader picks one of the three outcomes in `summary`.
     */
    | { kind: "conflict"; local: StoryProgress; remote: StoryProgress; summary: SyncConflictSummary }
    | { kind: "failed"; reason: SyncFailure };

/**
 * What the last successful pull said about the GAME's own read marks: how many
 * stories the account has read in the Arknights client, how many of those the
 * game's own Archive lists, and when the import happened. Null before any pull
 * has succeeded, and `at` null when the account has never refreshed its game
 * data at all.
 *
 * `count` and `archived` come from two different records in the same payload.
 * The Archive block covers events and records but omits the mainline
 * entirely; the client's played-script flags cover the mainline. `count` is
 * their union and `archived` is the Archive half of it, so `archived` is
 * always the smaller of the two.
 */
export type GameImport = { count: number; archived: number; at: number | null } | null;

/**
 * The merge rules and the conflict prompt live in `syncMerge.ts`; they are
 * re-exported here because `#/lib/story/sync` is the one import site the
 * library and the reader know.
 */
export { DEFAULT_SYNC_POLICY, merge, SYNC_POLICY_KEY, type SyncChoice, type SyncConflictSummary, type SyncPolicy, type SyncSideSummary, sameProgress } from "./syncMerge";

// ---------------------------------------------------------------------------
// The engine. One per browser session, not one per component: the reader and
// the library both mount the hook and must share a debounce, a state and an
// in-flight request.
// ---------------------------------------------------------------------------

const OFF: SyncState = { kind: "off" };

let state: SyncState = OFF;
let enabled = false;
let unsubscribeWrite: (() => void) | null = null;
let pushTimer: ReturnType<typeof setTimeout> | null = null;
let pendingPush: StoryProgress | null = null;
/** Canonical form of the last document the server accepted; null when unknown. */
let acknowledged: string | null = null;
/**
 * The pull in flight, shared by every caller that asks while it runs.
 *
 * The library mounts the hook and so does its Progress tab, and the reader
 * route mounts it too, so one navigation starts two or three pulls in the same
 * tick. Each of them would compute `needsPush` against a remote document none
 * of the others had written yet and PUT the same merge. One round trip per
 * trigger is the whole point of sharing the engine.
 */
let pullInFlight: Promise<void> | null = null;
/** A tab that comes back after this long re-pulls: the account may have moved under it. */
export const REPULL_AFTER_MS = 15_000;
let lastPullAt = 0;
/** The push in flight, and the one document waiting behind it. */
let pushInFlight: Promise<void> | null = null;
let queuedPush: StoryProgress | null = null;

let gameImport: GameImport = null;
/**
 * The game's own read verdict from the last pull, read beside the document
 * rather than merged into it.
 *
 * Empty until a pull succeeds, and empty again when the account signs out: it
 * belongs to the account, not to the browser, which is the whole difference
 * between it and the local document.
 */
const NO_GAME_READ: ReadonlySet<string> = new Set<string>();
let gameRead: ReadonlySet<string> = NO_GAME_READ;

const stateListeners = new Set<() => void>();

function setState(next: SyncState): void {
    state = next;
    for (const fn of stateListeners) fn();
}

function setGameImport(next: GameImport): void {
    gameImport = next;
    for (const fn of stateListeners) fn();
}

function setGameRead(next: ReadonlySet<string>): void {
    gameRead = next;
    for (const fn of stateListeners) fn();
}

function subscribeState(fn: () => void): () => void {
    stateListeners.add(fn);
    return () => stateListeners.delete(fn);
}

const getState = (): SyncState => state;
const getServerState = (): SyncState => OFF;
const getGameImport = (): GameImport => gameImport;
const getServerGameImport = (): GameImport => null;
const getGameRead = (): ReadonlySet<string> => gameRead;
const getServerGameRead = (): ReadonlySet<string> => NO_GAME_READ;

/**
 * The conflict policy, read once and then held.
 *
 * Null means NOT READ YET rather than the default: localStorage is unavailable
 * on the server and a stored value has to survive the first render, so the
 * read is deferred to the first pull or the first subscriber instead of
 * running at module load.
 */
let policy: SyncPolicy | null = null;
/** The `gameUnread` list the pending conflict was computed against, so resolving merges by the same rule the prompt described. */
let pendingUnread: readonly string[] = [];

/** A stored value or {@link DEFAULT_SYNC_POLICY}. A junk value reads as the default, so no hand-edited key can leave the sync waiting on a prompt nobody sees. */
function storedPolicy(): SyncPolicy {
    if (typeof window === "undefined") return DEFAULT_SYNC_POLICY;
    try {
        const raw = window.localStorage.getItem(SYNC_POLICY_KEY);
        return raw !== null && (SYNC_POLICIES as readonly string[]).includes(raw) ? (raw as SyncPolicy) : DEFAULT_SYNC_POLICY;
    } catch {
        return DEFAULT_SYNC_POLICY;
    }
}

function currentPolicy(): SyncPolicy {
    if (policy === null) policy = storedPolicy();
    return policy;
}

/** Choose what happens the next time the two sides disagree. Takes effect on the next pull; a conflict already on screen is not resolved by it. */
export function setStorySyncPolicy(next: SyncPolicy): void {
    policy = next;
    try {
        window.localStorage.setItem(SYNC_POLICY_KEY, next);
    } catch {
        // Storage blocked: the choice holds for this session and no longer.
    }
    for (const fn of stateListeners) fn();
}

const getPolicy = (): SyncPolicy => currentPolicy();
const getServerPolicy = (): SyncPolicy => DEFAULT_SYNC_POLICY;

/**
 * PUT a document, unless the server already holds exactly it.
 *
 * At most one PUT is ever in flight. A document offered while one runs replaces
 * whatever was waiting behind it and goes out after, which is what makes the
 * writes COALESCE instead of race: two overlapping PUTs of the same row differ
 * only in which one the database sees last.
 */
async function push(doc: StoryProgress): Promise<void> {
    if (!enabled) return;
    if (canonical(doc) === acknowledged) return;
    if (pushInFlight) {
        queuedPush = doc;
        return pushInFlight;
    }
    pushInFlight = sendPush(doc).finally(() => {
        pushInFlight = null;
        const next = queuedPush;
        queuedPush = null;
        if (next) void push(next);
    });
    return pushInFlight;
}

async function sendPush(doc: StoryProgress): Promise<void> {
    const key = canonical(doc);
    setState({ kind: "syncing" });
    const res = await putStoryProgressFn({ data: doc });
    if (!enabled) return;
    if (res.ok) {
        acknowledged = key;
        setState({ kind: "idle", at: Date.now() });
    } else {
        // Forget the acknowledgement: an unchanged document has to be
        // retryable, and the only record that it was never stored is this one.
        acknowledged = null;
        setState({ kind: "failed", reason: res.reason });
    }
}

/**
 * GET, merge, and write the result back to whichever side differs from it.
 *
 * Concurrent callers share one round trip: see `pullInFlight`.
 */
function pull(reverdict = false): Promise<void> {
    if (!enabled) return Promise.resolve();
    if (pullInFlight) return pullInFlight;
    pullInFlight = runPull(reverdict).finally(() => {
        pullInFlight = null;
    });
    return pullInFlight;
}

/**
 * `reverdict` asks the backend to re-derive the game's read verdict from the
 * game data it already stores before answering; the automatic pulls read what
 * is stored as it is. Neither reaches the game server.
 */
async function runPull(reverdict: boolean): Promise<void> {
    lastPullAt = Date.now();
    setState({ kind: "syncing" });
    const res = await (reverdict ? importStoryProgressFn() : getStoryProgressFn());
    if (!enabled) return;
    if (!res.ok) {
        setState({ kind: "failed", reason: res.reason });
        return;
    }
    setGameRead(new Set(res.gameRead));
    setGameImport({ count: res.gameRead.length, archived: res.gameArchived, at: res.gameSyncedAt });
    const local = loadProgress();
    const remote = res.progress === null ? emptyProgress() : coerceProgress(res.progress);
    // Two histories that cannot both survive a merge are the reader's to
    // settle, not ours. `conflictOf` returns null for every pull that has
    // nothing to decide, which is almost all of them, and the policy decides
    // what a real disagreement costs: `ask` stops here with both documents
    // untouched, and the other three apply one outcome without a prompt.
    const disagreement = conflictOf(local, remote, res.gameUnread);
    if (disagreement !== null) {
        const choice = currentPolicy();
        pendingUnread = res.gameUnread;
        // Truthful bookkeeping, and the reason a later choice reaches the
        // account: the server holds `remote`, so a push of anything else is a
        // real change and a push of this document is not. Every branch below
        // reads it, including one taken minutes later from the prompt.
        acknowledged = canonical(remote);
        if (choice === "ask") {
            setState({ kind: "conflict", local: disagreement.local, remote: disagreement.remote, summary: disagreement.summary });
            return;
        }
        await applyChoice(choice, disagreement.local, disagreement.remote);
        return;
    }
    // The game's verdict stays BESIDE the document: nothing from `gameRead` is
    // written into `read`, and `gameUnread` withdraws what the old import wrote
    // there. Both sides of the merge are upgraded, so one pull is enough
    // whichever side still carries a v1 document.
    const merged = merge(local, remote, res.gameUnread);
    const mergedKey = canonical(merged);
    const needsPush = mergedKey !== canonical(remote);
    // The local write notifies, which would schedule a debounced PUT of this
    // exact document. `schedulePush` drops a document the server already holds,
    // and this push acknowledges before the debounce fires, so the echo costs
    // nothing; a PUT slower than the debounce coalesces into the one in flight.
    if (mergedKey !== canonical(local)) saveProgress(merged);
    if (needsPush) {
        await push(merged);
        return;
    }
    acknowledged = mergedKey;
    setState({ kind: "idle", at: Date.now() });
}

/**
 * Carry out one of the three outcomes, on the two upgraded documents the
 * conflict was computed from.
 *
 * `acknowledged` is the pivot. It holds the canonical form of what the account
 * last said it had, which the conflict path sets to the document it just
 * pulled, so `merge` can tell whether its result still needs to go out,
 * `account` can let an unchanged document echo away for free, and `browser`
 * clears it because the account is about to be replaced by something it has
 * never seen.
 */
async function applyChoice(choice: SyncChoice, local: StoryProgress, remote: StoryProgress): Promise<void> {
    if (choice === "account") {
        setState({ kind: "syncing" });
        // The account wins and the browser copy is replaced. Nothing is pushed:
        // the document written here is the one the account already holds, so
        // `schedulePush` drops its own echo. The one exception is an account
        // still holding a v1 document, where what lands locally is its upgrade
        // and the upgrade is worth a PUT.
        saveProgress(remote);
        setState({ kind: "idle", at: Date.now() });
        return;
    }
    if (choice === "browser") {
        // The browser wins and the account is replaced. The local document is
        // not touched at all, which is the whole promise of this button.
        acknowledged = null;
        await push(local);
        return;
    }
    const merged = merge(local, remote, pendingUnread);
    const mergedKey = canonical(merged);
    const needsPush = mergedKey !== acknowledged;
    if (mergedKey !== canonical(loadProgress())) saveProgress(merged);
    if (needsPush) {
        await push(merged);
        return;
    }
    acknowledged = mergedKey;
    setState({ kind: "idle", at: Date.now() });
}

/**
 * Settle a conflict the reader is looking at. A no-op when there is none,
 * which is what a second click on a button already answered costs.
 */
export function resolveStorySync(choice: SyncChoice): void {
    void resolveConflict(choice);
}

function resolveConflict(choice: SyncChoice): Promise<void> {
    if (!enabled || state.kind !== "conflict") return Promise.resolve();
    const { local, remote } = state;
    return applyChoice(choice, local, remote);
}

/**
 * Hold a local write for `PUSH_DEBOUNCE_MS` and then push the LAST one.
 *
 * Trailing only: every write restarts the window and only the document the
 * window closes on is sent, so a reader tapping through dialogue costs one PUT
 * rather than one per tap. Nothing is sent on the leading edge, and a document
 * the server has already acknowledged is not scheduled at all, which is what
 * keeps a pull's own write from echoing back as a second PUT.
 */
function schedulePush(doc: StoryProgress): void {
    if (!enabled) return;
    // A conflict on screen means the account holds a history this document
    // would REPLACE, and replacing it is one of the three things the reader is
    // being asked about. Reading on while the prompt sits there is allowed and
    // costs nothing: the write lands locally, and the next pull sees a local
    // document that has grown, which is a superset rather than a conflict.
    if (state.kind === "conflict") return;
    if (canonical(doc) === acknowledged) {
        if (pushTimer) clearTimeout(pushTimer);
        pushTimer = null;
        pendingPush = null;
        return;
    }
    pendingPush = doc;
    if (pushTimer) clearTimeout(pushTimer);
    pushTimer = setTimeout(() => {
        pushTimer = null;
        const next = pendingPush;
        pendingPush = null;
        if (next) void push(next);
    }, PUSH_DEBOUNCE_MS);
}

/** Send whatever the debounce is holding, now. */
export function flushStorySync(): void {
    if (pushTimer) {
        clearTimeout(pushTimer);
        pushTimer = null;
    }
    const next = pendingPush;
    pendingPush = null;
    if (next) void push(next);
}

/** PUT one document immediately, skipping the debounce. Used by the reset. */
export function pushStoryProgressNow(doc: StoryProgress): void {
    if (!enabled) return;
    if (pushTimer) {
        clearTimeout(pushTimer);
        pushTimer = null;
    }
    pendingPush = null;
    void push(doc);
}

/** Pull, merge and push. The manual "Sync now" button and every mount call this. */
/**
 * The "Sync now" button. It is a RE-DERIVATION, not a refresh: the backend
 * rebuilds the game's verdict from the marks and stage records it stored on
 * the last refresh and the document is pulled after it. The game server is
 * never called, which is why the button is safe while the game is open.
 */
export function syncStoryProgressNow(): void {
    void pull(true);
}

function onVisibilityChange(): void {
    if (document.visibilityState === "hidden") {
        flushStorySync();
        return;
    }
    onReturn();
}

/**
 * A tab regaining focus or visibility pulls again when its last pull is older
 * than {@link REPULL_AFTER_MS}: a merge decided in another tab, a story read
 * on another device, or a "Sync now" pressed elsewhere used to stay invisible
 * here until a reload, which read as the merge not happening.
 */
function onReturn(): void {
    if (!enabled || pullInFlight) return;
    if (Date.now() - lastPullAt < REPULL_AFTER_MS) return;
    void pull();
}

function enable(): void {
    if (enabled || typeof window === "undefined") return;
    enabled = true;
    unsubscribeWrite = onProgressWritten(schedulePush);
    document.addEventListener("visibilitychange", onVisibilityChange);
    window.addEventListener("focus", onReturn);
}

function disable(): void {
    if (!enabled) return;
    enabled = false;
    unsubscribeWrite?.();
    unsubscribeWrite = null;
    document.removeEventListener("visibilitychange", onVisibilityChange);
    window.removeEventListener("focus", onReturn);
    if (pushTimer) clearTimeout(pushTimer);
    pushTimer = null;
    pendingPush = null;
    queuedPush = null;
    acknowledged = null;
    pendingUnread = [];
    setGameRead(NO_GAME_READ);
    setGameImport(null);
    setState(OFF);
}

/**
 * Mount the sync. Called by the library and by the reader; both may be mounted
 * at once and the engine behind them is the same one.
 *
 * Signed in is read off `authStore` rather than through `useAuth`, which pulls
 * in the server-only session module: a unit test cannot load that, and one
 * reader test imports the story route transitively. The store carries the same
 * answer one render later, which for a background sync costs nothing.
 *
 * Signing out disables the sync and clears every trace of the account side. The
 * local document is never touched by that, because it is the browser's, not the
 * account's.
 */
export function useStoryProgressSync(): { state: SyncState; syncNow: () => void } {
    const signedIn = useStore(authStore, (s) => s.user !== null);
    const current = useSyncExternalStore(subscribeState, getState, getServerState);

    useEffect(() => {
        if (!signedIn) {
            disable();
            return;
        }
        enable();
        void pull();
        // Leaving the reader or the library sends what the debounce still
        // holds, rather than waiting for a visibility change that may never come.
        return flushStorySync;
    }, [signedIn]);

    return { state: current, syncNow: syncStoryProgressNow };
}

/** What happens the next time the two sides disagree. `ask` until the reader says otherwise. */
export function useStorySyncPolicy(): SyncPolicy {
    return useSyncExternalStore(subscribeState, getPolicy, getServerPolicy);
}

/** The sync state alone, for a component that only reports it. */
export function useStorySyncState(): SyncState {
    return useSyncExternalStore(subscribeState, getState, getServerState);
}

/** What the game's own read marks contributed, for the Progress tab's second line. */
export function useStoryGameImport(): GameImport {
    return useSyncExternalStore(subscribeState, getGameImport, getServerGameImport);
}

/**
 * The game's read verdict, for every place that decides whether a story is
 * read. Empty on the server and before the first pull, so the first render
 * shows the local document alone and the account's half arrives with the pull.
 */
export function useStoryGameRead(): ReadonlySet<string> {
    return useSyncExternalStore(subscribeState, getGameRead, getServerGameRead);
}

/** Reset every module-level value. Tests only: the engine is a singleton by design. */
export function __resetStorySyncForTests(): void {
    if (pushTimer) clearTimeout(pushTimer);
    enabled = false;
    unsubscribeWrite?.();
    unsubscribeWrite = null;
    pushTimer = null;
    pendingPush = null;
    pullInFlight = null;
    lastPullAt = 0;
    pushInFlight = null;
    queuedPush = null;
    acknowledged = null;
    gameImport = null;
    gameRead = NO_GAME_READ;
    policy = null;
    pendingUnread = [];
    state = OFF;
}

/** Settle a pending conflict and wait for the write it causes. Tests only: the public resolver is fire-and-forget. */
export function __resolveStorySyncForTests(choice: SyncChoice): Promise<void> {
    return resolveConflict(choice);
}

/** The sync state as the module holds it. Tests only, for the same reason `useStorySyncState` needs React. */
export function __storySyncStateForTests(): SyncState {
    return state;
}

/** Start the engine outside React. Tests only. */
export function __enableStorySyncForTests(): void {
    // The real path, so the visibility and focus listeners are on too.
    enabled = false;
    unsubscribeWrite?.();
    unsubscribeWrite = null;
    enable();
}

/**
 * What the last pull reported the game had read. Tests only: the component
 * reads it through `useStoryGameImport`, which needs React.
 */
export function __storyGameImportForTests(): GameImport {
    return gameImport;
}

/** The game's read verdict from the last pull. Tests only, for the same reason. */
export function __storyGameReadForTests(): ReadonlySet<string> {
    return gameRead;
}

/** Pull once and wait for it. Tests only. */
/** The "Sync now" pull, the one that re-derives first. Tests only. */
export function __reverdictStorySyncForTests(): Promise<void> {
    return pull(true);
}

export function __pullStorySyncForTests(): Promise<void> {
    return pull();
}
