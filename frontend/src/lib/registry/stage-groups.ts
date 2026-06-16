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

/** Index of a group key in display order (unknown keys sort last). */
export function stageGroupOrder(key: string): number {
    const i = STAGE_GROUPS.findIndex((g) => g.key === key);
    return i === -1 ? STAGE_GROUPS.length : i;
}
