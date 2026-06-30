import * as PIXI from "pixi.js";
import type { Spine } from "pixi-spine";
import { useEffect, useRef } from "react";
import { loadSpineWithEncodedURLs } from "#/components/operators/detail/impl/components/chibi/helpers";
import type { EnemyHoverFn } from "../impl/types";
import { TILE_SIZE } from "../impl/util/geometry";

/** One enemy spawn to render as a walking chibi. */
export interface IChibiWalker {
    position: number;
    enemyKey: string;
    /** SVG path (in board/viewBox coordinates) the chibi walks along. */
    d: string;
    skel: string;
    atlas: string;
    /** Travel speed in board px/sec (derived from the enemy's move-speed stat). */
    speed: number;
    /** Pauses along the route: `dist` = path distance (px), `wait` = display seconds to idle. */
    waits: { dist: number; wait: number }[];
}

/** On-canvas footprint of one chibi, in board units (generous so larger enemies fit). */
const SPRITE_W = TILE_SIZE * 2.4;
const SPRITE_H = TILE_SIZE * 2.6;
/**
 * Global spine→board scale. A single constant (rather than normalizing every
 * chibi to one height) preserves each enemy's authored relative size, so small
 * enemies like Originium Slugs stay small and bosses stay large.
 */
const CHIBI_SCALE = 0.16;
/** Lift toward the camera so chibis render in front of raised/high-ground tiles. */
const Z_LIFT = TILE_SIZE;
const HIT_PAD = 6;
const WALK_ANIMS = ["Move", "Run", "Walk", "move", "run", "walk"];
const IDLE_ANIMS = ["Idle", "idle", "Default", "default", "Relax", "Stand"];

function pickWalkAnimation(names: string[]): string | null {
    for (const want of WALK_ANIMS) {
        const hit = names.find((n) => n === want);
        if (hit) return hit;
    }
    const moveLike = names.filter((n) => /move|run|walk/i.test(n));
    if (moveLike.length) {
        return moveLike.find((n) => /loop/i.test(n)) ?? moveLike.find((n) => !/begin|start|end|stop|finish/i.test(n)) ?? moveLike[0];
    }
    return names.find((n) => !/idle|default|die|death/i.test(n)) ?? names[0] ?? null;
}

function pickIdleAnimation(names: string[]): string | null {
    for (const want of IDLE_ANIMS) {
        const hit = names.find((n) => n === want);
        if (hit) return hit;
    }
    return names.find((n) => /idle|default|stand|relax/i.test(n)) ?? null;
}

/** Shared offscreen SVG so getPointAtLength has an attached element in every browser. */
let measureSvg: SVGSVGElement | null = null;
function makePath(d: string): SVGPathElement {
    if (!measureSvg) {
        measureSvg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
        measureSvg.setAttribute("width", "0");
        measureSvg.setAttribute("height", "0");
        Object.assign(measureSvg.style, { position: "absolute", width: "0", height: "0", overflow: "hidden", pointerEvents: "none" });
        document.body.appendChild(measureSvg);
    }
    const path = document.createElementNS("http://www.w3.org/2000/svg", "path");
    path.setAttribute("d", d);
    measureSvg.appendChild(path);
    return path;
}

/**
 * Reusable pool of PIXI applications. Each chibi needs a WebGL context, and creating/destroying one per
 * chibi exhausts the browser's hard context limit (~16) while scrubbing waves - the browser then
 * force-loses the oldest contexts, which pixi doesn't restore, leaving blank WHITE canvases flashing
 * where chibis were. Keeping a bounded set of apps alive and reusing them caps the live context count
 * (peak ≈ the on-screen chibi limit) so contexts are never churned or lost.
 */
const APP_POOL: PIXI.Application[] = [];
const MAX_POOLED_APPS = 12;

function acquireChibiApp(): PIXI.Application {
    const pooled = APP_POOL.pop();
    if (pooled) {
        pooled.stage.removeChildren();
        pooled.renderer.render(pooled.stage); // clear to transparent before reuse
        return pooled;
    }
    const app = new PIXI.Application({ width: SPRITE_W, height: SPRITE_H, backgroundAlpha: 0, antialias: true, resolution: window.devicePixelRatio || 1, autoDensity: true });
    app.renderer.render(app.stage);
    return app;
}

function releaseChibiApp(app: PIXI.Application): void {
    app.stage.removeChildren();
    app.renderer.render(app.stage); // leave it cleared so the next borrower never shows a stale/white frame
    const canvas = app.view as HTMLCanvasElement;
    canvas.parentNode?.removeChild(canvas);
    if (APP_POOL.length < MAX_POOLED_APPS) APP_POOL.push(app);
    else app.destroy(true, { children: true, texture: true });
}

/**
 * A single enemy chibi that walks its route. The spine animation plays in a
 * small fixed PIXI canvas; the surrounding element is moved along the path and
 * counter-rotated by `tilt` so the chibi stands upright on the tilted board
 * (the parent layer is rotated by +tilt, this cancels it).
 */
function ChibiWalkerSprite({ walker, padY, tilt, onEnemyHover }: { walker: IChibiWalker; padY: number; tilt: number; onEnemyHover?: EnemyHoverFn }) {
    const wrapRef = useRef<HTMLDivElement>(null);
    const mountRef = useRef<HTMLDivElement>(null);
    const hitRef = useRef<HTMLDivElement>(null);

    useEffect(() => {
        const mount = mountRef.current;
        const wrap = wrapRef.current;
        const hit = hitRef.current;
        if (!mount || !wrap || !hit) return;

        // Re-hide on every run: when a route position is reused across spawn groups the effect re-runs
        // in place (no unmount), and a previous run may have left the wrap visible - which would show
        // the fresh, empty pixi canvas as a white square until the new spine finishes loading.
        wrap.style.visibility = "hidden";
        // Disarm the hover target until the new spine loads and we know its footprint;
        // also ensures a failed load never leaves an invisible, full-size box blocking neighbors.
        hit.style.pointerEvents = "none";

        let cancelled = false;
        let spine: Spine | null = null;
        let moveAnim: string | null = null;
        let idleAnim: string | null = null;
        let curAnim: string | null = null;
        let raf = 0;
        let last = performance.now();
        let t = 0;
        let dir = 1;
        let waitTimer = 0;
        let nextWait = 0;
        const path = makePath(walker.d);
        const length = path.getTotalLength() || 1;
        const waits = walker.waits;

        const app = acquireChibiApp();
        mount.appendChild(app.view as HTMLCanvasElement);

        const setAnim = (name: string | null) => {
            if (spine && name && name !== curAnim) {
                spine.state.setAnimation(0, name, true);
                curAnim = name;
            }
        };

        const tick = (now: number) => {
            const dt = Math.min((now - last) / 1000, 0.1);
            last = now;

            // Advance along the path, pausing for the enemy's wait points.
            const moving = waitTimer <= 0;
            if (!moving) {
                waitTimer -= dt;
            } else if (nextWait < waits.length && t + walker.speed * dt >= waits[nextWait].dist) {
                t = waits[nextWait].dist;
                waitTimer = waits[nextWait].wait;
                nextWait += 1;
            } else {
                t += walker.speed * dt;
                if (t >= length) {
                    t -= length;
                    nextWait = 0;
                }
            }

            const p = path.getPointAtLength(t);
            const ahead = path.getPointAtLength(Math.min(t + 2, length));
            dir = ahead.x >= p.x ? 1 : -1;
            // Feet (bottom-center) sit on the path point; counter-rotate to stand upright,
            // and lift toward the camera so raised tiles don't clip the chibi.
            wrap.style.transform = `translate(${p.x}px, ${p.y + padY}px) translate(-50%, -100%) rotateX(${-tilt}deg) translateZ(${Z_LIFT}px) scaleX(${dir})`;

            if (spine) {
                const stopped = waitTimer > 0;
                setAnim(stopped ? (idleAnim ?? moveAnim) : moveAnim);
                // Freeze the frame while waiting if the chibi has no idle animation.
                spine.update(stopped && !idleAnim ? 0 : dt);
                if (spine.alpha < 1) spine.alpha = Math.min(1, spine.alpha + dt * 6);
            }
            app.renderer.render(app.stage);
            raf = requestAnimationFrame(tick);
        };
        raf = requestAnimationFrame(tick);

        loadSpineWithEncodedURLs(walker.skel, walker.atlas)
            .then((s) => {
                if (cancelled) {
                    s.destroy({ children: true, texture: true, baseTexture: true });
                    return;
                }
                s.autoUpdate = false;
                s.alpha = 0;
                const names = s.spineData.animations.map((a) => a.name);
                moveAnim = pickWalkAnimation(names);
                idleAnim = pickIdleAnimation(names);
                if (moveAnim) {
                    s.state.setAnimation(0, moveAnim, true);
                    curAnim = moveAnim;
                }
                s.update(0);
                const b = s.getLocalBounds();
                const scale = Math.min(CHIBI_SCALE, b.height > 0 ? (SPRITE_H * 0.92) / b.height : CHIBI_SCALE);
                s.scale.set(scale);
                s.position.set(SPRITE_W / 2 - (b.x + b.width / 2) * scale, SPRITE_H - 1 - (b.y + b.height) * scale);
                app.stage.addChild(s);
                app.renderer.render(app.stage);
                spine = s;
                // Tighten the hover hitbox to the chibi's actual rendered footprint (it sits
                // bottom-center of the sprite). The full SPRITE_W×SPRITE_H wrap is mostly empty
                // space, so leaving it hoverable made adjacent enemies' boxes overlap badly.
                const footW = Math.min(SPRITE_W, Math.max(b.width * scale + HIT_PAD * 2, TILE_SIZE * 0.7));
                const footH = Math.min(SPRITE_H, Math.max(b.height * scale + HIT_PAD, TILE_SIZE * 0.9));
                hit.style.width = `${footW}px`;
                hit.style.height = `${footH}px`;
                hit.style.left = `${(SPRITE_W - footW) / 2}px`;
                hit.style.top = `${SPRITE_H - footH}px`;
                hit.style.pointerEvents = "auto";
                wrap.style.visibility = "visible";
            })
            .catch(() => {
                /* missing/failed chibi - silently skip */
            });

        return () => {
            cancelled = true;
            cancelAnimationFrame(raf);
            if (spine) {
                app.stage.removeChild(spine);
                spine.destroy({ children: true, texture: true, baseTexture: true });
            }
            path.remove();
            // Return the app (and its WebGL context) to the pool instead of destroying it - see APP_POOL.
            releaseChibiApp(app);
        };
    }, [walker, padY, tilt]);

    return (
        <div ref={wrapRef} className="absolute top-0 left-0" style={{ width: SPRITE_W, height: SPRITE_H, visibility: "hidden", transformOrigin: "bottom center", transformStyle: "preserve-3d", willChange: "transform", pointerEvents: "none" }}>
            <div ref={mountRef} className="h-full w-full" style={{ pointerEvents: "none" }} />
            {/* Hover target sized to the chibi's actual footprint (set on load) - kept small so adjacent enemies don't fight over one big rect. */}
            <div
                ref={hitRef}
                className="absolute"
                style={{ pointerEvents: "none", cursor: "help" }}
                onPointerEnter={(e) => onEnemyHover?.(walker.enemyKey, e.clientX, e.clientY, e.movementX !== 0 || e.movementY !== 0)}
                onPointerMove={(e) => onEnemyHover?.(walker.enemyKey, e.clientX, e.clientY, e.movementX !== 0 || e.movementY !== 0)}
                onPointerLeave={() => onEnemyHover?.(null, 0, 0, false)}
            />
        </div>
    );
}

export function ChibiLayer({ walkers, width, height, padY, tilt = 30, onEnemyHover }: { walkers: IChibiWalker[]; width: number; height: number; padY: number; tilt?: number; onEnemyHover?: EnemyHoverFn }) {
    return (
        <div className="transform-3d pointer-events-none absolute top-0 left-0" style={{ width, height }}>
            {walkers.map((w) => (
                <ChibiWalkerSprite key={w.position} walker={w} padY={padY} tilt={tilt} onEnemyHover={onEnemyHover} />
            ))}
        </div>
    );
}
