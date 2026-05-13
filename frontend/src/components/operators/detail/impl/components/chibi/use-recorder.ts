import type * as PIXI from "pixi.js";
import { useCallback, useEffect, useRef, useState } from "react";
import { downloadBlob } from "#/lib/utils";
import { ANIMATION_SPEED, type IExportSettings } from "./constants";
import type { ExportFormat } from "./recorder";

interface IUseRecorderOptions {
    appRef: React.RefObject<PIXI.Application | null>;
    spineRef: React.RefObject<import("pixi-spine").Spine | null>;
    selectedAnimation: string;
    recordingRef: React.RefObject<boolean>;
}

interface IUseRecorderReturn {
    isRecording: boolean;
    progress: number;
    startRecording: (format: ExportFormat, settings?: Partial<IExportSettings>) => Promise<void>;
    cancelRecording: () => void;
}

export function useRecorder({ appRef, spineRef, selectedAnimation, recordingRef }: IUseRecorderOptions): IUseRecorderReturn {
    const [isRecording, setIsRecording] = useState(false);
    const [progress, setProgress] = useState(0);

    const abortControllerRef = useRef<AbortController | null>(null);
    const isMountedRef = useRef(true);

    const selectedAnimationRef = useRef(selectedAnimation);
    selectedAnimationRef.current = selectedAnimation;

    useEffect(() => {
        isMountedRef.current = true;
        return () => {
            isMountedRef.current = false;
            abortControllerRef.current?.abort();
        };
    }, []);

    const startRecording = useCallback(
        async (format: ExportFormat, settings?: Partial<IExportSettings>) => {
            if (abortControllerRef.current) return;

            const app = appRef.current;
            const spine = spineRef.current;
            const animationName = selectedAnimationRef.current;
            if (!app || !spine || !animationName) return;

            const controller = new AbortController();
            abortControllerRef.current = controller;
            recordingRef.current = true;
            setIsRecording(true);
            setProgress(0);

            let pendingProgress: number | null = null;
            let rafId = 0;
            const flushProgress = () => {
                rafId = 0;
                if (pendingProgress !== null && isMountedRef.current) {
                    setProgress(pendingProgress);
                    pendingProgress = null;
                }
            };
            const onProgress = (p: number) => {
                pendingProgress = p;
                if (rafId === 0) {
                    rafId = requestAnimationFrame(flushProgress);
                }
            };

            try {
                const { recordAsGif, recordAsVideo } = await import("./recorder");
                const record = format === "gif" ? recordAsGif : recordAsVideo;

                const result = await record({
                    liveApp: app,
                    spine,
                    animationName,
                    format,
                    settings,
                    onProgress,
                    signal: controller.signal,
                });

                downloadBlob(result.blob, result.filename);
            } catch (err) {
                if (err instanceof DOMException && err.name === "AbortError") {
                    spine.state.timeScale = ANIMATION_SPEED;
                    spine.state.setAnimation(0, animationName, true);
                } else {
                    console.error("Recording failed:", err);
                }
            } finally {
                if (rafId !== 0) cancelAnimationFrame(rafId);
                abortControllerRef.current = null;
                recordingRef.current = false;
                if (isMountedRef.current) {
                    setIsRecording(false);
                    setProgress(0);
                }
            }
        },
        [appRef, spineRef, recordingRef],
    );

    const cancelRecording = useCallback(() => {
        abortControllerRef.current?.abort();
    }, []);

    return { isRecording, progress, startRecording, cancelRecording };
}
