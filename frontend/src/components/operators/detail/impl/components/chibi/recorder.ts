import { ArrayBufferTarget, Muxer } from "mp4-muxer";
import * as PIXI from "pixi.js";
import type { Spine } from "pixi-spine";
import { CHIBI_OFFSET_X, CHIBI_OFFSET_Y, DEFAULT_EXPORT_SETTINGS, EXPORT_BG_COLOR, EXPORT_PADDING, type IExportSettings } from "./constants";
import { computeExportLayout, gifFrameDelayMs, measureAnimationBounds } from "./helpers";
import { type IWebmFrame, muxWebm, type WebmCodec } from "./webm-mux";

export type ExportFormat = "gif" | "mp4" | "webm";

export interface IRecordingOptions {
    liveApp: PIXI.Application;
    spine: import("pixi-spine").Spine;
    animationName: string;
    format: ExportFormat;
    settings?: Partial<IExportSettings>;
    onProgress?: (progress: number) => void;
    signal?: AbortSignal;
    /** File name without its extension; the animation name when unset. */
    filename?: string;
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

/** Spine state to restore after recording. Captured once, restored once. */
interface ISpineSnapshot {
    timeScale: number;
    autoUpdate: boolean;
    x: number;
    y: number;
    scaleX: number;
    scaleY: number;
    parent: PIXI.Container | null;
}

function snapshotSpine(spine: Spine): ISpineSnapshot {
    return {
        timeScale: spine.state.timeScale,
        autoUpdate: spine.autoUpdate,
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
    const target = snapshot.parent ?? liveApp?.stage ?? null;
    target?.addChild(spine);

    spine.state.timeScale = snapshot.timeScale;
    spine.autoUpdate = snapshot.autoUpdate;
    spine.x = snapshot.x;
    spine.y = snapshot.y;
    spine.scale.set(snapshot.scaleX, snapshot.scaleY);
    spine.state.setAnimation(0, animationName, true);
}

/**
 * Spine plus an offscreen PIXI app for export. Setup failures are the caller's to handle
 * through the returned cleanup.
 */
interface IRenderContext {
    app: PIXI.Application;
    width: number;
    height: number;
    cleanup: () => void;
}

function createRenderContext(spine: Spine, animationName: string, settings: IExportSettings, transparent: boolean): IRenderContext {
    spine.autoUpdate = false;
    spine.state.timeScale = 1;

    const bounds = measureAnimationBounds(spine, animationName, undefined, { wholeClip: true });
    const { width, height, chibiScale } = computeExportLayout(bounds, settings.scale);

    const app = new PIXI.Application({
        width,
        height,
        backgroundColor: transparent ? 0x000000 : (settings.backgroundColor ?? EXPORT_BG_COLOR),
        backgroundAlpha: transparent ? 0 : 1,
        antialias: true,
        preserveDrawingBuffer: true,
        resolution: 1,
        autoDensity: false,
    });

    // A mirrored skeleton keeps its box centred: the centre maps to the same x either way.
    const flip = settings.flipX ? -1 : 1;
    spine.scale.set(chibiScale * flip, chibiScale);
    if (bounds) {
        spine.x = width / 2 - (bounds.x + bounds.width / 2) * chibiScale * flip;
        spine.y = height - EXPORT_PADDING - (bounds.y + bounds.height) * chibiScale;
    } else {
        spine.x = width * CHIBI_OFFSET_X;
        spine.y = height * CHIBI_OFFSET_Y;
    }

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

function advanceAndRender(app: PIXI.Application, spine: Spine, dt: number): void {
    spine.update(dt);
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
    // frameDelta MUST match frameDelayMs, or the captured spine state and the
    // GIF playback timeline drift apart.
    const frameDelayMs = gifFrameDelayMs(settings.fps);
    const frameDelta = frameDelayMs / 1000;

    const loopDuration = getAnimationDuration(spine, animationName);
    const totalFrames = Math.max(1, Math.round(loopDuration / frameDelta));
    const stepDelta = totalFrames > 0 ? loopDuration / totalFrames : 0;

    const snapshot = snapshotSpine(spine);
    let ctx: IRenderContext | null = null;
    let gif: IGIFInstance | null = null;
    let abortHandler: (() => void) | null = null;

    try {
        checkAborted(signal);

        const [{ default: GIF }] = await Promise.all([import("gif.js") as Promise<{ default: IGIFConstructor }>]);

        ctx = createRenderContext(spine, animationName, settings, settings.transparentBg);
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
            advanceAndRender(ctx.app, spine, frame === 0 ? 0 : stepDelta);

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
                    filename: `${options.filename ?? animationName}.gif`,
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
 *   3.1 -> ≤ 1280×720
 *   4.0 -> ≤ 2048×1024
 *   5.1 -> ≤ 4096×2048
 */
function pickAvcLevel(width: number, height: number): string {
    const pixels = width * height;
    if (pixels > 2_097_152) return "33"; // 5.1
    if (pixels > 921_600) return "28"; // 4.0
    return "1f"; // 3.1
}

/** VP9 where the browser encodes it, else VP8; `null` when neither (no WebM export). */
async function pickWebmCodec(width: number, height: number, fps: number): Promise<{ codec: string; container: WebmCodec } | null> {
    for (const [codec, container] of [
        ["vp09.00.10.08", "V_VP9"],
        ["vp8", "V_VP8"],
    ] as const) {
        const { supported } = await VideoEncoder.isConfigSupported({ codec, width, height, bitrate: 5_000_000, framerate: fps });
        if (supported) return { codec, container };
    }
    return null;
}

/** H.264 in MP4 for `mp4`, VP9 (or VP8) in WebM for `webm`; one frame loop for both. */
export async function recordAsVideo(options: IRecordingOptions): Promise<IRecordingResult> {
    const { spine, animationName, onProgress, signal, liveApp } = options;
    const webm = options.format === "webm";
    const settings: IExportSettings = { ...DEFAULT_EXPORT_SETTINGS, ...options.settings };

    const loopDuration = getAnimationDuration(spine, animationName);
    const framesPerLoop = Math.max(1, Math.round(loopDuration * settings.fps));
    const totalFrames = framesPerLoop * settings.loopCount;
    const frameDelta = loopDuration > 0 ? loopDuration / framesPerLoop : 0;
    const microsecondsPerFrame = 1_000_000 / settings.fps;
    const KEYFRAME_INTERVAL = 30;

    const snapshot = snapshotSpine(spine);
    let ctx: IRenderContext | null = null;
    let videoEncoder: VideoEncoder | null = null;
    let encoderError: Error | null = null;

    try {
        checkAborted(signal);
        ctx = createRenderContext(spine, animationName, settings, false);
        primeAnimation(spine, animationName);

        let muxer: Muxer<ArrayBufferTarget> | null = null;
        let webmCodec: { codec: string; container: WebmCodec } | null = null;
        const webmFrames: IWebmFrame[] = [];
        let output: EncodedVideoChunkOutputCallback;
        if (webm) {
            webmCodec = await pickWebmCodec(ctx.width, ctx.height, settings.fps);
            if (!webmCodec) throw new Error("This browser cannot encode VP8 or VP9 video");
            output = (chunk) => {
                const data = new Uint8Array(chunk.byteLength);
                chunk.copyTo(data);
                webmFrames.push({ data, timestamp: chunk.timestamp, key: chunk.type === "key" });
            };
        } else {
            // Stream encoded chunks straight into the muxer instead of buffering
            // the entire encoded video in a temporary array.
            const activeMuxer = new Muxer({
                target: new ArrayBufferTarget(),
                video: { codec: "avc", width: ctx.width, height: ctx.height },
                fastStart: "in-memory",
            });
            muxer = activeMuxer;
            output = (chunk, meta) => activeMuxer.addVideoChunk(chunk, meta);
        }

        videoEncoder = new VideoEncoder({
            output,
            error: (e) => {
                encoderError = e;
            },
        });

        videoEncoder.configure({
            codec: webmCodec ? webmCodec.codec : `avc1.4200${pickAvcLevel(ctx.width, ctx.height)}`,
            width: ctx.width,
            height: ctx.height,
            bitrate: 5_000_000,
            framerate: settings.fps,
        });

        // VideoFrame takes the WebGL canvas directly, no 2D context copy needed.
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

        const filename = options.filename ?? animationName;
        let result: IRecordingResult;
        if (webmCodec) {
            const bytes = muxWebm({ codec: webmCodec.container, width: ctx.width, height: ctx.height, duration: totalFrames * microsecondsPerFrame, frames: webmFrames });
            result = { blob: new Blob([bytes], { type: "video/webm" }), filename: `${filename}.webm`, mimeType: "video/webm" };
        } else if (muxer) {
            muxer.finalize();
            result = { blob: new Blob([muxer.target.buffer], { type: "video/mp4" }), filename: `${filename}.mp4`, mimeType: "video/mp4" };
        } else {
            throw new Error("Video muxer was not initialized");
        }
        onProgress?.(100);
        return result;
    } finally {
        if (videoEncoder && videoEncoder.state !== "closed") {
            videoEncoder.close();
        }
        restoreSpine(spine, snapshot, liveApp, animationName);
        ctx?.cleanup();
    }
}
