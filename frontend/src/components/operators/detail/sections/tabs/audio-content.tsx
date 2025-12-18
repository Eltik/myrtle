"use client";

import { Pause, Play, Volume2, VolumeX } from "lucide-react";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { Button } from "~/components/ui/shadcn/button";
import { ScrollArea } from "~/components/ui/shadcn/scroll-area";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "~/components/ui/shadcn/select";
import { Skeleton } from "~/components/ui/shadcn/skeleton";
import { Slider } from "~/components/ui/shadcn/slider";
import { cn } from "~/lib/utils";
import type { Operator } from "~/types/api";
import type { LangType, Voice, VoiceData, Voices } from "~/types/api/impl/voice";

interface AudioContentProps {
    operator: Operator;
}

interface VoiceLine {
    id: string;
    title: string;
    text: string;
    data?: VoiceData[];
    languages?: LangType[];
}

// Map backend LangType to display labels
const LANGUAGE_LABELS: Record<LangType, string> = {
    JP: "Japanese",
    CN_MANDARIN: "Chinese",
    EN: "English",
    KR: "Korean",
    CN_TOPOLECT: "Chinese (Regional)",
    GER: "German",
    ITA: "Italian",
    RUS: "Russian",
    FRE: "French",
    LINKAGE: "Collaboration",
};

export function AudioContent({ operator }: AudioContentProps) {
    const [voices, setVoices] = useState<VoiceLine[]>([]);
    const [isLoading, setIsLoading] = useState(true);
    const [selectedLanguage, setSelectedLanguage] = useState<LangType>("JP");
    const [playingId, setPlayingId] = useState<string | null>(null);
    const [volume, setVolume] = useState(80);
    const [isMuted, setIsMuted] = useState(false);
    const [progress, setProgress] = useState(0);

    const audioRef = useRef<HTMLAudioElement | null>(null);
    const animationFrameRef = useRef<number | null>(null);

    const updateProgress = useCallback(() => {
        if (audioRef.current && !audioRef.current.paused) {
            const audio = audioRef.current;
            if (audio.duration > 0) {
                setProgress((audio.currentTime / audio.duration) * 100);
            }
            animationFrameRef.current = requestAnimationFrame(updateProgress);
        }
    }, []);

    const stopProgressAnimation = useCallback(() => {
        if (animationFrameRef.current) {
            cancelAnimationFrame(animationFrameRef.current);
            animationFrameRef.current = null;
        }
    }, []);

    // Derive available languages from voice data
    const availableLanguages = useMemo(() => {
        if (voices.length === 0) return [];

        // Collect all unique languages from all voice lines
        const langSet = new Set<LangType>();
        for (const voice of voices) {
            if (voice.languages) {
                for (const lang of voice.languages) {
                    langSet.add(lang);
                }
            }
        }

        // Sort languages in a sensible order
        const order: LangType[] = ["JP", "CN_MANDARIN", "EN", "KR", "CN_TOPOLECT", "GER", "ITA", "RUS", "FRE", "LINKAGE"];
        return order.filter((lang) => langSet.has(lang));
    }, [voices]);

    // Get the voice actor name for the selected language
    const voiceActorName = useMemo(() => {
        if (voices.length === 0) return null;

        // Find the first voice line that has data for the selected language
        for (const voice of voices) {
            const voiceData = voice.data?.find((d) => d.language === selectedLanguage);
            if (voiceData?.cvName && voiceData.cvName.length > 0) {
                return voiceData.cvName.join(", ");
            }
        }
        return null;
    }, [voices, selectedLanguage]);

    // Set default language when available languages change
    useEffect(() => {
        const firstLang = availableLanguages[0];
        if (firstLang && !availableLanguages.includes(selectedLanguage)) {
            setSelectedLanguage(firstLang);
        }
    }, [availableLanguages, selectedLanguage]);

    // Stop audio when language changes
    // biome-ignore lint/correctness/useExhaustiveDependencies: intentionally run when selectedLanguage changes
    useEffect(() => {
        if (audioRef.current) {
            audioRef.current.pause();
            audioRef.current = null;
        }
        stopProgressAnimation();
        setPlayingId(null);
        setProgress(0);
    }, [selectedLanguage, stopProgressAnimation]);

    // Fetch voice data
    useEffect(() => {
        const fetchVoices = async () => {
            setIsLoading(true);
            try {
                const res = await fetch("/api/static", {
                    method: "POST",
                    headers: { "Content-Type": "application/json" },
                    body: JSON.stringify({ type: "voices", id: operator.id }),
                });
                const data = await res.json();

                if (data.voices) {
                    const formattedVoices = formatVoices(data.voices);
                    setVoices(formattedVoices);
                }
            } catch (error) {
                console.error("Failed to fetch voices:", error);
            }
            setIsLoading(false);
        };

        if (operator.id) {
            fetchVoices();
        }
    }, [operator.id]);

    const playVoice = (voice: VoiceLine) => {
        if (playingId === voice.id) {
            // Pause current
            audioRef.current?.pause();
            stopProgressAnimation();
            setPlayingId(null);
            setProgress(0);
            return;
        }

        // Play new
        if (audioRef.current) {
            audioRef.current.pause();
        }
        stopProgressAnimation();

        const voiceData = voice.data?.find((d) => d.language === selectedLanguage);
        if (!voiceData?.voiceUrl) {
            console.error("No voice URL available for language:", selectedLanguage);
            return;
        }

        const audioUrl = `/api/cdn/upk${voiceData.voiceUrl}`;

        const audio = new Audio(audioUrl);
        audio.volume = isMuted ? 0 : volume / 100;
        audio.onended = () => {
            stopProgressAnimation();
            setPlayingId(null);
            setProgress(0);
        };
        audio.onerror = () => {
            console.error("Failed to load audio:", audioUrl);
            stopProgressAnimation();
            setPlayingId(null);
            setProgress(0);
        };

        audioRef.current = audio;
        setProgress(0);
        audio
            .play()
            .then(() => {
                animationFrameRef.current = requestAnimationFrame(updateProgress);
            })
            .catch(() => {
                stopProgressAnimation();
                setPlayingId(null);
                setProgress(0);
            });
        setPlayingId(voice.id);
    };

    useEffect(() => {
        if (audioRef.current) {
            audioRef.current.volume = isMuted ? 0 : volume / 100;
        }
    }, [volume, isMuted]);

    useEffect(() => {
        return () => {
            audioRef.current?.pause();
            stopProgressAnimation();
        };
    }, [stopProgressAnimation]);

    return (
        <div className="min-w-0 overflow-hidden p-4 md:p-6">
            {/* Header */}
            <div className="mb-6">
                <h2 className="font-semibold text-foreground text-xl">Voice Lines</h2>
                <p className="text-muted-foreground text-sm">Listen to operator voice lines and audio</p>
            </div>

            {/* Controls */}
            <div className="mb-6 space-y-3">
                {voiceActorName && (
                    <p className="text-muted-foreground text-sm">
                        <span className="text-foreground/60">CV:</span> <span className="font-medium text-foreground">{voiceActorName}</span>
                    </p>
                )}
                <div className="flex flex-wrap items-center gap-4">
                    <Select onValueChange={(value) => setSelectedLanguage(value as LangType)} value={selectedLanguage}>
                        <SelectTrigger className="w-48">
                            <SelectValue placeholder="Language" />
                        </SelectTrigger>
                        <SelectContent>
                            {availableLanguages.map((lang) => (
                                <SelectItem key={lang} value={lang}>
                                    {LANGUAGE_LABELS[lang] ?? lang}
                                </SelectItem>
                            ))}
                        </SelectContent>
                    </Select>

                    <div className="flex items-center gap-2">
                        <Button onClick={() => setIsMuted(!isMuted)} size="icon" variant="ghost">
                            {isMuted ? <VolumeX className="h-4 w-4" /> : <Volume2 className="h-4 w-4" />}
                        </Button>
                        <Slider className="w-24" max={100} min={0} onValueChange={(val) => setVolume(val[0] ?? 80)} step={1} value={[volume]} />
                    </div>
                </div>
            </div>

            {/* Voice Lines List */}
            {isLoading ? (
                <div className="space-y-2">
                    {[1, 2, 3, 4, 5].map((i) => (
                        <Skeleton className="h-16 w-full rounded-lg" key={i} />
                    ))}
                </div>
            ) : voices.length > 0 ? (
                <ScrollArea className="h-[500px]">
                    <div className="space-y-2 pr-4">
                        {voices.map((voice) => (
                            <div className={cn("group relative overflow-hidden rounded-lg border border-border transition-colors", playingId === voice.id ? "border-primary bg-primary/10" : "bg-card/30 hover:bg-secondary/30")} key={voice.id}>
                                <div className="flex items-start gap-3 p-3">
                                    <Button className="shrink-0" onClick={() => playVoice(voice)} size="icon" variant={playingId === voice.id ? "default" : "secondary"}>
                                        {playingId === voice.id ? <Pause className="h-4 w-4" /> : <Play className="h-4 w-4" />}
                                    </Button>
                                    <div className="min-w-0 flex-1">
                                        <h4 className="mb-1 font-medium text-foreground text-sm">{voice.title}</h4>
                                        <p className={cn("overflow-hidden text-muted-foreground text-xs duration-300 ease-out", playingId === voice.id ? "max-h-96" : "line-clamp-2 max-h-10")}>{voice.text}</p>
                                    </div>
                                </div>
                                {/* Progress bar */}
                                {playingId === voice.id && (
                                    <div className="absolute inset-x-0 bottom-0 h-1 bg-primary/20">
                                        <div className="h-full bg-primary" style={{ width: `${progress}%` }} />
                                    </div>
                                )}
                            </div>
                        ))}
                    </div>
                </ScrollArea>
            ) : (
                <div className="py-12 text-center text-muted-foreground">No voice data available for this operator.</div>
            )}
        </div>
    );
}

function formatVoices(voiceData: Voices | Voice[]): VoiceLine[] {
    const voiceLines: VoiceLine[] = [];

    // If we have actual voice data, use it
    if (Array.isArray(voiceData)) {
        for (const voice of voiceData) {
            voiceLines.push({
                id: voice.voiceId ?? voice.id ?? "",
                title: voice.voiceTitle ?? "Voice Line",
                text: voice.voiceText ?? "",
                data: voice.data ?? undefined,
                languages: voice.languages ?? undefined,
            });
        }
    } else if (voiceData && typeof voiceData === "object") {
        // Handle Voices object format (with charWords map)
        const voices = "charWords" in voiceData ? voiceData.charWords : voiceData;

        for (const [_key, voice] of Object.entries(voices)) {
            if (voice && typeof voice === "object" && "voiceId" in voice) {
                const v = voice as Voice;
                voiceLines.push({
                    id: v.voiceId ?? v.id ?? _key,
                    title: v.voiceTitle ?? "Voice Line",
                    text: v.voiceText ?? "",
                    data: v.data ?? undefined,
                    languages: v.languages ?? undefined,
                });
            }
        }
    }

    // Sort by voice index if available (CN_001, CN_002, etc.)
    voiceLines.sort((a, b) => {
        const numA = Number.parseInt(a.id.replace(/\D/g, ""), 10) || 0;
        const numB = Number.parseInt(b.id.replace(/\D/g, ""), 10) || 0;
        return numA - numB;
    });

    return voiceLines;
}
