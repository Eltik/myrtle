/**
 * Stage group taxonomy shared across enemy-stage UIs (and a future stage
 * viewer). The keys mirror the backend's `EnemyStageRef.group`, which is the
 * single source of truth - the frontend never re-derives a group from ids.
 */

export type StageGroupKey = "story" | "events" | "annihilation" | "is" | "ra" | "sss" | "paradox" | "cc" | "supplies" | "other";

export interface IStageGroup {
    key: StageGroupKey;
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

/** One-line blurb per group, shown in the stage-list detail dialog. */
export const STAGE_GROUP_DESCRIPTION: Record<StageGroupKey, string> = {
    story: "The mainline campaign - Rhodes Island's canonical operations, told episode by episode across Terra.",
    events: "Limited-time side stories, intermezzi and reruns, each with its own maps, mechanics and rewards.",
    annihilation: "Endless-wave defense operations scored by total kills - the weekly source of Orundum.",
    is: "Roguelike expeditions: assemble a squad node by node across randomized floors, one run at a time.",
    ra: "A survival sandbox - scout the wilds, gather resources, build defenses and endure each cycle.",
    sss: "Deck-building defense runs against escalating security directives at fixed installations.",
    paradox: "Operator-specific challenge simulations that stress-test a single unit's kit.",
    cc: "Score-attack contracts - stack risk modifiers on a fixed map for higher rewards.",
    supplies: "Daily and weekly resource runs for LMD, battle records, skill summaries and chips.",
    other: "Special and uncategorized operations that sit outside the regular rotation.",
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
