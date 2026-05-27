import { useQuery } from "@tanstack/react-query";
import { AudioLines, ChevronDown, ChevronUp, Download, Layers, Loader2, Pause, Play, Search, Volume2, VolumeX, X } from "lucide-react";
import { Fragment, memo, type ReactNode, useCallback, useEffect, useLayoutEffect, useMemo, useRef, useState } from "react";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "#/components/ui/select";
import { Skeleton } from "#/components/ui/skeleton";
import { Slider } from "#/components/ui/slider";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "#/components/ui/tabs";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import { voicesQueryOptions } from "#/lib/api/voices";
import { cn } from "#/lib/utils";
import type { AudioCategory, IOperatorAudio, IOperatorListItem } from "#/types/operators";
import type { IVoice, LangType } from "#/types/voices";
import { audioURL } from "../../assets";
import { SFX_CATEGORY_LABELS, SFX_CATEGORY_ORDER, SFX_EVENT_LABELS, VOICE_CATEGORY_MAP, VOICE_CATEGORY_ORDER, VOICE_LANGUAGE_LABELS, VOICE_LANGUAGE_ORDER, VOICE_LANGUAGE_SHORT } from "../../constants";

const ALL_CATEGORY_ID = "all";

type AudioMode = "voice" | "sfx";

function sanitize(name: string): string {
    return name.replace(/[^a-zA-Z0-9\-_]/g, "_");
}

/** Extension of a clip URL (without query string), falling back when absent. */
function fileExtension(url: string, fallback: string): string {
    return url.split(".").pop()?.split("?")[0] ?? fallback;
}

function operatorDisplayName(operator: IOperatorListItem): string {
    return operator.name ?? operator.id ?? "operator";
}

// =============================================================================
// Shared audio engine - a single <audio> element + transport state reused by
// both the voice-lines and battle-SFX panels so only one clip plays at a time
// and the volume setting persists when switching between them.
// =============================================================================

interface IPlayer {
    playingId: string | null;
    playingUrl: string | null;
    progress: number;
    downloadingId: string | null;
    erroredURLs: Set<string>;
    play: (id: string, url: string) => void;
    download: (id: string, url: string, filename: string) => void;
}

function useAudioPlayer() {
    const audioRef = useRef<HTMLAudioElement | null>(null);
    const [playingId, setPlayingId] = useState<string | null>(null);
    const [playingUrl, setPlayingUrl] = useState<string | null>(null);
    const [progress, setProgress] = useState(0);
    const [volume, setVolume] = useState(80);
    const [isMuted, setIsMuted] = useState(false);
    const [downloadingId, setDownloadingId] = useState<string | null>(null);
    const [erroredURLs, setErroredURLs] = useState<Set<string>>(() => new Set());

    useEffect(() => {
        const a = audioRef.current;
        if (a) a.volume = isMuted ? 0 : volume / 100;
    }, [volume, isMuted]);

    const markErrored = useCallback((url: string) => {
        setErroredURLs((prev) => {
            if (prev.has(url)) return prev;
            const next = new Set(prev);
            next.add(url);
            return next;
        });
    }, []);

    const resetPlayback = useCallback(() => {
        setPlayingId(null);
        setPlayingUrl(null);
        setProgress(0);
    }, []);

    const stop = useCallback(() => {
        const a = audioRef.current;
        if (a) {
            a.pause();
            a.removeAttribute("src");
            a.load();
        }
        resetPlayback();
    }, [resetPlayback]);

    const play = useCallback(
        (id: string, url: string) => {
            const a = audioRef.current;
            if (!a) return;
            if (playingId === id && playingUrl === url) {
                a.pause();
                resetPlayback();
                return;
            }
            if (erroredURLs.has(url)) return;
            a.src = url;
            setProgress(0);
            setPlayingId(id);
            setPlayingUrl(url);
            a.play().catch(() => {
                markErrored(url);
                resetPlayback();
            });
        },
        [playingId, playingUrl, erroredURLs, markErrored, resetPlayback],
    );

    const download = useCallback(
        async (id: string, url: string, filename: string) => {
            if (erroredURLs.has(url)) return;
            setDownloadingId(id);
            try {
                const res = await fetch(url);
                if (!res.ok) {
                    markErrored(url);
                    throw new Error(`Audio file unavailable (${res.status})`);
                }
                const blob = await res.blob();
                const objectURL = URL.createObjectURL(blob);
                const link = document.createElement("a");
                link.href = objectURL;
                link.download = filename;
                document.body.appendChild(link);
                link.click();
                document.body.removeChild(link);
                URL.revokeObjectURL(objectURL);
            } catch (e) {
                console.error("Failed to download audio", e);
            } finally {
                setDownloadingId(null);
            }
        },
        [erroredURLs, markErrored],
    );

    const player: IPlayer = { playingId, playingUrl, progress, downloadingId, erroredURLs, play, download };

    return { audioRef, player, volume, setVolume, isMuted, setIsMuted, setProgress, markErrored, resetPlayback, stop };
}

interface IAudioContentProps {
    operator: IOperatorListItem;
}

export const AudioContent = memo(function AudioContent({ operator }: IAudioContentProps) {
    const { audioRef, player, volume, setVolume, isMuted, setIsMuted, setProgress, markErrored, resetPlayback, stop } = useAudioPlayer();

    const hasSfx = (operator.audio?.length ?? 0) > 0;
    const [mode, setMode] = useState<AudioMode>("voice");

    // Stop playback when switching panels or moving to another operator so a
    // hidden clip never keeps playing without a visible control.
    // biome-ignore lint/correctness/useExhaustiveDependencies: intentionally reset on mode/operator change only
    useEffect(() => {
        stop();
    }, [mode, operator.id]);

    return (
        <div className="min-w-0 overflow-hidden p-4 md:p-6">
            {/* biome-ignore lint/a11y/useMediaCaption: game audio clips have no transcript track available */}
            <audio
                ref={audioRef}
                preload="auto"
                onTimeUpdate={(e) => {
                    const el = e.currentTarget;
                    if (el.duration > 0) setProgress((el.currentTime / el.duration) * 100);
                }}
                onEnded={resetPlayback}
                onError={() => {
                    const src = audioRef.current?.currentSrc;
                    if (src) markErrored(src);
                    resetPlayback();
                }}
            />

            <div className="mb-6 flex flex-wrap items-start justify-between gap-4">
                <div>
                    <h2 className="font-semibold text-foreground text-xl">Audio / SFX</h2>
                    <p className="text-muted-foreground text-sm">Operator voice lines and battle sound effects.</p>
                </div>
                <VolumeControl volume={volume} setVolume={setVolume} isMuted={isMuted} setIsMuted={setIsMuted} />
            </div>

            {hasSfx && (
                <Tabs value={mode} onValueChange={(v) => setMode(String(v) as AudioMode)} className="mb-6 w-full">
                    <TabsList>
                        <TabsTrigger value="voice" className="gap-1.5">
                            <Volume2 className="h-4 w-4" />
                            Voice Lines
                        </TabsTrigger>
                        <TabsTrigger value="sfx" className="gap-1.5">
                            <AudioLines className="h-4 w-4" />
                            Battle SFX
                        </TabsTrigger>
                    </TabsList>
                </Tabs>
            )}

            {mode === "sfx" && hasSfx ? <SfxPanel operator={operator} player={player} /> : <VoiceLinesPanel operator={operator} player={player} />}
        </div>
    );
});

function VolumeControl({ volume, setVolume, isMuted, setIsMuted }: { volume: number; setVolume: (v: number) => void; isMuted: boolean; setIsMuted: (fn: (m: boolean) => boolean) => void }) {
    return (
        <div className="flex items-center gap-2">
            <button type="button" aria-label={isMuted ? "Unmute" : "Mute"} onClick={() => setIsMuted((m) => !m)} className="rounded-md p-1.5 text-muted-foreground hover:bg-accent hover:text-foreground">
                {isMuted ? <VolumeX className="h-4 w-4" /> : <Volume2 className="h-4 w-4" />}
            </button>
            <Slider className="w-24" min={0} max={100} step={1} value={[volume]} onValueChange={(v) => setVolume(Array.isArray(v) ? (v[0] ?? 80) : v)} aria-label="Volume" />
        </div>
    );
}

// =============================================================================
// Shared row primitives
// =============================================================================

/** Small uppercase status/category badge. */
function Pill({ children, className }: { children: ReactNode; className?: string }) {
    return <span className={cn("rounded bg-secondary/60 px-1.5 py-0.5 text-[10px] text-muted-foreground uppercase tracking-wide", className)}>{children}</span>;
}

/** Centered placeholder shown when a list has no items / matches. */
function EmptyState({ children }: { children: ReactNode }) {
    return <div className="py-12 text-center text-muted-foreground text-sm">{children}</div>;
}

interface ICategoryTab {
    id: string;
    label: string;
    count: number;
}

/** Scrollable underline tab bar with per-tab counts, shared by both panels. */
function CategoryTabsList({ tabs }: { tabs: ICategoryTab[] }) {
    return (
        <div className="mb-4 overflow-x-auto overflow-y-hidden">
            <TabsList variant="underline">
                {tabs.map((t) => (
                    <TabsTrigger key={t.id} value={t.id}>
                        {t.label}
                        <span className="ml-1.5 text-muted-foreground text-xs">({t.count})</span>
                    </TabsTrigger>
                ))}
            </TabsList>
        </div>
    );
}

/**
 * The card chrome shared by every audio entry: play/pause + download controls,
 * the playing/unavailable styling, and the bottom progress bar. The row body
 * (titles, badges, expanders) is supplied via `children`.
 */
interface IAudioRowProps {
    isPlaying: boolean;
    isUnavailable: boolean;
    isDownloading: boolean;
    progress: number;
    downloadLabel: string;
    onPlay: () => void;
    onDownload: () => void;
    children: ReactNode;
}

function AudioRow({ isPlaying, isUnavailable, isDownloading, progress, downloadLabel, onPlay, onDownload, children }: IAudioRowProps) {
    return (
        <div className={cn("group relative overflow-hidden rounded-lg border transition-colors", isPlaying ? "border-primary bg-primary/10" : "border-border bg-card/30 hover:bg-secondary/30", isUnavailable && "opacity-60")}>
            <div className="flex items-start gap-3 p-3">
                <div className="flex shrink-0 gap-1">
                    <button
                        type="button"
                        onClick={onPlay}
                        disabled={isUnavailable && !isPlaying}
                        aria-label={isPlaying ? "Pause" : isUnavailable ? "Unavailable" : "Play"}
                        className={cn("flex h-9 w-9 items-center justify-center rounded-md transition-colors disabled:cursor-not-allowed disabled:opacity-50", isPlaying ? "bg-primary text-primary-foreground" : "bg-secondary text-foreground hover:bg-secondary/80")}
                    >
                        {isPlaying ? <Pause className="h-4 w-4" /> : <Play className="h-4 w-4" />}
                    </button>
                    <Tooltip>
                        <TooltipTrigger
                            render={(props) => (
                                <button
                                    type="button"
                                    onClick={onDownload}
                                    disabled={isUnavailable || isDownloading}
                                    aria-label={downloadLabel}
                                    className="flex h-9 w-9 items-center justify-center rounded-md text-muted-foreground hover:bg-accent hover:text-foreground disabled:cursor-not-allowed disabled:opacity-50"
                                    {...props}
                                >
                                    {isDownloading ? <Loader2 className="h-4 w-4 animate-spin" /> : <Download className="h-4 w-4" />}
                                </button>
                            )}
                        />
                        <TooltipPopup>Download</TooltipPopup>
                    </Tooltip>
                </div>
                <div className="min-w-0 flex-1">{children}</div>
            </div>
            {isPlaying && (
                <div className="absolute inset-x-0 bottom-0 h-1 bg-primary/20">
                    <div className="h-full bg-primary" style={{ width: `${progress}%` }} />
                </div>
            )}
        </div>
    );
}

// =============================================================================
// Voice lines (from the /static/voices endpoint)
// =============================================================================

interface IVoiceCategory {
    id: string;
    name: string;
    lines: IVoice[];
}

function VoiceLinesPanel({ operator, player }: { operator: IOperatorListItem; player: IPlayer }) {
    const { data: voicesData, isLoading } = useQuery(voicesQueryOptions());

    const operatorVoices: IVoice[] = useMemo(() => {
        if (!voicesData) return [];
        // For Amiya, every voice line has `charId == "char_002_amiya"` regardless
        // of form, but the line's key (`charWordId`) is prefixed with the
        // form-specific id (e.g. `char_1001_amiya2_CN_001`). Filter by that
        // prefix so each form only shows its own lines.
        const prefix = `${operator.id}_`;
        return Object.values(voicesData.charWords).filter((v) => v.charWordId?.startsWith(prefix));
    }, [voicesData, operator.id]);

    const availableLanguages = useMemo(() => {
        const set = new Set<LangType>();
        for (const v of operatorVoices) {
            for (const l of v.languages ?? []) set.add(l);
        }
        return VOICE_LANGUAGE_ORDER.filter((l) => set.has(l));
    }, [operatorVoices]);

    const [selectedLanguage, setSelectedLanguage] = useState<LangType>("JP");
    useEffect(() => {
        if (availableLanguages.length > 0 && !availableLanguages.includes(selectedLanguage)) {
            setSelectedLanguage(availableLanguages[0]);
        }
    }, [availableLanguages, selectedLanguage]);

    const voiceActor = useMemo(() => {
        const dict = voicesData?.voiceLangDict?.[operator.id ?? ""]?.dict?.[selectedLanguage];
        return dict?.cvName?.join(", ") ?? null;
    }, [voicesData, operator.id, selectedLanguage]);

    const categories: IVoiceCategory[] = useMemo(() => {
        const map = new Map<string, IVoice[]>();
        for (const v of operatorVoices) {
            const name = VOICE_CATEGORY_MAP[v.placeType] ?? "Other";
            const id = name.toLowerCase().replace(/\s+/g, "-");
            const list = map.get(id) ?? [];
            list.push(v);
            map.set(id, list);
        }
        const ordered: IVoiceCategory[] = [];
        for (const name of VOICE_CATEGORY_ORDER) {
            const id = name.toLowerCase().replace(/\s+/g, "-");
            const list = map.get(id);
            if (list?.length) ordered.push({ id, name, lines: list });
        }
        if (ordered.length > 0) {
            ordered.unshift({ id: ALL_CATEGORY_ID, name: "All", lines: operatorVoices });
        }
        return ordered;
    }, [operatorVoices]);

    const [searchQuery, setSearchQuery] = useState("");
    const normalizedQuery = searchQuery.trim().toLowerCase();

    const filteredCategories: IVoiceCategory[] = useMemo(() => {
        if (!normalizedQuery) return categories;
        return categories.map((c) => ({
            ...c,
            lines: c.lines.filter((v) => v.voiceTitle.toLowerCase().includes(normalizedQuery) || v.voiceText.toLowerCase().includes(normalizedQuery)),
        }));
    }, [categories, normalizedQuery]);

    const [activeCategory, setActiveCategory] = useState<string | null>(null);
    useEffect(() => {
        if (categories.length > 0 && !categories.some((c) => c.id === activeCategory)) {
            setActiveCategory(categories[0].id);
        }
    }, [categories, activeCategory]);

    const operatorName = operatorDisplayName(operator);

    const playVoice = (voice: IVoice) => {
        if (!voice.id) return;
        const url = voice.data?.find((d) => d.language === selectedLanguage)?.voiceUrl;
        if (!url) return;
        player.play(voice.id, audioURL(url));
    };

    const downloadVoice = (voice: IVoice) => {
        const url = voice.data?.find((d) => d.language === selectedLanguage)?.voiceUrl;
        if (!url || !voice.id) return;
        const langLabel = VOICE_LANGUAGE_LABELS[selectedLanguage] ?? selectedLanguage;
        player.download(voice.id, audioURL(url), `${sanitize(operatorName)}_${sanitize(voice.voiceTitle)}_${sanitize(langLabel)}.${fileExtension(url, "mp3")}`);
    };

    if (isLoading) return <AudioSkeleton />;

    return (
        <div>
            <div className="mb-6 space-y-3">
                {voiceActor && (
                    <p className="text-muted-foreground text-sm">
                        <span className="text-foreground/60">CV:</span> <span className="font-medium text-foreground">{voiceActor}</span>
                    </p>
                )}
                <div className="flex flex-wrap items-center gap-4">
                    <Select value={selectedLanguage} onValueChange={(v) => setSelectedLanguage(String(v) as LangType)}>
                        <SelectTrigger className="w-48">
                            <SelectValue placeholder="Language" />
                        </SelectTrigger>
                        <SelectContent>
                            {availableLanguages.map((lang) => (
                                <SelectItem key={lang} value={lang}>
                                    {VOICE_LANGUAGE_LABELS[lang] ?? lang}
                                </SelectItem>
                            ))}
                        </SelectContent>
                    </Select>
                    <SearchBox value={searchQuery} onChange={setSearchQuery} placeholder="Search voice lines..." label="Search voice lines" />
                </div>
            </div>

            {categories.length > 0 && activeCategory ? (
                <Tabs value={activeCategory} onValueChange={(v) => setActiveCategory(String(v))} className="w-full">
                    <CategoryTabsList tabs={filteredCategories.map((c) => ({ id: c.id, label: c.name, count: c.lines.length }))} />
                    {filteredCategories.map((c) => (
                        <TabsContent key={c.id} value={c.id} className="mt-0">
                            {c.lines.length === 0 ? (
                                <EmptyState>No voice lines match &ldquo;{searchQuery}&rdquo;.</EmptyState>
                            ) : (
                                <div className="max-h-112 space-y-2 overflow-y-auto pr-2">
                                    {c.lines.map((voice) => {
                                        const url = voice.data?.find((d) => d.language === selectedLanguage)?.voiceUrl;
                                        const fullURL = url ? audioURL(url) : null;
                                        const isUnavailable = !url || (fullURL !== null && player.erroredURLs.has(fullURL));
                                        const categoryLabel = c.id === ALL_CATEGORY_ID ? (VOICE_CATEGORY_MAP[voice.placeType] ?? "Other") : null;
                                        return (
                                            <VoiceLineRow
                                                key={voice.id ?? voice.charWordId}
                                                voice={voice}
                                                isPlaying={player.playingId === voice.id}
                                                progress={player.progress}
                                                isDownloading={player.downloadingId === voice.id}
                                                isUnavailable={isUnavailable}
                                                categoryLabel={categoryLabel}
                                                highlight={normalizedQuery}
                                                onPlay={() => playVoice(voice)}
                                                onDownload={() => downloadVoice(voice)}
                                            />
                                        );
                                    })}
                                </div>
                            )}
                        </TabsContent>
                    ))}
                </Tabs>
            ) : (
                <EmptyState>No voice data available for this operator.</EmptyState>
            )}
        </div>
    );
}

interface IVoiceLineRowProps {
    voice: IVoice;
    isPlaying: boolean;
    progress: number;
    isDownloading: boolean;
    isUnavailable: boolean;
    categoryLabel: string | null;
    highlight: string;
    onPlay: () => void;
    onDownload: () => void;
}

function VoiceLineRow({ voice, isPlaying, progress, isDownloading, isUnavailable, categoryLabel, highlight, onPlay, onDownload }: IVoiceLineRowProps) {
    const hasQuery = highlight.length > 0;
    const [isExpanded, setIsExpanded] = useState(false);
    const [canExpand, setCanExpand] = useState(false);
    const textRef = useRef<HTMLParagraphElement | null>(null);

    const showFull = isExpanded || isPlaying || hasQuery;

    useLayoutEffect(() => {
        if (showFull) return;
        const el = textRef.current;
        if (!el) return;
        const measure = () => {
            setCanExpand(el.scrollHeight > el.clientHeight + 1);
        };
        measure();
        const ro = new ResizeObserver(measure);
        ro.observe(el);
        return () => ro.disconnect();
    }, [showFull]);

    const showToggle = isExpanded || (canExpand && !isPlaying && !hasQuery);

    return (
        <AudioRow isPlaying={isPlaying} isUnavailable={isUnavailable} isDownloading={isDownloading} progress={progress} downloadLabel="Download voice line" onPlay={onPlay} onDownload={onDownload}>
            <div className="flex flex-wrap items-center gap-2">
                <div className="font-medium text-foreground text-sm">
                    <HighlightedText text={voice.voiceTitle} query={highlight} />
                </div>
                {categoryLabel && <Pill>{categoryLabel}</Pill>}
                {isUnavailable && <Pill className="bg-muted">Unavailable</Pill>}
            </div>
            <p ref={textRef} className={cn("text-muted-foreground text-xs duration-300", showFull ? "" : "line-clamp-2")}>
                <HighlightedText text={voice.voiceText} query={highlight} />
            </p>
            {showToggle && (
                <button type="button" onClick={() => setIsExpanded((v) => !v)} aria-expanded={isExpanded} className="mt-1 inline-flex items-center gap-1 rounded font-medium text-[11px] text-muted-foreground transition-colors hover:text-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring">
                    {isExpanded ? (
                        <>
                            Show less <ChevronUp className="h-3 w-3" />
                        </>
                    ) : (
                        <>
                            Show more <ChevronDown className="h-3 w-3" />
                        </>
                    )}
                </button>
            )}
        </AudioRow>
    );
}

// =============================================================================
// Battle SFX (from operator.audio - resolved from the game's FSB5 banks)
// =============================================================================

interface ISfxLanguageTrack {
    lang: LangType;
    variants: string[];
}

interface ISfxItem {
    id: string;
    category: AudioCategory;
    label: string;
    /** Raw battle event token, shown as the technical detail line. */
    event: string;
    skillSlot: number | null;
    skinVariant: boolean;
    /** Resolved clip URLs for non-voice SFX (one bank may carry random variants). */
    variants: string[];
    /** Per-language tracks for in-battle voice barks; null for plain SFX. */
    languages: ISfxLanguageTrack[] | null;
}

function humanizeEvent(event: string): string {
    return (
        SFX_EVENT_LABELS[event] ??
        event
            .replace(/^ON_/, "")
            .toLowerCase()
            .replace(/_/g, " ")
            .replace(/\b\w/g, (c) => c.toUpperCase())
    );
}

function sfxLabel(entry: IOperatorAudio): string {
    if (entry.category === "skill") return entry.skillSlot ? `Skill ${entry.skillSlot}` : "Skill";
    if (entry.category === "deploy") return "Deployment";
    return humanizeEvent(entry.event);
}

/** Banks for alternate-skin forms carry a `#` segment (e.g. `…iteration#2`). */
function isSkinVariant(bankName: string): boolean {
    return bankName.includes("#");
}

function buildSfxItems(operator: IOperatorListItem): ISfxItem[] {
    const audio = operator.audio ?? [];
    const items: ISfxItem[] = [];
    const seenSignatures = new Set<string>();

    interface IVoiceAcc {
        key: string;
        event: string;
        skillSlot: number | null;
        skinVariant: boolean;
        langs: Map<LangType, string[]>;
    }
    const voiceGroups = new Map<string, IVoiceAcc>();

    for (const entry of audio) {
        const urls = entry.sounds.flatMap((s) => s.urls).map(audioURL);
        if (urls.length === 0) continue;

        if (entry.category === "voice" && entry.language) {
            const groupKey = entry.bankName.replace(/@[^.]*$/, "");
            const acc = voiceGroups.get(groupKey) ?? {
                key: groupKey,
                event: entry.event,
                skillSlot: entry.skillSlot ?? null,
                skinVariant: isSkinVariant(entry.bankName),
                langs: new Map<LangType, string[]>(),
            };
            if (!acc.langs.has(entry.language)) acc.langs.set(entry.language, urls);
            voiceGroups.set(groupKey, acc);
            continue;
        }

        const signature = `${entry.category}|${entry.skillSlot ?? ""}|${entry.event}|${urls.join(",")}`;
        if (seenSignatures.has(signature)) continue;
        seenSignatures.add(signature);

        items.push({
            id: entry.bankName,
            category: entry.category,
            label: sfxLabel(entry),
            event: entry.event,
            skillSlot: entry.skillSlot ?? null,
            skinVariant: isSkinVariant(entry.bankName),
            variants: urls,
            languages: null,
        });
    }

    for (const acc of voiceGroups.values()) {
        const languages = VOICE_LANGUAGE_ORDER.filter((l) => acc.langs.has(l)).map((l) => ({ lang: l, variants: acc.langs.get(l) ?? [] }));
        if (languages.length === 0) continue;
        items.push({
            id: acc.key,
            category: "voice",
            label: acc.skillSlot ? `Skill ${acc.skillSlot} Voice` : "Battle Voice",
            event: acc.event,
            skillSlot: acc.skillSlot,
            skinVariant: acc.skinVariant,
            variants: [],
            languages,
        });
    }

    items.sort((a, b) => {
        const ca = SFX_CATEGORY_ORDER.indexOf(a.category);
        const cb = SFX_CATEGORY_ORDER.indexOf(b.category);
        if (ca !== cb) return ca - cb;
        const sa = a.skillSlot ?? 99;
        const sb = b.skillSlot ?? 99;
        if (sa !== sb) return sa - sb;
        if (a.label !== b.label) return a.label.localeCompare(b.label);
        return a.event.localeCompare(b.event);
    });

    return items;
}

function SfxPanel({ operator, player }: { operator: IOperatorListItem; player: IPlayer }) {
    const items = useMemo(() => buildSfxItems(operator), [operator]);

    const tabs: ICategoryTab[] = useMemo(() => {
        const present = SFX_CATEGORY_ORDER.filter((c) => items.some((i) => i.category === c));
        const base: ICategoryTab[] = [{ id: ALL_CATEGORY_ID, label: "All", count: items.length }];
        for (const c of present) base.push({ id: c, label: SFX_CATEGORY_LABELS[c], count: items.filter((i) => i.category === c).length });
        return base;
    }, [items]);

    const [activeCategory, setActiveCategory] = useState<string>(ALL_CATEGORY_ID);
    useEffect(() => {
        if (!tabs.some((t) => t.id === activeCategory)) setActiveCategory(ALL_CATEGORY_ID);
    }, [tabs, activeCategory]);

    const [searchQuery, setSearchQuery] = useState("");
    const normalizedQuery = searchQuery.trim().toLowerCase();

    const visibleItems = useMemo(() => {
        let list = activeCategory === ALL_CATEGORY_ID ? items : items.filter((i) => i.category === activeCategory);
        if (normalizedQuery) {
            list = list.filter((i) => {
                const hay = `${i.label} ${i.event} ${SFX_CATEGORY_LABELS[i.category]} ${i.skillSlot ? `skill ${i.skillSlot}` : ""}`.toLowerCase();
                return hay.includes(normalizedQuery);
            });
        }
        return list;
    }, [items, activeCategory, normalizedQuery]);

    const operatorName = operatorDisplayName(operator);

    if (items.length === 0) {
        return <EmptyState>No battle audio available for this operator.</EmptyState>;
    }

    return (
        <div>
            <div className="mb-6">
                <SearchBox value={searchQuery} onChange={setSearchQuery} placeholder="Search sound effects..." label="Search sound effects" />
            </div>

            <Tabs value={activeCategory} onValueChange={(v) => setActiveCategory(String(v))} className="w-full">
                <CategoryTabsList tabs={tabs} />
            </Tabs>

            {visibleItems.length === 0 ? (
                <EmptyState>No sound effects match &ldquo;{searchQuery}&rdquo;.</EmptyState>
            ) : (
                <div className="max-h-112 space-y-2 overflow-y-auto pr-2">
                    {visibleItems.map((item) => (
                        <SfxRow key={item.id} item={item} player={player} operatorName={operatorName} showCategory={activeCategory === ALL_CATEGORY_ID} />
                    ))}
                </div>
            )}
        </div>
    );
}

function SfxRow({ item, player, operatorName, showCategory }: { item: ISfxItem; player: IPlayer; operatorName: string; showCategory: boolean }) {
    const [selectedLang, setSelectedLang] = useState<LangType | null>(item.languages?.[0]?.lang ?? null);
    const [variantIndex, setVariantIndex] = useState(0);
    const [expanded, setExpanded] = useState(false);

    const activeVariants = useMemo(() => {
        if (item.languages) return item.languages.find((l) => l.lang === selectedLang)?.variants ?? item.languages[0]?.variants ?? [];
        return item.variants;
    }, [item, selectedLang]);

    const safeIndex = Math.min(variantIndex, Math.max(0, activeVariants.length - 1));
    const currentUrl = activeVariants[safeIndex] ?? null;
    const variantCount = activeVariants.length;

    const isRowPlaying = player.playingId === item.id;
    const isUnavailable = !currentUrl || player.erroredURLs.has(currentUrl);

    const onPlay = () => {
        if (currentUrl) player.play(item.id, currentUrl);
    };

    const onDownload = () => {
        if (!currentUrl) return;
        const langPart = selectedLang ? `_${selectedLang}` : "";
        const variantPart = variantCount > 1 ? `_v${safeIndex + 1}` : "";
        player.download(item.id, currentUrl, `${sanitize(operatorName)}_${sanitize(item.label)}${langPart}${variantPart}.${fileExtension(currentUrl, "ogg")}`);
    };

    return (
        <AudioRow isPlaying={isRowPlaying} isUnavailable={isUnavailable} isDownloading={player.downloadingId === item.id} progress={player.progress} downloadLabel="Download sound effect" onPlay={onPlay} onDownload={onDownload}>
            <div className="flex flex-wrap items-center gap-2">
                <span className="font-medium text-foreground text-sm">{item.label}</span>
                {showCategory && <Pill>{SFX_CATEGORY_LABELS[item.category]}</Pill>}
                {item.skinVariant && <Pill>Alt. skin</Pill>}
                {isUnavailable && <Pill className="bg-muted">Unavailable</Pill>}
                {variantCount > 1 && (
                    <button type="button" onClick={() => setExpanded((v) => !v)} aria-expanded={expanded} className="inline-flex items-center gap-1 rounded bg-secondary/60 px-1.5 py-0.5 font-medium text-[10px] text-muted-foreground uppercase tracking-wide transition-colors hover:text-foreground">
                        <Layers className="h-3 w-3" />
                        {variantCount} variants
                        {expanded ? <ChevronUp className="h-3 w-3" /> : <ChevronDown className="h-3 w-3" />}
                    </button>
                )}
            </div>

            <p className="mt-0.5 font-mono text-[11px] text-muted-foreground">{item.event}</p>

            {item.languages && (
                <div className="mt-2 flex flex-wrap gap-1">
                    {item.languages.map((track) => {
                        const active = track.lang === selectedLang;
                        return (
                            <button
                                key={track.lang}
                                type="button"
                                title={VOICE_LANGUAGE_LABELS[track.lang] ?? track.lang}
                                onClick={() => {
                                    setSelectedLang(track.lang);
                                    setVariantIndex(0);
                                }}
                                className={cn("rounded px-2 py-0.5 font-medium text-[11px] transition-colors", active ? "bg-primary text-primary-foreground" : "bg-secondary/50 text-muted-foreground hover:bg-secondary hover:text-foreground")}
                            >
                                {VOICE_LANGUAGE_SHORT[track.lang] ?? track.lang}
                            </button>
                        );
                    })}
                </div>
            )}

            {expanded && variantCount > 1 && (
                <div className="mt-2 flex flex-wrap gap-1">
                    {activeVariants.map((url, i) => {
                        const playingThis = player.playingId === item.id && player.playingUrl === url;
                        const selected = i === safeIndex;
                        const errored = player.erroredURLs.has(url);
                        return (
                            <button
                                // biome-ignore lint/suspicious/noArrayIndexKey: variant clips have no stable id beyond their order
                                key={i}
                                type="button"
                                disabled={errored}
                                aria-label={`Play variant ${i + 1}`}
                                onClick={() => {
                                    setVariantIndex(i);
                                    player.play(item.id, url);
                                }}
                                className={cn(
                                    "flex h-6 min-w-6 items-center justify-center rounded px-1.5 font-mono text-[11px] transition-colors",
                                    playingThis ? "bg-primary text-primary-foreground" : selected ? "bg-secondary text-foreground" : "bg-secondary/40 text-muted-foreground hover:bg-secondary hover:text-foreground",
                                    errored && "cursor-not-allowed line-through opacity-40",
                                )}
                            >
                                {i + 1}
                            </button>
                        );
                    })}
                </div>
            )}
        </AudioRow>
    );
}

// =============================================================================
// Shared bits
// =============================================================================

function SearchBox({ value, onChange, placeholder, label }: { value: string; onChange: (v: string) => void; placeholder: string; label: string }) {
    return (
        <div className="flex h-9 min-w-60 max-w-115 flex-1 items-center gap-2 rounded-lg border border-border bg-[color-mix(in_oklch,var(--secondary)_60%,transparent)] px-3 transition-[border-color,box-shadow] duration-150 focus-within:border-primary focus-within:shadow-[0_0_0_1px_var(--primary)] [&>svg]:shrink-0 [&>svg]:text-muted-foreground">
            <Search className="h-3.75 w-3.75" aria-hidden="true" />
            <input type="text" value={value} onChange={(e) => onChange(e.target.value)} placeholder={placeholder} aria-label={label} className="min-w-0 flex-1 appearance-none border-0 bg-transparent p-0 font-sans text-foreground text-sm leading-none outline-none placeholder:text-muted-foreground" />
            {value && (
                <button type="button" aria-label="Clear search" onClick={() => onChange("")} className="rounded p-0.5 text-muted-foreground hover:bg-accent hover:text-foreground">
                    <X className="h-3.5 w-3.5" />
                </button>
            )}
        </div>
    );
}

function HighlightedText({ text, query }: { text: string; query: string }) {
    if (!query) return <>{text}</>;
    const lower = text.toLowerCase();
    const q = query.toLowerCase();
    if (!lower.includes(q)) return <>{text}</>;
    const segments: { value: string; match: boolean; offset: number }[] = [];
    let i = 0;
    while (i < text.length) {
        const idx = lower.indexOf(q, i);
        if (idx === -1) {
            segments.push({ value: text.slice(i), match: false, offset: i });
            break;
        }
        if (idx > i) segments.push({ value: text.slice(i, idx), match: false, offset: i });
        segments.push({ value: text.slice(idx, idx + q.length), match: true, offset: idx });
        i = idx + q.length;
    }
    return (
        <>
            {segments.map((seg) => (
                <Fragment key={`${seg.offset}-${seg.match ? "m" : "t"}`}>{seg.match ? <mark className="rounded-sm bg-primary/30 px-0.5 text-foreground">{seg.value}</mark> : seg.value}</Fragment>
            ))}
        </>
    );
}

function AudioSkeleton() {
    return (
        <div>
            <div className="flex gap-2">
                <Skeleton className="h-9 w-48" />
                <Skeleton className="h-9 w-60" />
            </div>
            <div className="mt-6 space-y-2">
                {[0, 1, 2, 3, 4].map((i) => (
                    <Skeleton key={i} className="h-16 w-full rounded-lg" />
                ))}
            </div>
        </div>
    );
}
