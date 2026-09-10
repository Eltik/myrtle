import { Store } from "@tanstack/store";
import { countMissedNotes, LATEST_ANNOUNCED_NOTE } from "#/content/changelog/entries";

/**
 * Tracks the newest release note the visitor has been SHOWN. The value is the
 * entry id verbatim, so it is readable in devtools and sorts against the ids in
 * `entries.ts` with a plain string compare.
 */
export const RELEASE_NOTE_STORAGE_KEY = "myrtle-changelog-seen";

/**
 * `pending` until the one-shot read runs, `unavailable` when localStorage
 * throws. Storage failing hides the DOT, because a visitor who cannot persist a
 * dismissal would otherwise carry an unread marker on every page forever. The
 * bell itself keeps working; only the unread signal goes quiet.
 */
type ReleaseNoteStatus = "pending" | "ready" | "unavailable";

interface IReleaseNoteState {
    status: ReleaseNoteStatus;
    /** Live: drives the header dot. `null` means never seen anything. */
    seenId: string | null;
    dialogOpen: boolean;
    /** Announced entries between `seenId` and the marquee, frozen when the dialog opens. */
    missed: number;
}

export const releaseNoteStore = new Store<IReleaseNoteState>({
    status: "pending",
    seenId: null,
    dialogOpen: false,
    missed: 0,
});

function readSeen(): string | null {
    return window.localStorage.getItem(RELEASE_NOTE_STORAGE_KEY);
}

function writeSeen(id: string): void {
    try {
        window.localStorage.setItem(RELEASE_NOTE_STORAGE_KEY, id);
    } catch {
        // Quota or a blocked store. Nothing to do: the visitor sees this once
        // per session rather than never, which is the acceptable degradation.
    }
}

let initialised = false;

/**
 * Reads storage exactly once per page load. Idempotent, so every consumer can
 * call it without caring which effect commits first.
 *
 * Writes nothing. The key is created by the first bell click, so an absent key
 * means unread rather than caught up.
 */
export function initReleaseNotes(): void {
    if (initialised || typeof window === "undefined") return;
    initialised = true;

    const latest = LATEST_ANNOUNCED_NOTE;
    if (latest === null) {
        releaseNoteStore.setState((s) => ({ ...s, status: "ready" }));
        return;
    }

    let seen: string | null;
    try {
        seen = readSeen();
    } catch {
        releaseNoteStore.setState((s) => ({ ...s, status: "unavailable" }));
        return;
    }

    // `seen` stays null until the visitor actually opens the dialog, and a null
    // seen id counts as UNSEEN. Suppressing the dot for a first-time visitor was
    // wrong: every existing visitor arrives without the key, so seeding them as
    // caught up would ship this feature silent and the dot would not appear
    // until the second announced entry. `countMissedNotes(null)` is 0, so a
    // newcomer gets one dot and no "N more you missed" line.
    releaseNoteStore.setState((s) => ({ ...s, status: "ready", seenId: seen }));
}

/** True when there is an announced entry newer than what the visitor has seen. */
export function hasUnseenReleaseNote(state: IReleaseNoteState): boolean {
    if (state.status !== "ready" || LATEST_ANNOUNCED_NOTE === null) return false;
    return state.seenId === null || state.seenId < LATEST_ANNOUNCED_NOTE.id;
}

/**
 * Marks the marquee seen the moment the dialog opens rather than on dismissal.
 * It was shown, so it counts, and the dot clears on the click that opened it.
 *
 * Opens even when storage is UNAVAILABLE. Only the dot depends on being able to
 * persist a dismissal; refusing an explicit click would leave a private-browsing
 * visitor with a bell that does nothing.
 */
export function openReleaseNoteDialog(): void {
    const latest = LATEST_ANNOUNCED_NOTE;
    if (latest === null) return;
    const { seenId } = releaseNoteStore.state;

    const missed = countMissedNotes(seenId);
    if (seenId !== latest.id) writeSeen(latest.id);
    releaseNoteStore.setState((s) => ({ ...s, dialogOpen: true, missed, seenId: latest.id }));
}

export function setReleaseNoteDialogOpen(open: boolean): void {
    if (open) {
        openReleaseNoteDialog();
        return;
    }
    releaseNoteStore.setState((s) => ({ ...s, dialogOpen: false }));
}

/** Test seam: lets a fresh page load re-run `initReleaseNotes`. */
export function resetReleaseNotesForTest(): void {
    initialised = false;
    releaseNoteStore.setState(() => ({ status: "pending", seenId: null, dialogOpen: false, missed: 0 }));
}
