// Pool type classification
export type PoolType = "limited" | "regular" | "special";

// Sort modes for banner explorer
export type BannerSortMode = "date" | "activity" | "duration";

// Filter tab values
export const POOL_TYPE_TABS = ["all", "limited", "regular", "special"] as const;
export type PoolTypeFilter = (typeof POOL_TYPE_TABS)[number];

// Badge styles per pool type
export const POOL_TYPE_STYLES: Record<PoolType, { label: string; className: string; color: string }> = {
    limited: { label: "Limited", className: "border-orange-500/30 bg-orange-500/10 text-orange-500", color: "#f97316" },
    regular: { label: "Regular", className: "border-blue-500/30 bg-blue-500/10 text-blue-500", color: "#3b82f6" },
    special: { label: "Special", className: "border-purple-500/30 bg-purple-500/10 text-purple-500", color: "#a855f7" },
};

// Initial number of banners to display before "Show more"
export const INITIAL_BANNER_DISPLAY = 15;
