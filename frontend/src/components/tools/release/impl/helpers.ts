import { env } from "#/env";
import type { LagModel } from "#/types/generated/LagModel";
import type { Resolution } from "#/types/generated/Resolution";

const DAY_MS = 86_400_000;

export const PAST_GRACE_DAYS = 14;

export function fromUnix(seconds: number): Date {
    return new Date(seconds * 1000);
}

export function formatDate(seconds: number): string {
    return fromUnix(seconds).toLocaleDateString("en-US", { month: "short", day: "numeric", year: "numeric" });
}

function formatMonthDay(seconds: number): string {
    return fromUnix(seconds).toLocaleDateString("en-US", { month: "short", day: "numeric" });
}

export function formatDateRange(start: number, end: number | null | undefined): string {
    if (!end || end <= start) return formatDate(start);
    const a = fromUnix(start);
    const b = fromUnix(end);
    if (a.getFullYear() === b.getFullYear()) return `${formatMonthDay(start)} to ${formatDate(end)}`;
    return `${formatDate(start)} to ${formatDate(end)}`;
}

export function daysFromToday(seconds: number, today: Date): number {
    const anchor = new Date(today.getFullYear(), today.getMonth(), today.getDate());
    const d = fromUnix(seconds);
    const target = new Date(d.getFullYear(), d.getMonth(), d.getDate());
    return Math.round((target.getTime() - anchor.getTime()) / DAY_MS);
}

export function relativeDays(days: number): string {
    if (days === 0) return "today";
    if (days === 1) return "tomorrow";
    if (days === -1) return "yesterday";
    return days > 0 ? `in ${days} days` : `${-days} days ago`;
}

export function resolvedEnStart(r: Resolution): number | null {
    return r.status === "unmodelled" || r.status === "independent" ? null : r.enStart;
}

export function sortKey(r: Resolution, cnStart: number, model: LagModel | null | undefined): number {
    const en = resolvedEnStart(r);
    if (en !== null) return en;
    return cnStart + (model?.medianDays ?? 0) * 86_400;
}

export function isPast(keySeconds: number, today: Date): boolean {
    return daysFromToday(keySeconds, today) < -PAST_GRACE_DAYS;
}

export function formatDays(n: number): string {
    return Number.isInteger(n) ? String(n) : n.toFixed(1);
}

export function formatPercent(fraction: number): string {
    return `${Math.round(fraction * 100)}%`;
}

export function humanizeTag(raw: string): string {
    const s = raw.replace(/_/g, " ").toLowerCase();
    return s.charAt(0).toUpperCase() + s.slice(1);
}

export function assetUrl(path: string | null): string | null {
    if (!path) return null;
    const encoded = path.split("/").map(encodeURIComponent).join("/");
    return `${env.VITE_BACKEND_URL ?? ""}/api${encoded.startsWith("/") ? "" : "/"}${encoded}`;
}
