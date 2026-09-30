import type { BannersResponse } from "#/types/generated/BannersResponse";
import type { EventsResponse } from "#/types/generated/EventsResponse";
import type { Resolution } from "#/types/generated/Resolution";
import type { SkinsResponse } from "#/types/generated/SkinsResponse";
import { daysFromToday } from "./helpers";

type Estimate = Extract<Resolution, { status: "estimated" }>;

/**
 * An estimate whose day has passed while EN has still not run it. It is due
 * now, not history: the backend caches its responses for up to a day and knows
 * nothing EN has not shipped, so a passed estimate means only that the guess
 * was early. `estimatedStart` keeps the date the model gave.
 */
export type OverdueEstimate = Estimate & { overdue: true; estimatedStart: number };

export function isOverdue(r: Resolution): r is OverdueEstimate {
    return r.status === "estimated" && "overdue" in r && r.overdue === true;
}

function startOfDay(today: Date): number {
    return Math.floor(new Date(today.getFullYear(), today.getMonth(), today.getDate()).getTime() / 1000);
}

/** An estimate dated before today moves to today, its window clamped to start no earlier. */
export function dueNow(r: Resolution, today: Date): Resolution {
    if (r.status !== "estimated" || isOverdue(r) || daysFromToday(r.enStart, today) >= 0) return r;
    const start = startOfDay(today);
    const out: OverdueEstimate = { ...r, enStart: start, lo: Math.max(r.lo, start), hi: Math.max(r.hi, start), overdue: true, estimatedStart: r.enStart };
    return out;
}

function withResolution<T extends { resolution: Resolution }>(row: T, today: Date): T {
    const resolution = dueNow(row.resolution, today);
    return resolution === row.resolution ? row : { ...row, resolution };
}

export function eventsDueNow(res: EventsResponse, today: Date = new Date()): EventsResponse {
    return { ...res, events: res.events.map((e) => withResolution(e, today)) };
}

export function bannersDueNow(res: BannersResponse, today: Date = new Date()): BannersResponse {
    return { ...res, banners: res.banners.map((b) => withResolution(b, today)) };
}

export function skinsDueNow(res: SkinsResponse, today: Date = new Date()): SkinsResponse {
    return {
        ...res,
        batches: res.batches.map((b) => withResolution(b, today)),
        newSkins: res.newSkins.map((s) => withResolution(s, today)),
        rerunForecasts: res.rerunForecasts.map((r) => {
            const next = dueNow(r.next, today);
            return next === r.next ? r : { ...r, next };
        }),
        reviews: res.reviews.map((r) => withResolution(r, today)),
    };
}
