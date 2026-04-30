import { useQuery } from "@tanstack/react-query";
import { Download, Loader2, Pause, Play, Volume2, VolumeX } from "lucide-react";
import { memo, useEffect, useMemo, useRef, useState } from "react";
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
        return Object.values(voicesData.charWords).filter((v) => v.charId === operator.id);
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
        return ordered;
    }, [operatorVoices]);

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
    const [erroredUrls, setErroredUrls] = useState<Set<string>>(() => new Set());

    useEffect(() => {
        setErroredUrls(new Set());
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
        const fullUrl = voiceAudio(url);
        if (erroredUrls.has(fullUrl)) return;

        a.src = fullUrl;
        setProgress(0);
        setPlayingId(voice.id);
        a.play().catch(() => {
            setErroredUrls((prev) => {
                if (prev.has(fullUrl)) return prev;
                const next = new Set(prev);
                next.add(fullUrl);
                return next;
            });
            setPlayingId(null);
            setProgress(0);
        });
    };

    const onDownload = async (voice: IVoice) => {
        const url = voice.data?.find((d) => d.language === selectedLanguage)?.voiceUrl;
        if (!url || !voice.id) return;
        const fullUrl = voiceAudio(url);
        if (erroredUrls.has(fullUrl)) return;
        setDownloadingId(voice.id);
        try {
            const ext = url.split(".").pop()?.split("?")[0] ?? "mp3";
            const langLabel = VOICE_LANGUAGE_LABELS[selectedLanguage] ?? selectedLanguage;
            const res = await fetch(fullUrl);
            if (!res.ok) {
                setErroredUrls((prev) => {
                    if (prev.has(fullUrl)) return prev;
                    const next = new Set(prev);
                    next.add(fullUrl);
                    return next;
                });
                throw new Error(`Voice file unavailable (${res.status})`);
            }
            const blob = await res.blob();
            const objectUrl = URL.createObjectURL(blob);
            const link = document.createElement("a");
            link.href = objectUrl;
            link.download = `${sanitize(operatorName)}_${sanitize(voice.voiceTitle)}_${sanitize(langLabel)}.${ext}`;
            document.body.appendChild(link);
            link.click();
            document.body.removeChild(link);
            URL.revokeObjectURL(objectUrl);
        } catch (e) {
            console.error("Failed to download voice line", e);
        } finally {
            setDownloadingId(null);
        }
    };

    if (isLoading) return <AudioSkeleton />;

    return (
        <div className="min-w-0 overflow-hidden p-4 md:p-6">
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
                        setErroredUrls((prev) => {
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
                </div>
            </div>

            {categories.length > 0 && activeCategory ? (
                <Tabs value={activeCategory} onValueChange={(v) => setActiveCategory(String(v))} className="w-full">
                    <div className="mb-4 overflow-x-auto">
                        <TabsList variant="underline">
                            {categories.map((c) => (
                                <TabsTrigger key={c.id} value={c.id}>
                                    {c.name}
                                    <span className="ml-1.5 text-muted-foreground text-xs">({c.lines.length})</span>
                                </TabsTrigger>
                            ))}
                        </TabsList>
                    </div>
                    {categories.map((c) => (
                        <TabsContent key={c.id} value={c.id} className="mt-0">
                            <div className="max-h-112 space-y-2 overflow-y-auto pr-2">
                                {c.lines.map((voice) => {
                                    const url = voice.data?.find((d) => d.language === selectedLanguage)?.voiceUrl;
                                    const fullUrl = url ? voiceAudio(url) : null;
                                    const isUnavailable = !url || (fullUrl !== null && erroredUrls.has(fullUrl));
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
                                            onPlay={() => playVoice(voice)}
                                            onDownload={() => onDownload(voice)}
                                        />
                                    );
                                })}
                            </div>
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
    onPlay: () => void;
    onDownload: () => void;
}

function VoiceLineRow({ voice, isPlaying, progress, canPlay, canDownload, isDownloading, isUnavailable, onPlay, onDownload }: IVoiceLineRowProps) {
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
                    <div className="flex items-center gap-2">
                        <div className="font-medium text-foreground text-sm">{voice.voiceTitle}</div>
                        {isUnavailable && <span className="rounded bg-muted px-1.5 py-0.5 text-[10px] text-muted-foreground uppercase tracking-wide">Unavailable</span>}
                    </div>
                    <p className={cn("text-muted-foreground text-xs duration-300", isPlaying ? "" : "line-clamp-2")}>{voice.voiceText}</p>
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
