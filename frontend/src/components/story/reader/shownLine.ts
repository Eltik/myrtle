/**
 * WHICH LINE THE TEXT BOX PRINTS. It is not always the halt the reader is on.
 *
 * When a line is followed by a hide-and-cut (`[dialog]`, the characters swap,
 * the next `[name]` brings the box back), the player already holds the NEXT
 * halt while the box is fading out over the OLD scene. Printing that halt at
 * once drew the next speaker's plate, over an emptied box, for the fade's
 * 200 ms: the flash the 2026-09-25 recording of `act51side_st01` shows, an
 * "Ursine Police" plate popping into the narration box before the cut. The
 * client fades the box out with what it had and only shows the new name when
 * the box returns.
 *
 * So the box prints the current halt from the first frame in which the scene
 * shows the box for it, and holds the previous line until then. A step whose
 * box never hides (ordinary dialogue) switches on its first frame, as it did.
 * Once switched, a line is LATCHED on its reveal key: a later frame that hides
 * the box again within the same step cannot bring the old line back.
 */

export interface ShownLine {
    speaker?: string;
    text: string;
    isNarration: boolean;
    /** The reveal key of the halt this line came from: what the box's typewriter is stamped with. */
    revealKey: number;
}

export interface CurrentLine {
    speaker?: string;
    text: string;
    isNarration: boolean;
}

/**
 * The line to print now. `prev` is what was printed on the last render,
 * `current` the halt the player is on, `boxVisible` the scene's dialog flag
 * in the frame being shown.
 */
export function nextShownLine(prev: ShownLine | null, current: CurrentLine, revealKey: number, boxVisible: boolean): ShownLine {
    if (prev !== null && prev.revealKey === revealKey) return prev;
    if (prev === null || boxVisible) return { speaker: current.speaker, text: current.text, isNarration: current.isNarration, revealKey };
    return prev;
}
