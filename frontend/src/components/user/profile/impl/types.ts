import type { ProfileTabId } from "#/types/generated/ProfileTabId";

/** A profile tab. The backend's `ProfileTabId` is the allow-list, so the two cannot drift. */
export type TabId = ProfileTabId;

/**
 * Every `TabId`, in canonical order: the default profile's order, and the order
 * the backend appends a tab
 * missing from a saved layout (Showcase excepted: it goes first).
 * Also validates a persisted value before it is trusted.
 */
export const TAB_IDS = ["showcase", "stats", "score", "roster", "plans", "inventory", "enemies", "optimizer"] as const satisfies readonly TabId[];

export function isTabId(value: unknown): value is TabId {
    return typeof value === "string" && (TAB_IDS as readonly string[]).includes(value);
}
