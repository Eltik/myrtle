import { PauseIcon, PlayIcon, StepBackIcon, StepForwardIcon } from "lucide-react";
import type React from "react";
import { useEffect, useRef } from "react";
import { Button } from "#/components/ui/button";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { messages } from "./frame-stepper.messages";

/** Where the clip is: `frame` counts from 0, `seconds` is the time into the clip of `duration` seconds. */
export interface IFramePosition {
    frame: number;
    total: number;
    seconds: number;
    duration: number;
}

interface IFrameStepperProps {
    paused: boolean;
    onPausedChange: (paused: boolean) => void;
    /** Move one frame. Called after playback is paused, so it only has to seek. */
    onStep: (direction: -1 | 1) => void;
    /**
     * Read once per animation frame for the readout, which is written straight
     * to the DOM: a state update per frame would re-render the whole viewer at
     * the display's refresh rate. `null` blanks it (nothing loaded).
     */
    getPosition: () => IFramePosition | null;
    /** The viewer. `,` and `.` step only while the pointer is over it or focus is inside it. */
    scopeRef: React.RefObject<HTMLElement | null>;
    /**
     * Jump to a frame (0-based). Given, a full-width scrubber sits over the buttons; like
     * `onStep` it is called after playback is paused.
     */
    onSeek?: (frame: number) => void;
    /** Further controls (loop, speed) at the end of the button row. */
    children?: React.ReactNode;
    disabled?: boolean;
    className?: string;
}

/**
 * Play/pause and one-frame steps for an animation viewer, with the `,` and
 * `.` shortcuts video players use. Every sprite viewer mounts this one so the
 * controls and keys read the same everywhere.
 */
export function FrameStepper({ paused, onPausedChange, onStep, getPosition, scopeRef, onSeek, children, disabled, className }: IFrameStepperProps): React.ReactElement {
    const t: TypedT<typeof messages> = useT("common");
    const readoutRef = useRef<HTMLSpanElement>(null);
    const scrubberRef = useRef<HTMLInputElement>(null);
    // While the pointer holds the scrubber its value is the user's, not the clip's.
    const scrubbingRef = useRef(false);

    // Latest props for the listeners below, so they bind once per scope.
    const latest = useRef({ paused, onPausedChange, onStep, getPosition, onSeek, disabled, t });
    latest.current = { paused, onPausedChange, onStep, getPosition, onSeek, disabled, t };

    const step = (direction: -1 | 1) => {
        const { paused, onPausedChange, onStep, disabled } = latest.current;
        if (disabled) return;
        if (!paused) onPausedChange(true);
        onStep(direction);
    };
    const stepRef = useRef(step);
    stepRef.current = step;

    const seek = (frame: number) => {
        const { paused, onPausedChange, onSeek, disabled } = latest.current;
        if (disabled || !onSeek) return;
        if (!paused) onPausedChange(true);
        onSeek(frame);
    };

    useEffect(() => {
        const scope = scopeRef.current;
        if (!scope) return;
        let hovered = false;
        const enter = () => {
            hovered = true;
        };
        const leave = () => {
            hovered = false;
        };
        const onKey = (event: KeyboardEvent) => {
            if (event.key !== "," && event.key !== ".") return;
            if (event.ctrlKey || event.metaKey || event.altKey) return;
            if (!hovered && !scope.contains(document.activeElement)) return;
            if (isTypingTarget(event.target)) return;
            event.preventDefault();
            stepRef.current(event.key === "," ? -1 : 1);
        };
        scope.addEventListener("pointerenter", enter);
        scope.addEventListener("pointerleave", leave);
        window.addEventListener("keydown", onKey);
        return () => {
            scope.removeEventListener("pointerenter", enter);
            scope.removeEventListener("pointerleave", leave);
            window.removeEventListener("keydown", onKey);
        };
    }, [scopeRef]);

    useEffect(() => {
        let raf = 0;
        let shown = "";
        const loop = () => {
            const { getPosition, t } = latest.current;
            const pos = getPosition();
            const text = pos ? t("frameStepper.readout", { frame: pos.frame + 1, total: pos.total, seconds: pos.seconds.toFixed(2), duration: pos.duration.toFixed(2) }) : "";
            if (text !== shown && readoutRef.current) {
                readoutRef.current.textContent = text;
                shown = text;
            }
            const scrubber = scrubberRef.current;
            if (scrubber && !scrubbingRef.current) {
                const max = String(Math.max(0, (pos?.total ?? 1) - 1));
                const value = String(pos?.frame ?? 0);
                if (scrubber.max !== max) scrubber.max = max;
                if (scrubber.value !== value) scrubber.value = value;
            }
            raf = requestAnimationFrame(loop);
        };
        raf = requestAnimationFrame(loop);
        return () => cancelAnimationFrame(raf);
    }, []);

    const buttons = (
        <div className="flex flex-wrap items-center gap-1 pointer-coarse:gap-4">
            <ControlButton disabled={disabled} label={t("frameStepper.back")} onClick={() => step(-1)}>
                <StepBackIcon />
            </ControlButton>
            <ControlButton disabled={disabled} label={paused ? t("frameStepper.play") : t("frameStepper.pause")} onClick={() => onPausedChange(!paused)}>
                {paused ? <PlayIcon /> : <PauseIcon />}
            </ControlButton>
            <ControlButton disabled={disabled} label={t("frameStepper.forward")} onClick={() => step(1)}>
                <StepForwardIcon />
            </ControlButton>
            <span className="ml-1 text-muted-foreground text-xs tabular-nums" ref={readoutRef} />
            {children && <div className="ml-auto flex flex-wrap items-center gap-1">{children}</div>}
        </div>
    );

    if (!onSeek) return <div className={className}>{buttons}</div>;

    return (
        <div className={cn("flex flex-col gap-1.5", className)}>
            <input
                aria-label={t("frameStepper.scrub")}
                className="h-4 w-full cursor-pointer accent-primary disabled:cursor-default disabled:opacity-64"
                defaultValue={0}
                disabled={disabled}
                max={0}
                min={0}
                onInput={(event) => seek(Number(event.currentTarget.value))}
                onPointerCancel={() => {
                    scrubbingRef.current = false;
                }}
                onPointerDown={() => {
                    scrubbingRef.current = true;
                }}
                onPointerUp={() => {
                    scrubbingRef.current = false;
                }}
                ref={scrubberRef}
                step={1}
                type="range"
            />
            {buttons}
        </div>
    );
}

/** An icon button named by `label`, as its accessible name and its tooltip both. */
function ControlButton({ label, disabled, onClick, children }: { label: string; disabled: boolean | undefined; onClick: () => void; children: React.ReactNode }): React.ReactElement {
    return (
        <Tooltip>
            <TooltipTrigger render={<Button aria-label={label} disabled={disabled} onClick={onClick} size="icon-xs" variant="ghost" />}>{children}</TooltipTrigger>
            <TooltipPopup>{label}</TooltipPopup>
        </Tooltip>
    );
}

/** A key pressed in a text field, an open select list, a menu or a dialog belongs to that control, not the viewer. */
function isTypingTarget(target: EventTarget | null): boolean {
    if (!(target instanceof HTMLElement)) return false;
    if (target.isContentEditable) return true;
    if (target instanceof HTMLInputElement || target instanceof HTMLTextAreaElement || target instanceof HTMLSelectElement) return true;
    return target.closest("[role=listbox],[role=menu],[role=dialog]") !== null;
}
