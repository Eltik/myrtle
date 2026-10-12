/**
 * THE COVERFLOW'S LAW, as pure numbers so it can be tested without a DOM.
 *
 * Every pixel constant is the reference's at a 190 px cover and scales with
 * the cover size `k = cover / 190`, so the phone layout (a smaller cover from
 * the CSS variable) keeps the same fan. Angles, scales and brightness do not
 * scale.
 */
export const BASE_COVER = 190;
/** The first neighbour's offset from centre, px at a 190 px cover. */
const FIRST_OFFSET = 150;
/** Each further neighbour's extra offset, px. Under a cover's width, so the sides overlap like a fanned stack. */
const STEP_OFFSET = 92;
const CENTRE_Z = 60;
const Z_DROP = 210;
const SIDE_ANGLE = 58;
const CENTRE_SCALE = 1.15;
const SCALE_DROP = 0.35;
/** Drag distance per cover, px at a 190 px cover. */
export const DRAG_PER_COVER = 120;
/** Wheel `deltaY` per cover. */
export const WHEEL_PER_COVER = 120;
/** How far past either end a drag or wheel travels, per unit of overscroll. */
export const RUBBER_BAND = 0.35;
/** Velocity samples older than this are ignored on release, ms. */
export const VELOCITY_WINDOW_MS = 150;
/** Momentum keeps `MOMENTUM_DECAY ** dt` of its speed per second. */
export const MOMENTUM_DECAY = 0.08;
/** Below this speed momentum ends and the snap begins, covers per second. */
export const STOP_SPEED = 0.05;
/** A pointer that moved less than this is a click, px. */
export const CLICK_SLOP = 10;
/** The wheel snaps to a cover once no event arrived for this long, ms. Short, because the covers keep gliding meanwhile. */
export const WHEEL_IDLE_MS = 60;
/** How fast the covers close on the wheel's goal: this share of the gap per second, like the snap's. */
export const WHEEL_FOLLOW = 10;
/** Held arrow keys: the first repeat after this, ms. */
export const KEY_FIRST_REPEAT_MS = 280;
/** Held arrow keys: the fastest repeat, ms. */
export const KEY_MIN_REPEAT_MS = 60;

export interface ICoverPose {
    x: number;
    z: number;
    rotateY: number;
    scale: number;
    brightness: number;
    zIndex: number;
    /** Distance from the centre in covers. */
    distance: number;
}

/** Where cover `index` sits when the float position is `pos`. */
export function coverPose(index: number, pos: number, k = 1): ICoverPose {
    const d = index - pos;
    const s = Math.abs(d);
    const sign = Math.sign(d);
    const a = Math.min(s, 1);
    const x = sign * (s <= 1 ? FIRST_OFFSET * s : FIRST_OFFSET + STEP_OFFSET * (s - 1)) * k;
    return {
        x,
        z: (CENTRE_Z - Z_DROP * a) * k,
        rotateY: -SIDE_ANGLE * sign * a,
        scale: CENTRE_SCALE - SCALE_DROP * a,
        brightness: s <= 1 ? 1 - 0.45 * a : Math.max(0.35, 0.55 - 0.04 * (s - 1)),
        zIndex: Math.round(1000 - 10 * s),
        distance: s,
    };
}

export function poseTransform(p: ICoverPose, lift = 0): string {
    return `translateX(${p.x.toFixed(2)}px) translateY(${lift}px) translateZ(${p.z.toFixed(2)}px) rotateY(${p.rotateY.toFixed(2)}deg) scale(${p.scale.toFixed(4)})`;
}

/** Covers drawn on each side of the centre for a stage `width` px wide, between 3 and 20. */
export function visibleRadius(width: number, k = 1): number {
    const n = Math.ceil(((1350 * (width / 2)) / 1200 - FIRST_OFFSET * k) / (STEP_OFFSET * k)) + 2;
    return Math.min(20, Math.max(3, Number.isFinite(n) ? n : 3));
}

/** A raw position pulled back past the ends: the overscroll counts at `RUBBER_BAND`. */
export function rubberBand(raw: number, max: number): number {
    if (raw < 0) return raw * RUBBER_BAND;
    if (raw > max) return max + (raw - max) * RUBBER_BAND;
    return raw;
}

export interface ISample {
    t: number;
    pos: number;
}

/** Covers per second from the samples inside the window ending at `now`. */
export function velocityOf(samples: readonly ISample[], now: number): number {
    const recent = samples.filter((s) => now - s.t <= VELOCITY_WINDOW_MS);
    if (recent.length < 2) return 0;
    const first = recent[0];
    const last = recent[recent.length - 1];
    if (!first || !last) return 0;
    const dt = (last.t - first.t) / 1000;
    return dt > 0 ? (last.pos - first.pos) / dt : 0;
}

/** The delay before the next auto-repeat of a held arrow key, `elapsed` ms after the press. */
export function repeatDelay(elapsed: number): number {
    return Math.max(KEY_MIN_REPEAT_MS, KEY_FIRST_REPEAT_MS - 0.5 * elapsed);
}

export function clampIndex(i: number, count: number): number {
    return Math.min(Math.max(0, i), Math.max(0, count - 1));
}
