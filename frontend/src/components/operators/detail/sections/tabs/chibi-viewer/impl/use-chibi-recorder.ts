import type * as PIXI from "pixi.js";
import { useCallback, useRef, useState } from "react";
import type { ExportFormat } from "./recorder";

interface UseChibiRecorderOptions {
    appRef: React.RefObject<PIXI.Application | null>;
    spineRef: React.RefObject<import("pixi-spine").Spine | null>;
    selectedAnimation: string;
    recordingRef: React.RefObject<boolean>;
}

interface UseChibiRecorderReturn {
    isRecording: boolean;
    progress: number;
    startRecording: (format: ExportFormat) => Promise<void>;
    cancelRecording: () => void;
}

export function useChibiRecorder({ appRef, spineRef, selectedAnimation, recordingRef }: UseChibiRecorderOptions): UseChibiRecorderReturn {
    const [isRecording, setIsRecording] = useState(false);
    const [progress, setProgress] = useState(0);
    const abortControllerRef = useRef<AbortController | null>(null);

    const startRecording = useCallback(
        async (format: ExportFormat) => {
            const app = appRef.current;
            const spine = spineRef.current;
            if (!app || !spine || !selectedAnimation) return;

            setIsRecording(true);
            setProgress(0);
            recordingRef.current = true;

            const controller = new AbortController();
            abortControllerRef.current = controller;

            try {
                const { recordAsGif, recordAsVideo } = await import("./recorder");

                const record = format === "gif" ? recordAsGif : recordAsVideo;
                const result = await record({
                    liveApp: app,
                    spine,
                    animationName: selectedAnimation,
                    format,
                    onProgress: setProgress,
                    signal: controller.signal,
                });

                // Trigger download
                const url = URL.createObjectURL(result.blob);
                const a = document.createElement("a");
                a.href = url;
                a.download = result.filename;
                document.body.appendChild(a);
                a.click();
                document.body.removeChild(a);
                URL.revokeObjectURL(url);
            } catch (err) {
                if (err instanceof DOMException && err.name === "AbortError") {
                    // Cancelled by user, restore animation
                    if (spine && app) {
                        try {
                            const { ANIMATION_SPEED } = await import("./constants");
                            spine.state.timeScale = ANIMATION_SPEED;
                            spine.state.setAnimation(0, selectedAnimation, true);
                        } catch {
                            // Best effort restoration
                        }
                    }
                } else {
                    console.error("Recording failed:", err);
                }
            } finally {
                recordingRef.current = false;
                setIsRecording(false);
                setProgress(0);
                abortControllerRef.current = null;
            }
        },
        [appRef, spineRef, selectedAnimation, recordingRef],
    );

    const cancelRecording = useCallback(() => {
        abortControllerRef.current?.abort();
    }, []);

    return { isRecording, progress, startRecording, cancelRecording };
}
