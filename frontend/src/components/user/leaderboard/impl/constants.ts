export { DEFAULT_AVATAR_ID } from "#/lib/utils";

export const PAGE_SIZE = 25;

export type LeaderboardScope = "global" | "friends";

export const SERVERS = ["EN", "JP", "CN", "KR", "TW"] as const;
export type ServerCode = (typeof SERVERS)[number];

export const SERVER_TINTS: Record<string, { fg: string; bg: string }> = {
    EN: { fg: "#4f74e0", bg: "color-mix(in srgb, #4f74e0 15%, transparent)" },
    JP: { fg: "#e04f74", bg: "color-mix(in srgb, #e04f74 15%, transparent)" },
    CN: { fg: "#b87a2c", bg: "color-mix(in srgb, #e0a04f 15%, transparent)" },
    KR: { fg: "var(--lagoon-deep)", bg: "color-mix(in srgb, #4fb8b2 18%, transparent)" },
    TW: { fg: "#7a5cb0", bg: "color-mix(in srgb, #9e7ad4 18%, transparent)" },
};
