/**
 * Pulls-tab settings, kept in localStorage.
 *
 * The Originite Prime planner round-trips through `PUT /release/plan` because the
 * backend has a column for it. There is no such column for headhunting settings, and
 * adding one needs a migration plus regenerated ts-rs bindings, so this stays local
 * to the browser. `release-planner:pulls:v1` is deliberately a separate key from
 * `release-planner:plan:v1`: the two tabs spend different currencies and clearing one
 * should not disturb the other.
 */

import * as React from "react";
import { DEFAULT_INCOME, type IIncomeSettings } from "./income";

const STORAGE_KEY = "release-planner:pulls:v1";

export interface IPullsSettings extends IIncomeSettings {
    /** Rolls since the user's last 6* on the standard/kernel counter. */
    pity: number;
    /** Whether the resource figures were typed by hand rather than synced. */
    manual: boolean;
    /** Horizon for the projection, in days. */
    horizonDays: number;
    /** Whether a banner's own free pulls count toward its odds. */
    countFreePulls: boolean;
    /** Rolls committed per banner pool id. This is the plan itself. */
    allocations: Record<string, number>;
    /**
     * Copies wanted per featured operator, per banner pool id. A count of N means
     * potential N; 6 is maximum potential. An operator absent from the map, or at
     * zero, is not being chased.
     */
    targets: Record<string, Record<string, number>>;
}

export const DEFAULT_PULLS: IPullsSettings = {
    ...DEFAULT_INCOME,
    pity: 0,
    manual: false,
    horizonDays: 180,
    countFreePulls: true,
    allocations: {},
    targets: {},
};

/**
 * A stored plan outlives the banners it was made against: a pool id that has since
 * left the forecast never matches a row, so stale keys are harmless and are KEPT
 * rather than pruned, in case the banner returns to the window.
 */
function sanitiseAllocations(raw: unknown): Record<string, number> {
    if (raw === null || typeof raw !== "object") return {};
    const out: Record<string, number> = {};
    for (const [key, value] of Object.entries(raw as Record<string, unknown>)) {
        const n = Number(value);
        if (Number.isFinite(n) && n > 0) out[key] = Math.min(9999, Math.floor(n));
    }
    return out;
}

/**
 * Copies wanted per operator, per banner.
 *
 * An earlier build stored this as a plain list of char ids, meaning "chasing" with no
 * count. A stored plan in that shape is READ rather than discarded: each listed id
 * becomes one copy, which is what it meant.
 */
function sanitiseTargets(raw: unknown): Record<string, Record<string, number>> {
    if (raw === null || typeof raw !== "object") return {};
    const out: Record<string, Record<string, number>> = {};
    for (const [key, value] of Object.entries(raw as Record<string, unknown>)) {
        const copies: Record<string, number> = {};
        if (Array.isArray(value)) {
            for (const id of value) if (typeof id === "string" && id.length > 0) copies[id] = 1;
        } else if (value !== null && typeof value === "object") {
            for (const [id, n] of Object.entries(value as Record<string, unknown>)) {
                const count = Number(n);
                if (id.length > 0 && Number.isFinite(count) && count > 0) copies[id] = Math.min(6, Math.floor(count));
            }
        }
        if (Object.keys(copies).length > 0) out[key] = copies;
    }
    return out;
}

function sanitise(raw: Partial<IPullsSettings> | null): IPullsSettings {
    if (!raw) return DEFAULT_PULLS;
    const num = (v: unknown, fallback: number, min = 0, max = Number.MAX_SAFE_INTEGER) => {
        const n = Number(v);
        return Number.isFinite(n) ? Math.min(max, Math.max(min, n)) : fallback;
    };
    return {
        orundum: num(raw.orundum, 0),
        permits: num(raw.permits, 0),
        tenPermits: num(raw.tenPermits, 0),
        originite: num(raw.originite, 0),
        spendOriginite: raw.spendOriginite !== false,
        monthlyCard: raw.monthlyCard === true,
        annihilation: num(raw.annihilation, DEFAULT_PULLS.annihilation, 0, 5000),
        extraPerDay: num(raw.extraPerDay, 0, 0, 100_000),
        store: raw.store !== false,
        goldCertsPerDay: num(raw.goldCertsPerDay, 1.5, 0, 500),
        goldCertShop: raw.goldCertShop !== false,
        greenCertsPerWeek: num(raw.greenCertsPerWeek, 20, 0, 5000),
        greenCertShop: raw.greenCertShop === "phase1" || raw.greenCertShop === "phase2" ? raw.greenCertShop : "off",
        countFreePulls: raw.countFreePulls !== false,
        // Pity above the hard-pity roll is not a state the game can be in.
        pity: num(raw.pity, 0, 0, 98),
        manual: raw.manual === true,
        horizonDays: num(raw.horizonDays, 180, 7, 730),
        allocations: sanitiseAllocations(raw.allocations),
        targets: sanitiseTargets(raw.targets),
    };
}

export function loadPulls(): IPullsSettings {
    try {
        const raw = localStorage.getItem(STORAGE_KEY);
        return raw ? sanitise(JSON.parse(raw) as Partial<IPullsSettings>) : DEFAULT_PULLS;
    } catch {
        return DEFAULT_PULLS;
    }
}

export function savePulls(settings: IPullsSettings): void {
    try {
        localStorage.setItem(STORAGE_KEY, JSON.stringify(settings));
    } catch {}
}

/**
 * Hydrates in an effect rather than during render, because the server pass has no
 * localStorage and a mismatched first paint would warn.
 */
export function usePullsSettings(): [IPullsSettings, React.Dispatch<React.SetStateAction<IPullsSettings>>, boolean] {
    const [settings, setSettings] = React.useState<IPullsSettings>(DEFAULT_PULLS);
    const [ready, setReady] = React.useState(false);

    React.useEffect(() => {
        setSettings(loadPulls());
        setReady(true);
    }, []);

    React.useEffect(() => {
        if (ready) savePulls(settings);
    }, [settings, ready]);

    return [settings, setSettings, ready];
}
