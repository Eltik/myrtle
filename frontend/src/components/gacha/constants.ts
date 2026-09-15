import type { ClientGachaGroup } from "#/lib/api/gacha";
import type { messages as gachaConstantsMessages } from "./constants.messages";

/** A key in `constants.messages.ts`; resolved by whichever component renders it. */
export type GachaMessageKey = keyof typeof gachaConstantsMessages & string;

/** Banner status, as both screens label a banner's run window. */
export type BannerStatus = "active" | "upcoming" | "ended";

/** The bucket label for each rule-type group, shared by both gacha screens. */
export const BANNER_GROUP_LABEL_KEYS: Record<ClientGachaGroup, GachaMessageKey> = {
    limited: "banner.group.limited",
    linkage: "banner.group.linkage",
    regular: "banner.group.regular",
    special: "banner.group.special",
};

export const BANNER_STATUS_LABEL_KEYS: Record<BannerStatus, GachaMessageKey> = {
    active: "banner.status.active",
    upcoming: "banner.status.upcoming",
    ended: "banner.status.ended",
};

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
};
