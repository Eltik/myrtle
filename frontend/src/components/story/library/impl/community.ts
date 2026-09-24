/**
 * The community reading aggregate, shaped into the four lists the tab draws.
 *
 * Pure over its two inputs, so every ordering rule and every "we cannot
 * measure that" verdict is testable without a DOM or a backend.
 *
 * THE DENOMINATOR IS `players` and nothing else. Every share this module
 * computes is readers over synced players, and the tab labels it that way; no
 * number here is a share of anything narrower, and none of them is a
 * probability.
 */

import type { StoryCommunity } from "#/types/generated/StoryCommunity";
import type { LibGroup, LibIndex } from "./derive";
import { kindOf, type StoryKind } from "./sections";

/**
 * How many rows each ranked list holds, and the reason the tab needs no
 * virtualisation: it ranks all 1,887 stories and all 451 groups, then RENDERS
 * at most 60 rows. The ranking is one pass and one sort per list inside a
 * `useMemo` on the index, so the cost is paid on the payload landing and never
 * on a re-render.
 */
export const TOP_GROUPS = 25;
export const TOP_STORIES = 25;
export const BOTTOM_GROUPS = 10;

/** The chapter the depth strip opens on, when the index carries a curve for it. */
export const DEFAULT_DEPTH_GROUP = "main_1";

export interface ICommunityGroupRow {
    id: string;
    name: string;
    /** The badge the Browse ticket wears, so the same group reads the same in both places. */
    kind: StoryKind;
    readers: number;
    /** `readers / players`, in [0, 1]. */
    readerShare: number;
    /**
     * Accounts that finished the group, or NULL when finishing it cannot be
     * measured at all. See {@link finishedIsMeasurable}.
     */
    finished: number | null;
    finishedShare: number | null;
}

export interface ICommunityStoryRow {
    id: string;
    name: string;
    code?: string;
    groupName: string;
    /** False for the 25 listed stories no script file backs: those have no reader route to link to. */
    hasScript: boolean;
    readers: number;
    readerShare: number;
}

export interface ICommunityDepthBar {
    id: string;
    name: string;
    code?: string;
    readers: number;
    readerShare: number;
}

export interface ICommunityDepthGroup {
    id: string;
    name: string;
    bars: ICommunityDepthBar[];
}

export interface ICommunityRanked {
    players: number;
    /** Unix SECONDS, as the wire sends it. The tab multiplies by 1,000 itself. */
    computedAt: number;
    top: ICommunityGroupRow[];
    bottom: ICommunityGroupRow[];
    /** Ranked groups whose `readers` is 0, which the bottom list leaves out rather than filling itself with ties. */
    skippedZero: number;
    stories: ICommunityStoryRow[];
    depth: ICommunityDepthGroup[];
    /** The group the picker opens on: `main_1` when it has a curve, else the first that does. */
    defaultDepth: string | null;
}

/**
 * Whether "finished" means anything for this group.
 *
 * The aggregate knows 2,313 of its 2,576 accounts only through their STAGE
 * RECORDS, so a story that no stage gates is invisible to those accounts and
 * the group can never count a finisher from them. The index says exactly which
 * groups those are: `StoryEntry.requiredStages` is the gate, and a group with
 * no gated story at all is one whose `finished` is structurally 0 rather than
 * measured 0.
 *
 * Measured on the :3060 index on 2026-09-24: 494 of the 1,887 stories carry no
 * gate, and they fall into 384 of the 451 groups, being the 364 record sets and
 * the 20 MINISTORY vignettes (`act4d0`, `act7mini`, ...). `main_0` keeps its
 * two ungated tutorial entries but is gated elsewhere, so it stays measurable,
 * which is why the rule is ANY gated story rather than all of them.
 *
 * `requiredStages` is read defensively: a backend older than the field sends
 * no array, and absent must read as "no gate found", never as a crash.
 */
export function finishedIsMeasurable(group: Pick<LibGroup, "stories">): boolean {
    return group.stories.some((story) => Array.isArray(story.requiredStages) && story.requiredStages.length > 0);
}

function share(readers: number, players: number): number {
    return players > 0 ? readers / players : 0;
}

/**
 * The tab's four lists, or `null` when the backend served no aggregate.
 *
 * The group rankings cover the NON-RECORD groups only, 87 of the 451 on EN.
 * Two reasons, both structural: a heading that says "chapters and events" has
 * no business ranking an operator's records, and the wire lists every record
 * set twice, once under its `story_*_set_*` group id and once under the
 * operator's `charId`, so a combined ranking would carry each of them at two
 * different positions. The story ranking has no such duplicate, every story id
 * appearing once, so it ranks all 1,887.
 */
export function rankCommunity(index: LibIndex, community: StoryCommunity | null | undefined): ICommunityRanked | null {
    if (!community) return null;

    const players = community.players;
    const groupReaders = new Map(community.groups.map((g) => [g.id, g]));
    const storyReaders = new Map(community.stories.map((s) => [s.id, s.readers]));

    // Index order is the input order, and `sort` is stable in every engine this
    // ships to, so a tie between two groups keeps the order the library lists
    // them in rather than an arbitrary one.
    const ranked: ICommunityGroupRow[] = [];
    for (const group of index.groups) {
        const kind = kindOf(group);
        if (kind === "record") continue;
        const row = groupReaders.get(group.id);
        if (!row) continue;
        const measurable = finishedIsMeasurable(group);
        ranked.push({
            id: group.id,
            name: group.name,
            kind,
            readers: row.readers,
            readerShare: share(row.readers, players),
            finished: measurable ? row.finished : null,
            finishedShare: measurable ? share(row.finished, players) : null,
        });
    }

    const byReaders = [...ranked].sort((a, b) => b.readers - a.readers);
    const withReaders = byReaders.filter((row) => row.readers > 0);
    const bottom = [...withReaders].reverse().slice(0, BOTTOM_GROUPS);

    const names = new Map(index.groups.map((g) => [g.id, g.name]));
    const stories: ICommunityStoryRow[] = [];
    for (const group of index.groups) {
        for (const story of group.stories) {
            const readers = storyReaders.get(story.id);
            if (readers === undefined) continue;
            stories.push({
                id: story.id,
                name: story.name,
                code: story.code,
                groupName: names.get(story.groupId) ?? group.name,
                hasScript: story.hasScript,
                readers,
                readerShare: share(readers, players),
            });
        }
    }
    stories.sort((a, b) => b.readers - a.readers);

    // A curve is only drawable against the stories it was counted over, so a
    // group whose `depth` does not line up with the index one for one is left
    // out rather than drawn against the wrong titles. `depth` is ABSENT on
    // every operator record group by contract, which is the usual reason a
    // group has none.
    const depth: ICommunityDepthGroup[] = [];
    for (const group of index.groups) {
        const row = groupReaders.get(group.id);
        const curve = row?.depth;
        if (!curve || curve.length !== group.stories.length || curve.length < 2) continue;
        depth.push({
            id: group.id,
            name: group.name,
            bars: group.stories.map((story, at) => ({
                id: story.id,
                name: story.name,
                code: story.code,
                readers: curve[at],
                readerShare: share(curve[at], players),
            })),
        });
    }

    return {
        players,
        computedAt: community.computedAt,
        top: byReaders.slice(0, TOP_GROUPS),
        bottom,
        skippedZero: byReaders.length - withReaders.length,
        stories: stories.slice(0, TOP_STORIES),
        depth,
        defaultDepth: depth.some((g) => g.id === DEFAULT_DEPTH_GROUP) ? DEFAULT_DEPTH_GROUP : (depth[0]?.id ?? null),
    };
}
