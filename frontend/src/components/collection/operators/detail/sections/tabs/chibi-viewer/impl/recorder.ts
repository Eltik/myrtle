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
    // Sample every 4th pixel for performance (RGBA = 4 bytes per pixel)
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

    // Look for a frame similar to the first frame (after minimum frames to avoid false matches)
    for (let i = minFrameIndex; i < frames.length; i++) {
        const frame = frames[i];
        if (!frame) continue;
        const score = compareFrames(firstFrame, frame);
        if (score < bestMatchScore) {
            bestMatchScore = score;
            bestMatchIndex = i;
        }
    }

    // Only use the detected loop point if it's significantly similar (threshold)
    // If no good match found, use all frames
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

    // Calculate frames to cover exactly one loop
    const totalFrames = Math.max(1, Math.round(singleLoopDuration / frameDelta));

    // Save original state
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

        // Dynamically import gif.js (browser-only)
        const GIF = (await import("gif.js")).default as GIFConstructor;

        // Create offscreen PIXI application
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

        // Configure spine for export
        spine.state.timeScale = 1;
        spine.x = width * CHIBI_OFFSET_X;
        spine.y = height * CHIBI_OFFSET_Y;
        const scale = Math.min(width / EXPORT_WIDTH, height / EXPORT_HEIGHT) * 0.75;
        spine.scale.set(scale);

        // Move spine to offscreen app
        if (spine.parent) {
            spine.parent.removeChild(spine);
        }
        offscreenApp.stage.addChild(spine);

        // Create temp canvas for frame extraction
        tempCanvas = document.createElement("canvas");
        tempCanvas.width = width;
        tempCanvas.height = height;
        const tempCtx = tempCanvas.getContext("2d", { willReadFrequently: true });
        if (!tempCtx) {
            throw new Error("Failed to create 2D context");
        }

        // Reset animation (same as MP4)
        spine.state.clearTracks();
        spine.skeleton.setToSetupPose();
        spine.state.setAnimation(0, animationName, true);
        spine.state.apply(spine.skeleton);
        spine.skeleton.updateWorldTransform();

        // Capture all frames first (matching MP4 approach exactly)
        const frames: ImageData[] = [];

        for (let frame = 0; frame < totalFrames; frame++) {
            checkAborted(signal);

            // Same as MP4: update only after frame 0
            if (frame > 0) {
                spine.state.update(frameDelta);
                spine.state.apply(spine.skeleton);
                spine.skeleton.updateWorldTransform();
            }
            spine.updateTransform();
            offscreenApp.renderer.render(offscreenApp.stage);

            // Clear temp canvas before drawing to avoid artifacts
            tempCtx.clearRect(0, 0, width, height);

            // Extract frame to canvas
            const webglCanvas = offscreenApp.view as HTMLCanvasElement;
            tempCtx.drawImage(webglCanvas, 0, 0);

            // Get image data for gif.js
            const imageData = tempCtx.getImageData(0, 0, width, height);
            frames.push(imageData);

            onProgress?.(((frame + 1) / totalFrames) * 50); // 0-50% for capture

            // Yield to UI every few frames
            if (frame % 5 === 0) {
                await new Promise((resolve) => setTimeout(resolve, 0));
            }
        }

        checkAborted(signal);

        // Create GIF encoder
        gif = new GIF({
            workers: 4,
            quality: 10,
            width,
            height,
            workerScript: "/gif.worker.js",
            repeat: 0, // Loop forever
            transparent: settings.transparentBg ? 0x000000 : null,
            dither: false,
            debug: false,
        });

        // Add abort listener
        const abortHandler = () => {
            gif?.abort();
        };
        signal?.addEventListener("abort", abortHandler);

        // Find the visual loop point - where the animation returns to a pose similar to frame 0
        // This handles animations that have a "hold" period at the end
        const visualLoopPoint = findVisualLoopPoint(frames);
        const loopFrames = frames.slice(0, visualLoopPoint);

        // Add frames to GIF with uniform delay
        // dispose: 2 = restore to background (works better for transparency)
        loopFrames.forEach((frameData) => {
            checkAborted(signal);
            gif?.addFrame(frameData, {
                delay: frameDelayMs,
                copy: true,
                dispose: 2,
            });
        });

        // Render GIF and wait for completion
        const result = await new Promise<RecordingResult>((resolve, reject) => {
            if (!gif) {
                reject(new Error("GIF encoder not initialized"));
                return;
            }

            gif.on("progress", (progress) => {
                onProgress?.(50 + progress * 50); // 50-100% for encoding
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
        // Restore spine to original app
        if (spine.parent) {
            spine.parent.removeChild(spine);
        }
        if (options.liveApp?.stage) {
            options.liveApp.stage.addChild(spine);
        }

        // Restore original state
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

    // Save original state
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

        // Create offscreen PIXI application
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

        // Configure spine for export
        spine.state.timeScale = 1;
        spine.x = width * CHIBI_OFFSET_X;
        spine.y = height * CHIBI_OFFSET_Y;
        const scale = Math.min(width / EXPORT_WIDTH, height / EXPORT_HEIGHT) * 0.75;
        spine.scale.set(scale);

        // Move spine to offscreen app
        if (spine.parent) {
            spine.parent.removeChild(spine);
        }
        offscreenApp.stage.addChild(spine);

        // Create temp canvas for frame extraction
        tempCanvas = document.createElement("canvas");
        tempCanvas.width = width;
        tempCanvas.height = height;
        const tempCtx = tempCanvas.getContext("2d", { willReadFrequently: true });
        if (!tempCtx) {
            throw new Error("Failed to create 2D context");
        }

        // Setup MP4 muxer
        muxer = new Muxer({
            target: new ArrayBufferTarget(),
            video: {
                codec: "avc",
                width,
                height,
            },
            fastStart: "in-memory",
        });

        // Setup video encoder with proper error tracking
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
        let codecLevel = "1f"; // Level 3.1 default
        if (pixels > 921600) {
            codecLevel = "28"; // Level 4.0 for larger resolutions
        }
        if (pixels > 2097152) {
            codecLevel = "33"; // Level 5.1 for very large resolutions
        }

        videoEncoder.configure({
            codec: `avc1.4200${codecLevel}`, // H.264 Baseline with dynamic level
            width,
            height,
            bitrate: 5_000_000,
            framerate: settings.fps,
        });

        // Reset animation
        spine.state.clearTracks();
        spine.skeleton.setToSetupPose();
        spine.state.setAnimation(0, animationName, true); // Loop enabled
        spine.state.apply(spine.skeleton);
        spine.skeleton.updateWorldTransform();

        // Capture frames
        const microsecondsPerFrame = 1_000_000 / settings.fps;

        for (let frame = 0; frame < totalFrames; frame++) {
            checkAborted(signal);

            // Check for encoder errors before each frame
            if (encoderError) {
                throw encoderError;
            }

            // Check encoder state
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

            // Extract frame to canvas
            const webglCanvas = offscreenApp.view as HTMLCanvasElement;
            tempCtx.drawImage(webglCanvas, 0, 0);

            // Create video frame
            const videoFrame = new VideoFrame(tempCanvas, {
                timestamp: frame * microsecondsPerFrame,
                duration: microsecondsPerFrame,
            });

            // Encode frame (keyframe every 30 frames)
            const isKeyframe = frame % 30 === 0;
            videoEncoder.encode(videoFrame, { keyFrame: isKeyframe });
            videoFrame.close();

            onProgress?.(((frame + 1) / totalFrames) * 90); // 0-90% for capture

            // Yield to UI and let encoder process - more frequently for high res
            // This gives the encoder time to process frames without blocking
            if (frame % 2 === 0) {
                await new Promise((resolve) => setTimeout(resolve, 0));
            }
        }

        // Check for any final errors before flushing
        if (encoderError) {
            throw encoderError;
        }

        // Flush encoder - wait for all pending frames to be encoded
        await videoEncoder.flush();

        // Check for errors after flush
        if (encoderError) {
            throw encoderError;
        }

        // Add all chunks to muxer
        for (const { chunk, meta } of encodedChunks) {
            muxer.addVideoChunk(chunk, meta);
        }

        // Finalize muxer
        muxer.finalize();

        onProgress?.(100);

        // Get the final MP4 data
        const { buffer } = muxer.target;
        const blob = new Blob([buffer], { type: "video/mp4" });

        return {
            blob,
            filename: `${animationName}.mp4`,
            mimeType: "video/mp4",
        };
    } finally {
        // Close encoder
        if (videoEncoder && videoEncoder.state !== "closed") {
            videoEncoder.close();
        }

        // Restore spine to original app
        if (spine.parent) {
            spine.parent.removeChild(spine);
        }
        if (options.liveApp?.stage) {
            options.liveApp.stage.addChild(spine);
        }

        // Restore original state
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
