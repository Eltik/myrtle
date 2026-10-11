/**
 * Map labels at a constant screen size: each label's box is placed in screen
 * pixels from its world anchor, kept inside the stage, and the labels that
 * would overlap a more important one are hidden. Sizes are measured from the
 * rendered labels (never estimated from character counts), so long CJK and
 * Hangul names take the room they need.
 */

import type { IBox, IPoint } from "./geometry";

/** A view of the world: the visible world box and the stage size in pixels. */
export interface IView {
    box: IBox;
    width: number;
    height: number;
}

export interface ILabelBox {
    id: string;
    x: number;
    y: number;
    w: number;
    h: number;
}

/** Screen position of a world point. */
export function project(p: IPoint, view: IView): IPoint {
    return { x: ((p.x - view.box.x) / view.box.w) * view.width, y: ((p.y - view.box.y) / view.box.h) * view.height };
}

/**
 * The box of a `w` by `h` label whose left edge sits at `anchor` and which is
 * centred on it vertically (the game's own card placement), moved the least
 * needed to stay `margin` inside the stage. `null` when the anchor is off the
 * stage: a label is never pulled in for a cluster that isn't in view.
 */
export function placeLabel(id: string, anchor: IPoint, w: number, h: number, view: IView, margin: number): ILabelBox | null {
    if (anchor.x < 0 || anchor.y < 0 || anchor.x > view.width || anchor.y > view.height) return null;
    const x = Math.max(margin, Math.min(anchor.x, view.width - w - margin));
    const y = Math.max(margin, Math.min(anchor.y - h / 2, view.height - h - margin));
    return { id, x, y, w, h };
}

function overlaps(a: ILabelBox, b: ILabelBox, gap: number): boolean {
    return a.x < b.x + b.w + gap && b.x < a.x + a.w + gap && a.y < b.y + b.h + gap && b.y < a.y + a.h + gap;
}

/**
 * The ids to show: labels are taken in `priority` order (highest first) and
 * one that comes within `gap` pixels of a label already taken is hidden.
 * `always` is taken first whatever its priority (the hovered cluster).
 */
export function declutter(boxes: readonly ILabelBox[], priority: (id: string) => number, gap: number, always?: string | null): Set<string> {
    const order = [...boxes].sort((a, b) => Number(b.id === always) - Number(a.id === always) || priority(b.id) - priority(a.id) || a.id.localeCompare(b.id));
    const taken: ILabelBox[] = [];
    for (const box of order) {
        if (box.id === always || !taken.some((t) => overlaps(t, box, gap))) taken.push(box);
    }
    return new Set(taken.map((b) => b.id));
}
