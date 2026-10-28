/**
 * The chibi viewer's clip list: every animation in a skeleton, grouped and ordered from its
 * name alone, with its frame count and the spine events it fires. Nothing here is per
 * operator; a clip whose name fits no pattern lands in "other".
 */

export type AnimationGroupId = "idle" | "move" | "attack" | "skill" | "other";

export const ANIMATION_GROUP_ORDER: readonly AnimationGroupId[] = ["idle", "move", "attack", "skill", "other"];

/** First match wins. Dorm clips (Relax, Sit, Sleep, Interact, Special) rest, so they read as idle. */
const GROUP_PATTERNS: readonly [AnimationGroupId, RegExp][] = [
    ["skill", /^skill/i],
    ["attack", /^(attack|combat)/i],
    ["move", /^(move|run|walk|start)/i],
    ["idle", /^(idle|default|relax|sit|sleep|interact|special|stand)/i],
];

export function animationGroup(name: string): AnimationGroupId {
    for (const [group, pattern] of GROUP_PATTERNS) if (pattern.test(name)) return group;
    return "other";
}

/** Phases sort in play order, so chain play runs Begin, Loop, Idle, End. */
const PHASE_RANK: Record<string, number> = { begin: 0, start: 0, pre: 0, loop: 1, idle: 2, end: 3, post: 3 };
const LOOP_RANK = 1;

/** Tokens split on `_`, `-`, spaces and letter/number boundaries: `Skill_2_Begin` -> skill, 2, begin. */
function tokens(name: string): string[] {
    return name
        .split(/[_\-\s]+|(?<=[a-z])(?=\d)|(?<=\d)(?=[a-z])/i)
        .filter((token) => token.length > 0)
        .map((token) => token.toLowerCase());
}

/**
 * Orders two clips of one group token by token: a bare stem sits where a loop would (after
 * `_Begin`, before `_End`), a phase comes before a number (an unnumbered `Skill_Begin` is skill one, so it
 * comes before `Skill_2`), numbers by value (`Skill_2` before `Skill_10`), phases in play
 * order (Begin, Loop, Idle, End), anything else by name.
 */
export function compareAnimationNames(a: string, b: string): number {
    const ta = tokens(a);
    const tb = tokens(b);
    for (let i = 0; i < Math.max(ta.length, tb.length); i++) {
        const x = ta[i];
        const y = tb[i];
        // A bare stem beside its phases is the clip's body: it plays as the loop does, so
        // `Attack_Begin`, `Attack`, `Attack_End` chain in order.
        if (x === undefined) return y !== undefined && PHASE_RANK[y] !== undefined && PHASE_RANK[y] < LOOP_RANK ? 1 : -1;
        if (y === undefined) return PHASE_RANK[x] !== undefined && PHASE_RANK[x] < LOOP_RANK ? -1 : 1;
        if (x === y) continue;
        const nx = /^\d+$/.test(x) ? Number(x) : null;
        const ny = /^\d+$/.test(y) ? Number(y) : null;
        const px = PHASE_RANK[x];
        const py = PHASE_RANK[y];
        if (nx !== null && ny !== null) return nx - ny;
        if (nx !== null) return py !== undefined ? 1 : -1;
        if (ny !== null) return px !== undefined ? -1 : 1;
        if (px !== undefined && py !== undefined && px !== py) return px - py;
        if (px !== undefined && py === undefined) return -1;
        if (py !== undefined && px === undefined) return 1;
        return x < y ? -1 : 1;
    }
    return 0;
}

export interface IClipEvent {
    name: string;
    /** Seconds into the clip. */
    time: number;
}

export interface IClipInfo {
    name: string;
    group: AnimationGroupId;
    duration: number;
    frames: number;
    events: IClipEvent[];
}

export interface IAnimationGroup {
    id: AnimationGroupId;
    clips: IClipInfo[];
}

/** The slice of a runtime animation this module reads; the event timeline is the one with `events`. */
export interface IAnimationLike {
    name: string;
    duration: number;
    timelines: readonly unknown[];
}

interface IEventTimelineLike {
    events: readonly ({ data?: { name?: string }; time?: number } | null)[];
}

function isEventTimeline(timeline: unknown): timeline is IEventTimelineLike {
    return typeof timeline === "object" && timeline !== null && Array.isArray((timeline as { events?: unknown }).events);
}

/** A clip's frames at `fps`, at least one: the frame stepper counts the same way. */
export function clipFrameCount(duration: number, fps: number): number {
    return Math.max(1, Math.round(duration * fps));
}

export function clipInfo(animation: IAnimationLike, fps: number): IClipInfo {
    const events: IClipEvent[] = [];
    for (const timeline of animation.timelines) {
        if (!isEventTimeline(timeline)) continue;
        for (const event of timeline.events) {
            if (!event) continue;
            events.push({ name: event.data?.name ?? "", time: event.time ?? 0 });
        }
    }
    events.sort((a, b) => a.time - b.time);
    return { name: animation.name, group: animationGroup(animation.name), duration: animation.duration, frames: clipFrameCount(animation.duration, fps), events };
}

/** Non-empty groups in {@link ANIMATION_GROUP_ORDER}, each sorted by {@link compareAnimationNames}. */
export function groupAnimations(clips: readonly IClipInfo[]): IAnimationGroup[] {
    return ANIMATION_GROUP_ORDER.map((id) => ({ id, clips: clips.filter((clip) => clip.group === id).sort((a, b) => compareAnimationNames(a.name, b.name)) })).filter((group) => group.clips.length > 0);
}

/** The clip after `current` in list order, wrapping at the end; chain play follows it. */
export function nextClipName(groups: readonly IAnimationGroup[], current: string): string | null {
    const order = groups.flatMap((group) => group.clips.map((clip) => clip.name));
    if (order.length === 0) return null;
    const index = order.indexOf(current);
    return order[(index + 1) % order.length] ?? null;
}
