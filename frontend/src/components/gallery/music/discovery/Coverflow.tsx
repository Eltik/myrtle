import { DiscIcon } from "lucide-react";
import type React from "react";
import { memo, useCallback, useEffect, useLayoutEffect, useRef, useState } from "react";
import { useMediaQuery } from "#/hooks/use-media-query";
import { isEditableTarget } from "#/lib/hotkeys";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import { sizedCover } from "../impl/cover";
import type { IAlbum } from "../impl/model";
import type { messages } from "../MusicGallery.messages";
import { usePlayer } from "../player/engine";
import { JumpPalette } from "./JumpPalette";
import { BASE_COVER, CLICK_SLOP, clampIndex, coverPose, DRAG_PER_COVER, type ISample, KEY_FIRST_REPEAT_MS, MOMENTUM_DECAY, poseTransform, repeatDelay, rubberBand, STOP_SPEED, velocityOf, visibleRadius, WHEEL_FOLLOW, WHEEL_IDLE_MS, WHEEL_PER_COVER } from "./physics";

type MusicT = TypedT<typeof messages>;

/** Covers kept mounted (with their image) past the drawn radius, so a short swipe never shows a blank. */
const MOUNT_SLACK = 6;
/** Velocity samples kept during a drag or a wheel gesture. */
const MAX_SAMPLES = 24;
/** The snap ends when the position is this close to the target, covers. */
const SNAP_EPSILON = 0.0015;

type Mode = "idle" | "drag" | "wheel" | "coast" | "snap";

interface ISim {
    pos: number;
    vel: number;
    mode: Mode;
    target: number;
    raf: number;
    last: number;
    samples: ISample[];
    pointerId: number | null;
    startX: number;
    startPos: number;
    moved: number;
    wheelRaw: number;
    wheelTimer: ReturnType<typeof setTimeout> | null;
    /** A frame requested by an input event (drag, wheel) while the loop is idle; one write per frame however fast the events come. */
    pending: number;
}

/** What was last written to a cover, so a frame touches only what changed. */
interface IWritten {
    shown: boolean;
    transform: string;
    zIndex: string;
    shade: string;
}

/**
 * THE DISCOVERY COVERFLOW: every release, newest first, fanned in 3D around
 * the centre cover. The position is a float that one rAF loop moves (drag
 * momentum, wheel coasting, the snap to a whole cover) and stops once it
 * rests. Transforms are written to the DOM per frame; React only re-renders
 * when the centre cover changes, which moves the mounted window.
 *
 * A frame writes only compositor properties: the pose as a transform and the
 * side covers' dimming as the opacity of a black shade (a per-frame
 * `filter: brightness()` repainted every cover on every frame). Pointer and
 * wheel events only request a frame, so a 1 kHz mouse still costs one write
 * per display frame. The hover lift is CSS on an inner wrapper, so it eases
 * in and out and never fights the per-frame transform.
 */
export function Coverflow({ albums, onOpen }: { albums: readonly IAlbum[]; onOpen: (album: IAlbum) => void }): React.ReactElement {
    const t: MusicT = useT("gallery");
    const reduced = useMediaQuery("(prefers-reduced-motion: reduce)");
    const playingAlbum = usePlayer((s) => s.current?.albumId ?? null);
    const stageRef = useRef<HTMLDivElement>(null);
    const items = useRef(new Map<number, HTMLDivElement>());
    const written = useRef(new WeakMap<HTMLDivElement, IWritten>());
    const sim = useRef<ISim>({ pos: 0, vel: 0, mode: "idle", target: 0, raf: 0, last: 0, samples: [], pointerId: null, startX: 0, startPos: 0, moved: 0, wheelRaw: 0, wheelTimer: null, pending: 0 });
    const [centre, setCentre] = useState(0);
    const [layout, setLayout] = useState({ radius: 6, k: 1 });
    const [paletteOpen, setPaletteOpen] = useState(false);

    // The latest values the long-lived handlers read.
    const live = useRef({ albums, onOpen, reduced, layout, centre });
    live.current = { albums, onOpen, reduced, layout, centre };

    const max = Math.max(0, albums.length - 1);

    /** Write every mounted cover's transform for the current position. */
    const applyFrame = useCallback(() => {
        const s = sim.current;
        const { radius, k } = live.current.layout;
        for (const [idx, el] of items.current) {
            const pose = coverPose(idx, s.pos, k);
            let w = written.current.get(el);
            if (!w) {
                w = { shown: false, transform: "", zIndex: "", shade: "" };
                written.current.set(el, w);
            }
            const shown = pose.distance <= radius + 0.5;
            if (shown !== w.shown) {
                el.style.display = shown ? "" : "none";
                w.shown = shown;
            }
            if (!shown) continue;
            const transform = `translate(-50%, -50%) ${poseTransform(pose)}`;
            if (transform !== w.transform) {
                el.style.transform = transform;
                w.transform = transform;
            }
            const zIndex = String(pose.zIndex);
            if (zIndex !== w.zIndex) {
                el.style.zIndex = zIndex;
                w.zIndex = zIndex;
            }
            const shade = (1 - pose.brightness).toFixed(3);
            if (shade !== w.shade) {
                el.style.setProperty("--cf-shade", shade);
                w.shade = shade;
            }
        }
        const c = clampIndex(Math.round(s.pos), live.current.albums.length);
        if (c !== live.current.centre) {
            live.current.centre = c;
            setCentre(c);
        }
    }, []);

    const tick = useCallback(
        (now: number) => {
            const s = sim.current;
            const dt = Math.min(0.05, Math.max(0, (now - s.last) / 1000));
            s.last = now;
            const last = Math.max(0, live.current.albums.length - 1);
            if (s.mode === "wheel") {
                // The wheel moves a goal and the covers glide after it, so one notch is a
                // slide across frames, not a jump in one.
                s.pos += (rubberBand(s.wheelRaw, last) - s.pos) * Math.min(WHEEL_FOLLOW * dt, 1);
            }
            if (s.mode === "coast") {
                s.pos += s.vel * dt;
                s.vel *= MOMENTUM_DECAY ** dt;
                // Out of range springs back; slow enough settles on a cover.
                if (s.pos < 0 || s.pos > last || Math.abs(s.vel) < STOP_SPEED) {
                    s.mode = "snap";
                    s.target = clampIndex(Math.round(s.pos), last + 1);
                }
            }
            if (s.mode === "snap") {
                // Faster when what is left of the momentum already heads there.
                const heading = Math.sign(s.vel) === Math.sign(s.target - s.pos) ? 1.5 : 1;
                const step = Math.min(10 * dt * heading, 1);
                s.pos += (s.target - s.pos) * step;
                s.vel = 0;
                if (Math.abs(s.target - s.pos) < SNAP_EPSILON) {
                    s.pos = s.target;
                    s.mode = "idle";
                    // The wheel held the hover lift off while the covers moved under a still cursor.
                    if (stageRef.current) delete stageRef.current.dataset.dragging;
                }
            }
            applyFrame();
            s.raf = s.mode === "wheel" || s.mode === "coast" || s.mode === "snap" ? requestAnimationFrame(tick) : 0;
        },
        [applyFrame],
    );

    const run = useCallback(() => {
        const s = sim.current;
        if (s.raf) return;
        s.last = performance.now();
        s.raf = requestAnimationFrame(tick);
    }, [tick]);

    const halt = useCallback(() => {
        const s = sim.current;
        if (s.raf) cancelAnimationFrame(s.raf);
        s.raf = 0;
    }, []);

    /** One frame for the input events that arrive before the next paint; the running loop already draws every frame. */
    const requestFrame = useCallback(() => {
        const s = sim.current;
        if (s.raf || s.pending) return;
        s.pending = requestAnimationFrame(() => {
            s.pending = 0;
            applyFrame();
        });
    }, [applyFrame]);

    /** Settle on cover `index`: animated, or at once under reduced motion. */
    const goTo = useCallback(
        (index: number) => {
            const s = sim.current;
            s.target = clampIndex(index, live.current.albums.length);
            if (live.current.reduced) {
                halt();
                s.pos = s.target;
                s.vel = 0;
                s.mode = "idle";
                applyFrame();
                return;
            }
            s.mode = "snap";
            run();
        },
        [applyFrame, halt, run],
    );

    /** Let go with a velocity: momentum, or a direct snap under reduced motion. */
    const release = useCallback(
        (velocity: number) => {
            const s = sim.current;
            if (live.current.reduced || velocity === 0) {
                goTo(Math.round(s.pos));
                return;
            }
            s.vel = velocity;
            s.mode = "coast";
            run();
        },
        [goTo, run],
    );

    const step = useCallback(
        (dir: -1 | 1) => {
            const s = sim.current;
            const base = s.mode === "snap" ? s.target : Math.round(s.pos);
            goTo(base + dir);
        },
        [goTo],
    );

    // The album list changed (first load, a resync): keep the position in range.
    useEffect(() => {
        const s = sim.current;
        if (s.pos > max) {
            s.pos = max;
            s.target = max;
        }
        applyFrame();
    }, [max, applyFrame]);

    // Newly mounted covers get their pose before the browser paints.
    useLayoutEffect(() => {
        applyFrame();
    });

    // The stage's width and the cover size from its CSS variable.
    useEffect(() => {
        const el = stageRef.current;
        if (!el) return;
        const measure = () => {
            const cover = Number.parseFloat(getComputedStyle(el).getPropertyValue("--cf-cover")) || BASE_COVER;
            const k = cover / BASE_COVER;
            const radius = visibleRadius(el.clientWidth, k);
            setLayout((prev) => (prev.k === k && prev.radius === radius ? prev : { radius, k }));
        };
        measure();
        const ro = new ResizeObserver(measure);
        ro.observe(el);
        return () => ro.disconnect();
    }, []);

    // The wheel needs a non-passive listener to keep the page from scrolling.
    useEffect(() => {
        const el = stageRef.current;
        if (!el) return;
        const onWheel = (e: WheelEvent) => {
            const raw = Math.abs(e.deltaX) > Math.abs(e.deltaY) ? e.deltaX : e.deltaY;
            if (raw === 0) return;
            e.preventDefault();
            const delta = e.deltaMode === 1 ? raw * 40 : e.deltaMode === 2 ? raw * 800 : raw;
            const s = sim.current;
            const last = Math.max(0, live.current.albums.length - 1);
            if (live.current.reduced) {
                // No glide: each event lands where the wheel says, and the idle snap settles it.
                if (s.mode !== "wheel") {
                    halt();
                    s.mode = "wheel";
                    s.wheelRaw = s.pos;
                }
                s.wheelRaw += delta / WHEEL_PER_COVER;
                s.pos = rubberBand(s.wheelRaw, last);
                requestFrame();
            } else {
                if (s.mode !== "wheel") {
                    // A notch during a snap counts from where the snap is headed, so it is never swallowed.
                    s.wheelRaw = s.mode === "snap" ? s.target : s.pos;
                    s.mode = "wheel";
                    s.vel = 0;
                    el.dataset.dragging = "";
                }
                s.wheelRaw += delta / WHEEL_PER_COVER;
                run();
            }
            if (s.wheelTimer) clearTimeout(s.wheelTimer);
            // The wheel goal is already moving the covers, so the snap takes over from the
            // glide in place; there is no momentum to measure, and no pause to wait out.
            s.wheelTimer = setTimeout(() => {
                s.wheelTimer = null;
                if (s.mode !== "wheel") return;
                goTo(Math.round(s.wheelRaw));
            }, WHEEL_IDLE_MS);
        };
        el.addEventListener("wheel", onWheel, { passive: false });
        return () => {
            el.removeEventListener("wheel", onWheel);
            const timer = sim.current.wheelTimer;
            if (timer) clearTimeout(timer);
        };
    }, [requestFrame, halt, run, goTo]);

    // Arrow keys and Enter, document-level while Discovery is on screen.
    useEffect(() => {
        let held: { key: string; timer: ReturnType<typeof setTimeout> } | null = null;
        const stop = () => {
            if (held) clearTimeout(held.timer);
            held = null;
        };
        const blocked = (e: KeyboardEvent): boolean => {
            if (e.ctrlKey || e.metaKey || e.altKey || e.isComposing || e.defaultPrevented) return true;
            const active = document.activeElement;
            if (isEditableTarget(active)) return true;
            if (active instanceof HTMLElement && active.closest("[role=dialog],[role=alertdialog],[role=menu],[role=listbox],[role=slider]")) return true;
            return false;
        };
        const onKeyDown = (e: KeyboardEvent) => {
            if (blocked(e)) return;
            const stage = stageRef.current;
            const active = document.activeElement;
            const onStage = stage !== null && active instanceof Node && stage.contains(active);
            if (e.key === "Enter") {
                // Enter on a focused button belongs to the button.
                if (!onStage && active !== document.body && active !== null) return;
                const album = live.current.albums[live.current.centre];
                if (album) {
                    e.preventDefault();
                    live.current.onOpen(album);
                }
                return;
            }
            if (e.key !== "ArrowLeft" && e.key !== "ArrowRight") return;
            e.preventDefault();
            // The OS repeat is ignored; the hold runs on its own accelerating timer.
            if (e.repeat) return;
            stop();
            const dir = e.key === "ArrowLeft" ? -1 : 1;
            step(dir);
            const pressedAt = performance.now();
            const again = () => {
                step(dir);
                if (held) held.timer = setTimeout(again, repeatDelay(performance.now() - pressedAt));
            };
            held = { key: e.key, timer: setTimeout(again, KEY_FIRST_REPEAT_MS) };
        };
        const onKeyUp = (e: KeyboardEvent) => {
            if (held && e.key === held.key) stop();
        };
        document.addEventListener("keydown", onKeyDown);
        document.addEventListener("keyup", onKeyUp);
        window.addEventListener("blur", stop);
        return () => {
            stop();
            document.removeEventListener("keydown", onKeyDown);
            document.removeEventListener("keyup", onKeyUp);
            window.removeEventListener("blur", stop);
        };
    }, [step]);

    // The loop and any requested frame die with the component.
    useEffect(
        () => () => {
            halt();
            if (sim.current.pending) cancelAnimationFrame(sim.current.pending);
        },
        [halt],
    );

    const onPointerDown = (e: React.PointerEvent<HTMLDivElement>) => {
        if (e.button !== 0) return;
        const s = sim.current;
        halt();
        s.mode = "drag";
        s.pointerId = e.pointerId;
        s.startX = e.clientX;
        s.startPos = s.pos;
        s.moved = 0;
        s.samples = [{ t: performance.now(), pos: s.pos }];
        e.currentTarget.setPointerCapture(e.pointerId);
        e.currentTarget.dataset.dragging = "";
    };

    const onPointerMove = (e: React.PointerEvent<HTMLDivElement>) => {
        const s = sim.current;
        if (s.mode !== "drag" || s.pointerId !== e.pointerId) return;
        const dx = e.clientX - s.startX;
        s.moved = Math.max(s.moved, Math.abs(dx));
        s.pos = rubberBand(s.startPos - dx / (DRAG_PER_COVER * live.current.layout.k), Math.max(0, live.current.albums.length - 1));
        s.samples.push({ t: performance.now(), pos: s.pos });
        if (s.samples.length > MAX_SAMPLES) s.samples.shift();
        requestFrame();
    };

    const onPointerUp = (e: React.PointerEvent<HTMLDivElement>) => {
        const s = sim.current;
        if (s.mode !== "drag" || s.pointerId !== e.pointerId) return;
        s.pointerId = null;
        delete e.currentTarget.dataset.dragging;
        if (e.currentTarget.hasPointerCapture(e.pointerId)) e.currentTarget.releasePointerCapture(e.pointerId);
        if (s.moved < CLICK_SLOP) {
            // Pointer capture retargets the event to the stage, so the cover is found by position.
            const hit = document.elementFromPoint(e.clientX, e.clientY)?.closest<HTMLElement>("[data-cf-index]");
            const idx = hit ? Number(hit.dataset.cfIndex) : Number.NaN;
            const album = Number.isFinite(idx) ? live.current.albums[idx] : undefined;
            if (album && idx === Math.round(s.startPos)) {
                goTo(idx);
                live.current.onOpen(album);
            } else if (album) goTo(idx);
            else goTo(Math.round(s.pos));
            return;
        }
        release(velocityOf(s.samples, performance.now()));
    };

    const onPointerCancel = (e: React.PointerEvent<HTMLDivElement>) => {
        const s = sim.current;
        if (s.pointerId !== e.pointerId) return;
        s.pointerId = null;
        delete e.currentTarget.dataset.dragging;
        goTo(Math.round(s.pos));
    };

    const registerItem = useCallback((idx: number, el: HTMLDivElement | null) => {
        if (el) items.current.set(idx, el);
        else items.current.delete(idx);
    }, []);

    const span = layout.radius + MOUNT_SLACK;
    const from = Math.max(0, centre - span);
    const to = Math.min(max, centre + span);
    const mounted: number[] = [];
    if (albums.length > 0) for (let i = from; i <= to; i += 1) mounted.push(i);
    const current = albums[centre];

    return (
        <section aria-label={t("music.discovery.aria")}>
            {/* biome-ignore lint/a11y/useSemanticElements: a carousel is a focusable group, not a form fieldset */}
            <div
                ref={stageRef}
                // biome-ignore lint/a11y/noNoninteractiveTabindex: the stage takes focus so the arrow keys and Enter drive it
                tabIndex={0}
                role="group"
                aria-roledescription={t("music.discovery.roledescription")}
                aria-label={current ? t("music.discovery.current", { name: current.name, position: centre + 1, total: albums.length }) : t("music.discovery.aria")}
                onPointerDown={onPointerDown}
                onPointerMove={onPointerMove}
                onPointerUp={onPointerUp}
                onPointerCancel={onPointerCancel}
                className="relative h-[calc(var(--cf-cover)*1.58)] w-full touch-pan-y select-none overflow-hidden rounded-xl outline-none [--cf-cover:190px] focus-visible:ring-2 focus-visible:ring-ring/40 max-sm:[--cf-cover:128px]"
                style={{
                    perspective: "1200px",
                    perspectiveOrigin: "50% 42%",
                    background: "radial-gradient(ellipse 70% 90% at 50% 35%, color-mix(in oklab, var(--color-foreground) 9%, transparent), transparent 70%)",
                    maskImage: "linear-gradient(to right, transparent, #000 12%, #000 88%, transparent), linear-gradient(to bottom, transparent, #000 15%, #000 70%, transparent)",
                    maskComposite: "intersect",
                    WebkitMaskComposite: "source-in",
                }}
            >
                {mounted.map((idx) => {
                    const album = albums[idx];
                    if (!album) return null;
                    return <Cover key={album.id} index={idx} album={album} playing={album.id === playingAlbum} register={registerItem} />;
                })}
            </div>
            <div className="mt-1 flex min-h-11 flex-col items-center px-4 text-center" aria-live="polite">
                <span className="max-w-full truncate font-sans font-semibold text-[15px] text-foreground">{current?.name ?? ""}</span>
                {current ? <span className="font-mono text-[10.5px] text-primary/80 uppercase tracking-[0.14em]">{t("music.list.trackCount", { count: current.trackCount })}</span> : null}
            </div>
            <p className="mt-2 mb-0 text-center font-sans text-[11.5px] text-muted-foreground">
                <span className="pointer-coarse:hidden">
                    {t("music.discovery.hint.browse")}
                    <span aria-hidden="true"> · </span>
                    {t("music.discovery.hint.step")}
                    <span aria-hidden="true"> · </span>
                    {t("music.discovery.hint.open")}
                    <span aria-hidden="true"> · </span>
                </span>
                <button type="button" onClick={() => setPaletteOpen(true)} className="cursor-pointer text-primary underline-offset-2 hover:underline max-sm:min-h-11">
                    {t("music.discovery.jump")}
                </button>
            </p>
            <JumpPalette
                open={paletteOpen}
                onOpenChange={setPaletteOpen}
                albums={albums}
                onPick={(index) => {
                    setPaletteOpen(false);
                    goTo(index);
                }}
            />
        </section>
    );
}

/**
 * One cover with its reflection. Its pose is written by the coverflow, not by
 * React; `--cf-shade` (written per frame) dims it through two black shades,
 * one over the cover and one over the reflection. The images load eagerly:
 * only the window around the centre is mounted, and a hidden cover's lazy
 * image would not start until it is already on screen.
 */
const Cover = memo(function Cover({ index, album, playing, register }: { index: number; album: IAlbum; playing: boolean; register: (idx: number, el: HTMLDivElement | null) => void }): React.ReactElement {
    // The centre cover draws at 1.15 x 190 px, the largest pose.
    const cover = sizedCover(album.coverUrl, 220);
    const markLoaded = (img: HTMLImageElement | null) => {
        if (img?.complete && img.naturalWidth > 0) img.closest<HTMLElement>("[data-cf-index]")?.setAttribute("data-loaded", "");
    };
    return (
        <div ref={(el) => register(index, el)} data-cf-index={index} className="group/cf absolute top-[46%] left-1/2 size-(--cf-cover) cursor-pointer" style={{ display: "none" }}>
            <div className="relative size-full transition-transform duration-200 ease-out not-in-data-dragging:group-hover/cf:-translate-y-2 motion-reduce:transition-none">
                <div className={cn("relative size-full overflow-hidden rounded-[6px] bg-muted shadow-lg/20", playing && "ring-2 ring-primary")}>
                    <DiscIcon aria-hidden="true" className="absolute inset-0 m-auto size-1/3 text-muted-foreground/40" />
                    <img
                        referrerPolicy="no-referrer"
                        ref={markLoaded}
                        src={cover}
                        alt={album.name}
                        decoding="async"
                        draggable={false}
                        onLoad={(e) => e.currentTarget.closest<HTMLElement>("[data-cf-index]")?.setAttribute("data-loaded", "")}
                        className="relative size-full object-cover opacity-0 transition-opacity duration-200 ease-out group-data-loaded/cf:opacity-100"
                    />
                    <span aria-hidden="true" className="pointer-events-none absolute inset-0 bg-black opacity-[var(--cf-shade,0)]" />
                    <span aria-hidden="true" className="pointer-events-none absolute inset-0 rounded-[6px] opacity-0 ring-2 ring-primary/50 ring-inset transition-opacity duration-150 ease-out not-in-data-dragging:group-hover/cf:opacity-100" />
                </div>
                <div aria-hidden="true" className="pointer-events-none absolute top-full left-0 size-full -scale-y-100 overflow-hidden rounded-[6px] opacity-0 transition-opacity duration-200 ease-out group-data-loaded/cf:opacity-50" style={{ maskImage: "linear-gradient(to top, rgb(0 0 0 / .32), transparent 55%)" }}>
                    <img referrerPolicy="no-referrer" src={cover} alt="" decoding="async" draggable={false} className="size-full object-cover" />
                    <span className="absolute inset-0 bg-black opacity-[var(--cf-shade,0)]" />
                </div>
            </div>
        </div>
    );
});
