import { useQuery } from "@tanstack/react-query";
import { ChevronDown, ChevronUp, Download, Loader2, Pause, Play, Search, Volume2, VolumeX, X } from "lucide-react";
import { Fragment, memo, useEffect, useLayoutEffect, useMemo, useRef, useState } from "react";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "#/components/ui/select";
import { Skeleton } from "#/components/ui/skeleton";
import { Slider } from "#/components/ui/slider";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "#/components/ui/tabs";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import { voicesQueryOptions } from "#/lib/api/voices";
import { cn } from "#/lib/utils";
import type { IOperatorListItem } from "#/types/operators";
import type { IVoice, LangType } from "#/types/voices";
import { voiceAudio } from "../../assets";
import { VOICE_CATEGORY_MAP, VOICE_CATEGORY_ORDER, VOICE_LANGUAGE_LABELS, VOICE_LANGUAGE_ORDER } from "../../constants";

const ALL_CATEGORY_ID = "all";

function sanitize(name: string): string {
    return name.replace(/[^a-zA-Z0-9\-_]/g, "_");
}

interface IAudioContentProps {
    operator: IOperatorListItem;
}

interface IVoiceCategory {
    id: string;
    name: string;
    lines: IVoice[];
}

export const AudioContent = memo(function AudioContent({ operator }: IAudioContentProps) {
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

    const audioRef = useRef<HTMLAudioElement | null>(null);
    const [playingId, setPlayingId] = useState<string | null>(null);
    const [downloadingId, setDownloadingId] = useState<string | null>(null);
    const [progress, setProgress] = useState(0);
    const [volume, setVolume] = useState(80);
    const [isMuted, setIsMuted] = useState(false);
    const [erroredURLs, setErroredURLs] = useState<Set<string>>(() => new Set());

    useEffect(() => {
        setErroredURLs(new Set());
    }, []);

    useEffect(() => {
        const a = audioRef.current;
        if (a) a.volume = isMuted ? 0 : volume / 100;
    }, [volume, isMuted]);

    useEffect(() => {
        const a = audioRef.current;
        if (a) {
            a.pause();
            a.removeAttribute("src");
            a.load();
        }
        setPlayingId(null);
        setProgress(0);
    }, []);

    const operatorName = operator.name ?? operator.id ?? "voice";

    const playVoice = (voice: IVoice) => {
        const a = audioRef.current;
        if (!a) return;

        if (playingId === voice.id) {
            a.pause();
            setPlayingId(null);
            setProgress(0);
            return;
        }

        const url = voice.data?.find((d) => d.language === selectedLanguage)?.voiceUrl;
        if (!url) return;
        const fullURL = voiceAudio(url);
        if (erroredURLs.has(fullURL)) return;

        a.src = fullURL;
        setProgress(0);
        setPlayingId(voice.id);
        a.play().catch(() => {
            setErroredURLs((prev) => {
                if (prev.has(fullURL)) return prev;
                const next = new Set(prev);
                next.add(fullURL);
                return next;
            });
            setPlayingId(null);
            setProgress(0);
        });
    };

    const onDownload = async (voice: IVoice) => {
        const url = voice.data?.find((d) => d.language === selectedLanguage)?.voiceUrl;
        if (!url || !voice.id) return;
        const fullURL = voiceAudio(url);
        if (erroredURLs.has(fullURL)) return;
        setDownloadingId(voice.id);
        try {
            const ext = url.split(".").pop()?.split("?")[0] ?? "mp3";
            const langLabel = VOICE_LANGUAGE_LABELS[selectedLanguage] ?? selectedLanguage;
            const res = await fetch(fullURL);
            if (!res.ok) {
                setErroredURLs((prev) => {
                    if (prev.has(fullURL)) return prev;
                    const next = new Set(prev);
                    next.add(fullURL);
                    return next;
                });
                throw new Error(`Voice file unavailable (${res.status})`);
            }
            const blob = await res.blob();
            const objectURL = URL.createObjectURL(blob);
            const link = document.createElement("a");
            link.href = objectURL;
            link.download = `${sanitize(operatorName)}_${sanitize(voice.voiceTitle)}_${sanitize(langLabel)}.${ext}`;
            document.body.appendChild(link);
            link.click();
            document.body.removeChild(link);
            URL.revokeObjectURL(objectURL);
        } catch (e) {
            console.error("Failed to download voice line", e);
        } finally {
            setDownloadingId(null);
        }
    };

    if (isLoading) return <AudioSkeleton />;

    return (
        <div className="min-w-0 overflow-hidden p-4 md:p-6">
            {/* biome-ignore lint/a11y/useMediaCaption: voice line audio has no transcript track available */}
            <audio
                ref={audioRef}
                preload="auto"
                onTimeUpdate={(e) => {
                    const el = e.currentTarget;
                    if (el.duration > 0) setProgress((el.currentTime / el.duration) * 100);
                }}
                onEnded={() => {
                    setPlayingId(null);
                    setProgress(0);
                }}
                onError={() => {
                    const el = audioRef.current;
                    const src = el?.currentSrc;
                    if (src) {
                        setErroredURLs((prev) => {
                            if (prev.has(src)) return prev;
                            const next = new Set(prev);
                            next.add(src);
                            return next;
                        });
                    }
                    setPlayingId(null);
                    setProgress(0);
                }}
            />

            <div className="mb-6">
                <h2 className="font-semibold text-foreground text-xl">Voice Lines</h2>
                <p className="text-muted-foreground text-sm">Listen to operator voice lines.</p>
            </div>

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
                    <div className="flex items-center gap-2">
                        <button type="button" aria-label={isMuted ? "Unmute" : "Mute"} onClick={() => setIsMuted((m) => !m)} className="rounded-md p-1.5 text-muted-foreground hover:bg-accent hover:text-foreground">
                            {isMuted ? <VolumeX className="h-4 w-4" /> : <Volume2 className="h-4 w-4" />}
                        </button>
                        <Slider className="w-24" min={0} max={100} step={1} value={[volume]} onValueChange={(v) => setVolume(Array.isArray(v) ? (v[0] ?? 80) : v)} />
                    </div>
                    <div className="flex h-9 min-w-60 max-w-115 flex-1 items-center gap-2 rounded-lg border border-border bg-[color-mix(in_oklch,var(--secondary)_60%,transparent)] px-3 transition-[border-color,box-shadow] duration-150 focus-within:border-primary focus-within:shadow-[0_0_0_1px_var(--primary)] [&>svg]:shrink-0 [&>svg]:text-muted-foreground">
                        <Search className="h-3.75 w-3.75" aria-hidden="true" />
                        <input
                            type="text"
                            value={searchQuery}
                            onChange={(e) => setSearchQuery(e.target.value)}
                            placeholder="Search voice lines..."
                            aria-label="Search voice lines"
                            className="min-w-0 flex-1 appearance-none border-0 bg-transparent p-0 font-sans text-foreground text-sm leading-none outline-none placeholder:text-muted-foreground"
                        />
                        {searchQuery && (
                            <button type="button" aria-label="Clear search" onClick={() => setSearchQuery("")} className="rounded p-0.5 text-muted-foreground hover:bg-accent hover:text-foreground">
                                <X className="h-3.5 w-3.5" />
                            </button>
                        )}
                    </div>
                </div>
            </div>

            {categories.length > 0 && activeCategory ? (
                <Tabs value={activeCategory} onValueChange={(v) => setActiveCategory(String(v))} className="w-full">
                    <div className="mb-4 overflow-x-auto overflow-y-hidden">
                        <TabsList variant="underline">
                            {filteredCategories.map((c) => (
                                <TabsTrigger key={c.id} value={c.id}>
                                    {c.name}
                                    <span className="ml-1.5 text-muted-foreground text-xs">({c.lines.length})</span>
                                </TabsTrigger>
                            ))}
                        </TabsList>
                    </div>
                    {filteredCategories.map((c) => (
                        <TabsContent key={c.id} value={c.id} className="mt-0">
                            {c.lines.length === 0 ? (
                                <div className="py-12 text-center text-muted-foreground text-sm">No voice lines match &ldquo;{searchQuery}&rdquo;.</div>
                            ) : (
                                <div className="max-h-112 space-y-2 overflow-y-auto pr-2">
                                    {c.lines.map((voice) => {
                                        const url = voice.data?.find((d) => d.language === selectedLanguage)?.voiceUrl;
                                        const fullURL = url ? voiceAudio(url) : null;
                                        const isUnavailable = !url || (fullURL !== null && erroredURLs.has(fullURL));
                                        const categoryLabel = c.id === ALL_CATEGORY_ID ? (VOICE_CATEGORY_MAP[voice.placeType] ?? "Other") : null;
                                        return (
                                            <VoiceLineRow
                                                key={voice.id ?? voice.charWordId}
                                                voice={voice}
                                                isPlaying={playingId === voice.id}
                                                progress={playingId === voice.id ? progress : 0}
                                                canPlay={!isUnavailable}
                                                canDownload={!isUnavailable}
                                                isDownloading={downloadingId === voice.id}
                                                isUnavailable={isUnavailable}
                                                categoryLabel={categoryLabel}
                                                highlight={normalizedQuery}
                                                onPlay={() => playVoice(voice)}
                                                onDownload={() => onDownload(voice)}
                                            />
                                        );
                                    })}
                                </div>
                            )}
                        </TabsContent>
                    ))}
                </Tabs>
            ) : (
                <div className="py-12 text-center text-muted-foreground">No voice data available for this operator.</div>
            )}
        </div>
    );
});

interface IVoiceLineRowProps {
    voice: IVoice;
    isPlaying: boolean;
    progress: number;
    canPlay: boolean;
    canDownload: boolean;
    isDownloading: boolean;
    isUnavailable: boolean;
    categoryLabel: string | null;
    highlight: string;
    onPlay: () => void;
    onDownload: () => void;
}

function VoiceLineRow({ voice, isPlaying, progress, canPlay, canDownload, isDownloading, isUnavailable, categoryLabel, highlight, onPlay, onDownload }: IVoiceLineRowProps) {
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
        <div className={cn("group relative overflow-hidden rounded-lg border transition-colors", isPlaying ? "border-primary bg-primary/10" : "border-border bg-card/30 hover:bg-secondary/30", isUnavailable && "opacity-60")}>
            <div className="flex items-start gap-3 p-3">
                <div className="flex shrink-0 gap-1">
                    <button
                        type="button"
                        onClick={onPlay}
                        disabled={!canPlay && !isPlaying}
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
                                    disabled={!canDownload || isDownloading}
                                    aria-label="Download voice line"
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
                <div className="min-w-0 flex-1">
                    <div className="flex flex-wrap items-center gap-2">
                        <div className="font-medium text-foreground text-sm">
                            <HighlightedText text={voice.voiceTitle} query={highlight} />
                        </div>
                        {categoryLabel && <span className="rounded bg-secondary/60 px-1.5 py-0.5 text-[10px] text-muted-foreground uppercase tracking-wide">{categoryLabel}</span>}
                        {isUnavailable && <span className="rounded bg-muted px-1.5 py-0.5 text-[10px] text-muted-foreground uppercase tracking-wide">Unavailable</span>}
                    </div>
                    <p ref={textRef} className={cn("text-muted-foreground text-xs duration-300", showFull ? "" : "line-clamp-2")}>
                        <HighlightedText text={voice.voiceText} query={highlight} />
                    </p>
                    {showToggle && (
                        <button
                            type="button"
                            onClick={() => setIsExpanded((v) => !v)}
                            aria-expanded={isExpanded}
                            className="mt-1 inline-flex items-center gap-1 rounded font-medium text-[11px] text-muted-foreground transition-colors hover:text-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
                        >
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
                </div>
            </div>
            {isPlaying && (
                <div className="absolute inset-x-0 bottom-0 h-1 bg-primary/20">
                    <div className="h-full bg-primary" style={{ width: `${progress}%` }} />
                </div>
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
        <div className="min-w-0 overflow-hidden p-4 md:p-6">
            <Skeleton className="h-7 w-32" />
            <Skeleton className="mt-2 h-4 w-64" />
            <div className="mt-6 flex gap-2">
                <Skeleton className="h-9 w-48" />
                <Skeleton className="h-9 w-24" />
            </div>
            <div className="mt-6 space-y-2">
                {[0, 1, 2, 3, 4].map((i) => (
                    <Skeleton key={i} className="h-16 w-full rounded-lg" />
                ))}
            </div>
        </div>
    );
}
