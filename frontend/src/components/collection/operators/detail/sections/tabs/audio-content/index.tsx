"use client";

import { Pause, Play, Volume2, VolumeX } from "lucide-react";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { Button } from "~/components/ui/shadcn/button";
import { ScrollArea } from "~/components/ui/shadcn/scroll-area";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "~/components/ui/shadcn/select";
import { Skeleton } from "~/components/ui/shadcn/skeleton";
import { Slider } from "~/components/ui/shadcn/slider";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "~/components/ui/shadcn/tabs";
import { cn } from "~/lib/utils";
import type { Operator } from "~/types/api";
import type { LangType } from "~/types/api/impl/voice";
import { CATEGORY_ORDER, LANGUAGE_LABELS } from "./impl/constants";
import { formatVoices, getCategoryName } from "./impl/helpers";
import type { VoiceCategory, VoiceLine } from "./impl/types";

interface AudioContentProps {
    operator: Operator;
}

export function AudioContent({ operator }: AudioContentProps) {
    const [voices, setVoices] = useState<VoiceLine[]>([]);
    const [isLoading, setIsLoading] = useState(true);
    const [selectedLanguage, setSelectedLanguage] = useState<LangType>("JP");
    const [activeCategory, setActiveCategory] = useState<string>("greetings");
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

    const availableLanguages = useMemo(() => {
        if (voices.length === 0) return [];

        const langSet = new Set<LangType>();
        for (const voice of voices) {
            if (voice.languages) {
                for (const lang of voice.languages) {
                    langSet.add(lang);
                }
            }
        }

        const order: LangType[] = ["JP", "CN_MANDARIN", "EN", "KR", "CN_TOPOLECT", "GER", "ITA", "RUS", "FRE", "LINKAGE"];
        return order.filter((lang) => langSet.has(lang));
    }, [voices]);

    const voiceCategories = useMemo(() => {
        if (voices.length === 0) return [];

        const categoriesMap = new Map<string, VoiceLine[]>();

        for (const voice of voices) {
            const categoryName = voice.placeType ? getCategoryName(voice.placeType) : "Other";
            const categoryId = categoryName.toLowerCase().replace(/\s+/g, "-");

            if (!categoriesMap.has(categoryId)) {
                categoriesMap.set(categoryId, []);
            }
            categoriesMap.get(categoryId)?.push(voice);
        }

        const categories: VoiceCategory[] = [];
        for (const [id, lines] of categoriesMap) {
            const name = CATEGORY_ORDER.find((c) => c.toLowerCase().replace(/\s+/g, "-") === id) ?? id;
            categories.push({ id, name, lines });
        }

        categories.sort((a, b) => {
            const indexA = CATEGORY_ORDER.indexOf(a.name);
            const indexB = CATEGORY_ORDER.indexOf(b.name);
            return (indexA === -1 ? 999 : indexA) - (indexB === -1 ? 999 : indexB);
        });

        return categories;
    }, [voices]);

    useEffect(() => {
        if (voiceCategories.length > 0 && !voiceCategories.some((c) => c.id === activeCategory)) {
            setActiveCategory(voiceCategories[0]?.id ?? "greetings");
        }
    }, [voiceCategories, activeCategory]);

    const voiceActorName = useMemo(() => {
        if (voices.length === 0) return null;

        for (const voice of voices) {
            const voiceData = voice.data?.find((d) => d.language === selectedLanguage);
            if (voiceData?.cvName && voiceData.cvName.length > 0) {
                return voiceData.cvName.join(", ");
            }
        }
        return null;
    }, [voices, selectedLanguage]);

    useEffect(() => {
        const firstLang = availableLanguages[0];
        if (firstLang && !availableLanguages.includes(selectedLanguage)) {
            setSelectedLanguage(firstLang);
        }
    }, [availableLanguages, selectedLanguage]);

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
            audioRef.current?.pause();
            stopProgressAnimation();
            setPlayingId(null);
            setProgress(0);
            return;
        }

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

            {/* Voice Lines List with Categories */}
            {isLoading ? (
                <div className="space-y-2">
                    {[1, 2, 3, 4, 5].map((i) => (
                        <Skeleton className="h-16 w-full rounded-lg" key={i} />
                    ))}
                </div>
            ) : voiceCategories.length > 0 ? (
                <Tabs className="w-full" onValueChange={setActiveCategory} value={activeCategory}>
                    {/* Category Tabs */}
                    <div className="mb-4 overflow-x-auto">
                        <TabsList className="inline-flex h-auto w-auto min-w-full justify-start gap-1 bg-transparent p-0">
                            {voiceCategories.map((category) => (
                                <TabsTrigger className="shrink-0 rounded-lg border border-transparent px-3 py-1.5 text-sm data-[state=active]:border-primary data-[state=active]:bg-primary/10 data-[state=active]:text-primary" key={category.id} value={category.id}>
                                    {category.name}
                                    <span className="ml-1.5 text-muted-foreground text-xs">({category.lines.length})</span>
                                </TabsTrigger>
                            ))}
                        </TabsList>
                    </div>

                    {/* Category Content */}
                    {voiceCategories.map((category) => (
                        <TabsContent className="mt-0" key={category.id} value={category.id}>
                            <ScrollArea className="h-[450px]">
                                <div className="space-y-2 pr-4">
                                    {category.lines.map((voice) => (
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
                        </TabsContent>
                    ))}
                </Tabs>
            ) : (
                <div className="py-12 text-center text-muted-foreground">No voice data available for this operator.</div>
            )}
        </div>
    );
}
