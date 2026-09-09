import { queryOptions } from "@tanstack/react-query";
import { createServerFn } from "@tanstack/react-start";
import { backendFetch } from "#/lib/fetch";
import { optionalSiteToken } from "./_shared.server";
import type { IAssignedOperator, IBaseAssignment, IShiftRotation } from "./user";

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

// Response shapes are generated from `backend/src/app/services/base_planner.rs`.
// To change a field, edit the Rust struct and run `bun run gen:types`.
//
// `room_type` / `formula_type` are `String` there (the value sets are game
// constants the planner reads out of `building_data`, not a Rust enum), so they
// are narrowed here with `Refine` - which fails to compile if the field is
// renamed upstream, unlike a bare `Omit`.
import type { CatalogFormulaDto } from "#/types/generated/CatalogFormulaDto";
import type { CatalogPhaseDto } from "#/types/generated/CatalogPhaseDto";
import type { CatalogResponse } from "#/types/generated/CatalogResponse";
import type { CatalogRoomDto } from "#/types/generated/CatalogRoomDto";
import type { CatalogSlotDto } from "#/types/generated/CatalogSlotDto";
import type { CatalogStoreyDto } from "#/types/generated/CatalogStoreyDto";
import type { ClaimDto } from "#/types/generated/ClaimDto";
import type { ClaimIntervalDto } from "#/types/generated/ClaimIntervalDto";
import type { DormDto } from "#/types/generated/DormDto";
import type { DormsDto } from "#/types/generated/DormsDto";
import type { DraftRoom } from "#/types/generated/DraftRoom";
import type { DronesDto } from "#/types/generated/DronesDto";
import type { EvaluateResponse } from "#/types/generated/EvaluateResponse";
import type { LayoutResponse } from "#/types/generated/LayoutResponse";
import type { OptimizeResponse } from "#/types/generated/OptimizeResponse";
import type { PowerDto } from "#/types/generated/PowerDto";
import type { RoomDiffDto } from "#/types/generated/RoomDiffDto";
import type { RotationResponse } from "#/types/generated/RotationResponse";
import type { SlotPresetsDto } from "#/types/generated/SlotPresetsDto";
import type { SustainEntryDto } from "#/types/generated/SustainEntryDto";
import type { TrainerHintDto } from "#/types/generated/TrainerHintDto";
import type { Refine } from "#/types/refine";

/** One room of a layout the player is editing. */
export type IDraftRoom = Refine<DraftRoom, { room_type: RoomType; formula_type: FormulaType | null }>;

export type IPower = PowerDto;

/**
 * One operator's endurance in the room the draft puts them in. Drain is the
 * game's own per-hour figure, so this is not an estimate.
 */
export type ISustainEntry = Refine<SustainEntryDto, { room_type: RoomType }>;

/**
 * The dormitories' contribution. They produce nothing, so they never appear in
 * a `BaseAssignment` - but they set recovery speed and how many operators can
 * rest at once, which is what decides whether a staffing survives its rhythm.
 */
export type IDorms = DormsDto;
export type IDorm = DormDto;

export type IEvaluateResponse = EvaluateResponse;
export type ITrainerHint = TrainerHintDto;
export type IDrones = DronesDto;

/** How long the base runs unattended, and what each claim cadence loses. */
export type IClaim = ClaimDto;
export type IClaimInterval = ClaimIntervalDto;

/**
 * Player-declared account state the sync cannot read. Undeclared facts keep
 * the never-guess default: the dependent skills price 0.
 *
 * This is the REQUEST shape, which is why it stays hand-written: the fields are
 * optional going out (`facts ?? {}`), while the generated `AccountFactsReq`
 * describes them coming back, where the backend always fills them in.
 */
export interface IAccountFacts {
    /** Recruit slots purchased beyond the initial one (0-3). */
    open_recruit_slots?: number;
    /** The class currently training ("Guard", "Sniper", ...) - ranks trainer hints. */
    training_class?: string | null;
}

export type IRoomDiff = Refine<RoomDiffDto, { room_type: RoomType }>;
export type IOptimizeResponse = OptimizeResponse;

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

export type ICatalogPhase = CatalogPhaseDto;
export type ICatalogRoom = Refine<CatalogRoomDto, { room_type: FacilityType }>;
export type ICatalogFormula = Refine<CatalogFormulaDto, { formula_type: FormulaType }>;

/**
 * One slot of the base floorplan.
 *
 * Coordinates are in the game's own half-tile units: every room is 2 units
 * tall and an even number wide, while an elevator is 1 unit wide - which is
 * what makes the lift shafts half-width columns on the board.
 */
export type ICatalogSlot = CatalogSlotDto;
export type ICatalogStorey = CatalogStoreyDto;
export type ICatalogResponse = Refine<CatalogResponse, { rooms: ICatalogRoom[]; formulas: ICatalogFormula[] }>;

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

export type IRotationResponse = RotationResponse;

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
export type ISlotPresets = SlotPresetsDto;

export type ILayoutResponse = Refine<LayoutResponse, { rooms: IDraftRoom[] }>;

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
            if (res.status === 404 || res.status === 403) return { rooms: [], presets: [], facts: { open_recruit_slots: null, training_class: null } } satisfies ILayoutResponse;
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
        queryKey: ["base", "rotation", uid, layoutKey(layout), ignorePromotion, facts?.open_recruit_slots ?? 0, facts?.training_class ?? "", bearerToken ? "auth" : "anon"],
        queryFn: () => rotationPlanFn({ data: { uid, layout, ignorePromotion, facts, bearerToken } }),
        enabled: layout.length > 0,
        staleTime: 5 * 60 * 1000,
        placeholderData: (prev) => prev,
    });
}

export function evaluateLayoutQueryOptions(uid: string, layout: IDraftRoom[], ignorePromotion: boolean, facts?: IAccountFacts, claimIntervalHours?: number, bearerToken?: string) {
    return queryOptions({
        queryKey: ["base", "evaluate", uid, layoutKey(layout), ignorePromotion, facts?.open_recruit_slots ?? 0, facts?.training_class ?? "", claimIntervalHours ?? 0, bearerToken ? "auth" : "anon"],
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
