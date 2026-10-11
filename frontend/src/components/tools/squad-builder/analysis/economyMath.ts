/**
 * The Economy panel's arithmetic, kept out of React so it can be tested: the
 * cost ladder's scale, the starting-DP line's position and its inverse for a
 * drag, the opener count the ladder recolours from while a new analysis is on
 * its way, the redeploy range and the inputs of each cut's label.
 */

import type { Cut } from "#/types/generated/Cut";
import type { EconomyBonus } from "#/types/generated/EconomyBonus";
import type { EconomySeat } from "#/types/generated/EconomySeat";

import { START_DP_MAX, START_DP_MIN } from "../constants";

export { START_DP_MAX, START_DP_MIN } from "../constants";

/** An integer starting DP inside what the stepper allows. */
export function clampStartDp(n: number): number {
    if (!Number.isFinite(n)) return START_DP_MIN;
    return Math.max(START_DP_MIN, Math.min(START_DP_MAX, Math.round(n)));
}

/** The ladder's full width in DP: the dearest seat, at least 1 so nothing divides by zero. */
export function ladderScale(costs: ReadonlyArray<number>): number {
    let max = 1;
    for (const c of costs) if (c > max) max = c;
    return max;
}

/** A bar's length, or the line's position, as a fraction of the track. */
export function fractionOf(value: number, scale: number): number {
    if (scale <= 0) return 0;
    return Math.max(0, Math.min(1, value / scale));
}

/** The DP a pointer at `x` pixels into a track `width` pixels wide stands for. */
export function dpFromPointer(x: number, width: number, scale: number): number {
    if (width <= 0) return 0;
    return Math.round(fractionOf(x, width) * scale);
}

/** The DP talents add at the start just by being in the squad. */
export function startBonusTotal(bonus: ReadonlyArray<EconomyBonus>): number {
    let sum = 0;
    for (const b of bonus) sum += b.amount;
    return sum;
}

/**
 * The line sits at the DP the squad actually starts with (the page's number
 * plus squad bonuses); dragging it sets the page's number, so the bonus comes
 * back off before it is clamped to the stepper's range.
 */
export function baseDpFromLine(lineDp: number, bonus: number): number {
    return clampStartDp(lineDp - bonus);
}

/** Seats whose cost is at or under the starting DP, the same rule as `economy.openers`. */
export function countOpeners(costs: ReadonlyArray<number>, startDp: number): number {
    let n = 0;
    for (const c of costs) if (c <= startDp) n++;
    return n;
}

/** Shortest and longest redeploy time over the seats; `null` with no seats. */
export function respawnRange(respawns: ReadonlyArray<number>): { min: number; max: number } | null {
    if (respawns.length === 0) return null;
    let min = respawns[0];
    let max = respawns[0];
    for (const r of respawns) {
        if (r < min) min = r;
        if (r > max) max = r;
    }
    return { min, max };
}

/** The first squad-wide redeploy cut that scales (SilverAsh's -12%), for the strip's caption. */
export function squadRespawnCut(seats: ReadonlyArray<Pick<EconomySeat, "respawn_cuts">>): Cut | null {
    for (const s of seats) {
        for (const c of s.respawn_cuts) if (c.kind === "squad" && c.ratio !== null) return c;
    }
    return null;
}

/** `-3`, `+1`, `-12%`: the signed amount a cut's label ends with. */
export function cutAmountText(cut: Pick<Cut, "amount" | "ratio">): string {
    if (cut.ratio !== null) {
        const pct = Math.round(cut.ratio * 100);
        return `${pct > 0 ? "+" : pct < 0 ? "-" : ""}${Math.abs(pct)}%`;
    }
    const n = Math.round(cut.amount * 100) / 100;
    return `${n > 0 ? "+" : n < 0 ? "-" : ""}${Math.abs(n)}`;
}

export type CutLabelKey = "potential" | "talent" | "module" | "squad";

export interface ICutLabelInput {
    key: CutLabelKey;
    name: string;
    giver: string;
    amount: string;
}

/**
 * What a cut's label needs: which sentence, the talent or module name, the
 * giver's name for a squad cut, and the signed amount. `giverName` is looked
 * up by the caller from `from_seat`.
 */
export function cutLabelInput(cut: Cut, giverName: string | null): ICutLabelInput {
    return { key: cut.kind, name: cut.name ?? "", giver: giverName ?? "", amount: cutAmountText(cut) };
}
