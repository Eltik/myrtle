import { SettingsDialog } from "frontend";
import { type ReactNode, useEffect, useRef } from "react";
import { DEFAULT_SETTINGS } from "../../src/lib/story/settings";

// The reader's Settings sheet: reading style first (it writes the five rows
// under it), then text speed, size, width, spacing and margins, the light box
// and letterbox switches, playback and auto pace, volumes, cutscenes, the
// Doctor's name, font, text colour, speaker tint, box position, toolbar
// behaviour, Export, reading progress and the hotkey list, in one scrolling
// panel. Controlled, so `open` mounts it. The panel is 70dvh tall, so the
// later stories scroll it to show the lower sections the way a reader would.

const noop = () => {};
type Settings = typeof DEFAULT_SETTINGS;

function Stage({ children, scrollTo }: { children: ReactNode; scrollTo?: string }) {
    const ref = useRef<HTMLDivElement>(null);
    useEffect(() => {
        const timers = [60, 180, 400, 800].map((ms) =>
            window.setTimeout(() => {
                (document.activeElement as HTMLElement | null)?.blur();
                if (!scrollTo) return;
                // The popup is portalled: find the row label by its exact text,
                // then scroll the nearest scrolling ancestor (the panel) to it.
                const label = Array.from(document.querySelectorAll<HTMLElement>("span")).find((el) => el.children.length === 0 && el.textContent?.trim() === scrollTo);
                let box = label?.parentElement ?? null;
                while (box && !(box.scrollHeight > box.clientHeight + 4 && getComputedStyle(box).overflowY !== "visible")) box = box.parentElement;
                const row = label?.parentElement?.parentElement ?? label;
                if (row && box) box.scrollTop += row.getBoundingClientRect().top - box.getBoundingClientRect().top - 8;
            }, ms),
        );
        return () => {
            for (const t of timers) window.clearTimeout(t);
        };
    }, [scrollTo]);
    return (
        <div ref={ref} className="relative w-full bg-black" style={{ minHeight: 620 }}>
            {children}
        </div>
    );
}

const Sheet = ({ settings = {}, scrollTo }: { settings?: Partial<Settings>; scrollTo?: string }) => (
    <Stage scrollTo={scrollTo}>
        <SettingsDialog open onOpenChange={noop} settings={{ ...DEFAULT_SETTINGS, ...settings }} onChange={noop} onExport={noop} />
    </Stage>
);

// Opened fresh: the default reading style and the text rows under it.
export const Default = () => <Sheet />;

// Scrolled to the font, text colour and speaker colour rows, with
// OpenDyslexic chosen (the reading style now reads Custom).
export const FontAndColour = () => <Sheet settings={{ font: "dyslexic" }} scrollTo="Font" />;

// Scrolled to the end: toolbar behaviour with the idle fade on (its delay row
// appears), Export, reading progress and the hotkeys.
export const ToolbarAndProgress = () => <Sheet settings={{ autoHideToolbar: true }} scrollTo="Hide toolbar" />;
