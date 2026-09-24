/**
 * EVERY KEY THE READER ANSWERS TO, in one switch.
 *
 * The listener is re-bound on EVERY render and deliberately carries no
 * dependency array: each handler below closes over the reader's current state,
 * and a dependency list would have to name all of it to stay correct. Binding
 * once per render is a `removeEventListener` plus an `addEventListener` per
 * render, which is nothing next to the render itself.
 *
 * Three guards come before the switch and the ORDER is the rule. A dialog or an
 * editable target takes the key outright. A focused button inside the reader's
 * own UI handles Enter and Space itself, or Space on the Settings button would
 * both press it and turn the page. And in theater mode ANY key gives the UI
 * back and does nothing else, `Esc` included, because its toggle resolves to
 * the same restore.
 */
import { useEffect } from "react";
import { isEditableTarget, isPlainKey } from "#/lib/hotkeys";
import { theaterConsumes } from "./chrome";

export interface IReaderHotkeys {
    /** A dialog is open: every key belongs to it. */
    dialogOpen: boolean;
    theater: boolean;
    /** Space, Enter and ArrowRight: the same act as a tap on the stage. */
    onAdvance: () => void;
    onBack: () => void;
    onAutoPlay: () => void;
    /** `H` and `L`, the backlog. */
    onLog: () => void;
    /** `Esc` and the right-click, which toggle theater mode rather than leaving it. */
    onTheaterToggle: () => void;
    /** Theater mode swallowed a key: restore the UI and act on nothing. */
    onTheaterRestore: () => void;
    onSkip: () => void;
    onMute: () => void;
    onToggleToolbar: () => void;
    onFullscreen: () => void;
    /** `[` and `]`, the chapter's neighbours. They are plain keys: a bracket is nothing else in the reader. */
    onPrevious: () => void;
    onNext: () => void;
}

export function useReaderHotkeys(o: IReaderHotkeys): void {
    useEffect(() => {
        const onKey = (e: KeyboardEvent) => {
            if (o.dialogOpen || isEditableTarget(e.target) || !isPlainKey(e)) return;
            if (e.target instanceof Element && e.target.closest("[data-story-ui]") && (e.key === " " || e.key === "Enter")) return;
            if (theaterConsumes(o.theater, "activity")) {
                e.preventDefault();
                o.onTheaterRestore();
                return;
            }
            switch (e.key) {
                case " ":
                case "Enter":
                case "ArrowRight":
                    e.preventDefault();
                    o.onAdvance();
                    break;
                case "ArrowLeft":
                    e.preventDefault();
                    o.onBack();
                    break;
                case "a":
                case "A":
                    o.onAutoPlay();
                    break;
                case "h":
                case "H":
                case "l":
                case "L":
                    o.onLog();
                    break;
                case "Escape":
                    o.onTheaterToggle();
                    break;
                case "s":
                case "S":
                    o.onSkip();
                    break;
                case "m":
                case "M":
                    o.onMute();
                    break;
                case "t":
                case "T":
                    o.onToggleToolbar();
                    break;
                case "f":
                case "F":
                    o.onFullscreen();
                    break;
                case "[":
                    e.preventDefault();
                    o.onPrevious();
                    break;
                case "]":
                    e.preventDefault();
                    o.onNext();
                    break;
            }
        };
        window.addEventListener("keydown", onKey);
        return () => window.removeEventListener("keydown", onKey);
    });
}
