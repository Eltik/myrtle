import { queryOptions } from "@tanstack/react-query";
import { createServerFn } from "@tanstack/react-start";
import type { IOperator, ITierEntry, ITierList } from "#/components/home/impl/data";
import { backendFetch } from "#/lib/fetch";
import type { IOperatorIndexEntry, OperatorProfession } from "./operators";

interface BackendTierList {
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

interface BackendTier {
    id: string;
    tier_list_id: string;
    name: string;
    display_order: number;
    color: string | null;
    description: string | null;
    placements: BackendPlacement[];
}

interface BackendPlacement {
    tier_id: string;
    operator_id: string;
    sub_order: number;
    notes: string | null;
    updated_at: string;
}

interface BackendTierListStats {
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

interface BackendFlair {
    id: number;
    code: string;
    label: string;
    color: string | null;
    display_order: number;
    is_active: boolean;
}

interface BackendAuthor {
    id: string;
    uid: string;
    nickname: string | null;
    avatar_id: string | null;
}

interface BackendTierListDetail extends BackendTierList {
    tiers: BackendTier[];
    stats: BackendTierListStats | null;
    flair: BackendFlair | null;
    author: BackendAuthor | null;
}

const HOME_TIER_LIST_LIMIT = 6;

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

function formatRelative(iso: string): string {
    const then = new Date(iso).getTime();
    if (Number.isNaN(then)) return "recently";
    const diffMs = Date.now() - then;
    if (diffMs < 0) return "just now";
    const mins = Math.floor(diffMs / 60_000);
    if (mins < 1) return "just now";
    if (mins < 60) return `${mins}m ago`;
    const hrs = Math.floor(mins / 60);
    if (hrs < 24) return `${hrs}h ago`;
    const days = Math.floor(hrs / 24);
    if (days === 1) return "yesterday";
    if (days < 7) return `${days}d ago`;
    const weeks = Math.floor(days / 7);
    if (weeks === 1) return "last week";
    if (weeks < 5) return `${weeks}w ago`;
    const months = Math.floor(days / 30);
    if (months < 12) return `${months}mo ago`;
    const years = Math.floor(days / 365);
    return `${years}y ago`;
}

function tagFromDetail(detail: BackendTierListDetail): string {
    if (detail.flair?.label) return detail.flair.label;
    if (detail.list_type === "official") return "Official";
    return "Community";
}

function accentFromDetail(detail: BackendTierListDetail, index: number): string {
    const code = detail.flair?.code;
    if (code && FLAIR_ACCENT[code]) return FLAIR_ACCENT[code];
    return FALLBACK_ACCENTS[index % FALLBACK_ACCENTS.length] as string;
}

function mapTiers(tiers: BackendTier[], opById: Record<string, IOperator>): ITierEntry[] {
    return [...tiers]
        .sort((a, b) => a.display_order - b.display_order)
        .map((tier) => ({
            name: tier.name,
            operators: [...tier.placements]
                .sort((a, b) => a.sub_order - b.sub_order)
                .map((p) => opById[p.operator_id])
                .filter((op): op is IOperator => Boolean(op)),
        }));
}

function mapDetail(detail: BackendTierListDetail, index: number, opById: Record<string, IOperator>): ITierList {
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
        const res = await backendFetch(`/tier-lists/${slug}/view`, {
            method: "POST",
            body: "{}",
        });
        if (!res.ok) throw new Error(`Failed to record view: ${res.status}`);
        return (await res.json()) as { unique: boolean };
    });

export const getHomeTierListsFn = createServerFn({ method: "GET" }).handler(async (): Promise<ITierList[]> => {
    const [listRes, opsRes] = await Promise.all([backendFetch("/tier-lists"), backendFetch("/operators/index")]);
    if (!listRes.ok) throw new Error(`Failed to load tier lists: ${listRes.status}`);
    if (!opsRes.ok) throw new Error(`Failed to load operators index: ${opsRes.status}`);

    const lists = (await listRes.json()) as BackendTierList[];
    const operators = (await opsRes.json()) as IOperatorIndexEntry[];
    const opById: Record<string, IOperator> = Object.fromEntries(operators.map((op) => [op.id, toCardOperator(op)]));

    const top = lists.slice(0, HOME_TIER_LIST_LIMIT);
    const details = await Promise.all(
        top.map(async (tl) => {
            const res = await backendFetch(`/tier-lists/${tl.slug}`);
            if (!res.ok) throw new Error(`Failed to load tier list ${tl.slug}: ${res.status}`);
            return (await res.json()) as BackendTierListDetail;
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
