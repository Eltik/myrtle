import { PauseIcon, PlayIcon, StepBackIcon, StepForwardIcon } from "lucide-react";
import type React from "react";
import { useEffect, useRef } from "react";
import { Button } from "#/components/ui/button";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { messages } from "./frame-stepper.messages";

/** Where the clip is: `frame` counts from 0, `seconds` is the time into the clip. */
export interface IFramePosition {
    frame: number;
    total: number;
    seconds: number;
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
    disabled?: boolean;
    className?: string;
}

/**
 * Play/pause and one-frame steps for an animation viewer, with the `,` and
 * `.` shortcuts video players use. Every sprite viewer mounts this one so the
 * controls and keys read the same everywhere.
 */
export function FrameStepper({ paused, onPausedChange, onStep, getPosition, scopeRef, disabled, className }: IFrameStepperProps): React.ReactElement {
    const t: TypedT<typeof messages> = useT("common");
    const readoutRef = useRef<HTMLSpanElement>(null);

    // Latest props for the listeners below, so they bind once per scope.
    const latest = useRef({ paused, onPausedChange, onStep, getPosition, disabled, t });
    latest.current = { paused, onPausedChange, onStep, getPosition, disabled, t };

    const step = (direction: -1 | 1) => {
        const { paused, onPausedChange, onStep, disabled } = latest.current;
        if (disabled) return;
        if (!paused) onPausedChange(true);
        onStep(direction);
    };
    const stepRef = useRef(step);
    stepRef.current = step;

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
            const text = pos ? t("frameStepper.readout", { frame: pos.frame + 1, total: pos.total, seconds: pos.seconds.toFixed(2) }) : "";
            if (text !== shown && readoutRef.current) {
                readoutRef.current.textContent = text;
                shown = text;
            }
            raf = requestAnimationFrame(loop);
        };
        raf = requestAnimationFrame(loop);
        return () => cancelAnimationFrame(raf);
    }, []);

    return (
        <div className={cn("flex items-center gap-1 pointer-coarse:gap-4", className)}>
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
