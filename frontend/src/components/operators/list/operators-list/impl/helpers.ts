// Check if we're on mobile (matches Tailwind's md breakpoint)
export function getInitialViewMode(): "grid" | "list" {
    if (typeof window === "undefined") return "grid";
    return window.innerWidth < 768 ? "list" : "grid";
}

// Get initial list columns from localStorage
export function getInitialListColumns(): number {
    if (typeof window === "undefined") return 2;
    const saved = localStorage.getItem("operatorListColumns");
    return saved ? Number.parseInt(saved, 10) : 2;
}
