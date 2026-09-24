/**
 * THE CHARACTERS ON STAGE: the body plate, the face swap inside it, the slot's
 * own transform, and the DOTween actions that shake, jump, move or zoom it.
 *
 * The three readers at the top (`facePlacement`, `bodyTextureSize`,
 * `bodyPlate`) are pure and defensive: the wire fields they read are newer than
 * the running backend can be, so an absent one falls back to the slot template
 * rather than throwing inside a render. They are exported so the geometry can
 * be driven without a DOM; nothing outside this file imports them today, and
 * `stage.test.tsx` reaches them through a rendered `Stage`.
 */
import type React from "react";
import { useRef, useState } from "react";
import { asset } from "#/components/operators/detail/impl/assets";
import { BODY_PLATE, type BodyPlate, cpx, SLOT_X } from "#/lib/story/canvas";
import type { Slot, SlotState } from "#/lib/story/scene";
import { cn } from "#/lib/utils";
import { DIM_FILTER, fade, focusFilter, joinFilters, useOutgoing } from "./stageFx";

interface FacePlacement {
    x: number;
    y: number;
    w: number;
    h: number;
}

export function facePlacement(sprite: { faceUrl?: string; facePos?: unknown }): FacePlacement | null {
    if (!sprite.faceUrl) return null;
    const raw = sprite.facePos as { x?: unknown; y?: unknown; w?: unknown; h?: unknown } | undefined | null;
    if (!raw || typeof raw !== "object") return null;
    const n = (v: unknown): number => (typeof v === "number" && Number.isFinite(v) ? v : Number.NaN);
    const x = n(raw.x);
    const y = n(raw.y);
    const w = n(raw.w);
    const h = n(raw.h);
    if (!Number.isFinite(x) || !Number.isFinite(y) || x < 0 || y < 0 || !(w > 0) || !(h > 0)) return null;
    return { x, y, w, h };
}

/** The body TEXTURE's size in pixels, as the wire carries it. */
interface BodyTextureSize {
    w: number;
    h: number;
}

export function bodyTextureSize(sprite: { bodySize?: unknown }): BodyTextureSize | null {
    const raw = sprite.bodySize as { w?: unknown; h?: unknown } | undefined | null;
    if (!raw || typeof raw !== "object") return null;
    const n = (v: unknown): number => (typeof v === "number" && Number.isFinite(v) ? v : Number.NaN);
    const w = n(raw.w);
    const h = n(raw.h);
    if (!(w > 0) || !(h > 0)) return null;
    return { w, h };
}

/**
 * The body PLATE: the rect the sprite is drawn into, canvas px, centred
 * `x`,`y` above the slot origin at the canvas bottom with y up.
 *
 * MEASURED (`docs/story-reader-captures.md`, 2 and 3): three characters in two
 * frames come out 1091.7, 958.9 and 1094.5 canvas px, which no shared template
 * can produce, and Amiya's 1091.7 matches her own bundle's root RectTransform
 * of 1090 to 0.16%. The character's OWN prefab rect wins. It rides the wire as
 * `CharacterSprite.plate` and is read DEFENSIVELY, because the running backend
 * predates the binding: without it the slot template 1024 at (0,203) is what
 * shipped, and `?plate=0` forces that arm for an A/B.
 */
export function bodyPlate(sprite: { plate?: unknown }, fromWire = true): BodyPlate {
    if (!fromWire) return BODY_PLATE;
    const raw = sprite.plate as { x?: unknown; y?: unknown; w?: unknown; h?: unknown } | undefined | null;
    if (!raw || typeof raw !== "object") return BODY_PLATE;
    const n = (v: unknown): number => (typeof v === "number" && Number.isFinite(v) ? v : Number.NaN);
    const x = n(raw.x);
    const y = n(raw.y);
    const w = n(raw.w);
    const h = n(raw.h);
    if (!Number.isFinite(x) || !Number.isFinite(y) || !(w > 0) || !(h > 0)) return BODY_PLATE;
    return { x, y, w, h };
}

function Body({ state, sec, mode }: { state: SlotState; sec: number; mode: "in" | "hold" | "out" }): React.ReactElement {
    const face = facePlacement(state.sprite);
    const declared = bodyTextureSize(state.sprite);
    // A backend that predates `bodySize` sends none, so the texture is
    // measured off the <img> itself once it has decoded. The fallback is
    // NEVER 1024: that constant is the canvas PLATE, and reading it as the
    // texture put Amiya's patch 130 px right of and 50 px below her face on a
    // 1280 px body. Without a size there is no placement, so the patch waits
    // for the body rather than landing in the wrong place for a frame.
    const [measured, setMeasured] = useState<BodyTextureSize | null>(null);
    const size = declared ?? measured;
    // Both a ref and `onLoad`: a body already in the memory cache can finish
    // before React attaches the handler, and then only `complete` says so.
    const measure = (img: HTMLImageElement | null): void => {
        if (!img || declared || !(img.naturalWidth > 0) || !(img.naturalHeight > 0)) return;
        const next = { w: img.naturalWidth, h: img.naturalHeight };
        setMeasured((prev) => (prev && prev.w === next.w && prev.h === next.h ? prev : next));
    };
    return (
        <div className="absolute inset-0" style={mode === "hold" ? undefined : fade(mode, sec)}>
            <img ref={measure} onLoad={(e) => measure(e.currentTarget)} src={asset(state.sprite.bodyUrl)} alt={state.name} draggable={false} data-story-sprite={state.name} className="absolute inset-0 size-full select-none" />
            {face && size && state.sprite.faceUrl ? (
                <img src={asset(state.sprite.faceUrl)} alt="" draggable={false} data-story-face={state.name} className="absolute select-none" style={{ left: `${(face.x / size.w) * 100}%`, top: `${(face.y / size.h) * 100}%`, width: `${(face.w / size.w) * 100}%`, height: `${(face.h / size.h) * 100}%` }} />
            ) : null}
        </div>
    );
}

/**
 * One character slot. The prefab puts `l` at canvas x -200, `m` at 0 and `r`
 * at +200, anchored at the canvas BOTTOM; the sprite's own plate is centred
 * `plate.x`,`plate.y` above that origin, so its bottom edge sits
 * `plate.y - plate.h/2` from the canvas bottom, which for the 1024-at-203
 * template is 309 px BELOW it and the figure is cropped at the knees. The
 * lighting multiply is applied INSTANTLY on a new sprite and tweened over the
 * scaled duration when the same sprite changes focus.
 */
export function Sprite({ slot, state, sec, focus, plateFromWire }: { slot: Slot; state: SlotState; sec: number; focus?: number; plateFromWire: boolean }): React.ReactElement {
    // The wire field is not in the generated binding yet, so the cast is
    // where the backend agent's `plate` lands; `bodyPlate` validates it.
    const plate = bodyPlate(state.sprite as { plate?: unknown }, plateFromWire);
    const outgoing = useOutgoing(state, `${state.swap}`, sec);
    // -1 is "not here a frame ago", so the first render of a slot applies the
    // multiply instantly, which is what `SetFocus` does on a new sprite.
    const litRef = useRef<{ swap: number; lit: boolean }>({ swap: -1, lit: state.lit });
    // A render-time ref alone is not enough: the outgoing sprite arrives on a
    // second render, which would have already spent the "fresh" flag. A slot
    // that is crossfading is a new sprite by definition, so either says instant.
    const fresh = litRef.current.swap !== state.swap || outgoing !== null;
    litRef.current = { swap: state.swap, lit: state.lit };
    const action = state.action;
    return (
        <div
            data-story-slot={slot}
            data-story-lit={state.lit ? "true" : "false"}
            data-story-action={action?.kind}
            className="absolute transition-[left,bottom,transform,filter,opacity]"
            style={{
                left: `calc(50% + ${cpx(SLOT_X[slot] + state.x + plate.x)})`,
                bottom: cpx(state.y + plate.y - plate.h / 2),
                width: cpx(plate.w),
                height: cpx(plate.h),
                transform: `translateX(-50%) scale(${state.scale})`,
                transformOrigin: `${state.pivotX * 100}% ${(1 - state.pivotY) * 100}%`,
                opacity: state.alpha,
                // The multiply is instant on a sprite that was not here a frame
                // ago and tweened when only the focus moved.
                filter: joinFilters(state.lit ? undefined : DIM_FILTER, focusFilter(focus)),
                transitionDuration: `${fresh ? 0 : sec}s`,
            }}
        >
            {/* The one-shot action rides its own layer, keyed by the command's
                sequence, so replaying a jump cannot remount the sprite under it. */}
            <div key={action ? action.seq : 0} className={cn("absolute inset-0", action?.kind === "jump" && "story-jump", action?.kind === "shake" && "story-shake")} style={{ animationDuration: action ? `${Math.max(0.12, action.sec)}s` : undefined }}>
                {outgoing ? <Body key={`out-${outgoing.id}`} state={outgoing.item} sec={sec} mode="out" /> : null}
                <Body key={state.swap} state={state} sec={sec} mode={outgoing ? "in" : "hold"} />
            </div>
        </div>
    );
}

/**
 * A subtitle or a sticker. The origin is the canvas TOP-LEFT with y DOWNWARD,
 * `width` defaults to 1280 clamped to `1280 - x`, and `size` is a raw point
 * size at canvas scale. There is NO per-character reveal on these surfaces:
 * the executor fetches six arguments and `delay` is not one of them.
 */
