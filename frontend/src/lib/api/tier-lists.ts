import { randomBytes } from "node:crypto";
import { queryOptions } from "@tanstack/react-query";
import { createServerFn } from "@tanstack/react-start";
import { getCookie, getRequestIP, setCookie } from "@tanstack/react-start/server";
import type { IOperator, ITierEntry, ITierList } from "#/components/home/impl/data";
import { env } from "#/env";
import { backendFetch } from "#/lib/fetch";
import { formatRelative } from "#/lib/utils";
import type { IOperatorIndexEntry, OperatorPosition, OperatorProfession, OperatorRarity } from "#/types/operators";

const VIEW_SESSION_COOKIE = "mtl_sid";

function ensureViewSessionId(): string {
    const existing = getCookie(VIEW_SESSION_COOKIE);
    if (existing && /^[0-9a-f]{64}$/.test(existing)) return existing;
    const fresh = randomBytes(32).toString("hex");
    setCookie(VIEW_SESSION_COOKIE, fresh, {
        path: "/",
        httpOnly: true,
        sameSite: "lax",
        secure: env.NODE_ENV === "production",
        maxAge: 60 * 60 * 24 * 365,
    });
    return fresh;
}

interface IBackendTierList {
    id: string;
    name: string;
    slug: string;
    description: string | null;
    list_type: string;
    created_by: string | null;
    is_active: boolean;
    flair_id: number | null;
    created_at: string;
    updated_at: string;
}

interface IBackendTier {
    id: string;
    tier_list_id: string;
    name: string;
    display_order: number;
    color: string | null;
    description: string | null;
    placements: IBackendPlacement[];
}

interface IBackendPlacement {
    tier_id: string;
    operator_id: string;
    sub_order: number;
    notes: string | null;
    updated_at: string;
}

interface IBackendTierListStats {
    tier_list_id: string;
    view_count: number;
    unique_view_count: number;
    favorite_count: number;
    share_count: number;
    is_trending: boolean;
    trending_score: number;
    views_last_24h: number;
    views_last_7d: number;
    last_viewed_at: string | null;
    stats_updated_at: string;
}

interface IBackendFlair {
    id: number;
    code: string;
    label: string;
    color: string | null;
    display_order: number;
    is_active: boolean;
}

interface IBackendAuthor {
    id: string;
    uid: string;
    nickname: string | null;
    avatar_id: string | null;
}

interface IBackendTierListDetail extends IBackendTierList {
    tiers: IBackendTier[];
    stats: IBackendTierListStats | null;
    flair: IBackendFlair | null;
    author: IBackendAuthor | null;
}

const HOME_TIER_LIST_LIMIT = 6;
const BROWSE_TIER_LIST_LIMIT = 200;

const PROFESSION_TO_ROLE: Record<OperatorProfession, string> = {
    PIONEER: "Vanguard",
    WARRIOR: "Guard",
    TANK: "Defender",
    SNIPER: "Sniper",
    CASTER: "Caster",
    MEDIC: "Medic",
    SUPPORT: "Supporter",
    SPECIAL: "Specialist",
    TOKEN: "Specialist",
    TRAP: "Specialist",
};

const FLAIR_ACCENT: Record<string, string> = {
    endgame: "coral",
    meta: "coral",
    beginner: "mint",
    fun: "mint",
    roguelike: "violet",
    event: "violet",
    niche: "amber",
};

const FALLBACK_ACCENTS = ["coral", "mint", "amber", "violet"];

function toCardOperator(entry: IOperatorIndexEntry): IOperator {
    return {
        id: entry.id,
        name: entry.name,
        rarity: entry.rarity,
        role: PROFESSION_TO_ROLE[entry.profession] ?? "Specialist",
        arch: entry.subProfessionId,
    };
}

function tagFromDetail(detail: IBackendTierListDetail): string {
    if (detail.flair?.label) return detail.flair.label;
    if (detail.list_type === "official") return "Official";
    return "Community";
}

function accentFromDetail(detail: IBackendTierListDetail, index: number): string {
    const code = detail.flair?.code;
    if (code && FLAIR_ACCENT[code]) return FLAIR_ACCENT[code];
    return FALLBACK_ACCENTS[index % FALLBACK_ACCENTS.length] as string;
}

function mapTiers(tiers: IBackendTier[], opById: Record<string, IOperator>): ITierEntry[] {
    return [...tiers]
        .sort((a, b) => a.display_order - b.display_order)
        .map((tier) => ({
            name: tier.name,
            color: tier.color,
            operators: [...tier.placements]
                .sort((a, b) => a.sub_order - b.sub_order)
                .map((p) => opById[p.operator_id])
                .filter((op): op is IOperator => Boolean(op)),
        }));
}

function mapDetail(detail: IBackendTierListDetail, index: number, opById: Record<string, IOperator>): ITierList {
    return {
        id: detail.id,
        slug: detail.slug,
        title: detail.name,
        tag: tagFromDetail(detail),
        stage: detail.description ?? "",
        author: {
            name: detail.author?.nickname?.trim() || "Community",
            avatarId: detail.author?.avatar_id ?? null,
        },
        updated: formatRelative(detail.updated_at),
        votes: detail.stats?.favorite_count ?? 0,
        views: detail.stats?.view_count ?? 0,
        comments: detail.stats?.views_last_7d ?? 0,
        hot: detail.stats?.is_trending ?? false,
        accent: accentFromDetail(detail, index),
        tiers: mapTiers(detail.tiers, opById),
    };
}

export const recordTierListViewFn = createServerFn({ method: "POST" })
    .inputValidator((slug: string) => slug)
    .handler(async ({ data: slug }) => {
        const sessionId = ensureViewSessionId();
        const clientIp = getRequestIP({ xForwardedFor: true });
        const headers: Record<string, string> = { "X-Session-Id": sessionId };
        if (clientIp) headers["X-Forwarded-For"] = clientIp;
        const res = await backendFetch(`/tier-lists/${slug}/view`, {
            method: "POST",
            headers,
            body: "{}",
        });
        if (!res.ok) throw new Error(`Failed to record view: ${res.status}`);
        return (await res.json()) as { unique: boolean };
    });

export const getHomeTierListsFn = createServerFn({ method: "GET" }).handler(async (): Promise<ITierList[]> => {
    const [listRes, opsRes] = await Promise.all([backendFetch("/tier-lists"), backendFetch("/operators/index")]);
    if (!listRes.ok) throw new Error(`Failed to load tier lists: ${listRes.status}`);
    if (!opsRes.ok) throw new Error(`Failed to load operators index: ${opsRes.status}`);

    const lists = (await listRes.json()) as IBackendTierList[];
    const operators = (await opsRes.json()) as IOperatorIndexEntry[];
    const opById: Record<string, IOperator> = Object.fromEntries(operators.map((op) => [op.id, toCardOperator(op)]));

    const top = lists.slice(0, HOME_TIER_LIST_LIMIT);
    const details = await Promise.all(
        top.map(async (tl) => {
            const res = await backendFetch(`/tier-lists/${tl.slug}`);
            if (!res.ok) throw new Error(`Failed to load tier list ${tl.slug}: ${res.status}`);
            return (await res.json()) as IBackendTierListDetail;
        }),
    );

    return details.map((detail, i) => mapDetail(detail, i, opById));
});

export function homeTierListsQueryOptions() {
    return queryOptions({
        queryKey: ["tier-lists", "home"],
        queryFn: () => getHomeTierListsFn(),
        staleTime: 5 * 60 * 1000,
        gcTime: 60 * 60 * 1000,
    });
}

export type TierListType = "official" | "community";

export interface ITierListBrowseItem extends ITierList {
    listType: TierListType;
    description: string;
    createdAtMs: number;
    updatedAtMs: number;
    flairCode: string | null;
    flairLabel: string | null;
    flairColor: string | null;
    favorites: number;
    shares: number;
    views24h: number;
    views7d: number;
    trendingScore: number;
    isTrending: boolean;
}

function mapBrowseItem(detail: IBackendTierListDetail, index: number, opById: Record<string, IOperator>): ITierListBrowseItem {
    const base = mapDetail(detail, index, opById);
    const listType: TierListType = detail.list_type === "official" ? "official" : "community";
    return {
        ...base,
        listType,
        description: detail.description ?? "",
        createdAtMs: Date.parse(detail.created_at) || 0,
        updatedAtMs: Date.parse(detail.updated_at) || 0,
        flairCode: detail.flair?.code ?? null,
        flairLabel: detail.flair?.label ?? null,
        flairColor: detail.flair?.color ?? null,
        favorites: detail.stats?.favorite_count ?? 0,
        shares: detail.stats?.share_count ?? 0,
        views24h: detail.stats?.views_last_24h ?? 0,
        views7d: detail.stats?.views_last_7d ?? 0,
        trendingScore: detail.stats?.trending_score ?? 0,
        isTrending: detail.stats?.is_trending ?? false,
    };
}

export const getBrowseTierListsFn = createServerFn({ method: "GET" }).handler(async (): Promise<ITierListBrowseItem[]> => {
    const [listRes, opsRes] = await Promise.all([backendFetch("/tier-lists"), backendFetch("/operators/index")]);
    if (!listRes.ok) throw new Error(`Failed to load tier lists: ${listRes.status}`);
    if (!opsRes.ok) throw new Error(`Failed to load operators index: ${opsRes.status}`);

    const lists = (await listRes.json()) as IBackendTierList[];
    const operators = (await opsRes.json()) as IOperatorIndexEntry[];
    const opById: Record<string, IOperator> = Object.fromEntries(operators.map((op) => [op.id, toCardOperator(op)]));

    const subset = lists.slice(0, BROWSE_TIER_LIST_LIMIT);
    const settled = await Promise.allSettled(
        subset.map(async (tl) => {
            const res = await backendFetch(`/tier-lists/${tl.slug}`);
            if (!res.ok) throw new Error(`Failed to load tier list ${tl.slug}: ${res.status}`);
            return (await res.json()) as IBackendTierListDetail;
        }),
    );

    const details: IBackendTierListDetail[] = [];
    for (const result of settled) {
        if (result.status === "fulfilled") details.push(result.value);
    }
    return details.map((detail, i) => mapBrowseItem(detail, i, opById));
});

export function browseTierListsQueryOptions() {
    return queryOptions({
        queryKey: ["tier-lists", "browse"],
        queryFn: () => getBrowseTierListsFn(),
        staleTime: 5 * 60 * 1000,
        gcTime: 60 * 60 * 1000,
    });
}

export interface ITierListAuthor {
    id: string;
    uid: string;
    nickname: string | null;
    avatarId: string | null;
}

export interface ITierListFlair {
    id: number;
    code: string;
    label: string;
    color: string | null;
    displayOrder: number;
    isActive: boolean;
}

export interface ITierListStats {
    viewCount: number;
    uniqueViewCount: number;
    favoriteCount: number;
    shareCount: number;
    isTrending: boolean;
    trendingScore: number;
    viewsLast24h: number;
    viewsLast7d: number;
    lastViewedAt: string | null;
    statsUpdatedAt: string;
}

export interface ITierEntryFull {
    id: string;
    name: string;
    displayOrder: number;
    color: string | null;
    description: string | null;
    operators: ITierOperator[];
}

export interface ITierOperator {
    id: string;
    name: string;
    appellation: string | null;
    rarity: OperatorRarity;
    profession: OperatorProfession;
    subProfessionId: string;
    position: OperatorPosition;
    nationId: string | null;
    subOrder: number;
    notes: string | null;
    /** ISO timestamp of when this placement was last updated. */
    updatedAt: string;
}

export interface ITierListDetail {
    id: string;
    slug: string;
    title: string;
    description: string;
    listType: TierListType;
    createdBy: string | null;
    flair: ITierListFlair | null;
    author: ITierListAuthor | null;
    stats: ITierListStats | null;
    tiers: ITierEntryFull[];
    createdAt: string;
    updatedAt: string;
}

function mapTierDetail(detail: IBackendTierListDetail, opIndex: Record<string, IOperatorIndexEntry>): ITierListDetail {
    const tiers: ITierEntryFull[] = [...detail.tiers]
        .sort((a, b) => a.display_order - b.display_order)
        .map((t) => {
            const operators: ITierOperator[] = [...t.placements]
                .sort((a, b) => a.sub_order - b.sub_order)
                .map((p): ITierOperator | null => {
                    const op = opIndex[p.operator_id];
                    if (!op) return null;
                    return {
                        id: op.id,
                        name: op.name,
                        appellation: op.appellation || null,
                        rarity: op.rarity,
                        profession: op.profession,
                        subProfessionId: op.subProfessionId,
                        position: op.position,
                        nationId: op.nationId || null,
                        subOrder: p.sub_order,
                        notes: p.notes,
                        updatedAt: p.updated_at,
                    };
                })
                .filter((x): x is ITierOperator => x !== null);
            return {
                id: t.id,
                name: t.name,
                displayOrder: t.display_order,
                color: t.color,
                description: t.description,
                operators,
            };
        });

    return {
        id: detail.id,
        slug: detail.slug,
        title: detail.name,
        description: detail.description ?? "",
        listType: detail.list_type === "official" ? "official" : "community",
        createdBy: detail.created_by,
        flair: detail.flair
            ? {
                  id: detail.flair.id,
                  code: detail.flair.code,
                  label: detail.flair.label,
                  color: detail.flair.color,
                  displayOrder: detail.flair.display_order,
                  isActive: detail.flair.is_active,
              }
            : null,
        author: detail.author
            ? {
                  id: detail.author.id,
                  uid: detail.author.uid,
                  nickname: detail.author.nickname,
                  avatarId: detail.author.avatar_id,
              }
            : null,
        stats: detail.stats
            ? {
                  viewCount: detail.stats.view_count,
                  uniqueViewCount: detail.stats.unique_view_count,
                  favoriteCount: detail.stats.favorite_count,
                  shareCount: detail.stats.share_count,
                  isTrending: detail.stats.is_trending,
                  trendingScore: detail.stats.trending_score,
                  viewsLast24h: detail.stats.views_last_24h,
                  viewsLast7d: detail.stats.views_last_7d,
                  lastViewedAt: detail.stats.last_viewed_at,
                  statsUpdatedAt: detail.stats.stats_updated_at,
              }
            : null,
        tiers,
        createdAt: detail.created_at,
        updatedAt: detail.updated_at,
    };
}

export const getTierListDetailFn = createServerFn({ method: "GET" })
    .inputValidator((slug: string) => slug)
    .handler(async ({ data: slug }): Promise<ITierListDetail | null> => {
        const [listRes, opsRes] = await Promise.all([backendFetch(`/tier-lists/${encodeURIComponent(slug)}`), backendFetch("/operators/index")]);
        if (listRes.status === 404) return null;
        if (!listRes.ok) throw new Error(`Failed to load tier list: ${listRes.status}`);
        if (!opsRes.ok) throw new Error(`Failed to load operators index: ${opsRes.status}`);

        const detail = (await listRes.json()) as IBackendTierListDetail;
        const operators = (await opsRes.json()) as IOperatorIndexEntry[];
        const opIndex: Record<string, IOperatorIndexEntry> = Object.fromEntries(operators.map((op) => [op.id, op]));
        return mapTierDetail(detail, opIndex);
    });

export function tierListDetailQueryOptions(slug: string) {
    return queryOptions({
        queryKey: ["tier-lists", "detail", slug],
        queryFn: () => getTierListDetailFn({ data: slug }),
        staleTime: 5 * 60 * 1000,
        gcTime: 60 * 60 * 1000,
    });
}

export const toggleTierListFavoriteFn = createServerFn({ method: "POST" })
    .inputValidator((slug: string) => slug)
    .handler(async ({ data: slug }): Promise<{ favorited: boolean } | null> => {
        const token = getCookie("site_token");
        if (!token) return null;
        const res = await backendFetch(`/tier-lists/${encodeURIComponent(slug)}/favorite`, {
            method: "POST",
            body: "{}",
            bearerToken: token,
        });
        if (res.status === 401) return null;
        if (!res.ok) throw new Error(`Failed to toggle favorite: ${res.status}`);
        return (await res.json()) as { favorited: boolean };
    });

export const getMyTierListFavoriteFn = createServerFn({ method: "GET" })
    .inputValidator((slug: string) => slug)
    .handler(async ({ data: slug }): Promise<{ favorited: boolean } | null> => {
        const token = getCookie("site_token");
        if (!token) return null;
        const res = await backendFetch(`/tier-lists/${encodeURIComponent(slug)}/favorite`, { bearerToken: token });
        if (res.status === 401 || res.status === 404) return null;
        if (!res.ok) throw new Error(`Failed to load favorite state: ${res.status}`);
        return (await res.json()) as { favorited: boolean };
    });

export function myTierListFavoriteQueryOptions(slug: string, authed: boolean) {
    return queryOptions({
        queryKey: ["tier-lists", "favorite", slug, authed ? "auth" : "anon"],
        queryFn: () => (authed ? getMyTierListFavoriteFn({ data: slug }) : Promise.resolve(null)),
        enabled: authed,
        staleTime: 60 * 1000,
        gcTime: 5 * 60 * 1000,
    });
}

export const getTierListFlairsFn = createServerFn({ method: "GET" }).handler(async (): Promise<ITierListFlair[]> => {
    const res = await backendFetch("/tier-list-flairs");
    if (!res.ok) throw new Error(`Failed to load tier list flairs: ${res.status}`);
    const raw = (await res.json()) as IBackendFlair[];
    return raw.map((f) => ({
        id: f.id,
        code: f.code,
        label: f.label,
        color: f.color,
        displayOrder: f.display_order,
        isActive: f.is_active,
    }));
});

export function tierListFlairsQueryOptions() {
    return queryOptions({
        queryKey: ["tier-list-flairs"],
        queryFn: () => getTierListFlairsFn(),
        staleTime: 60 * 60 * 1000,
        gcTime: 24 * 60 * 60 * 1000,
    });
}
