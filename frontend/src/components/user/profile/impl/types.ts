import type { ProfileTabId } from "#/types/generated/ProfileTabId";

/**
 * A tab of the saved layout: one the owner orders and hides in the tab editor. The
 * backend's `ProfileTabId` is the allow-list, so the two cannot drift.
 */
export type LayoutTabId = ProfileTabId;

/**
 * The Pulls tab. Not part of the layout: its visibility is the owner's opt-in
 * `public_gacha` switch (off by default), which a layout cannot express, since a layout
 * shows every tab it omits. Shown after the layout's tabs, to a visitor only while the
 * switch is on, and always to the owner.
 */
export const PULLS_TAB = "pulls";

/** Any tab the bar can show: a layout tab or the Pulls tab. */
export type TabId = LayoutTabId | typeof PULLS_TAB;

/**
 * Every `LayoutTabId`, in canonical order: the default profile's order, and the order
 * the backend appends a tab missing from a saved layout (Showcase excepted: it
 * goes first).
 */
export const TAB_IDS = ["showcase", "stats", "score", "roster", "plans", "inventory", "enemies", "optimizer", "lookback"] as const satisfies readonly LayoutTabId[];

export function isLayoutTabId(value: unknown): value is LayoutTabId {
    return typeof value === "string" && (TAB_IDS as readonly string[]).includes(value);
}

/** Validates a persisted or linked tab before it is trusted. */
export function isTabId(value: unknown): value is TabId {
    return value === PULLS_TAB || isLayoutTabId(value);
}
