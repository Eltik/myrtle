import type { ClientGachaGroup } from "#/lib/api/gacha";
import inkStyles from "./bannerGroup.module.css";
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

/**
 * Each group's colour, shared by both gacha screens. `fill` paints bars, swatches and borders;
 * `inkClass` colours the group's label text, because every fill is under 2.2:1 as text on the
 * light card. Regular and special are the 4★ and 3★ rarity colours; limited, linkage and boot
 * match no token, so their hues are kept here and their ink in `bannerGroup.module.css`.
 */
export const BANNER_GROUP_COLOR: Record<ClientGachaGroup, { fill: string; inkClass: string }> = {
    limited: { fill: "oklch(0.85 0.18 80)", inkClass: inkStyles.limited },
    linkage: { fill: "oklch(0.78 0.16 320)", inkClass: inkStyles.linkage },
    regular: { fill: "var(--rarity-4)", inkClass: "text-rarity-4-ink" },
    special: { fill: "var(--rarity-3)", inkClass: "text-rarity-3-ink" },
    boot: { fill: "oklch(0.8 0.12 160)", inkClass: inkStyles.boot },
};

export const BANNER_STATUS_LABEL_KEYS: Record<BannerStatus, GachaMessageKey> = {
    active: "banner.status.active",
    upcoming: "banner.status.upcoming",
    ended: "banner.status.ended",
    returning: "banner.status.returning",
};

/** The status as text and its dot. The old status colours, oklch(0.78 0.18 145) / (0.78 0.16 220) / (0.78 0.12 300), measured 1.88 to 2.08 on the light card; `--banner-*-ink` clear 4.5 in both themes and keep those colours in dark. */
export const BANNER_STATUS_INK: Record<BannerStatus, string> = {
    active: "var(--banner-active-ink)",
    upcoming: "var(--banner-upcoming-ink)",
    ended: "var(--muted-foreground)",
    returning: "var(--banner-returning-ink)",
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
