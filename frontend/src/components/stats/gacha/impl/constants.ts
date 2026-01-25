import type { LucideIcon } from "lucide-react";
import { Minus, TrendingDown, TrendingUp } from "lucide-react";

// Official gacha rates for comparison
export const EXPECTED_RATES = {
    6: 0.02, // 2%
    5: 0.08, // 8%
    4: 0.5, // 50%
    3: 0.4, // 40%
} as const;

// Color configurations for each rarity
export const RARITY_COLORS = {
    6: {
        hex: "#f97316", // orange-500
        text: "text-orange-500",
        bg: "bg-orange-500/10",
        border: "border-orange-500/30",
        bgClass: "bg-orange-500",
    },
    5: {
        hex: "#eab308", // yellow-500
        text: "text-yellow-500",
        bg: "bg-yellow-500/10",
        border: "border-yellow-500/30",
        bgClass: "bg-yellow-500",
    },
    4: {
        hex: "#a855f7", // purple-500
        text: "text-purple-500",
        bg: "bg-purple-500/10",
        border: "border-purple-500/30",
        bgClass: "bg-purple-500",
    },
    3: {
        hex: "#3b82f6", // blue-500
        text: "text-blue-500",
        bg: "bg-blue-500/10",
        border: "border-blue-500/30",
        bgClass: "bg-blue-500",
    },
} as const;

// Day names for chart display
export const DAY_NAMES = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"] as const;

// Luck status thresholds and configurations
export interface LuckStatus {
    label: string;
    color: string;
    bg: string;
    icon: LucideIcon;
}

export function getLuckStatus(luckScore: number): LuckStatus {
    if (luckScore > 0.05) return { label: "Very Lucky", color: "text-green-500", bg: "bg-green-500/10", icon: TrendingUp };
    if (luckScore > 0.01) return { label: "Lucky", color: "text-green-400", bg: "bg-green-400/10", icon: TrendingUp };
    if (luckScore > -0.01) return { label: "Average", color: "text-muted-foreground", bg: "bg-muted", icon: Minus };
    if (luckScore > -0.05) return { label: "Unlucky", color: "text-yellow-500", bg: "bg-yellow-500/10", icon: TrendingDown };
    return { label: "Very Unlucky", color: "text-red-500", bg: "bg-red-500/10", icon: TrendingDown };
}

// Chart configurations
export const CHART_COLORS = {
    hourly: "#3b82f6", // blue-500
    daily: "#22c55e", // green-500
    activity: "#8b5cf6", // violet-500
} as const;
