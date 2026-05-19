import type { IOperator } from "#/components/home/impl/data";
import type { ITierListBrowseItem, ITierOperator } from "#/lib/api/tier-lists";
import { FALLBACK_TIER_COLORS } from "#/lib/utils";
import type { IOperatorIndexEntry } from "#/types/operators";

export const LIST_NAME_MAX = 80;
export const LIST_DESCRIPTION_MAX = 4000;
export const TIER_NAME_MAX = 24;
export const TIER_DESCRIPTION_MAX = 1000;
export const DESCRIPTION_CLAMP_THRESHOLD = 280;

export const MAX_THUMB_TIERS = 5;
const OPS_PER_ROW_BY_COUNT: Record<number, number> = { 1: 4, 2: 6, 3: 7 };
const DEFAULT_OPS_PER_ROW = 9;

export interface IThumbRow {
    name: string;
    color: string;
    visible: IOperator[];
    overflow: number;
}

export function buildThumbRows(tl: ITierListBrowseItem): IThumbRow[] {
    const slice = tl.tiers.slice(0, MAX_THUMB_TIERS);
    const opsPerRow = OPS_PER_ROW_BY_COUNT[slice.length] ?? DEFAULT_OPS_PER_ROW;
    return slice.map((t, idx): IThumbRow => {
        const color = t.color ?? FALLBACK_TIER_COLORS[idx % FALLBACK_TIER_COLORS.length] ?? "var(--primary)";
        const visible = t.operators.slice(0, opsPerRow);
        const overflow = Math.max(0, t.operators.length - visible.length);
        return { name: t.name, color, visible, overflow };
    });
}

export function indexEntryToTierOperator(entry: IOperatorIndexEntry): ITierOperator {
    return {
        id: entry.id,
        name: entry.name,
        appellation: entry.appellation || null,
        rarity: entry.rarity,
        profession: entry.profession,
        subProfessionId: entry.subProfessionId,
        position: entry.position,
        nationId: entry.nationId || null,
        subOrder: 0,
        notes: null,
        updatedAt: new Date().toISOString(),
    };
}

export type BrowseListSort = "trending" | "recent" | "newest" | "alpha" | "views" | "favorites" | "shares";

export function sortBrowseItems(lists: ITierListBrowseItem[], sort: BrowseListSort): ITierListBrowseItem[] {
    const arr = [...lists];
    switch (sort) {
        case "trending":
            return arr.sort((a, b) => {
                if (a.isTrending !== b.isTrending) return a.isTrending ? -1 : 1;
                if (a.trendingScore !== b.trendingScore) return b.trendingScore - a.trendingScore;
                return b.views24h - a.views24h;
            });
        case "recent":
            return arr.sort((a, b) => b.updatedAtMs - a.updatedAtMs);
        case "newest":
            return arr.sort((a, b) => b.createdAtMs - a.createdAtMs);
        case "alpha":
            return arr.sort((a, b) => a.title.localeCompare(b.title, "en", { sensitivity: "base" }));
        case "views":
            return arr.sort((a, b) => b.views - a.views);
        case "favorites":
            return arr.sort((a, b) => b.favorites - a.favorites);
        case "shares":
            return arr.sort((a, b) => b.shares - a.shares);
        default:
            return arr;
    }
}

export function matchesBrowseQuery(list: ITierListBrowseItem, q: string): boolean {
    if (!q) return true;
    const needle = q.toLowerCase();
    if (list.title.toLowerCase().includes(needle)) return true;
    if (list.description.toLowerCase().includes(needle)) return true;
    if (list.author.name.toLowerCase().includes(needle)) return true;
    if (list.flairLabel?.toLowerCase().includes(needle)) return true;
    return false;
}
