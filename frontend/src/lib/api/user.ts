import { queryOptions } from "@tanstack/react-query";
import { createServerFn } from "@tanstack/react-start";
import { backendFetch } from "#/lib/fetch";
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

export interface IRosterEntry {
    user_id: string;
    operator_id: string;
    elite: number;
    level: number;
    exp: number;
    potential: number;
    skill_level: number;
    favor_point: number;
    skin_id: string | null;
    default_skill: number | null;
    voice_lan: string | null;
    current_equip: string | null;
    current_tmpl: string | null;
    obtained_at: number | null;
    masteries: IRosterMastery[];
    modules: IRosterModule[];
}

export interface IUserScore {
    user_id: string;
    total_score: number;
    operator_score: number;
    stage_score: number;
    roguelike_score: number;
    sandbox_score: number;
    medal_score: number;
    base_score: number;
    skin_score: number;
    grade: string | null;
    calculated_at: string;
}

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

export interface IInventoryItem {
    item_id: string;
    quantity: number;
}

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

export interface IEncounteredEnemy {
    enemyId: string;
    name: string | null;
    enemyIndex: string | null;
    sortId: number | null;
}

export interface IEncounteredEnemies {
    encounteredCount: number;
    handbookTotal: number;
    enemies: IEncounteredEnemy[];
}

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

export interface IStageRotation {
    /** "active" = playable now, "past" = rotated out, "future" = not yet open. */
    status: "active" | "past" | "future";
    startTs: number;
    endTs: number;
}

export interface IStageGap {
    stage_id: string;
    code: string;
    name: string | null;
    zone_id: string;
    weight: number;
    state: number;
    /** Present only for rotating Annihilation maps (`camp_r_*`). */
    rotation?: IStageRotation | null;
}

export interface IStagePoolImprovements {
    total: number;
    cleared: number;
    three_starred: number;
    missing: IStageGap[];
    not_three_starred: IStageGap[];
}

export interface IStageImprovements {
    permanent: IStagePoolImprovements;
    event: IStagePoolImprovements;
}

export interface IProgressPair {
    current: number;
    max: number;
}

export interface IRoguelikeDifficulty {
    highest_cleared: number;
    max: number;
}

export interface IRoguelikeCollectibles {
    relics: IProgressPair;
    capsules: IProgressPair;
    bands: IProgressPair;
}

export interface IRoguelikeThemeImprovement {
    theme_id: string;
    theme_name: string;
    endings: IProgressPair;
    difficulty: IRoguelikeDifficulty;
    collectibles: IRoguelikeCollectibles;
    bp: IProgressPair;
    challenges: IProgressPair;
}

export interface ISandboxPart {
    label: string;
    current: number;
    max: number;
}

export interface ISandboxCategory {
    key: string;
    label: string;
    weight: number;
    score: number;
    parts: ISandboxPart[];
}

export interface ISandboxImprovements {
    total: number;
    categories: ISandboxCategory[];
}

export interface IMedalOperatorLock {
    operatorId: string;
    operatorName: string;
    /** Human-readable reason, e.g. "collab" or "event reward". */
    reason: string;
}

export interface IMedalGap {
    medal_id: string;
    name: string;
    rarity: string;
    get_method: string;
    description: string;
    is_hidden: boolean;
    end_time: number | null;
    /** Set when the medal is locked behind an operator that can't be obtained. */
    operator_lock?: IMedalOperatorLock | null;
}

export interface IMedalImprovements {
    permanent_missing: IMedalGap[];
    event_in_window_missing: IMedalGap[];
    /** Medals gated on a collab / one-time operator - shown for reference, excluded from scoring. */
    operator_locked: IMedalGap[];
}

export interface IUpgradeDelta {
    /** Matching one of the entries in `IOperatorGap.missing` (ELITE, MAX_LEVEL, M3, SL7, MOD3, POT6, TRUST). */
    tag: string;
    /** Change in this operator's own score if the milestone were reached (0.0-1.0). */
    operator_score_delta: number;
    /** Change in the user's operator_grade subscore (0.0-1.0). */
    operator_grade_delta: number;
    /** Change in the user's overall total_score (0.0-1.0). */
    total_score_delta: number;
}

export interface IOperatorGap {
    operator_id: string;
    name: string;
    rarity: number;
    current_elite: number;
    current_level: number;
    current_skill_level: number;
    max_mastery: number;
    max_module_level: number;
    current_trust: number;
    is_support: boolean;
    missing: string[];
    /** One delta per tag in `missing` (same order). */
    deltas: IUpgradeDelta[];
    /** Combined gain to the Operators subscore (0.0-1.0) if the user did every
     *  upgrade on this op - ELITE and MAX_LEVEL overlap are deduped. */
    subscore_potential_gain: number;
    /** Same combination as `subscore_potential_gain` but in overall total_score units. */
    total_potential_gain: number;
}

export interface IScoreDimension {
    /** Investment axis: elite, level, skill_level, mastery, module, potential, trust. */
    kind: string;
    /** Share of the Operators subscore this dimension carries (0.0-1.0; shares sum to 1.0). */
    weight_share: number;
    /** Rarity-weighted completion of this dimension (0.0-1.0). */
    completion: number;
    /** `weight_share x completion` - contributions sum to the Operators subscore. */
    contribution: number;
}

export interface IOperatorImprovements {
    /** Where the current Operators subscore comes from, one row per investment axis. */
    score_breakdown: IScoreDimension[];
    below_milestone: IOperatorGap[];
}

export interface IAssignedOperator {
    operator_id: string;
    name: string;
}

export interface IRoomAssignment {
    slot_id: string;
    room_type: string;
    level: number;
    formula_type: string | null;
    /** Order-acquisition speed % (the productivity bonus the game shows). */
    total_efficiency: number;
    /** Order-value % (LMD per order, e.g. Proviso) - lifts LMD, not speed. */
    order_value: number;
    /** True = fixed synergy squad (operators depend on each other); false = flexible. */
    locked: boolean;
    operators: IAssignedOperator[];
    /** Per-room natural yield (trading posts show potential LMD if gold-supplied). */
    yield_lmd_per_day: number;
    yield_gold_per_day: number;
    yield_exp_per_day: number;
}

export interface IBaseAssignment {
    rooms: IRoomAssignment[];
    total_production_efficiency: number;
    /** Realized daily output (gold→trade loop coupled: LMD = min(made, sold) × 500). */
    yield_lmd_per_day: number;
    yield_exp_per_day: number;
    /** LMD-equivalent of everything combined (LMD + EXP at 1:1). */
    yield_total_value: number;
}

export interface IRotationMember {
    operator: IAssignedOperator;
    /** Approximate hours this operator works before you rotate it out. */
    lasts_hours: number;
}

export interface IRoomRotation {
    slot_id: string;
    room_type: string;
    /** Main operators ordered by who needs swapping first (fastest-draining). */
    members: IRotationMember[];
    /** The backup to rotate in when a main needs rest. */
    backup: IAssignedOperator | null;
}

export interface IRotationSetRoom {
    slot_id: string;
    room_type: string;
    /** The operators working this room in this set. */
    working: IAssignedOperator[];
    /** The main resting this set (covered by the backup), if any. */
    resting: IAssignedOperator | null;
}

export interface IRotationSet {
    rooms: IRotationSetRoom[];
}

export interface IRotation {
    /** The main staffing - your best operators, working almost all the time. */
    main: IBaseAssignment;
    /** Per-room rotation plan: who to swap first, when, and the backup. */
    rooms: IRoomRotation[];
    /** The small shared bench that covers every room (one swap at a time). */
    shared_bench?: IAssignedOperator[];
    /** The rotation as a few overlapping staffings to cycle through, so the whole
     * base is never swapped at once. Consecutive sets share all-but-one operator. */
    sets?: IRotationSet[];
    /** Sustained 24/7 output - near peak, reduced only by backup-coverage time. */
    sustained_efficiency: number;
}

export interface IRoomLayoutEntry {
    room_type: string;
    count: number;
    levels: number[];
}

export interface IShiftRoom {
    slot_id: string;
    room_type: string;
    formula_type: string | null;
    /** False when the room is deliberately unstaffed this shift (CC's off shift). */
    active: boolean;
    recommended: IAssignedOperator[];
    /** The player's saved preset for this room and shift (empty if none). */
    current: IAssignedOperator[];
    /** Operators to ADD (in the recommendation, not the current preset). */
    swap_in: IAssignedOperator[];
    /** Operators to REMOVE (in the current preset, not the recommendation). */
    swap_out: IAssignedOperator[];
    /** True when the player's preset already matches the recommendation. */
    matches: boolean;
    /** True when the player's CURRENT team differs but produces at least as much (an exact tie),
     *  so no swap is suggested - e.g. a Dorothy-boosted Rhine operator matching Bryophyta. */
    equivalent: boolean;
}

export interface IShift {
    index: number;
    rooms: IShiftRoom[];
}

export interface IShiftRotation {
    shifts: IShift[];
    /** Operators the player runs 24/7 with a morale-swap manager (Fiammetta) - kept working every
     *  shift instead of resting the middle one. Badged as "24/7 · Fiammetta". */
    sustained: IAssignedOperator[];
}

/** A support operator to station outside production to feed the resource economy. */
export interface IPerceptionSupport {
    operator: IAssignedOperator;
    room_type: string;
}

/** A production operator the resource economy powers, with the bonus it gains. */
export interface IPerceptionConsumer {
    operator: IAssignedOperator;
    room_type: string;
    /** Peak bonus (fresh-operator snapshot). */
    bonus_pct: number;
    /** Sustained 24/7 bonus (peak x working uptime). */
    sustained_pct: number;
}

/** The base-wide resource economy plan (Rosmontis / Ebenholz "Perception Information"). */
export interface IPerceptionPlan {
    support: IPerceptionSupport[];
    consumers: IPerceptionConsumer[];
    /** The morale-swap operator (Fiammetta) sustaining the Ling/Dusk rotation, if owned. */
    rotation_manager?: IAssignedOperator | null;
    /** True when the plan uses Ling/Dusk but the roster has no morale-swap manager. */
    needs_rotation_manager: boolean;
}

export interface IBaseImprovements {
    /** The player's current base exactly as stationed right now. */
    current: IBaseAssignment | null;
    /** The player's planned rotation (their in-game preset shifts), if any. */
    current_rotation: IRotation | null;
    optimal: IBaseAssignment | null;
    rotation: IRotation | null;
    layout: IRoomLayoutEntry[];
    /** Recommended 3-shift rotation paired with the player's saved presets. */
    shift_rotation?: IShiftRotation | null;
    /** The base-wide resource economy plan, if a 243 roster can field it. */
    perception?: IPerceptionPlan | null;
}

export interface IImprovementsResponse {
    uid: string;
    stages: IStageImprovements;
    roguelike: IRoguelikeThemeImprovement[];
    sandbox: ISandboxImprovements;
    medals: IMedalImprovements;
    operators: IOperatorImprovements;
    base: IBaseImprovements;
}

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

export interface ISearchPage {
    entries: IUserProfile[];
    total: number;
}

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

export interface ILeaderboardEntry {
    id: string;
    uid: string;
    nickname: string | null;
    nick_number: string | null;
    level: number | null;
    avatar_id: string | null;
    secretary: string | null;
    secretary_skin_id: string | null;
    server: string;
    total_score: number | null;
    grade: string | null;
    operator_score: number | null;
    stage_score: number | null;
    roguelike_score: number | null;
    sandbox_score: number | null;
    medal_score: number | null;
    base_score: number | null;
    skin_score: number | null;
    rank_global: number | null;
    rank_server: number | null;
    /** Rank change vs. the snapshot baseline for the requested movement_interval.
     *  Positive = climbed. Null when movement wasn't requested or there's no baseline yet. */
    rank_delta: number | null;
}

export interface ILeaderboardPage {
    entries: ILeaderboardEntry[];
    total: number;
    updated_at: string | null;
}

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

export interface ILeaderboardMover {
    uid: string;
    nickname: string | null;
    nick_number: string | null;
    avatar_id: string | null;
    server: string;
    current_rank: number;
    previous_rank: number;
    /** Positive = climbed (previous_rank - current_rank). */
    rank_delta: number;
    current_score: number | null;
    score_delta: number | null;
}

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

export interface IServerShare {
    server: string;
    players: number;
}

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

export interface IPlayerStanding {
    player: ILeaderboardEntry;
    neighbors: ILeaderboardEntry[];
    /** 0.0 = top, 1.0 = bottom. */
    percentile: number;
    /** Positive = climbed since the requested interval's baseline. `null` if no baseline exists. */
    rank_delta: number | null;
}

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

export function playerStandingQueryOptions(input: IPlayerStandingInput) {
    return queryOptions({
        queryKey: ["user", "leaderboard", "standing", input.uid, input.server, input.window ?? null, input.interval ?? null],
        queryFn: () => getPlayerStandingFn({ data: input }),
        staleTime: 60 * 1000,
        gcTime: 5 * 60 * 1000,
    });
}
