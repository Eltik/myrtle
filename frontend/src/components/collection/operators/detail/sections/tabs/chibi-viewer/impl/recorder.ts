import GIFEncoder from "gif-encoder-2";
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

// ============ GIF RECORDING ============

export async function recordAsGif(options: RecordingOptions): Promise<RecordingResult> {
    const { spine, animationName, onProgress, signal } = options;
    const settings = resolveSettings(options.settings);

    const width = Math.round(EXPORT_WIDTH * settings.scale);
    const height = Math.round(EXPORT_HEIGHT * settings.scale);
    const singleLoopDuration = getAnimationDuration(spine, animationName);
    const totalFrames = Math.ceil(singleLoopDuration * settings.fps);
    const frameDelta = 1 / settings.fps;
    const frameDelayMs = Math.round(1000 / settings.fps);

    // Save original state
    const originalTimeScale = spine.state.timeScale;
    const originalX = spine.x;
    const originalY = spine.y;
    const originalScaleX = spine.scale.x;
    const originalScaleY = spine.scale.y;

    let offscreenApp: PIXI.Application | null = null;
    let tempCanvas: HTMLCanvasElement | null = null;

    try {
        checkAborted(signal);

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

        // Setup GIF encoder
        // Use "neuquant" algorithm for better color accuracy, enable optimizer
        const encoder = new GIFEncoder(width, height, "neuquant", true, totalFrames);
        encoder.setDelay(frameDelayMs);
        encoder.setQuality(10); // 1-20, lower is better quality but slower
        encoder.setRepeat(0); // 0 = loop forever, -1 = no repeat
        encoder.start();

        // Reset animation
        spine.state.clearTracks();
        spine.skeleton.setToSetupPose();
        spine.state.setAnimation(0, animationName, true);
        spine.state.apply(spine.skeleton);
        spine.skeleton.updateWorldTransform();

        // Capture frames
        for (let frame = 0; frame < totalFrames; frame++) {
            checkAborted(signal);

            if (frame > 0) {
                spine.state.update(frameDelta);
                spine.state.apply(spine.skeleton);
                spine.skeleton.updateWorldTransform();
            }
            spine.updateTransform();
            offscreenApp.renderer.render(offscreenApp.stage);

            // Extract frame to canvas
            const webglCanvas = offscreenApp.view as HTMLCanvasElement;
            tempCtx.clearRect(0, 0, width, height);
            tempCtx.drawImage(webglCanvas, 0, 0);

            // Add frame to encoder
            encoder.addFrame(tempCtx);

            onProgress?.(((frame + 1) / totalFrames) * 90); // 0-90% for capture

            // Yield to UI
            if (frame % 5 === 0) {
                await new Promise((resolve) => setTimeout(resolve, 0));
            }
        }

        // Finalize GIF
        encoder.finish();

        onProgress?.(100);

        // Get the final GIF data
        const data = encoder.out.getData();
        // Create a copy with a guaranteed ArrayBuffer type
        const blob = new Blob([new Uint8Array(data)], { type: "image/gif" });

        return {
            blob,
            filename: `${animationName}.gif`,
            mimeType: "image/gif",
        };
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

        // Setup video encoder
        const encodedChunks: { chunk: EncodedVideoChunk; meta?: EncodedVideoChunkMetadata }[] = [];

        videoEncoder = new VideoEncoder({
            output: (chunk, meta) => {
                encodedChunks.push({ chunk, meta });
            },
            error: (e) => {
                throw e;
            },
        });

        videoEncoder.configure({
            codec: "avc1.42001f", // H.264 Baseline
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

            // Yield to UI
            if (frame % 5 === 0) {
                await new Promise((resolve) => setTimeout(resolve, 0));
            }
        }

        // Flush encoder
        await videoEncoder.flush();

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
