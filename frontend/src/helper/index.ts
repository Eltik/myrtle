import type { PlayerData } from "~/types/types";

export function getInitials(name: string) {
    return name?.split(" ").map(word => word[0]?.toUpperCase()).join("");
}

export function isPlayerData(data: unknown): data is PlayerData {
    return (data && typeof data === "object" && "nickname" in data && "level" in data && "trust" in data && "friendship" in data && "potential" in data && "rank" in data && "tags" in data && "skills" in data && "operators" in data && "badges" in data && "base" in data && "stats" in data && "server" in data) as boolean;
}

export function capitalize(s: string) {
    s = s?.toLowerCase();
    return s && (s[0]?.toUpperCase() ?? "") + s.slice(1);
}

export function truncate(text: string, maxLength: number) {
    if (text.length > maxLength) {
        return text.substring(0, maxLength - 3) + "...";
    } else {
        return text;
    }
}

export function formatCompactNumber(number: number) {
    if (number < 1000) {
        return number;
    } else if (number >= 1000 && number < 1_000_000) {
        return (number / 1000).toFixed(1) + "k";
    } else if (number >= 1_000_000 && number < 1_000_000_000) {
        return (number / 1_000_000).toFixed(1) + "m";
    } else if (number >= 1_000_000_000 && number < 1_000_000_000_000) {
        return (number / 1_000_000_000).toFixed(1) + "b";
    } else if (number >= 1_000_000_000_000 && number < 1_000_000_000_000_000) {
        return (number / 1_000_000_000_000).toFixed(1) + "t";
    }
}

export function isValidDate(d: Date | number | string) {
    return d instanceof Date && !isNaN(Number(d));
}
