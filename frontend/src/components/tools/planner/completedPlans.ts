/**
 * The completed-plans notice. A plan is `met` once a roster sync shows every
 * target reached; the planner then offers to delete it, and never deletes it
 * on its own.
 *
 * Dismissing the notice stores the met operator ids it was showing. It comes
 * back only when a plan the player has not yet seen complete joins the set, so
 * deleting one of the kept plans elsewhere does not bring it back for the rest.
 */

const STORAGE_PREFIX = "planner:completed-dismissed";

/** Per account, so two players sharing a browser do not dismiss each other's notice. */
export function completedNoticeStorageKey(uid: string): string {
    return `${STORAGE_PREFIX}:${uid}`;
}

/** Operator ids of the met plans, sorted, so the same set always stores the same way. */
export function metOperatorIds(plans: { operator_id: string; met: boolean }[]): string[] {
    return plans
        .filter((p) => p.met)
        .map((p) => p.operator_id)
        .sort();
}

/** True when some met plan is missing from the dismissed set. */
export function shouldShowCompletedNotice(metIds: string[], dismissedIds: string[]): boolean {
    if (metIds.length === 0) return false;
    const dismissed = new Set(dismissedIds);
    return metIds.some((id) => !dismissed.has(id));
}

/** Storage can throw (private mode, blocked site data); a failed read means nothing was dismissed. */
export function readDismissedIds(key: string): string[] {
    try {
        const raw = window.localStorage.getItem(key);
        if (!raw) return [];
        const parsed: unknown = JSON.parse(raw);
        return Array.isArray(parsed) ? parsed.filter((id): id is string => typeof id === "string") : [];
    } catch {
        return [];
    }
}

/** A failed write only means the notice may show again next visit. */
export function writeDismissedIds(key: string, ids: string[]): void {
    try {
        window.localStorage.setItem(key, JSON.stringify([...ids].sort()));
    } catch {
        // Ignored: see above.
    }
}
