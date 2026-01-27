import * as PIXI from "pixi.js";
import { ANIMATION_SPEED, CHIBI_OFFSET_X, CHIBI_OFFSET_Y, CHIBI_SCALE, EXPORT_BG_COLOR, EXPORT_GIF_FPS, EXPORT_HEIGHT, EXPORT_MP4_FPS, EXPORT_WIDTH } from "./constants";

export type ExportFormat = "gif" | "mp4";

export interface RecordingOptions {
    liveApp: PIXI.Application;
    spine: import("pixi-spine").Spine;
    animationName: string;
    format: ExportFormat;
    onProgress?: (progress: number) => void;
    signal?: AbortSignal;
}

export interface RecordingResult {
    blob: Blob;
    filename: string;
    mimeType: string;
}

function getAnimationDuration(spine: import("pixi-spine").Spine, animationName: string): number {
    const anim = spine.spineData.animations.find((a: { name: string }) => a.name === animationName);
    if (!anim) throw new Error(`Animation "${animationName}" not found`);
    return (anim as unknown as { duration: number }).duration;
}

function positionSpine(spine: import("pixi-spine").Spine, width: number, height: number) {
    spine.x = width * CHIBI_OFFSET_X;
    spine.y = height * CHIBI_OFFSET_Y;
    const scale = Math.min(width / 600, height / 400) * CHIBI_SCALE;
    spine.scale.set(scale);
}

function createOffscreenApp(transparent: boolean): PIXI.Application {
    return new PIXI.Application({
        width: EXPORT_WIDTH,
        height: EXPORT_HEIGHT,
        backgroundColor: transparent ? 0x000000 : EXPORT_BG_COLOR,
        backgroundAlpha: transparent ? 0 : 1,
        antialias: true,
        resolution: 1,
        autoDensity: false,
        // Required for canvas.captureStream() and extract to work with WebGL
        preserveDrawingBuffer: true,
        forceCanvas: false,
    });
}

function resetAnimation(spine: import("pixi-spine").Spine, animationName: string) {
    spine.state.setAnimation(0, animationName, false);
    const track = spine.state.tracks[0];
    if (track) {
        track.trackTime = 0;
    }
    // Clear any listeners from the live viewer
    spine.state.clearListeners();
}

export async function recordAsGif(options: RecordingOptions): Promise<RecordingResult> {
    const { liveApp, spine, animationName, onProgress, signal } = options;
    const { GIFEncoder, quantize, applyPalette } = await import("gifenc");

    const rawDuration = getAnimationDuration(spine, animationName);
    const effectiveDuration = rawDuration / ANIMATION_SPEED;
    const totalFrames = Math.ceil(effectiveDuration * EXPORT_GIF_FPS);
    const frameDelta = 1 / EXPORT_GIF_FPS;
    const delay = Math.round(1000 / EXPORT_GIF_FPS);

    const offscreenApp = createOffscreenApp(true);

    try {
        // Re-parent spine to offscreen app
        liveApp.stage.removeChild(spine);
        offscreenApp.stage.addChild(spine);
        positionSpine(spine, EXPORT_WIDTH, EXPORT_HEIGHT);

        // Reset animation
        const savedTimeScale = spine.state.timeScale;
        spine.state.timeScale = ANIMATION_SPEED;
        resetAnimation(spine, animationName);

        const encoder = GIFEncoder();

        for (let frame = 0; frame < totalFrames; frame++) {
            if (signal?.aborted) {
                throw new DOMException("Recording cancelled", "AbortError");
            }

            if (frame > 0) {
                spine.update(frameDelta);
            }

            offscreenApp.renderer.render(offscreenApp.stage);

            // Extract pixels
            const extractedCanvas = offscreenApp.renderer.extract.canvas(offscreenApp.stage) as HTMLCanvasElement;
            const ctx2d = extractedCanvas.getContext("2d");
            if (!ctx2d) throw new Error("Failed to get 2D context for frame extraction");
            const imageData = ctx2d.getImageData(0, 0, extractedCanvas.width, extractedCanvas.height);
            const rgba = imageData.data;

            // Check if the frame has any transparent pixels
            let hasTransparency = false;
            for (let i = 3; i < rgba.length; i += 4) {
                if ((rgba[i] ?? 255) < 128) {
                    hasTransparency = true;
                    break;
                }
            }

            const palette = quantize(rgba, 256);
            const indexedPixels = applyPalette(rgba, palette);

            if (hasTransparency) {
                // Find the darkest palette entry to use as the transparent index
                let transparentIndex = 0;
                let minBrightness = Number.POSITIVE_INFINITY;
                for (let i = 0; i < palette.length; i++) {
                    const entry = palette[i];
                    if (!entry) continue;
                    const brightness = (entry[0] ?? 0) + (entry[1] ?? 0) + (entry[2] ?? 0);
                    if (brightness < minBrightness) {
                        minBrightness = brightness;
                        transparentIndex = i;
                    }
                }

                // Map transparent pixels to the transparent index
                for (let i = 0; i < rgba.length; i += 4) {
                    if ((rgba[i + 3] ?? 255) < 128) {
                        indexedPixels[i / 4] = transparentIndex;
                    }
                }

                encoder.writeFrame(indexedPixels, extractedCanvas.width, extractedCanvas.height, {
                    palette,
                    delay,
                    transparent: true,
                    transparentIndex,
                    dispose: 2, // Restore to background (important for transparency)
                });
            } else {
                encoder.writeFrame(indexedPixels, extractedCanvas.width, extractedCanvas.height, {
                    palette,
                    delay,
                });
            }

            onProgress?.(((frame + 1) / totalFrames) * 100);

            // Yield to main thread every 5 frames
            if (frame % 5 === 0) {
                await new Promise<void>((r) => setTimeout(r, 0));
            }
        }

        encoder.finish();

        // Restore spine to live app
        spine.state.timeScale = savedTimeScale;
        offscreenApp.stage.removeChild(spine);
        liveApp.stage.addChild(spine);
        positionSpine(spine, liveApp.screen.width, liveApp.screen.height);
        spine.state.setAnimation(0, animationName, true);

        return {
            blob: new Blob([encoder.bytes().buffer as ArrayBuffer], { type: "image/gif" }),
            filename: `chibi-${animationName}-${Date.now()}.gif`,
            mimeType: "image/gif",
        };
    } catch (err) {
        // Ensure spine is re-parented even on error
        if (!liveApp.stage.children.includes(spine)) {
            try {
                offscreenApp.stage.removeChild(spine);
            } catch {
                // Spine might already be removed
            }
            liveApp.stage.addChild(spine);
            positionSpine(spine, liveApp.screen.width, liveApp.screen.height);
            spine.state.timeScale = ANIMATION_SPEED;
            spine.state.setAnimation(0, animationName, true);
        }
        throw err;
    } finally {
        offscreenApp.destroy(true, { children: false, texture: false });
    }
}

function getSupportedMimeType(): string {
    const candidates = ["video/mp4;codecs=avc1", "video/mp4", "video/webm;codecs=vp9", "video/webm;codecs=vp8", "video/webm"];
    for (const type of candidates) {
        if (typeof MediaRecorder !== "undefined" && MediaRecorder.isTypeSupported(type)) return type;
    }
    return "video/webm";
}

export async function recordAsVideo(options: RecordingOptions): Promise<RecordingResult> {
    const { liveApp, spine, animationName, onProgress, signal } = options;

    const rawDuration = getAnimationDuration(spine, animationName);
    const effectiveDuration = rawDuration / ANIMATION_SPEED;
    const totalFrames = Math.ceil(effectiveDuration * EXPORT_MP4_FPS);
    const frameDelta = 1 / EXPORT_MP4_FPS;

    const offscreenApp = createOffscreenApp(false);

    try {
        // Re-parent spine
        liveApp.stage.removeChild(spine);
        offscreenApp.stage.addChild(spine);
        positionSpine(spine, EXPORT_WIDTH, EXPORT_HEIGHT);

        const savedTimeScale = spine.state.timeScale;
        spine.state.timeScale = ANIMATION_SPEED;
        resetAnimation(spine, animationName);

        const canvas = offscreenApp.view as HTMLCanvasElement;
        const mimeType = getSupportedMimeType();
        const isMP4 = mimeType.includes("mp4");
        const extension = isMP4 ? "mp4" : "webm";

        // Create capture stream - 0 means manual frame control
        const stream = canvas.captureStream(0);
        const videoTrack = stream.getVideoTracks()[0];
        const canRequestFrame = videoTrack && "requestFrame" in videoTrack;

        const chunks: Blob[] = [];
        const recorder = new MediaRecorder(stream, {
            mimeType,
            videoBitsPerSecond: 5_000_000,
        });

        recorder.ondataavailable = (e) => {
            if (e.data.size > 0) chunks.push(e.data);
        };

        const stopPromise = new Promise<void>((resolve) => {
            recorder.onstop = () => resolve();
        });

        recorder.start();

        if (canRequestFrame) {
            // Frame-by-frame mode (Chrome, Edge, Safari)
            for (let frame = 0; frame < totalFrames; frame++) {
                if (signal?.aborted) {
                    recorder.stop();
                    await stopPromise;
                    throw new DOMException("Recording cancelled", "AbortError");
                }

                if (frame > 0) {
                    spine.update(frameDelta);
                }

                offscreenApp.renderer.render(offscreenApp.stage);
                (videoTrack as unknown as { requestFrame: () => void }).requestFrame();

                onProgress?.(((frame + 1) / totalFrames) * 100);

                // Allow encoder to process
                await new Promise<void>((r) => setTimeout(r, 0));
            }
        } else {
            // Real-time fallback (Firefox)
            const frameInterval = 1000 / EXPORT_MP4_FPS;

            // Re-create with auto-capture since we can't use requestFrame
            const fallbackStream = canvas.captureStream(EXPORT_MP4_FPS);
            // We need to stop the old recorder and start a new one with the auto-capture stream
            recorder.stop();
            await stopPromise;
            chunks.length = 0;

            const fallbackRecorder = new MediaRecorder(fallbackStream, {
                mimeType,
                videoBitsPerSecond: 5_000_000,
            });

            const fallbackChunks: Blob[] = [];
            fallbackRecorder.ondataavailable = (e) => {
                if (e.data.size > 0) fallbackChunks.push(e.data);
            };

            const fallbackStopPromise = new Promise<void>((resolve) => {
                fallbackRecorder.onstop = () => resolve();
            });

            // Reset animation for the fallback path
            resetAnimation(spine, animationName);
            fallbackRecorder.start();

            for (let frame = 0; frame < totalFrames; frame++) {
                if (signal?.aborted) {
                    fallbackRecorder.stop();
                    await fallbackStopPromise;
                    throw new DOMException("Recording cancelled", "AbortError");
                }

                if (frame > 0) {
                    spine.update(frameDelta);
                }

                offscreenApp.renderer.render(offscreenApp.stage);
                onProgress?.(((frame + 1) / totalFrames) * 100);

                await new Promise<void>((r) => setTimeout(r, frameInterval));
            }

            fallbackRecorder.stop();
            await fallbackStopPromise;

            // Restore spine
            spine.state.timeScale = savedTimeScale;
            offscreenApp.stage.removeChild(spine);
            liveApp.stage.addChild(spine);
            positionSpine(spine, liveApp.screen.width, liveApp.screen.height);
            spine.state.setAnimation(0, animationName, true);

            return {
                blob: new Blob(fallbackChunks, { type: mimeType }),
                filename: `chibi-${animationName}-${Date.now()}.${extension}`,
                mimeType,
            };
        }

        recorder.stop();
        await stopPromise;

        // Restore spine to live app
        spine.state.timeScale = savedTimeScale;
        offscreenApp.stage.removeChild(spine);
        liveApp.stage.addChild(spine);
        positionSpine(spine, liveApp.screen.width, liveApp.screen.height);
        spine.state.setAnimation(0, animationName, true);

        return {
            blob: new Blob(chunks, { type: mimeType }),
            filename: `chibi-${animationName}-${Date.now()}.${extension}`,
            mimeType,
        };
    } catch (err) {
        // Ensure spine is re-parented even on error
        if (!liveApp.stage.children.includes(spine)) {
            try {
                offscreenApp.stage.removeChild(spine);
            } catch {
                // Spine might already be removed
            }
            liveApp.stage.addChild(spine);
            positionSpine(spine, liveApp.screen.width, liveApp.screen.height);
            spine.state.timeScale = ANIMATION_SPEED;
            spine.state.setAnimation(0, animationName, true);
        }
        throw err;
    } finally {
        offscreenApp.destroy(true, { children: false, texture: false });
    }
}
