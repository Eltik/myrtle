/**
 * READING A COMMAND'S ARGUMENTS. Every value on the wire is a string, and this
 * is the one place that decides what a missing, empty or malformed one means.
 *
 * The rule throughout is house rule 9's coercion trap: a MISSING argument is
 * checked for explicitly and falls back, never falsily, because `Number("")` is
 * 0 and finite while `Number(undefined)` is NaN. Each reader names the default
 * the client itself uses, so the engine's switch can write `num(a.x, 0)` and
 * not restate it.
 */
import { clamp01 } from "./num";
import type { ScreenAdapt, Slot } from "./scene";

export function num(value: string | undefined, fallback: number): number {
    if (value === undefined || value === "") return fallback;
    const n = Number(value);
    return Number.isFinite(n) ? n : fallback;
}

export function bool(value: string | undefined, fallback: boolean): boolean {
    if (value === undefined) return fallback;
    const v = value.trim().toLowerCase();
    if (v === "true" || v === "1") return true;
    if (v === "false" || v === "0") return false;
    return fallback;
}

const SLOT_ALIASES: Record<string, Slot> = { l: "l", left: "l", m: "m", middle: "m", r: "r", right: "r" };

/**
 * The wire `assets` maps are keyed by the TRIMMED name, because the backend
 * resolver trims before it looks up. The corpus writes trailing spaces in
 * `name=` values (3 in act42side_level_act42side_06_beg alone), so a raw
 * lookup misses a sprite that resolved perfectly well. Found in the browser:
 * `avg_npc_371_1#1$1 ` was reported unresolved while `avg_npc_371_1#1$1` was
 * in the map.
 */
export function key(raw: string): string {
    return raw.trim();
}

export function slotOf(value: string | undefined): Slot | undefined {
    if (value === undefined) return undefined;
    return SLOT_ALIASES[value.trim().toLowerCase()];
}

/**
 * A blocker channel. The floats reach `DOTweenModuleUI.DOColor` on a `Graphic`,
 * where Unity's Color32 conversion CLAMPS at 1, so `r=128` renders FULL red and
 * `r=255` beside `a=1` is white with an opaque alpha. There is no divide by 255
 * anywhere in `_ExecuteBlocker`, and the two scales MIX inside one command.
 */
export function channel01(value: string | undefined, fallback: number): number {
    const n = num(value, Number.NaN);
    if (!Number.isFinite(n)) return fallback;
    return clamp01(n);
}

export function adaptOf(value: string | undefined): ScreenAdapt {
    const v = (value ?? "").trim().toLowerCase();
    return v === "coverall" ? "coverall" : v === "showall" ? "showall" : v === "fill" ? "fill" : "native";
}
