import { queryOptions } from "@tanstack/react-query";
import { createServerFn } from "@tanstack/react-start";
import { backendFetch } from "#/lib/fetch";
import { optionalSiteToken } from "./_shared.server";
import type { IAssignedOperator, IBaseAssignment, IShiftRotation, ISustainability } from "./user";

/**
 * The interactive base planner.
 *
 * Every number here comes from the Rust clause engine - the same one behind
 * `/user/improvements`. This module deliberately contains no scoring math: it
 * ships the drafted layout to the backend and renders what comes back. If you
 * find yourself wanting to "just estimate" an efficiency client-side, that's the
 * seam where the two engines start to disagree.
 */

/** Game room constants, as they appear in `building_data`. */
export type RoomType = "CONTROL" | "MANUFACTURE" | "TRADING" | "POWER" | "DORMITORY" | "MEETING" | "HIRE" | "TRAINING" | "WORKSHOP";

/**
 * Every facility `building_data` defines - the nine staffable rooms plus the
 * three the planner never staffs: the Activity Room, and the structure itself.
 * The catalogue carries all twelve; only `RoomType` reaches the engine.
 */
export type FacilityType = RoomType | "PRIVATE" | "ELEVATOR" | "CORRIDOR";

export type FormulaType = "F_GOLD" | "F_EXP" | "F_DIAMOND";

/** One room of a layout the player is editing. */
export interface IDraftRoom {
    slot_id: string;
    room_type: RoomType;
    level: number;
    operators: string[];
    formula_type?: FormulaType | null;
    /** Dormitory ambience (0-5000), carried from the synced base. */
    comfort?: number;
}

export interface IPower {
    generated: number;
    consumed: number;
    net: number;
}

/**
 * One operator's endurance in the room the draft puts them in. Drain is the
 * game's own per-hour figure, so this is not an estimate.
 */
export interface ISustainEntry {
    operator_id: string;
    name: string;
    slot_id: string;
    room_type: RoomType;
    drain_per_hour: number;
    /** Hours until empty from the current bar (when the sync knows it), else from full. `null` = never depletes. */
    lasts_hours: number | null;
    /** Morale as of the last account sync (0-24), when known. */
    morale?: number;
}

/**
 * The dormitories' contribution. They produce nothing, so they never appear in
 * a `BaseAssignment` - but they set recovery speed and how many operators can
 * rest at once, which is what decides whether a staffing survives its rhythm.
 */
export interface IDorms {
    count: number;
    /** Sum of dorm levels - the `&dorm&lv` scaling the clause engine reads. */
    total_levels: number;
    /** Operators the base can rest at once. */
    total_capacity: number;
    /** Morale restored per hour to a resting operator. */
    recovery_per_hour: number;
    /** Each dormitory, best first - the order resters fill them. */
    per_dorm?: IDorm[];
}

export interface IDorm {
    slot_id: string;
    level: number;
    capacity: number;
    recovery_per_hour: number;
    /** Strongest whole-dorm recovery aura among the occupants, non-stacking. */
    occupant_aura_per_hour: number;
    /** Strongest single-target recovery among the occupants. */
    occupant_single_per_hour: number;
    /** Furniture ambience (0 to this level's cap). */
    comfort?: number;
    comfort_limit?: number;
    /** Recovery/hr still available if ambience were maxed. */
    comfort_upside_per_hour?: number;
}

export interface IEvaluateResponse {
    assignment: IBaseAssignment;
    power: IPower;
    sustain: ISustainEntry[];
    dorms: IDorms;
    /** Check-in economics; absent when nothing produces. */
    claim?: IClaim;
    /** The drafted crews simulated with NO rotation - "if you never swap".
     *  Seeded with projected live morale, so its clock starts now. */
    unrotated?: ISustainability;
    /** Hours since the newest morale write in the sync. */
    morale_synced_hours_ago?: number;
    /** Top trainers for the declared training class. */
    trainer_hints?: ITrainerHint[];
    /** Drone buffer, projected to now. */
    drones?: IDrones;
}

export interface ITrainerHint {
    operator: IAssignedOperator;
    /** Specialization training speed % of their best matching skill. */
    value_pct: number;
}

export interface IDrones {
    current: number;
    max: number;
    /** Hours until the buffer caps and recovery is wasted. Absent = full. */
    full_in_hours?: number;
}

/** How long the base runs unattended, and what each claim cadence loses. */
export interface IClaim {
    /** Hours from a claim now until the FIRST production room stalls. */
    next_full_hours: number;
    /** Steady-state losses at each check-in cadence (6h / 12h / 24h). */
    intervals: IClaimInterval[];
}

export interface IClaimInterval {
    hours: number;
    lost_lmd_per_day: number;
    lost_gold_per_day: number;
    lost_exp_per_day: number;
}

/**
 * Player-declared account state the sync cannot read. Undeclared facts keep
 * the never-guess default: the dependent skills price 0.
 */
export interface IAccountFacts {
    /** Recruit slots purchased beyond the initial one (0-3). */
    open_recruit_slots?: number;
    /** The class currently training ("Guard", "Sniper", ...) - ranks trainer hints. */
    training_class?: string | null;
}

export interface IRoomDiff {
    slot_id: string;
    room_type: RoomType;
    before: string[];
    after: string[];
    efficiency_before: number;
    efficiency_after: number;
    yield_before: number;
    yield_after: number;
}

export interface IOptimizeResponse {
    proposal: IBaseAssignment;
    baseline: IBaseAssignment;
    room_diffs: IRoomDiff[];
    power: IPower;
}

export interface IOptimizeInput {
    uid: string;
    layout: IDraftRoom[];
    /** Slot ids the optimizer may restaff. Empty = every room. */
    scope?: string[];
    /** Operators pinned exactly where the draft puts them. */
    locked?: string[];
    /** Operators the plan may not seat anywhere. */
    excluded?: string[];
    /**
     * Plan with every operator's highest base skills, whether or not the player has
     * promoted them that far - a target to build toward, not their base today.
     */
    ignorePromotion?: boolean;
    facts?: IAccountFacts;
}

export interface ICatalogPhase {
    level: number;
    max_stationed: number;
    /** Positive generates, negative consumes. */
    electricity: number;
    manpower_cost: number;
}

export interface ICatalogRoom {
    room_type: FacilityType;
    name: string;
    category: string;
    /** -1 means unlimited. */
    max_count: number;
    size_col: number;
    size_row: number;
    /** Index = level - 1. */
    phases: ICatalogPhase[];
}

export interface ICatalogFormula {
    formula_type: FormulaType;
    label: string;
}

/**
 * One slot of the base floorplan.
 *
 * Coordinates are in the game's own half-tile units: every room is 2 units
 * tall and an even number wide, while an elevator is 1 unit wide - which is
 * what makes the lift shafts half-width columns on the board.
 */
export interface ICatalogSlot {
    slot_id: string;
    /** Room category this slot accepts - matches `ICatalogRoom.category`. */
    category: string;
    storey_id: string;
    offset_col: number;
    /** Absolute, counting up from the bottom of the base: B4 is 0, 1F is 8. */
    offset_row: number;
    size_col: number;
    size_row: number;
}

export interface ICatalogStorey {
    storey_id: string;
    /** Control-centre level that unlocks this floor. */
    unlock_control_level: number;
}

export interface ICatalogResponse {
    /**
     * Shifts in a base day - a game constant, not a property of any one player.
     * A base whose rooms only queue two presets still runs three shifts,
     * alternating across them. Optional so an older backend still renders.
     */
    shift_count?: number;
    rooms: ICatalogRoom[];
    formulas: ICatalogFormula[];
    slots: ICatalogSlot[];
    storeys: ICatalogStorey[];
}

export const evaluateLayoutFn = createServerFn({ method: "POST" })
    .inputValidator((data: { uid: string; layout: IDraftRoom[]; ignorePromotion?: boolean; facts?: IAccountFacts; claimIntervalHours?: number; bearerToken?: string }) => data)
    .handler(async ({ data: { uid, layout, ignorePromotion, facts, claimIntervalHours, bearerToken } }) => {
        const token = bearerToken ?? optionalSiteToken();
        const res = await backendFetch(`/base/evaluate?uid=${encodeURIComponent(uid)}`, {
            method: "POST",
            bearerToken: token,
            body: JSON.stringify({ layout, ignore_promotion: ignorePromotion ?? false, facts: facts ?? {}, claim_interval_hours: claimIntervalHours ?? null }),
        });
        if (!res.ok) {
            const text = await res.text().catch(() => "");
            throw new Error(text || `Failed to evaluate layout: ${res.status}`);
        }
        return (await res.json()) as IEvaluateResponse;
    });

export const optimizeLayoutFn = createServerFn({ method: "POST" })
    .inputValidator((data: IOptimizeInput & { bearerToken?: string }) => data)
    .handler(async ({ data: { uid, layout, scope, locked, excluded, ignorePromotion, facts, bearerToken } }) => {
        const token = bearerToken ?? optionalSiteToken();
        const res = await backendFetch(`/base/optimize?uid=${encodeURIComponent(uid)}`, {
            method: "POST",
            bearerToken: token,
            body: JSON.stringify({
                ignore_promotion: ignorePromotion ?? false,
                facts: facts ?? {},
                layout,
                scope: scope ?? [],
                locked: locked ?? [],
                excluded: excluded ?? [],
            }),
        });
        if (!res.ok) {
            const text = await res.text().catch(() => "");
            throw new Error(text || `Failed to optimize layout: ${res.status}`);
        }
        return (await res.json()) as IOptimizeResponse;
    });

export interface IRotationInput {
    uid: string;
    layout: IDraftRoom[];
    locked?: string[];
    excluded?: string[];
    /**
     * Plan with every operator's highest base skills, whether or not the player has
     * promoted them that far - a target to build toward, not their base today.
     */
    ignorePromotion?: boolean;
    facts?: IAccountFacts;
}

export interface IRotationResponse {
    rotation: IShiftRotation;
    /** Shifts per day the rotation assumes - never hardcode it. */
    shift_count: number;
}

export const rotationPlanFn = createServerFn({ method: "POST" })
    .inputValidator((data: IRotationInput & { bearerToken?: string }) => data)
    .handler(async ({ data: { uid, layout, locked, excluded, ignorePromotion, facts, bearerToken } }) => {
        const token = bearerToken ?? optionalSiteToken();
        const res = await backendFetch(`/base/rotation?uid=${encodeURIComponent(uid)}`, {
            method: "POST",
            bearerToken: token,
            body: JSON.stringify({ layout, locked: locked ?? [], excluded: excluded ?? [], ignore_promotion: ignorePromotion ?? false, facts: facts ?? {} }),
        });
        if (!res.ok) {
            const text = await res.text().catch(() => "");
            throw new Error(text || `Failed to plan rotation: ${res.status}`);
        }
        return (await res.json()) as IRotationResponse;
    });

/** One slot's saved rotation: the crew the player has queued for each shift. */
export interface ISlotPresets {
    slot_id: string;
    shifts: string[][];
}

export interface ILayoutResponse {
    rooms: IDraftRoom[];
    /** The owner's saved account facts, so every viewer scores with them. */
    facts?: IAccountFacts;
    /**
     * The player's own shift rotation, out of the game's `presetQueue`. Kept
     * apart from `rooms` because that is the shape posted back for scoring;
     * presets are read-only context and never travel with a draft.
     *
     * Optional so a backend that predates the field still renders.
     */
    presets?: ISlotPresets[];
}

export const getBaseLayoutFn = createServerFn({ method: "GET" })
    .inputValidator((data: { uid: string; bearerToken?: string }) => data)
    .handler(async ({ data: { uid, bearerToken } }) => {
        const token = bearerToken ?? optionalSiteToken();
        const res = await backendFetch(`/base/layout?uid=${encodeURIComponent(uid)}`, { bearerToken: token });
        if (!res.ok) {
            // An empty payload, not `null`: a server function's `null` comes back
            // over the wire as `undefined`, which React Query rejects outright
            // ("Query data cannot be undefined") - turning "this profile has no
            // base" into a hard query error and a board with nothing on it.
            if (res.status === 404 || res.status === 403) return { rooms: [], presets: [] } satisfies ILayoutResponse;
            throw new Error(`Failed to load base layout: ${res.status}`);
        }
        return (await res.json()) as ILayoutResponse;
    });

/**
 * The player's real stationed base - every built slot, not just the rooms a
 * plan has an opinion about. This is the planner's seed; a `BaseAssignment`
 * from `/user/improvements` is NOT a substitute, because it carries only
 * production rooms and a staffed Control Center.
 */
export function baseLayoutQueryOptions(uid: string, bearerToken?: string) {
    return queryOptions({
        queryKey: ["base", "layout", uid, bearerToken ? "auth" : "anon"],
        queryFn: () => getBaseLayoutFn({ data: { uid, bearerToken } }),
        staleTime: 5 * 60 * 1000,
    });
}

export const getBaseCatalogFn = createServerFn({ method: "GET" }).handler(async () => {
    const res = await backendFetch("/base/catalog");
    if (!res.ok) throw new Error(`Failed to load base catalog: ${res.status}`);
    return (await res.json()) as ICatalogResponse;
});

/**
 * Facility definitions. Roster-independent and changes only when the game
 * does, so it is cached for the session rather than refetched per edit.
 */
export function baseCatalogQueryOptions() {
    return queryOptions({
        queryKey: ["base", "catalog"],
        queryFn: () => getBaseCatalogFn(),
        staleTime: Number.POSITIVE_INFINITY,
    });
}

/**
 * Score a drafted layout. Keyed by the layout itself, so react-query dedupes
 * repeat edits that land back on an arrangement already scored - dragging an
 * operator out and back costs nothing.
 */
export const saveBaseFactsFn = createServerFn({ method: "POST" })
    .inputValidator((data: { facts: IAccountFacts; bearerToken?: string }) => data)
    .handler(async ({ data: { facts, bearerToken } }) => {
        const token = bearerToken ?? optionalSiteToken();
        if (!token) return { saved: false };
        const res = await backendFetch("/base/facts", {
            method: "PUT",
            bearerToken: token,
            body: JSON.stringify(facts),
        });
        // Not being the owner (or signed out) is a normal outcome: the toggle
        // still works as a per-request what-if, it just doesn't persist.
        return { saved: res.ok };
    });

export function rotationPlanQueryOptions(uid: string, layout: IDraftRoom[], ignorePromotion: boolean, facts?: IAccountFacts, bearerToken?: string) {
    return queryOptions({
        queryKey: ["base", "rotation", uid, layoutKey(layout), ignorePromotion, facts?.open_recruit_slots ?? 0, bearerToken ? "auth" : "anon"],
        queryFn: () => rotationPlanFn({ data: { uid, layout, ignorePromotion, facts, bearerToken } }),
        enabled: layout.length > 0,
        staleTime: 5 * 60 * 1000,
        placeholderData: (prev) => prev,
    });
}

export function evaluateLayoutQueryOptions(uid: string, layout: IDraftRoom[], ignorePromotion: boolean, facts?: IAccountFacts, claimIntervalHours?: number, bearerToken?: string) {
    return queryOptions({
        queryKey: ["base", "evaluate", uid, layoutKey(layout), ignorePromotion, facts?.open_recruit_slots ?? 0, claimIntervalHours ?? 0, bearerToken ? "auth" : "anon"],
        queryFn: () => evaluateLayoutFn({ data: { uid, layout, ignorePromotion, facts, claimIntervalHours, bearerToken } }),
        enabled: layout.length > 0,
        staleTime: 5 * 60 * 1000,
        // A draft in progress is a sequence of near-identical layouts; keeping
        // the previous answer on screen avoids a flash of empty rooms between
        // one edit and the next.
        placeholderData: (prev) => prev,
    });
}

/**
 * A stable identity for a layout: room shape plus crew, order-insensitive
 * within a room. Two drafts that differ only in seat order score identically,
 * so they should share a cache entry.
 */
function layoutKey(layout: IDraftRoom[]): string {
    return layout
        .map((r) => `${r.slot_id}:${r.room_type}:${r.level}:${r.formula_type ?? ""}:${[...r.operators].sort().join(",")}`)
        .sort()
        .join("|");
}

export type { IAssignedOperator, IBaseAssignment, IShiftRotation };
