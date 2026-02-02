import type { EnemyLevel } from "~/types/api";

export const enemyLevelToNumber = (level: EnemyLevel) => {
    switch (level) {
        case "BOSS":
            return 3;
        case "ELITE":
            return 2;
        default:
            return 1;
    }
};

export function getInitialViewMode(): "grid" | "list" {
    if (typeof window === "undefined") return "grid";
    return window.innerWidth < 768 ? "list" : "grid";
}

// Get initial list columns from localStorage
export function getInitialListColumns(): number {
    if (typeof window === "undefined") return 2;
    const saved = localStorage.getItem("listColumns");
    return saved ? Number.parseInt(saved, 10) : 2;
}
