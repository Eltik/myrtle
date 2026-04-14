// Re-export pagination utilities from shared

export type { PaginationItem } from "../../shared/pagination";
export { generatePaginationItems } from "../../shared/pagination";

/**
 * Formats an account age from a timestamp
 */
export function formatAccountAge(registerTs: number | null | undefined): string | null {
    if (!registerTs) return null;

    // registerTs is a Unix timestamp in seconds
    const date = new Date(registerTs * 1000);

    if (Number.isNaN(date.getTime())) return null;

    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));

    if (diffDays < 30) return `${diffDays} days`;
    if (diffDays < 365) return `${Math.floor(diffDays / 30)} months`;
    const years = Math.floor(diffDays / 365);
    const remainingMonths = Math.floor((diffDays % 365) / 30);
    if (remainingMonths > 0) {
        return `${years}y ${remainingMonths}m`;
    }
    return `${years} years`;
}

/**
 * Gets the secretary character name from ID
 * Falls back to showing the operator ID formatted nicely
 */
export function formatSecretaryName(secretary: string | null | undefined): string | null {
    if (!secretary) return null;

    // Format the operator ID to be more readable
    // e.g., "char_002_amiya" -> "Amiya"
    const parts = secretary.split("_");
    if (parts.length >= 3) {
        const name = parts.slice(2).join(" ");
        return name.charAt(0).toUpperCase() + name.slice(1);
    }
    return secretary;
}

/**
 * Formats a date/timestamp into a human-readable relative time string.
 * @param value - ISO date string or Unix timestamp (seconds) to format
 * @returns Relative time string (e.g., "Today", "Yesterday", "3 days ago")
 */
export function formatRelativeTime(value: string | number): string {
    const date = typeof value === "number" ? new Date(value * 1000) : new Date(value);
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

/**
 * Formats a score number into a compact human-readable string.
 * @param score - The score to format
 * @returns Compact score string (e.g., "1.2M", "45.3K", "999")
 */
export function formatScore(score: number): string {
    if (score >= 1_000_000) {
        return `${(score / 1_000_000).toFixed(1)}M`;
    }
    if (score >= 1_000) {
        return `${(score / 1_000).toFixed(1)}K`;
    }
    return score.toString();
}

/**
 * Formats a Unix timestamp to a localized date string
 */
export function formatRegistrationDate(registerTs: number | null | undefined): string | null {
    if (!registerTs) return null;
    const date = new Date(registerTs * 1000);
    if (Number.isNaN(date.getTime())) return null;
    return date.toLocaleDateString();
}
