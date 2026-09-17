/**
 * Projecting headhunting resources forward in time.
 *
 * Every recurring source below is an officially disclosed figure. What is NOT here
 * is per-event orundum: the release backend models Originite Prime from
 * `OpStage.op` and event shop tokens, but it carries no orundum reward data at all,
 * so an event's one-off orundum cannot be derived from what this project ingests.
 * That is left to the user as `extraPerDay` rather than invented, and the UI says so.
 */

import { ORUNDUM_PER_ORIGINITE, ORUNDUM_PER_PULL } from "./rates";

/** Daily missions, every day. */
export const DAILY_MISSION_ORUNDUM = 100;
/** Weekly missions, once per weekly reset. */
export const WEEKLY_MISSION_ORUNDUM = 500;
/** Monthly card, per day, for as long as it is active. */
export const MONTHLY_CARD_ORUNDUM = 200;
/** Originite Prime the monthly card pays on renewal. */
export const MONTHLY_CARD_ORIGINITE = 6;
/** Green-certificate shop, refreshed monthly. */
export const STORE_MONTHLY_ORUNDUM = 600;
export const STORE_MONTHLY_PERMITS = 4;

/**
 * Weekly annihilation orundum. The cap starts at 1200 and rises with campaign
 * progress, so it is a user input rather than a constant; these are the reachable
 * values.
 */
export const ANNIHILATION_CAPS = [1200, 1400, 1600, 1700, 1800] as const;
export const DEFAULT_ANNIHILATION = 1800;

/**
 * Weekly reset lands on Monday. `Date.getUTCDay()` returns 1 for Monday.
 * imivi's calculator books weekly missions on Thursday instead; the choice shifts a
 * projection by at most one week's income and only at the boundary.
 */
const WEEKLY_RESET_DAY = 1;

const DAY_MS = 86_400_000;

/**
 * The monthly Headhunting Permit ladder in the Distinction (gold) certificate shop.
 * Marginal, not cumulative: buying every rung is 38 permits for 258 certificates,
 * which is the cumulative table imivi's calculator hardcodes as its cert-to-ticket
 * reference (10/28/68/138/258 certs for 1/3/8/18/38 permits). Stock is one of each
 * per month and they stack, so the rungs are bought cheapest first.
 */
export const GOLD_PERMIT_LADDER: { certs: number; permits: number }[] = [
    { certs: 10, permits: 1 },
    { certs: 18, permits: 2 },
    { certs: 40, permits: 5 },
    { certs: 70, permits: 10 },
    { certs: 120, permits: 20 },
];

/**
 * The pull-relevant rows of the Commendation (green) certificate shop.
 *
 * The shop runs in phases that reset monthly, and phase 2 only opens once phase 1 has
 * been bought out ENTIRELY, which costs 1490 certificates including a great deal that
 * has nothing to do with pulling. That gate is the whole reason the phase choice is
 * worth offering rather than assuming: at the 20 certificates a week that weekly
 * missions pay, a player banks about 87 a month and never reaches phase 2 at all.
 *
 * Within phase 1 the permit and the Orundum are exactly equal value, which is worth
 * knowing before agonising over the order: 240 certificates buys either one permit or
 * 600 Orundum, and a permit IS 600 Orundum. Permits are bought first only because a
 * whole pull is easier to reason about than a part of one.
 */
export const GREEN_PHASE1_PERMITS = { certs: 240, stock: 2 };
export const GREEN_PHASE1_ORUNDUM = { certs: 40, orundum: 100, stock: 6 };
export const GREEN_PHASE1_BUYOUT = 1490;
export const GREEN_PHASE2_PERMITS = { certs: 450, stock: 2 };

/** How deep into the monthly Commendation shop the player actually buys. */
export type GreenCertShop = "off" | "phase1" | "phase2";

export interface ICertPurchase {
    permits: number;
    orundum: number;
    goldSpent: number;
    greenSpent: number;
}

/** What one month's certificates buy, gold and green shops together. */
export function certPurchases(gold: number, green: number, greenShop: GreenCertShop): ICertPurchase {
    let goldLeft = Math.max(0, gold);
    let permits = 0;
    let orundum = 0;
    for (const rung of GOLD_PERMIT_LADDER) {
        if (goldLeft < rung.certs) continue;
        goldLeft -= rung.certs;
        permits += rung.permits;
    }

    let greenLeft = Math.max(0, green);
    if (greenShop !== "off") {
        for (let i = 0; i < GREEN_PHASE1_PERMITS.stock && greenLeft >= GREEN_PHASE1_PERMITS.certs; i++) {
            greenLeft -= GREEN_PHASE1_PERMITS.certs;
            permits += 1;
        }
        for (let i = 0; i < GREEN_PHASE1_ORUNDUM.stock && greenLeft >= GREEN_PHASE1_ORUNDUM.certs; i++) {
            greenLeft -= GREEN_PHASE1_ORUNDUM.certs;
            orundum += GREEN_PHASE1_ORUNDUM.orundum;
        }
        // Phase 2 is gated on clearing the whole of phase 1, junk included.
        if (greenShop === "phase2" && green >= GREEN_PHASE1_BUYOUT) {
            let afterBuyout = green - GREEN_PHASE1_BUYOUT;
            for (let i = 0; i < GREEN_PHASE2_PERMITS.stock && afterBuyout >= GREEN_PHASE2_PERMITS.certs; i++) {
                afterBuyout -= GREEN_PHASE2_PERMITS.certs;
                permits += 1;
            }
            greenLeft = afterBuyout;
        }
    }

    return { permits, orundum, goldSpent: Math.max(0, gold) - goldLeft, greenSpent: Math.max(0, green) - greenLeft };
}

export interface IIncomeSettings {
    orundum: number;
    permits: number;
    tenPermits: number;
    originite: number;
    /** Whether Originite Prime is converted into rolls in the projection. */
    spendOriginite: boolean;
    monthlyCard: boolean;
    annihilation: number;
    /** Anything the model does not carry: event orundum, farming, mail. */
    extraPerDay: number;
    /** Whether to count the monthly certificate-shop orundum and permits. */
    store: boolean;
    /** Distinction (gold) certificates earned per day. The player's own estimate. */
    goldCertsPerDay: number;
    /** Whether to spend gold certificates on the monthly permit ladder. */
    goldCertShop: boolean;
    /** Commendation (green) certificates earned per week. Weekly missions pay 20. */
    greenCertsPerWeek: number;
    /** How deep into the monthly Commendation shop the player buys. */
    greenCertShop: GreenCertShop;
}

export const DEFAULT_INCOME: IIncomeSettings = {
    orundum: 0,
    permits: 0,
    tenPermits: 0,
    originite: 0,
    spendOriginite: true,
    monthlyCard: false,
    annihilation: DEFAULT_ANNIHILATION,
    extraPerDay: 0,
    store: true,
    // imivi's calculator defaults to 1.5 a day with the note "1Y+ account: around 1.5
    // certs/day from recruitment". That is its author's heuristic, not a game figure,
    // and it is the reason this is an input rather than a constant.
    goldCertsPerDay: 1.5,
    goldCertShop: true,
    // 20 a week from weekly missions is the one officially documented rate; dupes pay
    // more and vary wildly by account, so the field stays editable.
    greenCertsPerWeek: 20,
    greenCertShop: "off",
};

export interface IProjectedDay {
    /** Unix seconds at the start of the day, UTC. */
    at: number;
    orundum: number;
    permits: number;
    originite: number;
    /** Rolls affordable without touching Originite Prime. */
    pulls: number;
    /**
     * Rolls affordable once Originite Prime is converted, after the outfit picks
     * have taken theirs. Reserved Originite is never converted.
     */
    pullsWithOriginite: number;
}

function startOfDayUTC(d: Date): number {
    return Date.UTC(d.getUTCFullYear(), d.getUTCMonth(), d.getUTCDate());
}

function pullsFrom(orundum: number, permits: number, tenPermits: number, originite: number, spendOriginite: boolean): number {
    const pool = orundum + (spendOriginite ? originite * ORUNDUM_PER_ORIGINITE : 0);
    return Math.floor(pool / ORUNDUM_PER_PULL) + permits + tenPermits * 10;
}

/**
 * Day-by-day resources from `from` through `to` inclusive.
 *
 * The first day accrues nothing: it is the user's balance as entered, so income is
 * booked from the following reset onward rather than handing them a day they have
 * already claimed.
 *
 * `reservedOriginite` is what the outfit picks in the Planner tab cost. It has first
 * claim on the Originite balance from day one: only what is left over is converted
 * into rolls, so a pick made in the other tab shows up here as fewer pulls rather
 * than as a warning. The reservation is booked up front rather than on each outfit's
 * sale day because the picks carry no date, and earmarking early is the conservative
 * reading. `originite` on each day is the raw balance, before the reservation.
 */
export function projectIncome(settings: IIncomeSettings, from: Date, to: Date, reservedOriginite = 0): IProjectedDay[] {
    const startMs = startOfDayUTC(from);
    const endMs = startOfDayUTC(to);
    if (!Number.isFinite(startMs) || !Number.isFinite(endMs) || endMs < startMs) return [];

    let orundum = Math.max(0, settings.orundum);
    let permits = Math.max(0, settings.permits);
    const tenPermits = Math.max(0, settings.tenPermits);
    let originite = Math.max(0, settings.originite);
    const reserved = Math.max(0, reservedOriginite);
    let goldCerts = 0;
    let greenCerts = 0;
    const days: IProjectedDay[] = [];
    const total = Math.min(Math.round((endMs - startMs) / DAY_MS), 1500);

    for (let i = 0; i <= total; i++) {
        const ms = startMs + i * DAY_MS;
        const day = new Date(ms);
        if (i > 0) {
            orundum += DAILY_MISSION_ORUNDUM + settings.extraPerDay;
            if (settings.monthlyCard) orundum += MONTHLY_CARD_ORUNDUM;
            if (day.getUTCDay() === WEEKLY_RESET_DAY) orundum += WEEKLY_MISSION_ORUNDUM + Math.max(0, settings.annihilation);
            if (day.getUTCDate() === 1) {
                if (settings.store) {
                    orundum += STORE_MONTHLY_ORUNDUM;
                    permits += STORE_MONTHLY_PERMITS;
                }
                if (settings.monthlyCard) originite += MONTHLY_CARD_ORIGINITE;
                // Certificate shops reset with the month, so a month's earnings are
                // spent as one basket rather than trickled in daily.
                const bought = certPurchases(settings.goldCertShop ? goldCerts : 0, greenCerts, settings.greenCertShop);
                permits += bought.permits;
                orundum += bought.orundum;
                goldCerts -= bought.goldSpent;
                greenCerts -= bought.greenSpent;
            }
            goldCerts += Math.max(0, settings.goldCertsPerDay);
            if (day.getUTCDay() === WEEKLY_RESET_DAY) greenCerts += Math.max(0, settings.greenCertsPerWeek);
        }
        days.push({
            at: Math.floor(ms / 1000),
            orundum,
            permits,
            originite,
            pulls: pullsFrom(orundum, permits, tenPermits, 0, false),
            pullsWithOriginite: pullsFrom(orundum, permits, tenPermits, Math.max(0, originite - reserved), settings.spendOriginite),
        });
    }
    return days;
}

/** The projected day at or immediately before `atSeconds`. */
export function dayAt(days: IProjectedDay[], atSeconds: number): IProjectedDay | null {
    if (days.length === 0) return null;
    if (atSeconds < days[0].at) return null;
    let lo = 0;
    let hi = days.length - 1;
    while (lo < hi) {
        const mid = Math.ceil((lo + hi) / 2);
        if (days[mid].at <= atSeconds) lo = mid;
        else hi = mid - 1;
    }
    return days[lo];
}

export interface IOriginiteWarning {
    /** Originite Prime the outfit picks need. */
    needed: number;
    /** Originite Prime that will be on hand at the end of the projection. */
    available: number;
    /** How far short the outfit picks leave the player. Always positive. */
    short: number;
}

/**
 * Whether the outfit picks can be paid for at all.
 *
 * The pull projection already reserves the outfit cost before converting anything
 * (`projectIncome`), so pulls and outfits no longer compete: the only thing left to
 * say is when the outfits alone cost more Originite Prime than the player will ever
 * hold over the horizon, in which case no amount of not-pulling fixes it.
 */
export function originiteWarning(committed: number, availableAtHorizon: number): IOriginiteWarning | null {
    const needed = Math.max(0, committed);
    if (needed === 0) return null;
    const available = Math.max(0, availableAtHorizon);
    const short = Math.max(0, needed - available);
    if (short === 0) return null;
    return { needed, available, short };
}
