/**
 * THE TWO PILLS over the stage, and the handle that brings them back.
 *
 * The grouping is the client's own, read off an emulator capture of the story
 * screen at 2340x1080: three icons at the top left, AUTO and SKIP at the top
 * right, no bar. One flex row holds both clusters, which is what keeps them
 * from overlapping; the row itself takes no pointer events, so the gap between
 * the pills is stage and a click there advances the line.
 *
 * WHEN the pills are on screen is not decided here. `chrome.ts` owns that rule
 * and the reader owns the timer; this file takes `shown` and renders. The three
 * ways the chrome goes (theater mode, the deliberate "Hide toolbar", and the
 * opt-in idle fade) are all written up there.
 */
import { ArrowLeftIcon, BookOpenIcon, ChevronDownIcon, ChevronLeftIcon, ChevronRightIcon, ChevronUpIcon, EyeOffIcon, FastForwardIcon, HistoryIcon, MaximizeIcon, MinimizeIcon, PlayIcon, SettingsIcon, Volume2Icon, VolumeXIcon } from "lucide-react";
import type React from "react";
import { memo, useId, useRef } from "react";
import { Button } from "#/components/ui/button";
import { Popover, PopoverPopup, PopoverTrigger } from "#/components/ui/popover";
import { Slider } from "#/components/ui/slider";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import { useMediaQuery } from "#/hooks/use-media-query";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { VolumeKey } from "#/lib/story/settings";
import { cn } from "#/lib/utils";
import { DARK_FOCUS, PILL, TOUCH_TARGET } from "./glass";
import type { messages } from "./reader.messages";

type ReaderT = TypedT<typeof messages>;

/** The pill's fast setting, and the other half of its 1x/2x toggle. */
export const FAST_RATIO = 2;

/** `1` prints as `1x` and `1.5` as `1.5x`: a trailing `.0` reads as precision that is not there. */
export function formatRatio(ratio: number): string {
    return Number.isInteger(ratio) ? String(ratio) : ratio.toFixed(1);
}

/**
 * A control in one of the pills. At 1024 and up it carries its label beside the
 * icon, which is the register the client uses for AUTO and SKIP; under 1024 it
 * is icon-only, so the label becomes a tooltip on the same target and the
 * accessible name either way.
 */
export function PillButton({
    label,
    state,
    active,
    compact,
    iconOnly,
    disabled,
    hook,
    className,
    onClick,
    render,
    children,
}: {
    label: string;
    state?: string;
    active?: boolean;
    compact: boolean;
    iconOnly?: boolean;
    disabled?: boolean;
    /** A `data-story-*` value, so a control the tests drive has a selector that is not its label. */
    hook?: string;
    className?: string;
    onClick?: () => void;
    render?: React.ReactElement;
    children: React.ReactNode;
}): React.ReactElement {
    const button = (
        <Button
            variant="ghost"
            size="sm"
            aria-label={label}
            aria-pressed={active}
            disabled={disabled}
            data-story-nav={hook}
            title={compact ? undefined : label}
            onClick={onClick}
            render={render}
            className={cn("h-8 min-w-9 justify-center gap-1.5 px-1.5 text-white/85 hover:bg-white/10 hover:text-white max-sm:min-w-8 lg:px-2", TOUCH_TARGET, DARK_FOCUS, active && "bg-white/20 text-white", disabled && "pointer-events-none opacity-35", className)}
        >
            {children}
            {/* Display:none under 1024, so the measured pill width is the
                icon-only one and nothing depends on a media-query hook. */}
            {iconOnly ? null : (
                <span className="hidden text-xs lg:inline">
                    {label}
                    {state ? <span className="ms-1 font-normal text-white/55">· {state}</span> : null}
                </span>
            )}
        </Button>
    );
    if (!compact && !iconOnly) return button;
    return (
        <Tooltip>
            <TooltipTrigger render={button} />
            <TooltipPopup side="bottom">{state ? `${label} · ${state}` : label}</TooltipPopup>
        </Tooltip>
    );
}

/**
 * `useT` is bound HERE rather than taken as a prop, and that is not a style
 * choice: `scripts/i18n-extract.mjs` binds a `t` to its namespace by the
 * `useT(...)` call in the SAME file, so a component handed its translator
 * through a prop takes every key it uses out of the catalogue's reach. Passing
 * it down measured 54 keys newly unused and 59 call sites lost.
 */
export interface IReaderToolbarProps {
    /** Under 1024 px the pills are icon-only and every label is a tooltip. */
    compact: boolean;
    /** Whether the chrome is on screen, from `chromeShown`. Opacity only, so nothing reflows on the way out. */
    shown: boolean;
    /** The scrubber measures the bar's bottom edge off this to place its tooltip. */
    barRef: React.Ref<HTMLDivElement>;
    onPointerEnter: () => void;
    onPointerLeave: () => void;
    /** The link back to the library, so the toolbar needs no router of its own. */
    backLink: React.ReactElement;
    hasPrevious: boolean;
    hasNext: boolean;
    onPrevious: () => void;
    onNext: () => void;
    onSettings: () => void;
    onLog: () => void;
    onChapter: () => void;
    theater: boolean;
    onTheater: () => void;
    onHideToolbar: () => void;
    autoPlay: boolean;
    onAutoPlay: () => void;
    animateRatio: number;
    onSpeed: () => void;
    /** The synopsis sheet is open. */
    skipping: boolean;
    /** Nothing to skip: the title card or the end card is up. */
    skipDisabled?: boolean;
    onSkip: () => void;
    fullscreen: boolean;
    onFullscreen: () => void;
    muted: boolean;
    onMute: () => void;
    musicVolume: number;
    sfxVolume: number;
    /** A volume slider in the mute button's popover moved (`withVolume` is the rule). */
    onVolume: (key: VolumeKey, value: number | readonly number[]) => void;
    /** The volume popover is open, controlled by the reader so it counts as a dialog for the hotkeys. */
    volumeOpen: boolean;
    onVolumeOpenChange: (open: boolean) => void;
}

export function ReaderToolbar(p: IReaderToolbarProps): React.ReactElement {
    const t: ReaderT = useT("story");
    const { compact, shown } = p;
    return (
        // UNDER 640 THE ROW WRAPS, and that is a correction: the claim that both
        // clusters fit on one line at 375 was wrong. The left pill is 232 px and
        // the right 168, which with the 6 px gap is 406 against the 382 px the
        // row has at a 390 px viewport, and the stage clips what runs past it,
        // so the collapse chevron at the right end was off screen and
        // unreachable. Wrapping costs a second 52 px row of chrome over the art
        // and keeps every control at the size it already had; `ms-auto` returns
        // the second line to the right edge, which `justify-between` cannot do
        // with one item on a line.
        <div
            ref={p.barRef}
            className={cn("story-toolbar pointer-events-none absolute inset-x-2 pointer-coarse:top-12 top-2 z-30 flex items-start justify-between gap-1.5 max-sm:inset-x-1 max-sm:flex-wrap", "transition-opacity", shown ? "opacity-100 duration-150" : "opacity-0 duration-200")}
            data-story-toolbar-hidden={shown ? "false" : "true"}
            aria-hidden={!shown}
        >
            <div className={cn(PILL, !shown && "pointer-events-none")} data-story-ui data-story-pill="left" role="toolbar" aria-label={t("reader.toolbar.sceneLabel")} onPointerEnter={p.onPointerEnter} onPointerLeave={p.onPointerLeave}>
                <PillButton compact={compact} label={t("reader.toolbar.back")} render={p.backLink}>
                    <ArrowLeftIcon />
                </PillButton>
                <PillButton compact={compact} label={t("reader.toolbar.settings")} onClick={p.onSettings}>
                    <SettingsIcon />
                </PillButton>
                <PillButton compact={compact} label={t("reader.toolbar.log")} onClick={p.onLog}>
                    <HistoryIcon />
                </PillButton>
                {/* The eye with the slash, the client's own Hide. */}
                <PillButton compact={compact} label={t("reader.toolbar.hide")} active={p.theater} onClick={p.onTheater}>
                    <EyeOffIcon />
                </PillButton>
                <PillButton compact={compact} label={t("reader.toolbar.chapter")} onClick={p.onChapter}>
                    <BookOpenIcon />
                </PillButton>
                {/* The chapter's own order, beside the chapter picker: one story
                    back and one forward, dead at the two ends rather than
                    absent, so the pill does not change width halfway through a
                    chapter. */}
                <PillButton compact={compact} iconOnly disabled={!p.hasPrevious} hook="previous" label={t("reader.toolbar.previousStory")} onClick={p.onPrevious}>
                    <ChevronLeftIcon />
                </PillButton>
                <PillButton compact={compact} iconOnly disabled={!p.hasNext} hook="next" label={t("reader.toolbar.nextStory")} onClick={p.onNext}>
                    <ChevronRightIcon />
                </PillButton>
                {/* The collapse chevron. It is on BOTH pills at 1024 and up;
                    under 1024 only the right pill carries one, because twelve
                    32 px controls and their gaps need 414 px of a 375 px
                    viewport. */}
                <PillButton compact={compact} iconOnly className="hidden lg:inline-flex" label={t("reader.toolbar.hideToolbar")} onClick={p.onHideToolbar}>
                    <ChevronUpIcon />
                </PillButton>
            </div>
            <div className={cn(PILL, "max-sm:ms-auto", !shown && "pointer-events-none")} data-story-ui data-story-pill="right" role="toolbar" aria-label={t("reader.toolbar.playbackLabel")} onPointerEnter={p.onPointerEnter} onPointerLeave={p.onPointerLeave}>
                {/* AUTO carries its state as a second word, the way the client
                    prints "AUTO  OFF", and the control tints while it runs. */}
                <PillButton compact={compact} label={t("reader.toolbar.auto")} state={p.autoPlay ? t("reader.toolbar.on") : t("reader.toolbar.off")} active={p.autoPlay} onClick={p.onAutoPlay}>
                    <PlayIcon />
                </PillButton>
                {/* Playback speed, the 1x/2x half of the setting. It only exists
                    at 1024 and up, where the two pills have the room for it; the
                    slider in Settings is still the full control. */}
                <Button
                    variant="ghost"
                    size="sm"
                    data-story-speed
                    aria-label={t("reader.toolbar.speed")}
                    aria-pressed={p.animateRatio === FAST_RATIO}
                    title={t("reader.toolbar.speed")}
                    onClick={p.onSpeed}
                    className={cn("hidden h-8 min-w-9 justify-center px-2 font-mono text-white/85 text-xs tabular-nums hover:bg-white/10 hover:text-white lg:inline-flex", DARK_FOCUS, p.animateRatio === FAST_RATIO && "bg-white/20 text-white")}
                >
                    {t("settings.multiplier", { value: formatRatio(p.animateRatio) })}
                </Button>
                <PillButton compact={compact} label={t("reader.toolbar.skip")} active={p.skipping} disabled={p.skipDisabled} hook="skip" onClick={p.onSkip}>
                    <FastForwardIcon />
                </PillButton>
                <PillButton compact={compact} label={p.fullscreen ? t("reader.toolbar.exitFullscreen") : t("reader.toolbar.fullscreen")} active={p.fullscreen} onClick={p.onFullscreen}>
                    {p.fullscreen ? <MinimizeIcon /> : <MaximizeIcon />}
                </PillButton>
                <span className="mx-0.5 hidden h-5 w-px bg-white/15 lg:block" />
                <VolumeControl compact={compact} muted={p.muted} musicVolume={p.musicVolume} sfxVolume={p.sfxVolume} volumeOpen={p.volumeOpen} onMute={p.onMute} onVolume={p.onVolume} onVolumeOpenChange={p.onVolumeOpenChange} />
                <PillButton compact={compact} iconOnly label={t("reader.toolbar.hideToolbar")} onClick={p.onHideToolbar}>
                    <ChevronUpIcon />
                </PillButton>
            </div>
        </div>
    );
}

/**
 * The MUTE pill with the volume under it. A click on the pill still mutes; the
 * two sliders (the same Music and Sound volumes Settings has) sit in a popover
 * that opens on HOVER and on keyboard focus for a fine pointer, so turning a
 * loud story down no longer needs the settings dialog.
 *
 * A touch screen has no hover, so there a tap is the mute it always was and a
 * small chevron beside it, 44 px tall like every coarse target in the reader,
 * opens the same popover. The popup is portalled, so it carries
 * `data-story-ui` itself: without it a click on a slider would bubble through
 * the React tree to the stage and turn the page.
 */
type IVolumeControlProps = Pick<IReaderToolbarProps, "compact" | "muted" | "musicVolume" | "sfxVolume" | "volumeOpen" | "onMute" | "onVolume" | "onVolumeOpenChange">;

// Memoised on its own props: the toolbar re-renders on every halt, and a
// base-ui popover trigger is dear enough to render that doing so per step made
// the reader tests 3x slower. The reader keeps the callbacks stable across a step.
const VolumeControl = memo(function VolumeControl(p: IVolumeControlProps): React.ReactElement {
    const t: ReaderT = useT("story");
    const coarse = useMediaQuery("(pointer: coarse)");
    const chevronRef = useRef<HTMLButtonElement>(null);
    const musicId = useId();
    const sfxId = useId();
    const label = p.muted ? t("reader.toolbar.unmute") : t("reader.toolbar.mute");
    const rows: { key: VolumeKey; id: string; label: string; value: number }[] = [
        { key: "musicVolume", id: musicId, label: t("reader.volume.music"), value: p.musicVolume },
        { key: "sfxVolume", id: sfxId, label: t("reader.volume.sound"), value: p.sfxVolume },
    ];
    return (
        <Popover
            open={p.volumeOpen}
            onOpenChange={(open: boolean, details: { reason: string; event: Event }) => {
                // The chevron toggles the popover itself; an outside press ON it
                // would close it here and the chevron's own click reopen it.
                if (!open && details.reason === "outside-press" && details.event.target instanceof Node && chevronRef.current?.contains(details.event.target)) return;
                // A press on the pill is the MUTE, never a popover toggle.
                if (details.reason === "trigger-press") return;
                p.onVolumeOpenChange(open);
            }}
        >
            <PopoverTrigger
                openOnHover={!coarse}
                delay={120}
                closeDelay={250}
                render={
                    <Button
                        variant="ghost"
                        size="sm"
                        aria-label={label}
                        aria-pressed={p.muted}
                        data-story-mute
                        title={p.compact ? undefined : label}
                        className={cn("h-8 min-w-9 justify-center px-1.5 text-white/85 hover:bg-white/10 hover:text-white max-sm:min-w-8 lg:px-2", TOUCH_TARGET, DARK_FOCUS, p.muted && "bg-white/20 text-white")}
                    />
                }
                onClick={(e: React.MouseEvent & { preventBaseUIHandler?: () => void }) => {
                    e.preventBaseUIHandler?.();
                    p.onMute();
                }}
                onFocus={(e: React.FocusEvent<HTMLElement>) => {
                    // Keyboard focus only: a mouse press also focuses the button,
                    // and that press is the mute, not a request for the sliders.
                    if (!coarse && e.currentTarget.matches(":focus-visible")) p.onVolumeOpenChange(true);
                }}
            >
                {p.muted ? <VolumeXIcon /> : <Volume2Icon />}
            </PopoverTrigger>
            {coarse ? (
                <Button
                    ref={chevronRef}
                    variant="ghost"
                    size="sm"
                    aria-label={t("reader.volume.label")}
                    aria-expanded={p.volumeOpen}
                    data-story-volume-toggle
                    onClick={() => p.onVolumeOpenChange(!p.volumeOpen)}
                    className={cn("h-11 w-7 min-w-0 justify-center px-0 text-white/70 hover:bg-white/10 hover:text-white", DARK_FOCUS)}
                >
                    <ChevronDownIcon className={cn("size-3.5 transition-transform", p.volumeOpen && "rotate-180")} />
                </Button>
            ) : null}
            <PopoverPopup side="bottom" align="end" sideOffset={8} initialFocus={false} className="w-60 border-white/15 bg-black/80 text-white backdrop-blur-md" data-story-ui data-story-volume aria-label={t("reader.volume.label")}>
                <div className="flex flex-col gap-3">
                    {rows.map((r) => (
                        <div key={r.key} className="flex flex-col gap-1.5">
                            <div className="flex items-center justify-between text-xs">
                                <span id={r.id} className="text-white/80">
                                    {r.label}
                                </span>
                                <span className="font-mono text-white/60 tabular-nums">{t("settings.percent", { value: Math.round(r.value * 100) })}</span>
                            </div>
                            <Slider aria-labelledby={r.id} className="pointer-coarse:[&_[data-slot=slider-control]]:h-11 pointer-coarse:[&_[data-slot=slider-control]]:items-center" min={0} max={1} step={0.05} value={[r.value]} onValueChange={(v: number | readonly number[]) => p.onVolume(r.key, v)} />
                        </div>
                    ))}
                </div>
            </PopoverPopup>
        </Popover>
    );
});

/**
 * The collapsed toolbar's handle: 24 px of frosted glass at the top edge,
 * centred between the two pills it brings back. It sits ABOVE the scrubber
 * (`z-50`) so revealing the chrome under the pointer does not hand the pointer
 * to the track and drop the reveal again.
 */
export function ToolbarHandle({ onPointerEnter, onPointerLeave, onClick }: { onPointerEnter: () => void; onPointerLeave: () => void; onClick: () => void }): React.ReactElement {
    const t: ReaderT = useT("story");
    return (
        <button
            type="button"
            data-story-ui
            data-story-toolbar-handle
            aria-label={t("reader.toolbar.showToolbar")}
            title={t("reader.toolbar.showToolbar")}
            onPointerEnter={onPointerEnter}
            onPointerLeave={onPointerLeave}
            onClick={onClick}
            className={cn("absolute top-0 left-1/2 z-50 flex h-6 pointer-coarse:h-11 w-16 -translate-x-1/2 items-center justify-center rounded-b-lg bg-black/45 text-white/70 backdrop-blur-sm hover:bg-black/60 hover:text-white", DARK_FOCUS)}
        >
            <ChevronDownIcon className="size-4" />
        </button>
    );
}
