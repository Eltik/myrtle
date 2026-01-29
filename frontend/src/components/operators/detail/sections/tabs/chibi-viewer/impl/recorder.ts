import * as PIXI from "pixi.js";
import { ANIMATION_SPEED, CHIBI_OFFSET_X, CHIBI_OFFSET_Y, CHIBI_SCALE, DEFAULT_EXPORT_SETTINGS, EXPORT_BG_COLOR, EXPORT_HEIGHT, EXPORT_WIDTH, type ExportSettings } from "./constants";

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

function createOffscreenApp(transparent: boolean, scale: number): PIXI.Application {
    const width = Math.round(EXPORT_WIDTH * scale);
    const height = Math.round(EXPORT_HEIGHT * scale);
    return new PIXI.Application({
        width,
        height,
        backgroundColor: transparent ? 0x000000 : EXPORT_BG_COLOR,
        backgroundAlpha: transparent ? 0 : 1,
        antialias: true,
        resolution: 1,
        autoDensity: false,
        preserveDrawingBuffer: true,
        forceCanvas: false,
    });
}

function resetAnimation(spine: import("pixi-spine").Spine, animationName: string) {
    spine.state.clearListeners();
    spine.state.setAnimation(0, animationName, false);
    const track = spine.state.tracks[0];
    if (track) {
        track.trackTime = 0;
    }
    spine.state.apply(spine.skeleton);
    spine.skeleton.updateWorldTransform();
}

export async function recordAsGif(options: RecordingOptions): Promise<RecordingResult> {
    const { liveApp, spine, animationName, settings, onProgress, signal } = options;
    const { GIFEncoder, quantize, applyPalette } = await import("gifenc");

    const mergedSettings = { ...DEFAULT_EXPORT_SETTINGS, ...settings };
    const { scale, fps, transparentBg } = mergedSettings;

    const exportWidth = Math.round(EXPORT_WIDTH * scale);
    const exportHeight = Math.round(EXPORT_HEIGHT * scale);

    const rawDuration = getAnimationDuration(spine, animationName);
    const effectiveDuration = rawDuration / ANIMATION_SPEED;
    const totalFrames = Math.ceil(effectiveDuration * fps);
    const frameDelta = 1 / fps;
    const delay = Math.round(1000 / fps);

    const offscreenApp = createOffscreenApp(transparentBg, scale);

    // Create a separate 2D canvas for pixel extraction (avoid WebGL extract issues)
    const extractCanvas = document.createElement("canvas");
    extractCanvas.width = exportWidth;
    extractCanvas.height = exportHeight;
    const extractCtx = extractCanvas.getContext("2d");
    if (!extractCtx) throw new Error("Failed to create 2D context for extraction");

    try {
        liveApp.stage.removeChild(spine);
        offscreenApp.stage.addChild(spine);
        positionSpine(spine, exportWidth, exportHeight);

        const savedTimeScale = spine.state.timeScale;
        spine.state.timeScale = ANIMATION_SPEED;
        resetAnimation(spine, animationName);

        const encoder = GIFEncoder();
        const pixiCanvas = offscreenApp.view as HTMLCanvasElement;

        for (let frame = 0; frame < totalFrames; frame++) {
            if (signal?.aborted) {
                throw new DOMException("Recording cancelled", "AbortError");
            }

            if (frame > 0) {
                spine.update(frameDelta);
            }

            offscreenApp.renderer.render(offscreenApp.stage);

            // Copy from WebGL canvas to 2D canvas for reliable pixel extraction
            extractCtx.clearRect(0, 0, exportWidth, exportHeight);
            extractCtx.drawImage(pixiCanvas, 0, 0);
            const imageData = extractCtx.getImageData(0, 0, exportWidth, exportHeight);
            const rgba = imageData.data;

            // Check for transparency
            let hasTransparency = false;
            if (transparentBg) {
                for (let i = 3; i < rgba.length; i += 4) {
                    if ((rgba[i] ?? 255) < 128) {
                        hasTransparency = true;
                        break;
                    }
                }
            }

            const palette = quantize(rgba, 256);
            const indexedPixels = applyPalette(rgba, palette);

            if (hasTransparency) {
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

                for (let i = 0; i < rgba.length; i += 4) {
                    if ((rgba[i + 3] ?? 255) < 128) {
                        indexedPixels[i / 4] = transparentIndex;
                    }
                }

                encoder.writeFrame(indexedPixels, exportWidth, exportHeight, {
                    palette,
                    delay,
                    transparent: true,
                    transparentIndex,
                    dispose: 2,
                });
            } else {
                encoder.writeFrame(indexedPixels, exportWidth, exportHeight, {
                    palette,
                    delay,
                });
            }

            onProgress?.(((frame + 1) / totalFrames) * 100);

            if (frame % 5 === 0) {
                await new Promise<void>((r) => setTimeout(r, 0));
            }
        }

        encoder.finish();

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
    const { liveApp, spine, animationName, settings, onProgress, signal } = options;

    const mergedSettings = { ...DEFAULT_EXPORT_SETTINGS, ...settings };
    const { scale, fps } = mergedSettings;

    const exportWidth = Math.round(EXPORT_WIDTH * scale);
    const exportHeight = Math.round(EXPORT_HEIGHT * scale);

    const rawDuration = getAnimationDuration(spine, animationName);
    const effectiveDuration = rawDuration / ANIMATION_SPEED;
    const totalFrames = Math.ceil(effectiveDuration * fps);
    const frameDelta = 1 / fps;
    const frameInterval = 1000 / fps; // ms between frames for proper timing

    const offscreenApp = createOffscreenApp(false, scale);

    try {
        liveApp.stage.removeChild(spine);
        offscreenApp.stage.addChild(spine);
        positionSpine(spine, exportWidth, exportHeight);

        const savedTimeScale = spine.state.timeScale;
        spine.state.timeScale = ANIMATION_SPEED;
        resetAnimation(spine, animationName);

        const canvas = offscreenApp.view as HTMLCanvasElement;
        const mimeType = getSupportedMimeType();
        const isMP4 = mimeType.includes("mp4");
        const extension = isMP4 ? "mp4" : "webm";

        // Use auto-capture at desired FPS for proper frame timing in output
        const stream = canvas.captureStream(fps);

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

        // Record in real-time to ensure proper frame timing
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
            onProgress?.(((frame + 1) / totalFrames) * 100);

            // Wait for the frame interval to ensure proper timing
            await new Promise<void>((r) => setTimeout(r, frameInterval));
        }

        recorder.stop();
        await stopPromise;

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
