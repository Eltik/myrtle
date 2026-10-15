import type { ClientGachaGroup } from "#/lib/api/gacha";
import type { messages as gachaConstantsMessages } from "./constants.messages";

/** A key in `constants.messages.ts`; resolved by whichever component renders it. */
export type GachaMessageKey = keyof typeof gachaConstantsMessages & string;

/**
 * Banner status, as both screens label a banner's run window. `returning` is a pool only
 * returning players see (`GachaPoolClient.returning`): the game times it per player, so its
 * years-long table window is not a run anyone can be "in", and it is never `active`.
 */
export type BannerStatus = "active" | "upcoming" | "ended" | "returning";

/** The bucket label for each rule-type group, shared by both gacha screens. */
export const BANNER_GROUP_LABEL_KEYS: Record<ClientGachaGroup, GachaMessageKey> = {
    limited: "banner.group.limited",
    linkage: "banner.group.linkage",
    regular: "banner.group.regular",
    special: "banner.group.special",
    boot: "banner.group.boot",
};

export const BANNER_STATUS_LABEL_KEYS: Record<BannerStatus, GachaMessageKey> = {
    active: "banner.status.active",
    upcoming: "banner.status.upcoming",
    ended: "banner.status.ended",
    returning: "banner.status.returning",
};

export const BANNER_STATUS_COLOR: Record<BannerStatus, string> = {
    active: "oklch(0.78 0.18 145)",
    upcoming: "oklch(0.78 0.16 220)",
    ended: "var(--muted-foreground)",
    returning: "oklch(0.78 0.12 300)",
};

/** Where `banner` stands at `nowSec` (unix seconds). */
export function bannerStatus(banner: { openTime: number; endTime: number; returning: boolean }, nowSec: number): BannerStatus {
    if (banner.returning) return "returning";
    if (nowSec < banner.openTime) return "upcoming";
    if (nowSec > banner.endTime) return "ended";
    return "active";
}

/** Readable name for a banner's raw `gachaRuleType`. Unknown types fall back to the raw value. */
export const BANNER_RULE_TYPE_LABEL_KEYS: Record<string, GachaMessageKey> = {
    NORMAL: "banner.rule.normal",
    SINGLE: "banner.rule.single",
    LIMITED: "banner.rule.limited",
    LINKAGE: "banner.rule.linkage",
    CLASSIC: "banner.rule.classic",
    CLASSIC_ATTAIN: "banner.rule.classicAttain",
    CLASSIC_DOUBLE: "banner.rule.classicDouble",
    FESCLASSIC: "banner.rule.fesclassic",
    ATTAIN: "banner.rule.attain",
    DOUBLE: "banner.rule.double",
    SPECIAL: "banner.rule.special",
    BACKFLOW: "banner.rule.backflow",
};
