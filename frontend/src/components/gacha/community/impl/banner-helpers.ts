import type { DatePullData, GachaPoolClient } from "~/types/api";
import type { BannerSortMode, PoolType, PoolTypeFilter } from "./banner-constants";

/** Processed banner entry with all derived fields for display */
export interface ProcessedBanner {
    gachaPoolId: string;
    gachaPoolName: string;
    poolType: PoolType;
    openTime: number;
    endTime: number;
    durationDays: number;
    openDateStr: string;
    endDateStr: string;
    estimatedPulls: number;
    gachaRuleType: string;
    cdPrimColor: string | null;
    cdSecColor: string | null;
    isActive: boolean;
}

/** Chart date data item (matches PullActivityChart's DateDataItem) */
export interface DateDataItem {
    date: string;
    fullDate: string;
    pulls: number;
}

/** Banner overlay region for the activity chart */
export interface BannerOverlay {
    gachaPoolId: string;
    gachaPoolName: string;
    poolType: PoolType;
    startDate: string;
    endDate: string;
    color: string;
}

/** Classify pool type from pool ID prefix */
export function classifyPoolType(gachaPoolId: string): PoolType {
    if (gachaPoolId.startsWith("LIMITED_")) return "limited";
    if (gachaPoolId.startsWith("SINGLE_") || gachaPoolId.startsWith("LINKAGE_")) return "special";
    return "regular";
}

/** Calculate banner duration in days from Unix timestamps (seconds) */
export function calculateDurationDays(openTime: number, endTime: number): number {
    const MS_PER_DAY = 86_400_000;
    const diffMs = (endTime - openTime) * 1000;
    return Math.max(1, Math.round(diffMs / MS_PER_DAY));
}

/** Format a Unix timestamp (seconds) to a short date string */
export function formatBannerDate(timestamp: number): string {
    return new Date(timestamp * 1000).toLocaleDateString(undefined, {
        year: "numeric",
        month: "short",
        day: "numeric",
    });
}

/** Convert a Unix timestamp (seconds) to YYYY-MM-DD */
function toDateString(timestampSeconds: number): string {
    const d = new Date(timestampSeconds * 1000);
    return d.toISOString().slice(0, 10);
}

/** Build a Map<YYYY-MM-DD, pullCount> from the byDate array for efficient lookup */
export function buildDatePullMap(byDate: DatePullData[] | undefined): Map<string, number> {
    const map = new Map<string, number>();
    if (!byDate) return map;
    for (const entry of byDate) {
        map.set(entry.date, entry.pullCount);
    }
    return map;
}

/** Estimate pull activity for a banner by summing byDate entries within its date range */
export function estimateBannerPulls(openTime: number, endTime: number, byDateMap: Map<string, number>): number {
    const startDate = toDateString(openTime);
    const endDate = toDateString(endTime);
    let totalPulls = 0;

    for (const [date, count] of byDateMap) {
        if (date >= startDate && date <= endDate) {
            totalPulls += count;
        }
    }

    return totalPulls;
}

/** Process raw GachaPoolClient[] into ProcessedBanner[] with all derived fields */
export function processPoolData(pools: GachaPoolClient[], byDateMap: Map<string, number>): ProcessedBanner[] {
    const now = Date.now() / 1000;

    return pools.map((pool) => ({
        gachaPoolId: pool.gachaPoolId,
        gachaPoolName: pool.gachaPoolName,
        poolType: classifyPoolType(pool.gachaPoolId),
        openTime: pool.openTime,
        endTime: pool.endTime,
        durationDays: calculateDurationDays(pool.openTime, pool.endTime),
        openDateStr: formatBannerDate(pool.openTime),
        endDateStr: formatBannerDate(pool.endTime),
        estimatedPulls: estimateBannerPulls(pool.openTime, pool.endTime, byDateMap),
        gachaRuleType: pool.gachaRuleType,
        cdPrimColor: pool.cdPrimColor,
        cdSecColor: pool.cdSecColor,
        isActive: now >= pool.openTime && now <= pool.endTime,
    }));
}

/** Sort banners by the specified mode */
export function sortBanners(banners: ProcessedBanner[], mode: BannerSortMode): ProcessedBanner[] {
    return [...banners].sort((a, b) => {
        switch (mode) {
            case "date":
                return b.openTime - a.openTime;
            case "activity":
                return b.estimatedPulls - a.estimatedPulls;
            case "duration":
                return b.durationDays - a.durationDays;
            default:
                return 0;
        }
    });
}

/** Filter banners by pool type */
export function filterBanners(banners: ProcessedBanner[], filter: PoolTypeFilter): ProcessedBanner[] {
    if (filter === "all") return banners;
    return banners.filter((b) => b.poolType === filter);
}

/** Search banners by name (case-insensitive) */
export function searchBanners(banners: ProcessedBanner[], query: string): ProcessedBanner[] {
    if (!query.trim()) return banners;
    const lower = query.toLowerCase();
    return banners.filter((b) => b.gachaPoolName.toLowerCase().includes(lower));
}

/** Build banner overlays for the chart (LIMITED banners only, clipped to data range) */
export function buildBannerOverlays(pools: GachaPoolClient[], dateData: DateDataItem[]): BannerOverlay[] {
    if (dateData.length === 0) return [];

    const firstEntry = dateData[0];
    const lastEntry = dateData[dateData.length - 1];
    if (!firstEntry || !lastEntry) return [];

    const firstDate = firstEntry.fullDate;
    const lastDate = lastEntry.fullDate;

    return pools
        .filter((pool) => pool.gachaPoolId.startsWith("LIMITED_"))
        .map((pool) => {
            const startDate = toDateString(pool.openTime);
            const endDate = toDateString(pool.endTime);
            return {
                gachaPoolId: pool.gachaPoolId,
                gachaPoolName: pool.gachaPoolName,
                poolType: "limited" as const,
                startDate: startDate < firstDate ? firstDate : startDate,
                endDate: endDate > lastDate ? lastDate : endDate,
                color: pool.cdPrimColor ? `#${pool.cdPrimColor}` : "#f97316",
            };
        })
        .filter((overlay) => overlay.startDate <= lastDate && overlay.endDate >= firstDate);
}
