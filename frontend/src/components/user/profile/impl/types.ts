export type TabId = "stats" | "score" | "roster" | "inventory" | "plans" | "enemies" | "optimizer";

/** Every `TabId`, so a persisted value can be validated before it is trusted. */
export const TAB_IDS = ["stats", "score", "roster", "inventory", "plans", "enemies", "optimizer"] as const satisfies readonly TabId[];

export function isTabId(value: unknown): value is TabId {
    return typeof value === "string" && (TAB_IDS as readonly string[]).includes(value);
}
