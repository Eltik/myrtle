/**
 * DOTWEEN'S `Ease` ENUM AS CSS TIMING FUNCTIONS, which is the whole file.
 *
 * It sits outside `engine.ts` because it is a lookup table with one function
 * over it and nothing in the interpreter's switch needs to be read to trust it.
 */

/**
 * DOTween's `Ease` enum, which is what `ease="1"` names: `_ExecuteImageRotate`
 * passes `mov w1,#1` into the same `SetEase`, pinning Unset 0, Linear 1. The
 * cubic-beziers are the standard approximations of each curve; DOTween's own
 * defaults for Unset come from `DOTween.defaultEaseType`, which is not
 * readable, so Unset falls through to the renderer's `ease-out`.
 */
const DOTWEEN_EASE: Record<number, string> = {
    1: "linear",
    2: "cubic-bezier(0.12, 0, 0.39, 0)",
    3: "cubic-bezier(0.61, 1, 0.88, 1)",
    4: "cubic-bezier(0.37, 0, 0.63, 1)",
    5: "cubic-bezier(0.11, 0, 0.5, 0)",
    6: "cubic-bezier(0.5, 1, 0.89, 1)",
    7: "cubic-bezier(0.45, 0, 0.55, 1)",
    8: "cubic-bezier(0.32, 0, 0.67, 0)",
    9: "cubic-bezier(0.33, 1, 0.68, 1)",
    10: "cubic-bezier(0.65, 0, 0.35, 1)",
};

export const EASE_LINEAR = DOTWEEN_EASE[1];
export const EASE_OUT_CUBIC = DOTWEEN_EASE[9];

/** `ease` written by name: `OutQuad` 99, `InOutCubic` 5, `InOutSine` 4, `OutSine` 3, `OutFlash` 2, `inSine` 1. */
const EASE_NAMES: Record<string, string> = {
    linear: DOTWEEN_EASE[1],
    insine: DOTWEEN_EASE[2],
    outsine: DOTWEEN_EASE[3],
    inoutsine: DOTWEEN_EASE[4],
    inquad: DOTWEEN_EASE[5],
    outquad: DOTWEEN_EASE[6],
    inoutquad: DOTWEEN_EASE[7],
    incubic: DOTWEEN_EASE[8],
    outcubic: DOTWEEN_EASE[9],
    inoutcubic: DOTWEEN_EASE[10],
    outflash: "cubic-bezier(0.22, 1, 0.36, 1)",
};

/** A tween's `ease`, either a DOTween enum ordinal or one of the names above. */
export function easeOf(value: string | undefined): string | undefined {
    const raw = (value ?? "").trim();
    if (raw === "") return undefined;
    const n = Number(raw);
    if (Number.isInteger(n)) return DOTWEEN_EASE[n];
    return EASE_NAMES[raw.toLowerCase()];
}
