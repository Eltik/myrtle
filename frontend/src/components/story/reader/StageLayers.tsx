/**
 * THE SCENE BEHIND THE CHARACTERS: backgrounds, CG images, panel strips,
 * cut-ins, the curtain wipe, the interlude panels and the blurred extend fill.
 *
 * Every rule here was READ out of the client, not inferred: `screenadapt` at
 * `_LoadImage`, the crossfade at `_SwapImages`, the curtain's direction table,
 * the interlude's masked panel. The measurement that fixed each one stays with
 * the component it fixed. `StageSprites.tsx` is the other half of the picture
 * and `Stage.tsx` is the frame both are drawn into.
 */
import type React from "react";
import { asset } from "#/components/operators/detail/impl/assets";
import { CANVAS_H, CANVAS_W, cpx } from "#/lib/story/canvas";
import type { Curtain, Cutin, ImageLayer, InterludePanel, Overlay, PanelLayer } from "#/lib/story/scene";
import { cn } from "#/lib/utils";
import { Body, bodyPlate } from "./StageSprites";
import { fade, focusFilter, useOutgoing } from "./stageFx";

/**
 * `screenadapt` (`docs/story-reader-il2cpp-scene.md`, 4): `coverall` covers,
 * `showall` contains, `fill` stretches, and an absent or unrecognised value
 * skips the adapt block entirely. EN writes `coverall` 20,091 times and
 * `showall` 3,249 over 21,947 background and image commands that carry
 * arguments, so 1,856 of them (8.46%, 665 backgrounds and 1,191 images) take
 * the skipped arm.
 *
 * The skipped arm is now MEASURED (`docs/story-reader-captures.md`, 1). Both
 * readings the reader carried are refuted: `bg_cher_1` is drawn 1500.7 x 843.5
 * canvas px before its script's 1.1, where the prefab's own 1280x720 rect and
 * `SetNativeSize` at ppu 80 both compute 1280x720, out by 17.2%. What fits is
 * the sprite's own texture at its own pixels-per-unit against a reference 100:
 * 1024 x 576 at ppu 68.25 is 1500.4 x 844.0, a 0.02% and 0.06% match. That
 * pair rides the wire as `StoryAssets.imageSizes` and reaches this component as
 * `layer.nativeW`/`nativeH`. With no size on the wire the rect falls back to
 * the prefab's 1280x720, which is what shipped, and the engine counts the name
 * as `imageSize:missing`.
 *
 * The two adapting arms run against the CANVAS rect, which is the 16:9 box, so
 * `coverall` covers that box and `showall` contains inside it, exactly as a
 * CanvasScaler at `matchWidthOrHeight` 0 makes them against the mask.
 */
function AdaptedImage({ layer }: { layer: ImageLayer }): React.ReactElement {
    const fit = layer.adapt === "coverall" ? "cover" : layer.adapt === "showall" ? "contain" : "fill";
    const native = layer.adapt === "native";
    // The image is POSITIONED, not a grid item: as a grid item its `height: 100%`
    // resolved against an auto row that had already been sized from the image's
    // own 16:9 aspect, so at a 2.5:1 stage a `showall` background was contained
    // inside a 3440x1935 box and overflowed the 1376 px stage by 559 px instead
    // of fitting inside it. Against an absolutely positioned box both
    // percentages resolve against the plate, which IS the canvas.
    //
    // `max-w-none` is LOAD-BEARING and the reason the native rect was invisible
    // on the page while the inline `width` was already right: the preflight
    // ships `img { max-width: 100% }`, `max-width` is a different property from
    // `width` so an inline `width` never overrides it, and 100% resolves
    // against the plate, which is the 1280 canvas px box. A native plate WIDER
    // than the canvas was clamped back to exactly 1280 canvas px while its
    // height, which no rule caps, kept its own number: `bg_cher_1` rendered
    // 1280 x 844.0 instead of 1500.4 x 844.0.
    return <img src={asset(layer.url)} alt="" draggable={false} className="absolute block max-w-none select-none" style={{ left: "50%", top: "50%", translate: "-50% -50%", width: native ? cpx(layer.nativeW ?? CANVAS_W) : "100%", height: native ? cpx(layer.nativeH ?? CANVAS_H) : "100%", objectFit: fit }} />;
}

function LayerPlate({ layer, sec, kind, focus, ease, mode }: { layer: ImageLayer; sec: number; kind: "background" | "image"; focus?: number; ease?: string; mode: "in" | "hold" | "out" }): React.ReactElement {
    // `width`/`height` are MULTIPLIERS on the adapted size and `xScale`/`yScale`
    // a separate localScale channel; both are a scale about the centre pivot here.
    const sx = layer.xScale * layer.widthMul;
    const sy = layer.yScale * layer.heightMul;
    return (
        <div
            data-story-layer={kind}
            data-story-adapt={layer.adapt}
            className="absolute size-full transition-[transform,filter]"
            style={{
                left: cpx(layer.x),
                top: cpx(-layer.y),
                transform: `scale(${sx}, ${sy}) rotate(${layer.rotate}deg)`,
                transitionDuration: `${sec}s`,
                transitionTimingFunction: ease,
                filter: focusFilter(focus),
                ...(mode === "hold" ? {} : fade(mode, sec)),
            }}
        >
            <AdaptedImage layer={layer} />
        </div>
    );
}

/**
 * A background or CG channel. A new `[Image]` fades IN over the still-opaque
 * old one, which is a sibling reorder plus one `DOFade`, not a crossfade; a
 * bare `[Image]` fades the current one OUT.
 */
export function Layer({ layer, sec, kind, focus, ease }: { layer: ImageLayer | undefined; sec: number; kind: "background" | "image"; focus?: number; ease?: string }): React.ReactElement | null {
    const outgoing = useOutgoing(layer, layer?.url, sec);
    return (
        <>
            {outgoing ? <LayerPlate key={`out-${outgoing.id}`} layer={outgoing.item} sec={sec} kind={kind} focus={focus} mode={layer ? "hold" : "out"} /> : null}
            {layer?.url ? <LayerPlate key={layer.url} layer={layer} sec={sec} kind={kind} focus={focus} ease={ease} mode="in" /> : null}
        </>
    );
}

/**
 * `largebg` and `gridbg`: 2 or 4 panels butted into one surface. `x` moves the
 * CONTAINER in the same canvas pixels as every other offset, so
 * `solidwidth="920/920"` is an 1840 px strip and `x=-720` pans it 720 px left.
 * The strip's own left edge starts at the canvas left edge, which is
 * `_InitPositionUpperLeft`; whether `largebg` runs that or the default init is
 * a 280 px AMBIGUITY the binary does not settle, and one capture of a
 * `solidwidth="920/920"` panel would.
 */
export function Panels({ panels, sec, focus }: { panels: PanelLayer; sec: number; focus?: number }): React.ReactElement {
    const cols = panels.rows === 2 ? panels.urls.length / 2 : panels.urls.length;
    const totalW = panels.widths.slice(0, cols).reduce((a, b) => a + b, 0);
    const totalH = panels.height * panels.rows;
    return (
        <div
            data-story-panels={panels.rows === 2 ? "grid" : "strip"}
            className="absolute top-0 left-0 flex flex-wrap transition-[transform,filter] ease-out"
            style={{ width: cpx(totalW), height: cpx(totalH), transform: `translate(${cpx(panels.x)}, ${cpx(-panels.y)})`, transitionDuration: `${sec}s`, filter: focusFilter(focus) }}
        >
            {panels.urls.map((url, i) => (
                // biome-ignore lint/suspicious/noArrayIndexKey: a panel's place in the group IS its identity; the same url repeats.
                <img key={`${url}-${i}`} src={asset(url)} alt="" draggable={false} className="block size-full select-none object-cover" style={{ width: `${(100 * (panels.widths[i] ?? totalW / cols)) / totalW}%`, height: `${100 / panels.rows}%` }} />
            ))}
        </div>
    );
}

/**
 * `charactercutin`: a WINDOW onto the character standing as they would on
 * stage. The mask is `width` wide (200 in every EN use) and `height` tall
 * (never written; the full canvas), centred at the offsets AS WRITTEN
 * (negative x is screen-left; the corpus test is in the engine), and it fades
 * alpha 0 to 1 with no slide. Inside it the body is drawn exactly as `Sprite`
 * draws a slot: the prefab's `cutin_charslot` anchors bottom-centre in the
 * mask at (0,0), so the plate sits centred on the strip with its bottom edge
 * `plate.y - plate.h/2` from the canvas bottom, the figure at stage scale
 * with its knees below the mask. The old plate stretched the whole body into
 * the strip (`object-cover`), a small full-length figure where the game
 * shows the head and shoulders at the size of everyone else on stage.
 */
export function CutinPlate({ cutin, sec }: { cutin: Cutin; sec: number }): React.ReactElement {
    const plate = bodyPlate(cutin.sprite as { plate?: unknown });
    return (
        <div data-story-cutin={cutin.name} className="absolute overflow-hidden" style={{ left: `calc(50% + ${cpx(cutin.x)})`, top: `calc(50% - ${cpx(cutin.y)})`, width: cpx(cutin.width), height: cpx(cutin.height), transform: "translate(-50%, -50%)", ...fade("in", Math.max(0, sec)) }}>
            <img src={asset(cutin.sprite.bodyUrl)} alt={cutin.name} draggable={false} className="absolute max-w-none select-none" style={{ left: `calc(50% + ${cpx(plate.x)})`, bottom: cpx(plate.y - plate.h / 2), width: cpx(plate.w), height: cpx(plate.h), transform: "translateX(-50%)" }} />
        </div>
    );
}

/**
 * `curtain`: a solid fill covering `fill` of the canvas from one edge. The
 * AXIS is read: `w8 = 1 << direction` against `tst w8, #0xbb` puts directions 2
 * and 6 on WIDTH and 0, 1, 3, 4, 5 and 7 on HEIGHT. The two WIDTH edges are now
 * MEASURED (`docs/story-reader-captures.md`, 4): at `direction=6` the visible
 * columns inside the mask run 0..1919, 1472..1919, 1677..1919, 1813..1919,
 * 1890..1919, so black grows from the LEFT; at `direction=2` they run 0..1919,
 * 0..1584, 0..1259, 0..447, 0..106, 0..29, so black grows from the RIGHT. Both
 * match what this table already shipped, so nothing moves.
 *
 * 0 = top and 4 = bottom stay the reader's compass guess: the corpus always
 * issues them as a PAIR on one halt (`main_14-17_end` lines 7 and 8,
 * `act33side_10_beg` lines 95 and 96), so a capture shows two bands and cannot
 * say which is which, and the guess is invisible unless their fill fractions
 * differ. EN uses 0 (338), 4 (332), 6 (178) and 2 (175), then 1/5/7/3 at
 * 6/5/2/2, so 13 uses ride on the odd fallbacks.
 */
const CURTAIN_EDGE: Record<number, "top" | "right" | "bottom" | "left"> = { 0: "top", 1: "top", 2: "right", 3: "bottom", 4: "bottom", 5: "bottom", 6: "left", 7: "top" };

export function CurtainFill({ curtain, sec }: { curtain: Curtain; sec: number }): React.ReactElement {
    const edge = CURTAIN_EDGE[curtain.direction] ?? "top";
    const pct = `${Math.round(curtain.fill * 100)}%`;
    const vertical = edge === "top" || edge === "bottom";
    return (
        <div
            data-story-curtain={edge}
            className="pointer-events-none absolute bg-black transition-[width,height] ease-linear"
            style={{
                [edge]: 0,
                width: vertical ? "100%" : pct,
                height: vertical ? pct : "100%",
                left: edge === "right" ? undefined : 0,
                top: edge === "bottom" ? undefined : 0,
                transitionDuration: `${sec}s`,
                backgroundImage: curtain.grad ? `linear-gradient(to ${edge}, rgba(0,0,0,0) 0%, #000 45%)` : undefined,
            }}
        />
    );
}

/**
 * `interlude`: the radio-call WINDOW (`InterludePanel` in `scene.ts` has the
 * read). The panel is a `frame` rect centred at the offset, scaled by the
 * window's `tsfrom`/`tsto` about its centre, with a dark back and a
 * `content` rect inside it that CLIPS the background and the figure. The
 * figure is placed in CANVAS coordinates, exactly as `Sprite` places a slot
 * (origin at the canvas bottom, the sprite's own plate above it), so the
 * window shows the part of the standing figure that falls inside it: in 15-13
 * Raidian's `pto -180,50` puts her face inside a square at `-200,170`.
 *
 * Not modelled, with what would settle each: the deco view (the wave that
 * `switch` toggles, `group_interclude_*_deco`) is exposed as
 * `data-story-speaking` only, because its placement against the window is a
 * separate prefab root the tree dump does not relate to the mask; the back and
 * gradient COLOURS are the prefab's `back_shadow` and `bg_defaul_gradient`
 * sprites, which were not exported, so the tones here are ours; the empty
 * window's `img_unknown` "UNKNOWN" plate is not drawn. One capture of a
 * two-way call (15-04 halt 211) decides all three.
 */
export function InterludePanels({ panels, sec }: { panels: Record<string, InterludePanel>; sec: number }): React.ReactElement | null {
    const rows = Object.entries(panels);
    if (rows.length === 0) return null;
    return <>{rows.map(([channel, p]) => (p.legacy ? <LegacyInterlude key={channel} channel={channel} legacy={p.legacy} sec={sec} /> : <InterludeWindow key={channel} channel={channel} panel={p} sec={sec} />))}</>;
}

function LegacyInterlude({ channel, legacy, sec }: { channel: string; legacy: NonNullable<InterludePanel["legacy"]>; sec: number }): React.ReactElement {
    return (
        <img
            src={asset(legacy.url)}
            alt=""
            draggable={false}
            data-story-interlude={channel}
            className="absolute inset-0 size-full select-none object-contain transition-transform ease-out"
            style={{ transform: `translate(${cpx(legacy.x)}, ${cpx(-legacy.y)})`, transitionDuration: `${Math.max(0.2, sec)}s`, ...fade("in", Math.max(0.2, sec)) }}
        />
    );
}

function InterludeWindow({ channel, panel: p, sec }: { channel: string; panel: InterludePanel; sec: number }): React.ReactElement {
    const framed = p.frameW !== p.contentW || p.frameH !== p.contentH;
    // The content rect's centre IS the window centre, so a point at canvas
    // (cx, cy) with y up sits at (cw/2 + cx - x, ch/2 - (cy - y)) inside it.
    const ch = p.character;
    const plate = ch ? bodyPlate(ch.sprite as { plate?: unknown }) : null;
    const img = p.image;
    const move = { transitionProperty: "left, top, transform, opacity", transitionDuration: `${sec}s`, transitionTimingFunction: "ease-out" };
    return (
        <div
            data-story-interlude={channel}
            data-story-mask={p.maskId}
            data-story-speaking={p.speaking ? "true" : "false"}
            className="pointer-events-none absolute transition-transform ease-out"
            style={{ left: `calc(50% + ${cpx(p.x)})`, top: `calc(50% - ${cpx(p.y)})`, width: cpx(p.frameW), height: cpx(p.frameH), transform: `translate(-50%, -50%) scale(${p.scaleX}, ${p.scaleY})`, transitionDuration: `${sec}s` }}
        >
            {framed ? <div className="absolute inset-0 bg-black/60 shadow-[0_0_24px_rgba(0,0,0,.5)]" /> : null}
            <div className="absolute overflow-hidden bg-linear-to-b from-[#3b4049] to-[#121418]" style={{ left: "50%", top: "50%", width: cpx(p.contentW), height: cpx(p.contentH), transform: "translate(-50%, -50%)" }}>
                {p.bg ? <img src={asset(p.bg.url)} alt="" draggable={false} data-story-interlude-bg={p.bg.name} className="absolute inset-0 size-full max-w-none select-none object-cover" /> : null}
                {img ? (
                    <img
                        src={asset(img.url)}
                        alt=""
                        draggable={false}
                        data-story-interlude-image={img.name}
                        className="absolute max-w-none select-none"
                        style={{ left: cpx(p.contentW / 2 + img.x - p.x), top: cpx(p.contentH / 2 - (img.y - p.y)), width: cpx(img.w ?? CANVAS_W), height: cpx(img.h ?? CANVAS_H), transform: `translate(-50%, -50%) scale(${img.scaleX}, ${img.scaleY})`, opacity: img.alpha, ...move }}
                    />
                ) : null}
                {ch && plate ? (
                    <div
                        data-story-interlude-char={ch.name}
                        className="absolute"
                        style={{
                            left: cpx(p.contentW / 2 + ch.x + plate.x - p.x - plate.w / 2),
                            top: cpx(p.contentH / 2 - (-CANVAS_H / 2 + ch.y + plate.y - p.y) - plate.h / 2),
                            width: cpx(plate.w),
                            height: cpx(plate.h),
                            transform: `scale(${ch.scaleX}, ${ch.scaleY})`,
                            opacity: ch.alpha,
                            ...move,
                        }}
                    >
                        <Body key={ch.name} state={{ sprite: ch.sprite, name: ch.name }} sec={sec} mode="hold" />
                    </div>
                ) : null}
            </div>
            {p.label ? (
                // `group_name`: anchored (-145.3, -85.19) from the 300 px root's
                // centre with pivot (0, 0.5), 23 px tall, the lower-left of the square.
                <div className="absolute whitespace-nowrap bg-black/80 px-[0.4em] font-medium text-white" style={{ left: `calc(50% + ${cpx(-145.3 * (p.w / 300))})`, top: `calc(50% + ${cpx(85.19 * (p.h / 300))})`, height: cpx(23), lineHeight: cpx(23), fontSize: cpx(15), transform: "translateY(-50%)" }}>
                    {p.label}
                </div>
            ) : null}
        </div>
    );
}

/**
 * A face patch. `facePos` is the patch's TOP-LEFT corner in BODY TEXTURE
 * pixels and `w`/`h` the on-body size the 128 px face texture is scaled into
 * (Haak: 512,120,51,71). Body texture pixels are NOT canvas pixels: the plate
 * is 1024 canvas px whatever the texture measures, so the fraction of the
 * plate is `facePos / bodySize` and Haak's 1024 body was right only by
 * coincidence. Read defensively: the running backend predates the binding and
 * sends no `facePos` at all, and the legacy hubs write the sentinels (-1,-1)
 * and (0,0).
 */

export function OverlayText({ overlay, kind }: { overlay: Overlay; kind: "subtitle" | "sticker" }): React.ReactElement {
    return (
        <div
            data-story-overlay={kind}
            className={cn("absolute whitespace-pre-wrap text-white/95 [text-shadow:0_1px_2px_rgba(0,0,0,.9),0_0_12px_rgba(0,0,0,.6)]", kind === "subtitle" && "font-medium")}
            // LINE HEIGHT IS 1.25 OF THE SIZE, read off the scripts rather than
            // chosen: `act51side_st01` stacks three 800 px stickers at size 24 on
            // a 120 px stride (y 150, 270, 390) and its first block is 233
            // characters, four lines in the game, so a line box is 30 px. The
            // `leading-relaxed` this carried (1.625, 39 px) made the block 156 px
            // and drew it over the sticker below.
            style={{ left: cpx(overlay.x), top: cpx(overlay.y), width: cpx(overlay.width), fontSize: cpx(overlay.size), lineHeight: 1.25, textAlign: overlay.alignment, ...fade("in", 0.25) }}
        >
            {overlay.text}
        </div>
    );
}
