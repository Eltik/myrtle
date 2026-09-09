import { queryOptions } from "@tanstack/react-query";
import { createServerFn } from "@tanstack/react-start";
import { backendFetch } from "#/lib/fetch";
// Generated from `backend/src/app/services/{improvements,stats,gacha}.rs`,
// `backend/src/database/models/*` and `backend/src/core/grade/grade_operators.rs`.
// To change a field, edit the Rust struct and run `bun run gen:types`.
import type { AssignedOperator } from "#/types/generated/AssignedOperator";
import type { BaseAssignmentDto } from "#/types/generated/BaseAssignmentDto";
import type { BaseImprovements } from "#/types/generated/BaseImprovements";
import type { DepletedOperatorDto } from "#/types/generated/DepletedOperatorDto";
import type { EncounteredEnemiesResponse } from "#/types/generated/EncounteredEnemiesResponse";
import type { EncounteredEnemy } from "#/types/generated/EncounteredEnemy";
import type { FacilityOutputDto } from "#/types/generated/FacilityOutputDto";
import type { ImprovementsResponse } from "#/types/generated/ImprovementsResponse";
import type { ItemEntry } from "#/types/generated/ItemEntry";
import type { LeaderboardEntry } from "#/types/generated/LeaderboardEntry";
import type { LeaderboardMover } from "#/types/generated/LeaderboardMover";
import type { LeaderboardPage } from "#/types/generated/LeaderboardPage";
import type { MedalGap } from "#/types/generated/MedalGap";
import type { MedalImprovements } from "#/types/generated/MedalImprovements";
import type { MedalOperatorLock } from "#/types/generated/MedalOperatorLock";
import type { MoraleTimelineDto } from "#/types/generated/MoraleTimelineDto";
import type { MovedOperator } from "#/types/generated/MovedOperator";
import type { NonProdEffectDto } from "#/types/generated/NonProdEffectDto";
import type { OperatorGap } from "#/types/generated/OperatorGap";
import type { OperatorImprovements } from "#/types/generated/OperatorImprovements";
import type { PerceptionConsumerDto } from "#/types/generated/PerceptionConsumerDto";
import type { PerceptionPlanDto } from "#/types/generated/PerceptionPlanDto";
import type { PerceptionSupportDto } from "#/types/generated/PerceptionSupportDto";
import type { PlayerStanding } from "#/types/generated/PlayerStanding";
import type { ProgressPair } from "#/types/generated/ProgressPair";
import type { RoguelikeCollectibles } from "#/types/generated/RoguelikeCollectibles";
import type { RoguelikeDifficulty } from "#/types/generated/RoguelikeDifficulty";
import type { RoguelikeThemeImprovement } from "#/types/generated/RoguelikeThemeImprovement";
import type { RoomAssignmentDto } from "#/types/generated/RoomAssignmentDto";
import type { RoomLayoutEntry } from "#/types/generated/RoomLayoutEntry";
import type { RoomRotationDto } from "#/types/generated/RoomRotationDto";
import type { RosterEntry } from "#/types/generated/RosterEntry";
import type { RotationDto } from "#/types/generated/RotationDto";
import type { RotationInfo } from "#/types/generated/RotationInfo";
import type { RotationMemberDto } from "#/types/generated/RotationMemberDto";
import type { RotationSetDto } from "#/types/generated/RotationSetDto";
import type { RotationSetRoomDto } from "#/types/generated/RotationSetRoomDto";
import type { SandboxCategory } from "#/types/generated/SandboxCategory";
import type { SandboxImprovements } from "#/types/generated/SandboxImprovements";
import type { SandboxPart } from "#/types/generated/SandboxPart";
import type { ScoreDimension } from "#/types/generated/ScoreDimension";
import type { ScoreHistoryPoint } from "#/types/generated/ScoreHistoryPoint";
import type { SearchPage } from "#/types/generated/SearchPage";
import type { ServerShare } from "#/types/generated/ServerShare";
import type { ShiftDto } from "#/types/generated/ShiftDto";
import type { ShiftRoomDto } from "#/types/generated/ShiftRoomDto";
import type { ShiftRotationDto } from "#/types/generated/ShiftRotationDto";
import type { SkillLineDto } from "#/types/generated/SkillLineDto";
import type { StageGap } from "#/types/generated/StageGap";
import type { StageImprovements } from "#/types/generated/StageImprovements";
import type { StagePoolImprovements } from "#/types/generated/StagePoolImprovements";
import type { SustainabilityDto } from "#/types/generated/SustainabilityDto";
import type { UpgradeDelta } from "#/types/generated/UpgradeDelta";
import type { UserScore } from "#/types/generated/UserScore";
import type { Refine } from "#/types/refine";
import type { IUserCheckin, IUserProfile } from "#/types/user";
import { optionalSiteToken } from "./_shared.server";

export interface IRosterMastery {
    index: number;
    mastery: number;
}

export interface IRosterModule {
    id: string;
    level: number;
    locked: boolean;
}

export type IRosterEntry = Refine<RosterEntry, { masteries: IRosterMastery[]; modules: IRosterModule[] }>;

export type IUserScore = UserScore;

export const getUserFn = createServerFn({ method: "GET" })
    .inputValidator((uid: string) => uid)
    .handler(async ({ data: uid }) => {
        const res = await backendFetch(`/get-user?uid=${encodeURIComponent(uid)}`);
        if (!res.ok) {
            if (res.status === 404) return null;
            throw new Error(`Failed to load user: ${res.status}`);
        }
        return (await res.json()) as IUserProfile;
    });

export function userQueryOptions(uid: string) {
    return queryOptions({
        queryKey: ["user", "profile", uid],
        queryFn: () => getUserFn({ data: uid }),
        staleTime: 60 * 1000,
        gcTime: 5 * 60 * 1000,
    });
}

export const getUserRosterFn = createServerFn({ method: "GET" })
    .inputValidator((data: { uid: string; bearerToken?: string }) => data)
    .handler(async ({ data: { uid, bearerToken } }) => {
        const token = bearerToken ?? optionalSiteToken();
        const res = await backendFetch(`/roster?uid=${encodeURIComponent(uid)}`, { bearerToken: token });
        if (!res.ok) {
            if (res.status === 404) return null;
            throw new Error(`Failed to load roster: ${res.status}`);
        }
        return (await res.json()) as IRosterEntry[];
    });

export function userRosterQueryOptions(uid: string, bearerToken?: string) {
    return queryOptions({
        queryKey: ["user", "roster", uid, bearerToken ? "auth" : "anon"],
        queryFn: () => getUserRosterFn({ data: { uid, bearerToken } }),
        staleTime: 60 * 1000,
        gcTime: 5 * 60 * 1000,
    });
}

export const getUserRosterOperatorFn = createServerFn({ method: "GET" })
    .inputValidator((data: { uid: string; operatorId: string }) => data)
    .handler(async ({ data: { uid, operatorId } }) => {
        const res = await backendFetch(`/roster/${encodeURIComponent(operatorId)}?uid=${encodeURIComponent(uid)}`, { bearerToken: optionalSiteToken() });
        if (!res.ok) {
            if (res.status === 404) return null;
            throw new Error(`Failed to load roster operator: ${res.status}`);
        }
        return (await res.json()) as IRosterEntry;
    });

export function userRosterOperatorQueryOptions(uid: string, operatorId: string) {
    return queryOptions({
        queryKey: ["user", "roster", uid, operatorId],
        queryFn: () => getUserRosterOperatorFn({ data: { uid, operatorId } }),
        staleTime: 60 * 1000,
        gcTime: 5 * 60 * 1000,
    });
}

export type IInventoryItem = ItemEntry;

export const getUserInventoryFn = createServerFn({ method: "GET" })
    .inputValidator((data: { uid: string; bearerToken?: string }) => data)
    .handler(async ({ data: { uid, bearerToken } }) => {
        const token = bearerToken ?? optionalSiteToken();
        const res = await backendFetch(`/inventory?uid=${encodeURIComponent(uid)}`, { bearerToken: token });
        if (!res.ok) {
            if (res.status === 404) return null;
            throw new Error(`Failed to load inventory: ${res.status}`);
        }
        return (await res.json()) as IInventoryItem[];
    });

export function userInventoryQueryOptions(uid: string, bearerToken?: string) {
    return queryOptions({
        queryKey: ["user", "inventory", uid, bearerToken ? "auth" : "anon"],
        queryFn: () => getUserInventoryFn({ data: { uid, bearerToken } }),
        staleTime: 60 * 1000,
        gcTime: 5 * 60 * 1000,
    });
}

export type IEncounteredEnemy = EncounteredEnemy;

export type IEncounteredEnemies = EncounteredEnemiesResponse;

export const getUserEncounteredEnemiesFn = createServerFn({ method: "GET" })
    .inputValidator((data: { uid: string; bearerToken?: string }) => data)
    .handler(async ({ data: { uid, bearerToken } }) => {
        const token = bearerToken ?? optionalSiteToken();
        const res = await backendFetch(`/encountered-enemies?uid=${encodeURIComponent(uid)}`, { bearerToken: token });
        if (!res.ok) {
            // 403 = private profile, 404 = no such user - treat both as "unavailable".
            if (res.status === 404 || res.status === 403) return null;
            throw new Error(`Failed to load encountered enemies: ${res.status}`);
        }
        return (await res.json()) as IEncounteredEnemies;
    });

export function userEncounteredEnemiesQueryOptions(uid: string, bearerToken?: string) {
    return queryOptions({
        queryKey: ["user", "encountered-enemies", uid, bearerToken ? "auth" : "anon"],
        queryFn: () => getUserEncounteredEnemiesFn({ data: { uid, bearerToken } }),
        staleTime: 60 * 1000,
        gcTime: 5 * 60 * 1000,
    });
}

export const getUserScoreFn = createServerFn({ method: "GET" })
    .inputValidator((uid: string) => uid)
    .handler(async ({ data: uid }) => {
        const res = await backendFetch(`/get-user-score?uid=${encodeURIComponent(uid)}`);
        if (!res.ok) {
            if (res.status === 404) return null;
            throw new Error(`Failed to load user score: ${res.status}`);
        }
        return (await res.json()) as IUserScore | null;
    });

export function userScoreQueryOptions(uid: string) {
    return queryOptions({
        queryKey: ["user", "score", uid],
        queryFn: () => getUserScoreFn({ data: uid }),
        staleTime: 60 * 1000,
        gcTime: 5 * 60 * 1000,
    });
}

export const getUserCheckinFn = createServerFn({ method: "GET" })
    .inputValidator((uid: string) => uid)
    .handler(async ({ data: uid }) => {
        const res = await backendFetch(`/get-user-checkin?uid=${encodeURIComponent(uid)}`);
        if (!res.ok) {
            if (res.status === 404) return null;
            throw new Error(`Failed to load user check-in: ${res.status}`);
        }
        return (await res.json()) as IUserCheckin | null;
    });

export function userCheckinQueryOptions(uid: string) {
    return queryOptions({
        queryKey: ["user", "checkin", uid],
        queryFn: () => getUserCheckinFn({ data: uid }),
        staleTime: 60 * 1000,
        gcTime: 5 * 60 * 1000,
    });
}

// ─── User improvements ──────────────────────────────────────────────────────

export type IStageRotation = Refine<RotationInfo, { status: "active" | "past" | "future" }>;

export type IStageGap = StageGap;

export type IStagePoolImprovements = StagePoolImprovements;

export type IStageImprovements = StageImprovements;

export type IProgressPair = ProgressPair;

export type IRoguelikeDifficulty = RoguelikeDifficulty;

export type IRoguelikeCollectibles = RoguelikeCollectibles;

export type IRoguelikeThemeImprovement = RoguelikeThemeImprovement;

export type ISandboxPart = SandboxPart;

export type ISandboxCategory = SandboxCategory;

export type ISandboxImprovements = SandboxImprovements;

export type IMedalOperatorLock = MedalOperatorLock;

export type IMedalGap = MedalGap;

export type IMedalImprovements = MedalImprovements;

export type IUpgradeDelta = UpgradeDelta;

export type IOperatorGap = OperatorGap;

export type IScoreDimension = ScoreDimension;

export type IOperatorImprovements = OperatorImprovements;

export type IAssignedOperator = AssignedOperator;

export type IRoomAssignment = RoomAssignmentDto;

/**
 * One line of a room's per-skill breakdown. Values are MARGINALS in this exact
 * crew - what the room loses if this one skill is removed - so pair riders,
 * non-stacking rules and faction gates are already folded in. Marginals of
 * coupled skills deliberately do NOT sum to the room total.
 */
export type ISkillLine = SkillLineDto;

export type INonProdEffect = NonProdEffectDto;

export type IBaseAssignment = BaseAssignmentDto;

export type IRotationMember = RotationMemberDto;

export type IRoomRotation = RoomRotationDto;

export type IRotationSetRoom = RotationSetRoomDto;

export type IRotationSet = RotationSetDto;

export type IRotation = RotationDto;

export type IRoomLayoutEntry = RoomLayoutEntry;

/** An operator leaving a cell for another room in the SAME shift. */
export type IMovedOperator = MovedOperator;

export type IShiftRoom = ShiftRoomDto;

export type IShift = ShiftDto;

export type IShiftRotation = ShiftRotationDto;

/** The rotation validated by a time-stepped morale simulation: honest evidence the
 *  plan survives its own rhythm instead of an unchecked recommendation. */
export type ISustainability = SustainabilityDto;

/** One production room's simulated totals over the sim horizon. */
export type IFacilityOutput = FacilityOutputDto;

export type IMoraleTimeline = MoraleTimelineDto;

export type IDepletedOperator = DepletedOperatorDto;

/** A support operator to station outside production to feed the resource economy. */
export type IPerceptionSupport = PerceptionSupportDto;

/** A production operator the resource economy powers, with the bonus it gains. */
export type IPerceptionConsumer = PerceptionConsumerDto;

/** The base-wide resource economy plan (Rosmontis / Ebenholz "Perception Information"). */
export type IPerceptionPlan = PerceptionPlanDto;

export type IBaseImprovements = BaseImprovements;

export type IImprovementsResponse = ImprovementsResponse;

export const getUserImprovementsFn = createServerFn({ method: "GET" })
    .inputValidator((data: { uid: string; bearerToken?: string }) => data)
    .handler(async ({ data: { uid, bearerToken } }) => {
        const token = bearerToken ?? optionalSiteToken();
        const res = await backendFetch(`/user/improvements?uid=${encodeURIComponent(uid)}`, { bearerToken: token });
        if (!res.ok) {
            // 403 = private profile, 404 = no such user - treat both as "unavailable".
            if (res.status === 404 || res.status === 403) return null;
            throw new Error(`Failed to load improvements: ${res.status}`);
        }
        return (await res.json()) as IImprovementsResponse;
    });

export function userImprovementsQueryOptions(uid: string, bearerToken?: string) {
    return queryOptions({
        queryKey: ["user", "improvements", uid, bearerToken ? "auth" : "anon"],
        queryFn: () => getUserImprovementsFn({ data: { uid, bearerToken } }),
        // Improvements depend on roster + score state; refresh sparingly.
        staleTime: 60 * 1000,
        gcTime: 5 * 60 * 1000,
    });
}

export interface ISearchUsersInput {
    q?: string;
    limit?: number;
    offset?: number;
}

export type ISearchPage = SearchPage;

export const searchUsersFn = createServerFn({ method: "GET" })
    .inputValidator((data: ISearchUsersInput) => data)
    .handler(async ({ data: { q, limit, offset } }) => {
        const params = new URLSearchParams();
        if (q) params.set("q", q);
        if (limit !== undefined) params.set("limit", String(limit));
        if (offset !== undefined) params.set("offset", String(offset));

        const res = await backendFetch(`/search?${params.toString()}`);
        if (!res.ok) throw new Error(`Failed to search users: ${res.status}`);
        return (await res.json()) as ISearchPage;
    });

export function searchUsersQueryOptions(input: ISearchUsersInput) {
    return queryOptions({
        queryKey: ["user", "search", input.q ?? null, input.limit ?? null, input.offset ?? null],
        queryFn: () => searchUsersFn({ data: input }),
        staleTime: 30 * 1000,
        gcTime: 5 * 60 * 1000,
    });
}

export type ILeaderboardEntry = LeaderboardEntry;

export type ILeaderboardPage = LeaderboardPage;

export interface ILeaderboardInput {
    sort?: string;
    server?: string;
    /** "1 day" | "7 days" | "30 days" - when set, each entry receives `rank_delta`. */
    movement_interval?: string;
    /** When true (requires `movement_interval`), only users with non-zero movement are returned. */
    movement_only?: boolean;
    /** Free-text filter on nickname / uid (ILIKE). */
    q?: string;
    limit?: number;
    offset?: number;
}

export const getLeaderboardFn = createServerFn({ method: "GET" })
    .inputValidator((data: ILeaderboardInput) => data)
    .handler(async ({ data: { sort, server, movement_interval, movement_only, q, limit, offset } }) => {
        const params = new URLSearchParams();
        if (sort) params.set("sort", sort);
        if (server) params.set("server", server);
        if (movement_interval) params.set("movement_interval", movement_interval);
        if (movement_only) params.set("movement_only", "true");
        if (q) params.set("q", q);
        if (limit !== undefined) params.set("limit", String(limit));
        if (offset !== undefined) params.set("offset", String(offset));

        const res = await backendFetch(`/leaderboard?${params.toString()}`);
        if (!res.ok) throw new Error(`Failed to load leaderboard: ${res.status}`);
        return (await res.json()) as ILeaderboardPage;
    });

export function leaderboardQueryOptions(input: ILeaderboardInput = {}) {
    return queryOptions({
        queryKey: ["user", "leaderboard", input.sort ?? null, input.server ?? null, input.movement_interval ?? null, input.movement_only ?? false, input.q ?? null, input.limit ?? null, input.offset ?? null],
        queryFn: () => getLeaderboardFn({ data: input }),
        staleTime: 60 * 1000,
        gcTime: 5 * 60 * 1000,
    });
}

export type LeaderboardMoverDirection = "up" | "down";
export type LeaderboardMoverInterval = "1 day" | "7 days" | "30 days";

export type ILeaderboardMover = LeaderboardMover;

export interface ILeaderboardMoversInput {
    direction?: LeaderboardMoverDirection;
    interval?: LeaderboardMoverInterval;
    server?: string;
    limit?: number;
}

export const getLeaderboardMoversFn = createServerFn({ method: "GET" })
    .inputValidator((data: ILeaderboardMoversInput) => data)
    .handler(async ({ data: { direction, interval, server, limit } }) => {
        const params = new URLSearchParams();
        if (direction) params.set("direction", direction);
        if (interval) params.set("interval", interval);
        if (server) params.set("server", server);
        if (limit !== undefined) params.set("limit", String(limit));

        const res = await backendFetch(`/leaderboard/movers?${params.toString()}`);
        if (!res.ok) throw new Error(`Failed to load leaderboard movers: ${res.status}`);
        return (await res.json()) as ILeaderboardMover[];
    });

export function leaderboardMoversQueryOptions(input: ILeaderboardMoversInput = {}) {
    return queryOptions({
        queryKey: ["user", "leaderboard", "movers", input.direction ?? null, input.interval ?? null, input.server ?? null, input.limit ?? null],
        queryFn: () => getLeaderboardMoversFn({ data: input }),
        staleTime: 60 * 1000,
        gcTime: 5 * 60 * 1000,
    });
}

export type IServerShare = ServerShare;

export interface ILeaderboardDistributionInput {
    top?: number;
}

export const getLeaderboardDistributionFn = createServerFn({ method: "GET" })
    .inputValidator((data: ILeaderboardDistributionInput) => data)
    .handler(async ({ data: { top } }) => {
        const params = new URLSearchParams();
        if (top !== undefined) params.set("top", String(top));

        const res = await backendFetch(`/leaderboard/distribution?${params.toString()}`);
        if (!res.ok) throw new Error(`Failed to load leaderboard distribution: ${res.status}`);
        return (await res.json()) as IServerShare[];
    });

export function leaderboardDistributionQueryOptions(input: ILeaderboardDistributionInput = {}) {
    return queryOptions({
        queryKey: ["user", "leaderboard", "distribution", input.top ?? null],
        queryFn: () => getLeaderboardDistributionFn({ data: input }),
        staleTime: 5 * 60 * 1000,
        gcTime: 10 * 60 * 1000,
    });
}

export type IPlayerStanding = PlayerStanding;

export interface IPlayerStandingInput {
    uid: string;
    server: string;
    window?: number;
    interval?: LeaderboardMoverInterval;
}

export const getPlayerStandingFn = createServerFn({ method: "GET" })
    .inputValidator((data: IPlayerStandingInput) => data)
    .handler(async ({ data: { uid, server, window, interval } }) => {
        const params = new URLSearchParams({ uid, server });
        if (window !== undefined) params.set("window", String(window));
        if (interval) params.set("interval", interval);

        const res = await backendFetch(`/leaderboard/standing?${params.toString()}`);
        if (!res.ok) {
            if (res.status === 404) return null;
            throw new Error(`Failed to load player standing: ${res.status}`);
        }
        return (await res.json()) as IPlayerStanding;
    });

export type IScoreHistoryPoint = ScoreHistoryPoint;

export const getScoreHistoryFn = createServerFn({ method: "GET" })
    .inputValidator((uid: string) => uid)
    .handler(async ({ data: uid }) => {
        const res = await backendFetch(`/leaderboard/history?uid=${encodeURIComponent(uid)}`);
        if (!res.ok) throw new Error(`Failed to load score history: ${res.status}`);
        return (await res.json()) as IScoreHistoryPoint[];
    });

export function scoreHistoryQueryOptions(uid: string) {
    return queryOptions({
        queryKey: ["user", "leaderboard", "history", uid],
        queryFn: () => getScoreHistoryFn({ data: uid }),
        staleTime: 5 * 60 * 1000,
        gcTime: 15 * 60 * 1000,
    });
}

export function playerStandingQueryOptions(input: IPlayerStandingInput) {
    return queryOptions({
        queryKey: ["user", "leaderboard", "standing", input.uid, input.server, input.window ?? null, input.interval ?? null],
        queryFn: () => getPlayerStandingFn({ data: input }),
        staleTime: 60 * 1000,
        gcTime: 5 * 60 * 1000,
    });
}
