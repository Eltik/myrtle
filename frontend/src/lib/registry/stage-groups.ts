/**
 * Stage group taxonomy shared across enemy-stage UIs (and a future stage
 * viewer). The keys mirror the backend's `EnemyStageRef.group`, which is the
 * single source of truth - the frontend never re-derives a group from ids.
 */

import type { messages as stageGroupMessages } from "#/lib/registry/stage-groups.messages";

export type StageGroupKey = "story" | "events" | "annihilation" | "is" | "ra" | "sss" | "paradox" | "cc" | "supplies" | "other";

/** A key in `stage-groups.messages.ts`; resolved by whichever component renders it. */
export type StageGroupMessageKey = keyof typeof stageGroupMessages & string;

export interface IStageGroup {
    key: StageGroupKey;
    /**
     * Official in-game mode name, deliberately NOT in the message catalog:
     * game vocabulary belongs to the game-data layer, which ships its own
     * name per region. Only the site's own prose about a group is a message
     * (see `STAGE_GROUP_DESCRIPTION_KEY`).
     */
    label: string;
}

/** Fine groups in display order. */
export const STAGE_GROUPS: IStageGroup[] = [
    { key: "story", label: "Main Story" },
    { key: "events", label: "Events" },
    { key: "annihilation", label: "Annihilation" },
    { key: "is", label: "Integrated Strategies" },
    { key: "ra", label: "Reclamation Algorithm" },
    { key: "sss", label: "Stationary Security Service" },
    { key: "paradox", label: "Paradox Simulation" },
    { key: "cc", label: "Contingency Contract" },
    { key: "supplies", label: "Supplies" },
    { key: "other", label: "Other Modes" },
];

export const STAGE_GROUP_LABEL: Record<StageGroupKey, string> = Object.fromEntries(STAGE_GROUPS.map((g) => [g.key, g.label])) as Record<StageGroupKey, string>;

/**
 * One-line blurb per group, shown in the stage-list detail dialog. These are
 * message KEYS, not text: this module has no React, so the dialog resolves
 * them with `t()` against `stage-groups.messages.ts`.
 */
export const STAGE_GROUP_DESCRIPTION_KEY: Record<StageGroupKey, StageGroupMessageKey> = {
    story: "stageGroup.story.desc",
    events: "stageGroup.events.desc",
    annihilation: "stageGroup.annihilation.desc",
    is: "stageGroup.is.desc",
    ra: "stageGroup.ra.desc",
    sss: "stageGroup.sss.desc",
    paradox: "stageGroup.paradox.desc",
    cc: "stageGroup.cc.desc",
    supplies: "stageGroup.supplies.desc",
    other: "stageGroup.other.desc",
};

/** Index of a group key in display order (unknown keys sort last). */
export function stageGroupOrder(key: string): number {
    const i = STAGE_GROUPS.findIndex((g) => g.key === key);
    return i === -1 ? STAGE_GROUPS.length : i;
}

/**
 * Per-group accent so a stage's code reads its category at a glance. Main Story
 * carries the coral brand; the rest are a teal-free categorical set.
 */
export const STAGE_GROUP_TONE: Record<StageGroupKey, string> = {
    story: "var(--primary)",
    events: "#f7a452",
    annihilation: "var(--destructive)",
    is: "#b78fe6",
    ra: "#5bbf86",
    sss: "#5a8fe8",
    paradox: "#bcabdb",
    cc: "#e8a23c",
    supplies: "var(--muted-foreground)",
    other: "var(--muted-foreground)",
};

/**
 * Derive a group from a zone's `type` (and optionally the stage's `stageType`).
 * The backend's `EnemyStageRef.group` remains the source of truth for the
 * enemy-stage UIs; this mapper exists for the Stage Viewer browser, where we
 * group the full `stage_table` and `zone_table` (which carry a `ZoneType`, not
 * a precomputed group key).
 */
export function stageGroupForZone(zoneType: string | undefined, stageType?: string): StageGroupKey {
    if (stageType === "DAILY") return "supplies";
    if (stageType === "CAMPAIGN") return "annihilation";
    if (stageType === "CLIMB_TOWER") return "other";
    switch (zoneType) {
        case "MAINLINE":
        case "MAINLINE_ACTIVITY":
            return "story";
        case "MAINLINE_RETRO":
        case "SIDESTORY":
        case "BRANCHLINE":
        case "ACTIVITY":
            return "events";
        case "WEEKLY":
            return "supplies";
        case "CAMPAIGN":
            return "annihilation";
        case "ROGUELIKE":
            return "is";
        default:
            return "other";
    }
}
