import { ArrayBufferTarget, Muxer } from "mp4-muxer";
import * as PIXI from "pixi.js";
import { CHIBI_OFFSET_X, CHIBI_OFFSET_Y, DEFAULT_EXPORT_SETTINGS, EXPORT_BG_COLOR, EXPORT_HEIGHT, EXPORT_WIDTH, type ExportSettings } from "./constants";

export type ExportFormat = "gif" | "mp4";

export interface RecordingOptions {
    liveApp: PIXI.Application;
    spine: import("pixi-spine").Spine;
    animationName: string;
    format: ExportFormat;
    settings?: Partial<ExportSettings>;
    onProgress?: (progress: number) => void;
    signal?: AbortSignal;
}

export interface RecordingResult {
    blob: Blob;
    filename: string;
    mimeType: string;
}

// ============ GIF.JS TYPE DEFINITIONS ============

interface GIFOptions {
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

interface GIFFrameOptions {
    delay?: number;
    copy?: boolean;
    dispose?: number;
}

interface GIFInstance {
    addFrame(image: CanvasRenderingContext2D | ImageData | HTMLCanvasElement | HTMLImageElement, options?: GIFFrameOptions): void;
    on(event: "start", callback: () => void): void;
    on(event: "progress", callback: (progress: number) => void): void;
    on(event: "finished", callback: (blob: Blob, data: Uint8Array) => void): void;
    on(event: "abort", callback: () => void): void;
    render(): void;
    abort(): void;
    running: boolean;
}

interface GIFConstructor {
    new (options?: GIFOptions): GIFInstance;
}

// ============ HELPER FUNCTIONS ============

function resolveSettings(settings?: Partial<ExportSettings>): ExportSettings {
    return { ...DEFAULT_EXPORT_SETTINGS, ...settings };
}

function checkAborted(signal?: AbortSignal): void {
    if (signal?.aborted) {
        throw new DOMException("Recording was cancelled", "AbortError");
    }
}

function getAnimationDuration(spine: import("pixi-spine").Spine, animationName: string): number {
    const animation = spine.spineData.findAnimation(animationName);
    return animation?.duration ?? 1;
}

// Compare two ImageData objects and return a similarity score (lower = more similar)
function compareFrames(frame1: ImageData, frame2: ImageData): number {
    const data1 = frame1.data;
    const data2 = frame2.data;
    let diff = 0;
    for (let i = 0; i < data1.length; i += 16) {
        const r1 = data1[i] ?? 0;
        const r2 = data2[i] ?? 0;
        const g1 = data1[i + 1] ?? 0;
        const g2 = data2[i + 1] ?? 0;
        const b1 = data1[i + 2] ?? 0;
        const b2 = data2[i + 2] ?? 0;
        diff += Math.abs(r1 - r2) + Math.abs(g1 - g2) + Math.abs(b1 - b2);
    }
    return diff / ((data1.length / 16) * 3);
}

// Find the frame index where the animation visually loops back to the start
function findVisualLoopPoint(frames: ImageData[], minFrameIndex: number = 10): number {
    if (frames.length <= minFrameIndex) return frames.length;

    const firstFrame = frames[0];
    if (!firstFrame) return frames.length;

    let bestMatchIndex = frames.length;
    let bestMatchScore = Infinity;

    for (let i = minFrameIndex; i < frames.length; i++) {
        const frame = frames[i];
        if (!frame) continue;
        const score = compareFrames(firstFrame, frame);
        if (score < bestMatchScore) {
            bestMatchScore = score;
            bestMatchIndex = i;
        }
    }

    if (bestMatchScore < 15) {
        return bestMatchIndex;
    }
    return frames.length;
}

// ============ GIF RECORDING ============

export async function recordAsGif(options: RecordingOptions): Promise<RecordingResult> {
    const { spine, animationName, onProgress, signal } = options;
    const settings = resolveSettings(options.settings);

    const width = Math.round(EXPORT_WIDTH * settings.scale);
    const height = Math.round(EXPORT_HEIGHT * settings.scale);
    const singleLoopDuration = getAnimationDuration(spine, animationName);

    // GIF delays are quantized to 10ms (min 20ms for browser compatibility)
    // CRITICAL: frameDelta must match frameDelayMs for consistent timing
    // Otherwise the animation plays at wrong speed and loop timing is off
    const frameDelayMs = Math.max(20, Math.round(1000 / settings.fps / 10) * 10);
    const frameDelta = frameDelayMs / 1000;

    const totalFrames = Math.max(1, Math.round(singleLoopDuration / frameDelta));

    const originalTimeScale = spine.state.timeScale;
    const originalX = spine.x;
    const originalY = spine.y;
    const originalScaleX = spine.scale.x;
    const originalScaleY = spine.scale.y;

    let offscreenApp: PIXI.Application | null = null;
    let tempCanvas: HTMLCanvasElement | null = null;
    let gif: GIFInstance | null = null;

    try {
        checkAborted(signal);

        const GIF = (await import("gif.js")).default as GIFConstructor;

        offscreenApp = new PIXI.Application({
            width,
            height,
            backgroundColor: settings.transparentBg ? 0x000000 : EXPORT_BG_COLOR,
            backgroundAlpha: settings.transparentBg ? 0 : 1,
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
        offscreenApp.stage.addChild(spine);

        tempCanvas = document.createElement("canvas");
        tempCanvas.width = width;
        tempCanvas.height = height;
        const tempCtx = tempCanvas.getContext("2d", { willReadFrequently: true });
        if (!tempCtx) {
            throw new Error("Failed to create 2D context");
        }

        spine.state.clearTracks();
        spine.skeleton.setToSetupPose();
        spine.state.setAnimation(0, animationName, true);
        spine.state.apply(spine.skeleton);
        spine.skeleton.updateWorldTransform();

        const frames: ImageData[] = [];

        for (let frame = 0; frame < totalFrames; frame++) {
            checkAborted(signal);

            if (frame > 0) {
                spine.state.update(frameDelta);
                spine.state.apply(spine.skeleton);
                spine.skeleton.updateWorldTransform();
            }
            spine.updateTransform();
            offscreenApp.renderer.render(offscreenApp.stage);

            tempCtx.clearRect(0, 0, width, height);

            const webglCanvas = offscreenApp.view as HTMLCanvasElement;
            tempCtx.drawImage(webglCanvas, 0, 0);

            const imageData = tempCtx.getImageData(0, 0, width, height);
            frames.push(imageData);

            onProgress?.(((frame + 1) / totalFrames) * 50);

            if (frame % 5 === 0) {
                await new Promise((resolve) => setTimeout(resolve, 0));
            }
        }

        checkAborted(signal);

        gif = new GIF({
            workers: 4,
            quality: 10,
            width,
            height,
            workerScript: "/gif.worker.js",
            repeat: 0,
            transparent: settings.transparentBg ? 0x000000 : null,
            dither: false,
            debug: false,
        });

        const abortHandler = () => {
            gif?.abort();
        };
        signal?.addEventListener("abort", abortHandler);

        const visualLoopPoint = findVisualLoopPoint(frames);
        const loopFrames = frames.slice(0, visualLoopPoint);

        loopFrames.forEach((frameData) => {
            checkAborted(signal);
            gif?.addFrame(frameData, {
                delay: frameDelayMs,
                copy: true,
                dispose: 2,
            });
        });

        const result = await new Promise<RecordingResult>((resolve, reject) => {
            if (!gif) {
                reject(new Error("GIF encoder not initialized"));
                return;
            }

            gif.on("progress", (progress) => {
                onProgress?.(50 + progress * 50);
            });

            gif.on("finished", (blob) => {
                signal?.removeEventListener("abort", abortHandler);
                resolve({
                    blob,
                    filename: `${animationName}.gif`,
                    mimeType: "image/gif",
                });
            });

            gif.on("abort", () => {
                signal?.removeEventListener("abort", abortHandler);
                reject(new DOMException("Recording was cancelled", "AbortError"));
            });

            gif.render();
        });

        onProgress?.(100);
        return result;
    } finally {
        if (spine.parent) {
            spine.parent.removeChild(spine);
        }
        if (options.liveApp?.stage) {
            options.liveApp.stage.addChild(spine);
        }

        spine.state.timeScale = originalTimeScale;
        spine.x = originalX;
        spine.y = originalY;
        spine.scale.set(originalScaleX, originalScaleY);
        spine.state.setAnimation(0, animationName, true);

        if (offscreenApp) {
            offscreenApp.destroy(true, { children: false, texture: false });
        }
        if (tempCanvas) {
            tempCanvas.width = 0;
            tempCanvas.height = 0;
        }
    }
}

// ============ MP4 RECORDING ============

export async function recordAsVideo(options: RecordingOptions): Promise<RecordingResult> {
    const { spine, animationName, onProgress, signal } = options;
    const settings = resolveSettings(options.settings);

    const width = Math.round(EXPORT_WIDTH * settings.scale);
    const height = Math.round(EXPORT_HEIGHT * settings.scale);
    const singleLoopDuration = getAnimationDuration(spine, animationName);
    const loopCount = settings.loopCount;
    const totalDuration = singleLoopDuration * loopCount;
    const totalFrames = Math.ceil(totalDuration * settings.fps);
    const frameDelta = 1 / settings.fps;

    const originalTimeScale = spine.state.timeScale;
    const originalX = spine.x;
    const originalY = spine.y;
    const originalScaleX = spine.scale.x;
    const originalScaleY = spine.scale.y;

    let offscreenApp: PIXI.Application | null = null;
    let tempCanvas: HTMLCanvasElement | null = null;
    let muxer: Muxer<ArrayBufferTarget> | null = null;
    let videoEncoder: VideoEncoder | null = null;
    let encoderError: Error | null = null;

    try {
        checkAborted(signal);

        offscreenApp = new PIXI.Application({
            width,
            height,
            backgroundColor: EXPORT_BG_COLOR,
            backgroundAlpha: 1,
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
        offscreenApp.stage.addChild(spine);

        tempCanvas = document.createElement("canvas");
        tempCanvas.width = width;
        tempCanvas.height = height;
        const tempCtx = tempCanvas.getContext("2d", { willReadFrequently: true });
        if (!tempCtx) {
            throw new Error("Failed to create 2D context");
        }

        muxer = new Muxer({
            target: new ArrayBufferTarget(),
            video: {
                codec: "avc",
                width,
                height,
            },
            fastStart: "in-memory",
        });

        const encodedChunks: { chunk: EncodedVideoChunk; meta?: EncodedVideoChunkMetadata }[] = [];

        videoEncoder = new VideoEncoder({
            output: (chunk, meta) => {
                encodedChunks.push({ chunk, meta });
            },
            error: (e) => {
                encoderError = e;
            },
        });

        // Select appropriate AVC level based on resolution
        // Level 3.1 (1f): max 1280x720 (921,600 pixels)
        // Level 4.0 (28): max 2048x1024 (2,097,152 pixels)
        // Level 5.1 (33): max 4096x2048 (8,388,608 pixels)
        const pixels = width * height;
        let codecLevel = "1f";
        if (pixels > 921600) {
            codecLevel = "28";
        }
        if (pixels > 2097152) {
            codecLevel = "33";
        }

        videoEncoder.configure({
            codec: `avc1.4200${codecLevel}`,
            width,
            height,
            bitrate: 5_000_000,
            framerate: settings.fps,
        });

        spine.state.clearTracks();
        spine.skeleton.setToSetupPose();
        spine.state.setAnimation(0, animationName, true);
        spine.state.apply(spine.skeleton);
        spine.skeleton.updateWorldTransform();

        const microsecondsPerFrame = 1_000_000 / settings.fps;

        for (let frame = 0; frame < totalFrames; frame++) {
            checkAborted(signal);

            if (encoderError) {
                throw encoderError;
            }

            if (videoEncoder.state === "closed") {
                throw new Error("Video encoder was unexpectedly closed");
            }

            if (frame > 0) {
                spine.state.update(frameDelta);
                spine.state.apply(spine.skeleton);
                spine.skeleton.updateWorldTransform();
            }
            spine.updateTransform();
            offscreenApp.renderer.render(offscreenApp.stage);

            const webglCanvas = offscreenApp.view as HTMLCanvasElement;
            tempCtx.drawImage(webglCanvas, 0, 0);

            const videoFrame = new VideoFrame(tempCanvas, {
                timestamp: frame * microsecondsPerFrame,
                duration: microsecondsPerFrame,
            });

            const isKeyframe = frame % 30 === 0;
            videoEncoder.encode(videoFrame, { keyFrame: isKeyframe });
            videoFrame.close();

            onProgress?.(((frame + 1) / totalFrames) * 90);

            if (frame % 2 === 0) {
                await new Promise((resolve) => setTimeout(resolve, 0));
            }
        }

        if (encoderError) {
            throw encoderError;
        }

        await videoEncoder.flush();

        if (encoderError) {
            throw encoderError;
        }

        for (const { chunk, meta } of encodedChunks) {
            muxer.addVideoChunk(chunk, meta);
        }

        muxer.finalize();

        onProgress?.(100);

        const { buffer } = muxer.target;
        const blob = new Blob([buffer], { type: "video/mp4" });

        return {
            blob,
            filename: `${animationName}.mp4`,
            mimeType: "video/mp4",
        };
    } finally {
        if (videoEncoder && videoEncoder.state !== "closed") {
            videoEncoder.close();
        }

        if (spine.parent) {
            spine.parent.removeChild(spine);
        }
        if (options.liveApp?.stage) {
            options.liveApp.stage.addChild(spine);
        }

        spine.state.timeScale = originalTimeScale;
        spine.x = originalX;
        spine.y = originalY;
        spine.scale.set(originalScaleX, originalScaleY);
        spine.state.setAnimation(0, animationName, true);

        if (offscreenApp) {
            offscreenApp.destroy(true, { children: false, texture: false });
        }
        if (tempCanvas) {
            tempCanvas.width = 0;
            tempCanvas.height = 0;
        }
    }
}
