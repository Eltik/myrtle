export { DEFAULT_AVATAR_ID } from "#/lib/utils";

export const PAGE_SIZE = 25;

export type LeaderboardScope = "global" | "friends";

export const SERVERS = ["EN", "JP", "CN", "KR", "TW"] as const;
export type ServerCode = (typeof SERVERS)[number];

export const INTERVALS = [
    { value: "1 day", short: "1d", label: "Today", subtitle: "today", since: "since yesterday" },
    { value: "7 days", short: "7d", label: "Past 7 days", subtitle: "7 days", since: "in the past 7 days" },
    { value: "30 days", short: "30d", label: "Past 30 days", subtitle: "30 days", since: "in the past 30 days" },
] as const;
export type LeaderboardInterval = (typeof INTERVALS)[number]["value"];

export const SERVER_TINTS: Record<string, { fg: string; bg: string }> = {
    EN: { fg: "#4f74e0", bg: "color-mix(in srgb, #4f74e0 15%, transparent)" },
    JP: { fg: "#e04f74", bg: "color-mix(in srgb, #e04f74 15%, transparent)" },
    CN: { fg: "#b87a2c", bg: "color-mix(in srgb, #e0a04f 15%, transparent)" },
    KR: { fg: "var(--lagoon-deep)", bg: "color-mix(in srgb, #4fb8b2 18%, transparent)" },
    TW: { fg: "#7a5cb0", bg: "color-mix(in srgb, #9e7ad4 18%, transparent)" },
};
