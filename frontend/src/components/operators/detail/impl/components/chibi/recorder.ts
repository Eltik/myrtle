import { ArrayBufferTarget, Muxer } from "mp4-muxer";
import * as PIXI from "pixi.js";
import type { Spine } from "pixi-spine";
import { CHIBI_OFFSET_X, CHIBI_OFFSET_Y, DEFAULT_EXPORT_SETTINGS, EXPORT_BG_COLOR, EXPORT_HEIGHT, EXPORT_WIDTH, type IExportSettings } from "./constants";

export type ExportFormat = "gif" | "mp4";

export interface IRecordingOptions {
    liveApp: PIXI.Application;
    spine: import("pixi-spine").Spine;
    animationName: string;
    format: ExportFormat;
    settings?: Partial<IExportSettings>;
    onProgress?: (progress: number) => void;
    signal?: AbortSignal;
}

export interface IRecordingResult {
    blob: Blob;
    filename: string;
    mimeType: string;
}

interface IGIFOptions {
    workers?: number;
    quality?: number;
    repeat?: number;
    background?: string;
    width?: number | null;
    height?: number | null;
    transparent?: number | null;
    dither?: boolean | string;
    debug?: boolean;
    workerScript?: string;
}

interface IGIFFrameOptions {
    delay?: number;
    copy?: boolean;
    dispose?: number;
}

interface IGIFInstance {
    addFrame(image: CanvasRenderingContext2D | ImageData | HTMLCanvasElement | HTMLImageElement, options?: IGIFFrameOptions): void;
    on(event: "start", callback: () => void): void;
    on(event: "progress", callback: (progress: number) => void): void;
    on(event: "finished", callback: (blob: Blob, data: Uint8Array) => void): void;
    on(event: "abort", callback: () => void): void;
    render(): void;
    abort(): void;
    running: boolean;
}

interface IGIFConstructor {
    new (options?: IGIFOptions): IGIFInstance;
}

function checkAborted(signal?: AbortSignal): void {
    if (signal?.aborted) {
        throw new DOMException("Recording was cancelled", "AbortError");
    }
}

/**
 * Snapshot of mutable spine state that we need to restore after recording.
 * Captured once, restored once - no scattered originalX/originalY locals.
 */
interface ISpineSnapshot {
    timeScale: number;
    x: number;
    y: number;
    scaleX: number;
    scaleY: number;
    parent: PIXI.Container | null;
}

function snapshotSpine(spine: Spine): ISpineSnapshot {
    return {
        timeScale: spine.state.timeScale,
        x: spine.x,
        y: spine.y,
        scaleX: spine.scale.x,
        scaleY: spine.scale.y,
        parent: spine.parent,
    };
}

function restoreSpine(spine: Spine, snapshot: ISpineSnapshot, liveApp: PIXI.Application | undefined, animationName: string): void {
    if (spine.parent) {
        spine.parent.removeChild(spine);
    }
    // Prefer the stage we were originally attached to; fall back to liveApp.
    const target = snapshot.parent ?? liveApp?.stage ?? null;
    target?.addChild(spine);

    spine.state.timeScale = snapshot.timeScale;
    spine.x = snapshot.x;
    spine.y = snapshot.y;
    spine.scale.set(snapshot.scaleX, snapshot.scaleY);
    spine.state.setAnimation(0, animationName, true);
}

/**
 * Configures spine + an offscreen PIXI app for export. Returns both so the
 * caller can render and clean up. Any failure here is the caller's
 * responsibility to handle via the returned cleanup function.
 */
interface IRenderContext {
    app: PIXI.Application;
    width: number;
    height: number;
    cleanup: () => void;
}

function createRenderContext(spine: Spine, settings: IExportSettings, transparent: boolean): IRenderContext {
    const width = Math.round(EXPORT_WIDTH * settings.scale);
    const height = Math.round(EXPORT_HEIGHT * settings.scale);

    const app = new PIXI.Application({
        width,
        height,
        backgroundColor: transparent ? 0x000000 : EXPORT_BG_COLOR,
        backgroundAlpha: transparent ? 0 : 1,
        antialias: true,
        preserveDrawingBuffer: true,
        resolution: 1,
        autoDensity: false,
    });

    spine.state.timeScale = 1;
    spine.x = width * CHIBI_OFFSET_X;
    spine.y = height * CHIBI_OFFSET_Y;
    const scale = Math.min(width / EXPORT_WIDTH, height / EXPORT_HEIGHT) * 0.75;
    spine.scale.set(scale);

    if (spine.parent) {
        spine.parent.removeChild(spine);
    }
    app.stage.addChild(spine);

    return {
        app,
        width,
        height,
        cleanup: () => app.destroy(true, { children: false, texture: false }),
    };
}

/** Reset and start the animation at frame 0. */
function primeAnimation(spine: Spine, animationName: string): void {
    spine.state.clearTracks();
    spine.skeleton.setToSetupPose();
    spine.state.setAnimation(0, animationName, true);
    spine.state.apply(spine.skeleton);
    spine.skeleton.updateWorldTransform();
}

/** Advance the animation by `dt` seconds and re-render the offscreen stage. */
function advanceAndRender(app: PIXI.Application, spine: Spine, dt: number): void {
    if (dt > 0) {
        spine.state.update(dt);
        spine.state.apply(spine.skeleton);
        spine.skeleton.updateWorldTransform();
    }
    spine.updateTransform();
    app.renderer.render(app.stage);
}

function getAnimationDuration(spine: Spine, animationName: string): number {
    return spine.spineData.findAnimation(animationName)?.duration ?? 1;
}

/**
 * Yield to the event loop without the 4ms setTimeout clamp.
 * MessageChannel posts are dispatched on the next tick with no minimum delay.
 */
const yieldToEventLoop: () => Promise<void> = (() => {
    if (typeof MessageChannel === "undefined") {
        return () => new Promise<void>((r) => setTimeout(r, 0));
    }
    return () =>
        new Promise<void>((resolve) => {
            const channel = new MessageChannel();
            channel.port1.onmessage = () => {
                channel.port1.close();
                resolve();
            };
            channel.port2.postMessage(null);
        });
})();

export async function recordAsGif(options: IRecordingOptions): Promise<IRecordingResult> {
    const { spine, animationName, onProgress, signal, liveApp } = options;
    const settings: IExportSettings = { ...DEFAULT_EXPORT_SETTINGS, ...options.settings };

    // GIF delays are quantized to 10ms (min 20ms for browser compatibility).
    // frameDelta MUST equal frameDelayMs / 1000 - otherwise the captured spine
    // state and the GIF playback timeline drift apart.
    const frameDelayMs = Math.max(20, Math.round(1000 / settings.fps / 10) * 10);
    const frameDelta = frameDelayMs / 1000;

    const loopDuration = getAnimationDuration(spine, animationName);
    const totalFrames = Math.max(1, Math.round(loopDuration / frameDelta));

    const snapshot = snapshotSpine(spine);
    let ctx: IRenderContext | null = null;
    let gif: IGIFInstance | null = null;
    let abortHandler: (() => void) | null = null;

    try {
        checkAborted(signal);

        const [{ default: GIF }] = await Promise.all([import("gif.js") as Promise<{ default: IGIFConstructor }>]);

        ctx = createRenderContext(spine, settings, settings.transparentBg);
        primeAnimation(spine, animationName);

        gif = new GIF({
            workers: 4,
            quality: 10,
            width: ctx.width,
            height: ctx.height,
            workerScript: "/gif.worker.js",
            repeat: 0,
            transparent: settings.transparentBg ? 0x000000 : null,
            dither: false,
            debug: false,
        });

        abortHandler = () => gif?.abort();
        signal?.addEventListener("abort", abortHandler);

        // Stream frames straight from the WebGL canvas into gif.js. The library
        // copies internally (copy: true) so we don't need our own canvas buffer.
        const canvas = ctx.app.view as HTMLCanvasElement;
        for (let frame = 0; frame < totalFrames; frame++) {
            checkAborted(signal);
            advanceAndRender(ctx.app, spine, frame === 0 ? 0 : frameDelta);

            gif.addFrame(canvas, {
                delay: frameDelayMs,
                copy: true,
                dispose: 2,
            });

            onProgress?.(((frame + 1) / totalFrames) * 50);

            // Yielding every ~8 frames is enough to keep the UI responsive
            // without paying the per-yield overhead of doing it every frame.
            if ((frame & 7) === 7) {
                await yieldToEventLoop();
            }
        }

        checkAborted(signal);

        const encoder = gif;
        if (!encoder) throw new Error("GIF encoder was not initialized");
        const result = await new Promise<IRecordingResult>((resolve, reject) => {
            encoder.on("progress", (p) => onProgress?.(50 + p * 50));
            encoder.on("finished", (blob) => {
                resolve({
                    blob,
                    filename: `${animationName}.gif`,
                    mimeType: "image/gif",
                });
            });
            encoder.on("abort", () => {
                reject(new DOMException("Recording was cancelled", "AbortError"));
            });
            encoder.render();
        });

        onProgress?.(100);
        return result;
    } finally {
        if (abortHandler) signal?.removeEventListener("abort", abortHandler);
        restoreSpine(spine, snapshot, liveApp, animationName);
        ctx?.cleanup();
    }
}

/**
 * Pick the lowest AVC level that fits the given resolution.
 * Browsers reject configures whose level is below the frame size.
 *   3.1 → ≤ 1280×720
 *   4.0 → ≤ 2048×1024
 *   5.1 → ≤ 4096×2048
 */
function pickAvcLevel(width: number, height: number): string {
    const pixels = width * height;
    if (pixels > 2_097_152) return "33"; // 5.1
    if (pixels > 921_600) return "28"; // 4.0
    return "1f"; // 3.1
}

export async function recordAsVideo(options: IRecordingOptions): Promise<IRecordingResult> {
    const { spine, animationName, onProgress, signal, liveApp } = options;
    const settings: IExportSettings = { ...DEFAULT_EXPORT_SETTINGS, ...options.settings };

    const loopDuration = getAnimationDuration(spine, animationName);
    const totalFrames = Math.ceil(loopDuration * settings.loopCount * settings.fps);
    const frameDelta = 1 / settings.fps;
    const microsecondsPerFrame = 1_000_000 / settings.fps;
    const KEYFRAME_INTERVAL = 30;

    const snapshot = snapshotSpine(spine);
    let ctx: IRenderContext | null = null;
    let videoEncoder: VideoEncoder | null = null;
    let muxer: Muxer<ArrayBufferTarget> | null = null;
    let encoderError: Error | null = null;

    try {
        checkAborted(signal);
        ctx = createRenderContext(spine, settings, false);
        primeAnimation(spine, animationName);

        muxer = new Muxer({
            target: new ArrayBufferTarget(),
            video: { codec: "avc", width: ctx.width, height: ctx.height },
            fastStart: "in-memory",
        });

        // Stream encoded chunks straight into the muxer instead of buffering
        // the entire encoded video in a temporary array.
        const activeMuxer = muxer;
        videoEncoder = new VideoEncoder({
            output: (chunk, meta) => activeMuxer.addVideoChunk(chunk, meta),
            error: (e) => {
                encoderError = e;
            },
        });

        videoEncoder.configure({
            codec: `avc1.4200${pickAvcLevel(ctx.width, ctx.height)}`,
            width: ctx.width,
            height: ctx.height,
            bitrate: 5_000_000,
            framerate: settings.fps,
        });

        // VideoFrame can take the WebGL canvas directly - the extra 2D context
        // copy in the original code was pure overhead.
        const canvas = ctx.app.view as HTMLCanvasElement;

        for (let frame = 0; frame < totalFrames; frame++) {
            checkAborted(signal);
            if (encoderError) throw encoderError;
            if (videoEncoder.state === "closed") {
                throw new Error("Video encoder was unexpectedly closed");
            }

            advanceAndRender(ctx.app, spine, frame === 0 ? 0 : frameDelta);

            const videoFrame = new VideoFrame(canvas, {
                timestamp: frame * microsecondsPerFrame,
                duration: microsecondsPerFrame,
            });
            videoEncoder.encode(videoFrame, {
                keyFrame: frame % KEYFRAME_INTERVAL === 0,
            });
            videoFrame.close();

            onProgress?.(((frame + 1) / totalFrames) * 90);

            if ((frame & 7) === 7) {
                await yieldToEventLoop();
            }
        }

        if (encoderError) throw encoderError;
        await videoEncoder.flush();
        if (encoderError) throw encoderError;

        muxer.finalize();
        onProgress?.(100);

        const blob = new Blob([muxer.target.buffer], { type: "video/mp4" });
        return {
            blob,
            filename: `${animationName}.mp4`,
            mimeType: "video/mp4",
        };
    } finally {
        if (videoEncoder && videoEncoder.state !== "closed") {
            videoEncoder.close();
        }
        restoreSpine(spine, snapshot, liveApp, animationName);
        ctx?.cleanup();
    }
}
