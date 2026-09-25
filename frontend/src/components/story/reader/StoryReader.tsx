/**
 * THE READER SHELL: the state every surface over the stage reads, and nothing
 * that draws.
 *
 * What this file OWNS is the wiring. The settings document, the audio module,
 * the player, the skip state machine, the chrome's idle timer and the peek
 * grace all live here because they are shared, and each of them has its rule in
 * a pure sibling that is tested without a DOM: `chrome.ts` decides whether the
 * chrome is up, `skip.ts` what a skip does next, `scrub.ts` where a fraction
 * lands, `settings.ts` what a stored document means.
 *
 * What draws lives beside it: `ReaderToolbar` (both pills and the reveal
 * handle), `Scrubber`, `ReaderCards` (the title and end cards), `TextBox`,
 * `Stage`, `Cutscene`, and `useReaderHotkeys` for the key switch. The measured
 * history behind each of those decisions is in `docs/story-reader.md` and the
 * capture notes it links, not repeated here.
 */
import { Link, useNavigate } from "@tanstack/react-router";
import type React from "react";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { AlertDialog, AlertDialogClose, AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogPopup, AlertDialogTitle } from "#/components/ui/alert-dialog";
import { Button } from "#/components/ui/button";
import { useMediaQuery } from "#/hooks/use-media-query";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { createStoryAudio, type StoryAudio } from "#/lib/story/audio";
import type { CameraShake } from "#/lib/story/engine";
import { clamp01 } from "#/lib/story/num";
import { autoPlayDelaySec, type BoxPosition, resolveNickname, useStorySettings } from "#/lib/story/settings";
import { parseStoryText, plainStoryText } from "#/lib/story/text";
import { cn } from "#/lib/utils";
import type { StoryCategory } from "#/types/generated/StoryCategory";
import type { StoryEntry } from "#/types/generated/StoryEntry";
import type { StoryScript } from "#/types/generated/StoryScript";
import type { messages as sharedMessages } from "../shared.messages";
import { BacklogDialog } from "./BacklogDialog";
import { ChapterDialog } from "./ChapterDialog";
import { Cutscene } from "./Cutscene";
import { chromeIdleMs, chromeShown, nextPeek, nextTheater, PEEK_GRACE_MS, revealHandleShown, revealsChrome, theaterConsumes } from "./chrome";
import { loadCustomFont, registerCustomFont } from "./fonts";
import { DARK_FOCUS } from "./glass";
import { EndCard, TitleCard } from "./ReaderCards";
import { FAST_RATIO, ReaderToolbar, ToolbarHandle } from "./ReaderToolbar";
import type { messages } from "./reader.messages";
import { Scrubber } from "./Scrubber";
import { SettingsDialog } from "./SettingsDialog";
import { type CanvasMode, Stage } from "./Stage";
import { nextSkip, SKIP_OFF, SKIP_STEP_MS, type SkipState, skipAdvances, skipRatio } from "./skip";
import { useSpeakerTint } from "./speaker";
import { nextShownLine, type ShownLine } from "./shownLine";
import { TextBox } from "./TextBox";
import { useReaderHotkeys } from "./useReaderHotkeys";
import { useStoryPlayer } from "./useStoryPlayer";
import "./reader.css";

export interface IStoryReaderProps {
    script: StoryScript;
    entry: StoryEntry | null;
    groupName: string;
    category: StoryCategory;
    /** Neighbours within the same group, for the end card. */
    previous: StoryEntry | null;
    next: StoryEntry | null;
    initialHalt?: number;
    /** `?ratio=`: overrides the Playback speed setting for a debugging run. */
    ratioOverride?: number;
    /** `?legacyclamp=1`: the pre-parity 1.5 s clamps, for an A/B against this build. */
    legacyClamp?: boolean;
    /** `?mask=1`: the client's own black outside the 16:9 canvas box, instead of the extend fill. */
    mask?: boolean;
    /** `?canvas=stretch`: the pre-capture canvas mapping, canvas width = stage width. */
    canvasMode?: CanvasMode;
    /** `?plate=0`: draw every body at the 1024-at-203 slot template instead of its own wire plate. */
    plateFromWire?: boolean;
    /** `?firstright=1`: the pre-capture legacy `[character]` sides, first name on the right. */
    firstNameRight?: boolean;
    /**
     * `?video=0`: never play a cutscene, skip every `[Video]`. Kill switch for
     * the cutscene layer, and it sits UNDER the settings toggle: either one off
     * turns cutscenes off.
     */
    video?: boolean;
}

/**
 * The skip confirm asks ONCE per session, as the client does. The flag lives in
 * `sessionStorage` so it survives a reload and a walk from one story to the
 * next inside the tab, and dies with the tab; every access is guarded because
 * a locked-down browser throws on the property itself.
 */
const SKIP_CONFIRMED_KEY = "myrtle.story.skipConfirmed";

/** A stable empty slot map, so a frameless render does not hand the tint hook a new object every time. */
const EMPTY_SLOTS = {};

function readSkipConfirmed(): boolean {
    try {
        return window.sessionStorage.getItem(SKIP_CONFIRMED_KEY) === "1";
    } catch {
        return false;
    }
}

function rememberSkipConfirmed(): void {
    try {
        window.sessionStorage.setItem(SKIP_CONFIRMED_KEY, "1");
    } catch {
        // A session that cannot remember asks again. That is the whole cost.
    }
}

export function StoryReader({ script, entry, groupName, category, previous, next, initialHalt, ratioOverride, legacyClamp, mask, canvasMode, plateFromWire, firstNameRight, video }: IStoryReaderProps): React.ReactElement {
    const t: TypedT<typeof messages> = useT("story");
    const tc: TypedT<typeof sharedMessages> = useT("story");
    const [settings, setSettings] = useStorySettings();
    const navigate = useNavigate();
    // An empty name field is "Doctor", resolved HERE rather than in storage, so
    // clearing it with the settings dialog open reads back on the next line.
    const nickname = resolveNickname(settings.nickname);
    // The chapter's neighbours in index order, already resolved by the route
    // for the end card. `[` and `]` and the two toolbar arrows are the same
    // navigation, so they read the same two entries and are dead at the ends.
    const goNeighbour = useCallback(
        (to: StoryEntry | null) => {
            if (to) void navigate({ to: "/stories/$storyId", params: { storyId: to.id } });
        },
        [navigate],
    );
    const audioRef = useRef<StoryAudio | null>(null);
    if (audioRef.current === null) audioRef.current = createStoryAudio();
    const audio = audioRef.current;

    // A shake carries its own parameters and never blocks, so it rides beside
    // the frame timeline rather than inside it; the sequence restarts it.
    const [shake, setShake] = useState<{ seq: number; params: CameraShake } | null>(null);
    const onShake = useCallback((params: CameraShake) => setShake((s) => ({ seq: (s?.seq ?? 0) + 1, params })), []);
    // SKIP. `confirmed` is seeded from the tab's session, so the reader asks
    // once per session and not once per story.
    const [skip, setSkip] = useState<SkipState>(() => ({ ...SKIP_OFF, confirmed: readSkipConfirmed() }));
    // SKIP is `animateRatio = 0`, the client's own fast-forward, so it rides the
    // one multiplier the engine already has rather than a second timeline.
    const animateRatio = skipRatio(skip, ratioOverride ?? settings.animateRatio);
    // Cutscenes are on unless the URL or the setting says otherwise; the check
    // is against an explicit `false`, so a missing search param is never off.
    const videosOn = video !== false && settings.playVideos !== false;
    const cutsceneLabel = t("reader.cutscene.label");
    const player = useStoryPlayer({ script, storyId: script.id, nickname, audio, initialHalt, animateRatio, legacyClamp, firstNameRight, videos: videosOn, cutsceneLabel, onShake });
    const { phase, halt, frame, playing, haltIndex, totalHalts, backlog, revealKey, advance, choose, back, start, resume, restart, savedHalt, haltSummaries, jumpTo } = player;
    // The box is the SCENE's, not the halt's: a bare `[dialog]` hides it and
    // the next line shows it again, so while a step's frames play out the box
    // follows `dialogVisible` and only the halt's own frame brings it back.
    const boxVisible = frame?.state.dialogVisible ?? true;
    // What the box PRINTS lags the halt through a hide-and-cut (`shownLine.ts`):
    // the old line stays in the fading box and the new plate waits for the box
    // to return. A ref written during render, because a state-and-effect pair
    // would paint the flash for one frame first; the rule is idempotent, so a
    // strict-mode double render lands on the same line.
    const shownRef = useRef<ShownLine | null>(null);
    const shown = halt?.kind === "line" ? nextShownLine(shownRef.current, halt, revealKey, boxVisible) : null;
    shownRef.current = shown;
    // The speaker's ink, read off the LIT sprite in the frame the stage is
    // showing. It is undefined unless the reader asked for it, so the default
    // reader never samples an image and the plate keeps its hashed hue.
    const speakerTint = useSpeakerTint({ speaker: shown?.speaker, slots: frame?.state.slots ?? EMPTY_SLOTS, settings, storyId: script.id });

    const [autoPlay, setAutoPlay] = useState(false);
    // Theater mode: the client's eye button. The text box, both pills and the
    // scrubber go; the scene stays; any tap or key hands them back.
    const [theater, setTheater] = useState(false);
    const [logOpen, setLogOpen] = useState(false);
    const [chapterOpen, setChapterOpen] = useState(false);
    const [settingsOpen, setSettingsOpen] = useState(false);
    const [fullscreen, setFullscreen] = useState(false);
    // The uploaded face is registered per MOUNT: a `FontFace` does not survive
    // a reload, so the bytes come back out of IndexedDB every time. Until it
    // resolves the box keeps the preset family, so a slow read never blanks a line.
    const [customFontLoaded, setCustomFontLoaded] = useState(false);
    // The idle wait with no pointer MOVEMENT. The ref mirrors the state so
    // a mousemove that changes nothing does not re-render the whole reader.
    const [idle, setIdle] = useState(false);
    const idleRef = useRef(false);
    const idleTimer = useRef(0);
    const [pointerOverChrome, setPointerOverChrome] = useState(false);
    // The handle is its own input: it PEEKS at a collapsed toolbar, which the
    // pills themselves must not do, or the chevron would have no visible effect
    // until the pointer left it. The peek is held over the UNION of the handle
    // and the pills it reveals, with `PEEK_GRACE_MS` of grace over the stage
    // between them, so reaching for a revealed pill does not drop the bar;
    // `chrome.ts` carries the measurement and the rule.
    const [pointerOverHandle, setPointerOverHandle] = useState(false);
    const peekTimer = useRef(0);
    const holdPeek = useCallback(() => {
        window.clearTimeout(peekTimer.current);
        setPointerOverHandle(nextPeek("enter"));
    }, []);
    const releasePeek = useCallback(() => {
        window.clearTimeout(peekTimer.current);
        peekTimer.current = window.setTimeout(() => setPointerOverHandle(nextPeek("expire")), PEEK_GRACE_MS);
    }, []);
    const endPeek = useCallback(() => {
        window.clearTimeout(peekTimer.current);
        setPointerOverHandle(nextPeek("open"));
    }, []);
    useEffect(() => () => window.clearTimeout(peekTimer.current), []);
    const toolbarRef = useRef<HTMLDivElement>(null);
    // The reveal is tracked by the KEY of the line that finished, never a boolean:
    // a boolean has to be reset when the line changes, and that reset races the
    // child's notification in either order. Comparing keys cannot race.
    const [revealDoneKey, setRevealDoneKey] = useState(-1);
    // A viewport under 500 px tall is a phone in landscape: the reader covers the
    // site header instead of subtracting it, so the stage gets the whole viewport.
    const shortViewport = useMediaQuery("(max-height: 500px)");
    const revealDone = revealDoneKey === revealKey;
    const [completeSignal, setCompleteSignal] = useState(0);
    const rootRef = useRef<HTMLDivElement>(null);
    const dialogOpen = logOpen || chapterOpen || settingsOpen || skip.phase === "confirming";
    // Under 1024 px the pills are icon-only and every label is a tooltip.
    const compact = useMediaQuery("(max-width: 1023px)");

    // The chrome's auto-hide. `wake` restarts the idle wait and is called ONLY
    // by the inputs `revealsChrome` admits: reading is not one of them, so a
    // Space that turns the page leaves a faded bar faded. The DECISION is
    // `chromeShown`, which is pure and tested.
    const idleMs = chromeIdleMs(settings.toolbarIdleSec);
    const wake = useCallback(() => {
        if (idleRef.current) {
            idleRef.current = false;
            setIdle(false);
        }
        window.clearTimeout(idleTimer.current);
        idleTimer.current = window.setTimeout(() => {
            idleRef.current = true;
            setIdle(true);
        }, idleMs);
    }, [idleMs]);
    useEffect(() => {
        wake();
        return () => window.clearTimeout(idleTimer.current);
    }, [wake]);
    /** A pointer DOWN wakes the chrome only inside the top band, which is how a phone reveals it. */
    const onReaderPointerDown = useCallback(
        (e: React.PointerEvent<HTMLDivElement>) => {
            if (revealsChrome("pointerdown", e.clientY - e.currentTarget.getBoundingClientRect().top)) wake();
        },
        [wake],
    );
    const chromeOn = chromeShown({ theater, collapsed: settings.toolbarHidden, alwaysShow: !settings.autoHideToolbar, dialogOpen, pointerOverChrome, pointerOverHandle, phase, idle });
    const handleOn = revealHandleShown({ theater, collapsed: settings.toolbarHidden });
    /** `T` and the chevron: the one control that hides ONLY the toolbar, and it is remembered. */
    const toggleToolbar = useCallback(() => {
        wake();
        setSettings({ ...settings, toolbarHidden: !settings.toolbarHidden });
    }, [settings, setSettings, wake]);

    // Settings volumes reach the audio channels; everything stops on unmount and on route change.
    useEffect(() => {
        audio.setMasterMusic(settings.musicVolume);
        audio.setMasterSfx(settings.sfxVolume);
        audio.setMuted(settings.muted);
    }, [audio, settings.musicVolume, settings.sfxVolume, settings.muted]);
    useEffect(() => () => audio.dispose(), [audio]);

    // The BGM is held at silence for the length of a cutscene, because the
    // clip carries its own soundtrack and the two played over each other. It
    // is a DUCK and not a mute: the viewer's own mute stays where they left it,
    // and the bus comes back up when the halt changes or the reader unmounts.
    const cutscene = halt?.kind === "video" ? halt : null;
    useEffect(() => {
        if (!cutscene) return;
        audio.setMusicDucked(true);
        return () => audio.setMusicDucked(false);
    }, [audio, cutscene]);

    useEffect(() => {
        if (settings.font !== "custom") {
            setCustomFontLoaded(false);
            return;
        }
        let live = true;
        void loadCustomFont().then(async (stored) => {
            const ok = stored ? await registerCustomFont(stored.data) : false;
            if (live) setCustomFontLoaded(ok);
        });
        return () => {
            live = false;
        };
    }, [settings.font]);

    const onRevealDone = useCallback((key: number) => setRevealDoneKey(key), []);

    // Click on the stage: arm audio, then either skip the timeline, finish the reveal, or advance.
    // Clicks inside reader UI (toolbar, cards, options) carry `data-story-ui` and never advance.
    const onStageClick = useCallback(
        (e?: React.MouseEvent) => {
            if (e && e.target instanceof Element && e.target.closest("[data-story-ui]")) return;
            // Theater mode: a tap gives the UI back and does nothing else. The
            // rule is `theaterConsumes`, so the "does nothing else" is tested.
            if (theaterConsumes(theater, "activity")) {
                setTheater(false);
                return;
            }
            // A click is one of the three things that end a skip, and it ends
            // it INSTEAD of advancing, so a stop never overshoots by a line.
            if (skip.phase === "on") {
                setSkip((v) => nextSkip(v, "click"));
                return;
            }
            audio.arm();
            if (phase === "title") return start();
            if (phase === "resume") return resume();
            if (phase === "end") return;
            if (halt?.kind === "decision") return;
            if (playing) return advance();
            // A cutscene has no line to reveal, so the reveal gate below would
            // swallow every advance key for as long as the clip is on screen.
            if (halt?.kind === "video") return advance();
            if (!revealDone) {
                setCompleteSignal((n) => n + 1);
                return;
            }
            advance();
        },
        [advance, audio, halt, phase, playing, resume, revealDone, skip.phase, start, theater],
    );

    // Auto-play: after the reveal, wait `max(minLineSec, chars / cps * pace)` then advance.
    useEffect(() => {
        if (!autoPlay || phase !== "reading" || !revealDone || playing || halt?.kind !== "line") return;
        const chars = plainStoryText(parseStoryText(halt.text)).length;
        const id = window.setTimeout(() => advance(), autoPlayDelaySec(chars, settings) * 1000);
        return () => window.clearTimeout(id);
    }, [advance, autoPlay, halt, phase, playing, revealDone, settings]);

    // The skip loop: one advance per frame while it runs. `skipAdvances` is the
    // rule and it only ever fires on a LINE of a story being read, so the loop
    // cannot spin on a decision or on the end card.
    useEffect(() => {
        if (!skipAdvances(skip, phase, halt?.kind)) return;
        const id = window.setTimeout(() => advance(), SKIP_STEP_MS);
        return () => window.clearTimeout(id);
    }, [advance, halt, phase, skip]);

    // The two stops that are not a click: a decision, and the end of the story.
    useEffect(() => {
        if (skip.phase !== "on") return;
        if (phase !== "reading") setSkip((v) => nextSkip(v, "boundary"));
        else if (halt?.kind === "decision") setSkip((v) => nextSkip(v, "decision"));
    }, [halt, phase, skip.phase]);

    useReaderHotkeys({
        dialogOpen,
        theater,
        onAdvance: onStageClick,
        onBack: back,
        onAutoPlay: () => setAutoPlay((v) => !v),
        onLog: () => setLogOpen(true),
        onTheaterToggle: () => setTheater((v) => nextTheater(v, "toggle")),
        onTheaterRestore: () => setTheater(false),
        onSkip: () => setSkip((v) => nextSkip(v, "press")),
        onMute: () => setSettings({ ...settings, muted: !settings.muted }),
        onToggleToolbar: toggleToolbar,
        onFullscreen: () => toggleFullscreen(),
        onPrevious: () => goNeighbour(previous),
        onNext: () => goNeighbour(next),
    });

    const toggleFullscreen = () => {
        const el = rootRef.current;
        if (!el) return;
        if (document.fullscreenElement) void document.exitFullscreen();
        else void el.requestFullscreen?.();
    };
    useEffect(() => {
        const onChange = () => setFullscreen(document.fullscreenElement === rootRef.current && rootRef.current !== null);
        document.addEventListener("fullscreenchange", onChange);
        return () => document.removeEventListener("fullscreenchange", onChange);
    }, []);

    const onContextMenu = (e: React.MouseEvent) => {
        e.preventDefault();
        setTheater((v) => nextTheater(v, "toggle"));
    };

    const progress = totalHalts > 0 ? clamp01((haltIndex + 1) / totalHalts) : 0;
    const title = entry?.name ?? script.name;
    const cardMeta = useMemo(() => [groupName, entry?.code, entry?.avgTag].filter((s): s is string => Boolean(s)).join(" · "), [groupName, entry]);

    return (
        // `--story-avail` is the height the stage may fill: the viewport minus the
        // site header (`reader.css` follows the header's own 3.5rem/4rem breakpoint),
        // or the whole viewport in fullscreen and on a short viewport. The STAGE is
        // that tall and the FULL viewport width, always; the CANVAS inside it is the
        // largest 16:9 box, centred, which is what `fit_mode="BLACK_MASK"` makes
        // (`docs/story-reader-captures.md`, 0). The region outside that box is our
        // named trade, extended background by default and the client's black under
        // `?mask=1`; `?canvas=stretch` puts the pre-capture mapping back.
        <div
            ref={rootRef}
            data-story-reader
            data-story-immersive={fullscreen || shortViewport ? "true" : "false"}
            className="relative flex h-[var(--story-avail)] w-full flex-col justify-center overflow-hidden bg-black text-white"
            // Any pointer movement anywhere in the reader, and any tap, wakes
            // the chrome. `onPointerDown` is what covers touch, where there is
            // no move before the tap. It sits on the ROOT and not on the stage
            // so a move that starts over the toolbar or a card counts too.
            onPointerMove={wake}
            onPointerDown={onReaderPointerDown}
        >
            {/* The halt ordinal, for tests and for the browser verification to read. */}
            <span hidden data-testid="story-halt" data-story-halt={haltIndex}>
                {haltIndex}
            </span>

            {/* The STAGE spans the viewport edge to edge; the CANVAS inside it
                is the largest 16:9 box, centred, which is what the client's
                `BLACK_MASK` makes. Outside that box the stage extends the
                background blurred, or paints the client's black under
                `?mask=1` and the "Letterbox like the game" setting. */}
            <div className="relative w-full">
                <Stage frame={frame} shake={shake} canvasMode={canvasMode} fill={mask || settings.letterbox ? "mask" : "extend"} plateFromWire={plateFromWire} onClick={onStageClick} onContextMenu={onContextMenu} label={t("reader.stage.label")} className="h-[var(--story-avail)]">
                    {settings.progressBar ? (
                        <Scrubber
                            shown={chromeOn}
                            summaries={haltSummaries}
                            totalHalts={totalHalts}
                            progress={progress}
                            barRect={() => toolbarRef.current?.getBoundingClientRect()}
                            onPointerEnter={() => setPointerOverChrome(true)}
                            onPointerLeave={() => setPointerOverChrome(false)}
                            onJump={(target) => {
                                audio.arm();
                                jumpTo(target);
                            }}
                        />
                    ) : null}

                    <ReaderToolbar
                        compact={compact}
                        shown={chromeOn}
                        barRef={toolbarRef}
                        onPointerEnter={() => {
                            setPointerOverChrome(true);
                            if (settings.toolbarHidden) holdPeek();
                        }}
                        onPointerLeave={() => {
                            setPointerOverChrome(false);
                            releasePeek();
                        }}
                        backLink={<Link to="/stories" />}
                        hasPrevious={Boolean(previous)}
                        hasNext={Boolean(next)}
                        onPrevious={() => goNeighbour(previous)}
                        onNext={() => goNeighbour(next)}
                        onSettings={() => setSettingsOpen(true)}
                        onLog={() => setLogOpen(true)}
                        onChapter={() => setChapterOpen(true)}
                        theater={theater}
                        onTheater={() => setTheater((v) => nextTheater(v, "toggle"))}
                        onHideToolbar={toggleToolbar}
                        autoPlay={autoPlay}
                        onAutoPlay={() => setAutoPlay((v) => !v)}
                        animateRatio={settings.animateRatio}
                        onSpeed={() => setSettings({ ...settings, animateRatio: settings.animateRatio === FAST_RATIO ? 1 : FAST_RATIO })}
                        skipping={skip.phase === "on"}
                        onSkip={() => setSkip((v) => nextSkip(v, "press"))}
                        fullscreen={fullscreen}
                        onFullscreen={toggleFullscreen}
                        muted={settings.muted}
                        onMute={() => setSettings({ ...settings, muted: !settings.muted })}
                    />

                    {handleOn ? (
                        <ToolbarHandle
                            onPointerEnter={holdPeek}
                            onPointerLeave={releasePeek}
                            onClick={() => {
                                endPeek();
                                setSettings({ ...settings, toolbarHidden: false });
                            }}
                        />
                    ) : null}

                    {phase === "reading" && halt?.kind === "line" && shown ? (
                        <TextBox
                            speaker={shown.speaker}
                            text={shown.text}
                            isNarration={shown.isNarration}
                            revealKey={shown.revealKey}
                            armed={!playing}
                            settings={settings}
                            onRevealDone={onRevealDone}
                            completeSignal={completeSignal}
                            continueLabel={t("reader.line.continue")}
                            hidden={theater || halt.surface !== "box" || !boxVisible}
                            customFontLoaded={customFontLoaded}
                            speakerTint={speakerTint}
                            onPositionChange={(p: BoxPosition) => setSettings({ ...settings, boxX: p.x, boxY: p.y })}
                            dragLabel={t("reader.box.drag")}
                        />
                    ) : null}

                    {phase === "reading" && halt?.kind === "decision" ? (
                        <div className="absolute inset-0 z-10 flex flex-col items-center justify-center gap-3 overflow-y-auto bg-black/35 px-4 py-6 sm:gap-2 sm:px-[10%]" data-story-ui>
                            {halt.options.map((opt, i) => (
                                <button
                                    key={halt.values[i] ?? opt}
                                    type="button"
                                    data-story-option
                                    onClick={() => {
                                        audio.arm();
                                        choose(halt.values[i] ?? opt);
                                    }}
                                    className={cn("min-h-11 w-full max-w-xl rounded-lg border border-white/25 bg-black/70 px-5 py-3 text-center text-white/95 backdrop-blur-md transition-colors hover:border-white/60 hover:bg-white/10", DARK_FOCUS, "focus-visible:ring-2")}
                                    style={{ fontSize: `${settings.textSize / 100}rem` }}
                                >
                                    {plainStoryText(parseStoryText(opt))}
                                </button>
                            ))}
                        </div>
                    ) : null}

                    {phase === "title" || phase === "resume" ? (
                        <TitleCard
                            meta={cardMeta}
                            title={title}
                            category={tc(`category.${category}`)}
                            savedHalt={phase === "resume" ? savedHalt : null}
                            totalHalts={totalHalts}
                            onResume={() => {
                                audio.arm();
                                resume();
                            }}
                            onStart={() => {
                                audio.arm();
                                start();
                            }}
                        />
                    ) : null}

                    {/* The clip's own volume control writes BACK: the reader's
                        Music slider and the player's are the same level, so a
                        viewer who turns a cutscene down finds the soundtrack
                        turned down after it. `simple` has no control to write
                        with, which is why the callback rides the same setting
                        that chose the layer. */}
                    {cutscene ? (
                        <Cutscene
                            key={cutscene.res}
                            sources={cutscene.sources}
                            label={cutsceneLabel}
                            skipLabel={t("reader.cutscene.skip")}
                            playLabel={t("reader.cutscene.play")}
                            onSkip={advance}
                            onEnded={advance}
                            volume={settings.musicVolume}
                            muted={settings.muted}
                            player={settings.cutscenePlayer}
                            onVolumeChange={(musicVolume, isMuted) => setSettings({ ...settings, musicVolume, muted: isMuted })}
                        />
                    ) : null}

                    {phase === "end" ? <EndCard title={title} previous={previous} next={next} onRestart={restart} /> : null}
                </Stage>
            </div>

            <BacklogDialog
                open={logOpen}
                onOpenChange={setLogOpen}
                entries={backlog}
                title={title}
                currentHaltIndex={haltIndex}
                onJump={(target) => {
                    audio.arm();
                    jumpTo(target);
                    setLogOpen(false);
                }}
            />
            <ChapterDialog open={chapterOpen} onOpenChange={setChapterOpen} currentStoryId={script.id} currentCategory={category} currentGroupId={category === "record" ? (entry?.groupId ?? script.groupId) : script.groupId} />
            <SettingsDialog open={settingsOpen} onOpenChange={setSettingsOpen} settings={settings} onChange={setSettings} />
            {/* The client asks before its first skip of a session; so does this,
                in one line, and never again in the same tab. */}
            <AlertDialog open={skip.phase === "confirming"} onOpenChange={(open: boolean) => !open && setSkip((v) => nextSkip(v, "cancel"))}>
                <AlertDialogPopup>
                    <AlertDialogHeader>
                        <AlertDialogTitle>{t("reader.skip.confirmTitle")}</AlertDialogTitle>
                        <AlertDialogDescription>{t("reader.skip.confirmBody")}</AlertDialogDescription>
                    </AlertDialogHeader>
                    <AlertDialogFooter>
                        <AlertDialogClose render={<Button variant="outline" />}>{t("reader.skip.cancel")}</AlertDialogClose>
                        <Button
                            onClick={() => {
                                audio.arm();
                                rememberSkipConfirmed();
                                setSkip((v) => nextSkip(v, "confirm"));
                            }}
                        >
                            {t("reader.skip.confirm")}
                        </Button>
                    </AlertDialogFooter>
                </AlertDialogPopup>
            </AlertDialog>
        </div>
    );
}
