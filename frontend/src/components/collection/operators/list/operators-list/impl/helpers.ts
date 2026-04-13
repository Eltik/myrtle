// Check if we're on mobile (matches Tailwind's md breakpoint)
export function getInitialViewMode(): "grid" | "list" | "compact" {
    if (typeof window === "undefined") return "grid";
    const saved = localStorage.getItem("viewMode");
    if (saved === "grid" || saved === "list" || saved === "compact") return saved;
    return window.innerWidth < 768 ? "list" : "grid";
}

// Get initial list columns from localStorage
export function getInitialListColumns(): number {
    if (typeof window === "undefined") return 2;
    const saved = localStorage.getItem("listColumns");
    return saved ? Number.parseInt(saved, 10) : 2;
}
