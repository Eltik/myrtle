// Re-export pagination utilities from shared

export type { PaginationItem } from "../../shared/pagination";
export { generatePaginationItems } from "../../shared/pagination";

/**
 * Formats a date string into a human-readable relative time string.
 * @param dateString - ISO date string to format
 * @returns Relative time string (e.g., "Today", "Yesterday", "3 days ago")
 */
export function formatRelativeTime(dateString: string): string {
    const date = new Date(dateString);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));

    if (diffDays === 0) return "Today";
    if (diffDays === 1) return "Yesterday";
    if (diffDays < 7) return `${diffDays} days ago`;
    if (diffDays < 30) return `${Math.floor(diffDays / 7)} weeks ago`;
    if (diffDays < 365) return `${Math.floor(diffDays / 30)} months ago`;
    return `${Math.floor(diffDays / 365)} years ago`;
}
