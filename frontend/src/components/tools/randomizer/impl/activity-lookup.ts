import type { IActivity, IRetroAct } from "#/types/stages";

const ZONE_ACTIVITY_RE = /^([a-z0-9]+)_zone\d+$/i;
const ZONE_PERMANENT_RE = /^(permanent_(?:sidestory|sub|main)_\d+)_zone\d+$/i;
const RETRO_PREFIX_RE = /^(permanent_(?:sidestory|sub|main)_\d+)(?:_|$)/i;

export interface PermanentEvent {
    retroId: string;
    name: string;
    type: "SIDESTORY" | "BRANCHLINE";
    index: number;
}

export interface ActivityLookup {
    /** activityId -> IActivity */
    activityById: Map<string, IActivity>;
    /** "permanent_sidestory_1" -> retro act */
    retroByZonePrefix: Map<string, IRetroAct>;
    /** activityId -> retro act (covers each linked event, including reruns) */
    retroByActivityId: Map<string, IRetroAct>;
}

export const EMPTY_ACTIVITY_LOOKUP: ActivityLookup = {
    activityById: new Map(),
    retroByZonePrefix: new Map(),
    retroByActivityId: new Map(),
};

function getRetroZonePrefix(retroId: string): string | null {
    return retroId.match(RETRO_PREFIX_RE)?.[1] ?? null;
}

export function buildActivityLookup(activities: IActivity[] | undefined, retroActs: IRetroAct[] | undefined): ActivityLookup {
    const activityById = new Map<string, IActivity>();
    for (const act of activities ?? []) {
        if (act.id) activityById.set(act.id, act);
    }

    const retroByZonePrefix = new Map<string, IRetroAct>();
    const retroByActivityId = new Map<string, IRetroAct>();
    for (const retro of retroActs ?? []) {
        const prefix = getRetroZonePrefix(retro.retroId);
        if (prefix) retroByZonePrefix.set(prefix, retro);
        for (const linked of retro.linkedActId ?? []) {
            if (linked) retroByActivityId.set(linked, retro);
        }
    }

    return { activityById, retroByZonePrefix, retroByActivityId };
}

/** "act43side_zone1" -> "act43side". Returns null for permanent or unrecognized zones. */
export function getActivityIdFromZoneId(zoneId: string): string | null {
    return zoneId.match(ZONE_ACTIVITY_RE)?.[1] ?? null;
}

/** "permanent_sidestory_1_zone2" -> "permanent_sidestory_1". */
export function getPermanentZonePrefix(zoneId: string): string | null {
    return zoneId.match(ZONE_PERMANENT_RE)?.[1] ?? null;
}

export function getEventNameFromZoneId(zoneId: string, lookup: ActivityLookup): string {
    const activityId = getActivityIdFromZoneId(zoneId);
    if (activityId) {
        const name = lookup.activityById.get(activityId)?.name;
        if (name) return name;
    }
    return zoneId;
}

export function isRerunActivity(zoneId: string, lookup: ActivityLookup): boolean {
    const activityId = getActivityIdFromZoneId(zoneId);
    if (!activityId) return false;
    return lookup.activityById.get(activityId)?.isReplicate ?? false;
}

export function getActivityStartTime(activityId: string, lookup: ActivityLookup): number {
    return lookup.activityById.get(activityId)?.startTime ?? 0;
}

export function isActivityCurrentlyOpen(activityId: string, lookup: ActivityLookup, currentTime?: number): boolean {
    const act = lookup.activityById.get(activityId);
    if (!act) return false;
    const now = currentTime ?? Math.floor(Date.now() / 1000);
    return act.startTime <= now && now <= act.endTime;
}

export function getActivityEventTimes(activityId: string, lookup: ActivityLookup): { start: number; end: number } | null {
    const act = lookup.activityById.get(activityId);
    return act ? { start: act.startTime, end: act.endTime } : null;
}

export function getPermanentZoneName(zoneId: string, lookup: ActivityLookup): string | null {
    const prefix = getPermanentZonePrefix(zoneId);
    return prefix ? (lookup.retroByZonePrefix.get(prefix)?.name ?? null) : null;
}

export function getPermanentZoneStartTime(zoneId: string, lookup: ActivityLookup): number {
    const prefix = getPermanentZonePrefix(zoneId);
    return prefix ? (lookup.retroByZonePrefix.get(prefix)?.startTime ?? 0) : 0;
}

export function getPermanentEventInfo(activityId: string, lookup: ActivityLookup): PermanentEvent | null {
    const retro = lookup.retroByActivityId.get(activityId);
    if (!retro) return null;
    if (retro.type !== "SIDESTORY" && retro.type !== "BRANCHLINE") return null;
    return {
        retroId: retro.retroId,
        name: retro.name,
        type: retro.type,
        index: retro.index,
    };
}

export function isPermanentEvent(activityId: string, lookup: ActivityLookup): boolean {
    return getPermanentEventInfo(activityId, lookup) !== null;
}

export function formatEventDate(timestamp: number): string {
    const date = new Date(timestamp * 1000);
    return date.toLocaleDateString("en-US", {
        month: "short",
        day: "numeric",
        year: "numeric",
    });
}
