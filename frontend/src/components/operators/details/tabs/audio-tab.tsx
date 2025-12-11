"use client";

import { ChevronDown, Download, Pause, Play, Volume2, VolumeX } from "lucide-react";
import { motion } from "motion/react";
import type React from "react";
import { useCallback, useEffect, useRef, useState } from "react";
import { AnimatedGroup } from "~/components/ui/motion-primitives/animated-group";
import { Button } from "~/components/ui/shadcn/button";
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from "~/components/ui/shadcn/dropdown-menu";
import { ScrollArea } from "~/components/ui/shadcn/scroll-area";
import { Separator } from "~/components/ui/shadcn/separator";
import type { LangType, Operator, Voice } from "~/types/api";

interface AudioTabProps {
    operator: Operator;
}

interface VoiceLine {
    id: string;
    name: string;
    transcript: string;
    url: string;
}

interface VoiceCategory {
    id: string;
    name: string;
    lines: VoiceLine[];
}

export function AudioTab({ operator }: AudioTabProps) {
    const [categories, setCategories] = useState<VoiceCategory[]>([]);
    const [activeCategory, setActiveCategory] = useState<string>("");
    const [activeLine, setActiveLine] = useState<string | null>(null);
    const [isPlaying, setIsPlaying] = useState(false);
    const [isMuted, setIsMuted] = useState(false);
    const [availableLanguages, setAvailableLanguages] = useState<LangType[]>([]);
    const [selectedLanguage, setSelectedLanguage] = useState<LangType | null>(null);
    const [isLoading, setIsLoading] = useState(true);
    const audioRef = useRef<HTMLAudioElement | null>(null);

    // Fetch voices
    useEffect(() => {
        const fetchVoices = async () => {
            try {
                const response = await fetch("/api/static", {
                    method: "POST",
                    headers: { "Content-Type": "application/json" },
                    body: JSON.stringify({ type: "voices", id: operator.id }),
                });
                const data = (await response.json()) as { voices: Voice[] };
                const voices = data.voices ?? [];

                // Extract available languages
                const langs = new Set<LangType>();
                voices.forEach((v) => {
                    v.data?.forEach((d) => {
                        if (d.language) langs.add(d.language);
                    });
                });
                const langArray = Array.from(langs);
                setAvailableLanguages(langArray);
                setSelectedLanguage(langArray[0] ?? null);

                // Group by category
                const categoryMap = new Map<string, VoiceLine[]>();
                voices.forEach((voice) => {
                    const categoryName = getCategoryName(voice.placeType);
                    const langData = voice.data?.find((d) => d.language === (langArray[0] ?? "JP"));

                    const line: VoiceLine = {
                        id: voice.charWordId ?? voice.voiceId,
                        name: voice.voiceTitle ?? voice.voiceId,
                        transcript: voice.voiceText ?? "",
                        url: langData?.voiceUrl ?? "",
                    };

                    if (!categoryMap.has(categoryName)) {
                        categoryMap.set(categoryName, []);
                    }
                    categoryMap.get(categoryName)?.push(line);
                });

                const cats: VoiceCategory[] = [];
                categoryMap.forEach((lines, name) => {
                    cats.push({
                        id: name.toLowerCase().replace(/\s+/g, "-"),
                        name,
                        lines,
                    });
                });

                // Sort categories
                const order = ["Greetings", "Combat", "Interaction", "Base", "Level Up", "Recruitment", "Squad", "Other"];
                cats.sort((a, b) => {
                    const indexA = order.indexOf(a.name);
                    const indexB = order.indexOf(b.name);
                    return (indexA === -1 ? 999 : indexA) - (indexB === -1 ? 999 : indexB);
                });

                setCategories(cats);
                if (cats.length > 0) {
                    setActiveCategory(cats[0]?.id);
                }
            } catch (error) {
                console.error("Failed to fetch voices:", error);
            } finally {
                setIsLoading(false);
            }
        };
        void fetchVoices();
    }, [operator.id]);

    const currentCategory = categories.find((c) => c.id === activeCategory);
    const _currentLine = currentCategory?.lines.find((l) => l.id === activeLine);

    const handleLineSelect = (lineId: string, url: string) => {
        if (activeLine === lineId && isPlaying) {
            audioRef.current?.pause();
            setIsPlaying(false);
        } else {
            setActiveLine(lineId);
            if (audioRef.current && url) {
                audioRef.current.src = url;
                audioRef.current.volume = isMuted ? 0 : 0.7;
                void audioRef.current.play();
                setIsPlaying(true);
            }
        }
    };

    const toggleMute = () => {
        if (audioRef.current) {
            audioRef.current.muted = !isMuted;
        }
        setIsMuted(!isMuted);
    };

    const handleDownload = useCallback(
        (e: React.MouseEvent, line: VoiceLine) => {
            e.stopPropagation();
            if (line.url) {
                const a = document.createElement("a");
                a.href = line.url;
                a.download = `${operator.name}-${line.name}.mp3`;
                document.body.appendChild(a);
                a.click();
                document.body.removeChild(a);
            }
        },
        [operator.name],
    );

    useEffect(() => {
        const audio = audioRef.current;
        if (audio) {
            const handleEnded = () => setIsPlaying(false);
            audio.addEventListener("ended", handleEnded);
            return () => audio.removeEventListener("ended", handleEnded);
        }
    }, []);

    return (
        <div className="w-full space-y-6">
            <div className="flex items-center justify-between">
                <h2 className="font-bold text-2xl md:text-3xl">Voice Lines</h2>
            </div>
            <Separator />

            <audio ref={audioRef} />

            {isLoading ? (
                <div className="flex h-64 items-center justify-center">
                    <div className="h-8 w-8 animate-spin rounded-full border-4 border-primary border-t-transparent" />
                </div>
            ) : categories.length > 0 ? (
                <AnimatedGroup className="space-y-4" preset="blur-slide">
                    {/* Language Selector */}
                    {availableLanguages.length > 1 && (
                        <div className="flex items-center gap-2">
                            <span className="text-muted-foreground text-sm">Language:</span>
                            <div className="flex gap-1">
                                {availableLanguages.map((lang) => (
                                    <Button key={lang} onClick={() => setSelectedLanguage(lang)} size="sm" variant={selectedLanguage === lang ? "default" : "outline"}>
                                        {getLangLabel(lang)}
                                    </Button>
                                ))}
                            </div>
                        </div>
                    )}

                    {/* Category Selector - Mobile */}
                    <div className="lg:hidden">
                        <DropdownMenu>
                            <DropdownMenuTrigger asChild>
                                <Button className="w-full justify-between bg-transparent" variant="outline">
                                    {currentCategory?.name ?? "Select Category"}
                                    <ChevronDown className="ml-2 h-4 w-4" />
                                </Button>
                            </DropdownMenuTrigger>
                            <DropdownMenuContent className="w-full">
                                {categories.map((cat) => (
                                    <DropdownMenuItem key={cat.id} onClick={() => setActiveCategory(cat.id)}>
                                        {cat.name}
                                    </DropdownMenuItem>
                                ))}
                            </DropdownMenuContent>
                        </DropdownMenu>
                    </div>

                    {/* Category Tabs - Desktop */}
                    <div className="hidden lg:block">
                        <ScrollArea className="w-full">
                            <div className="flex gap-1 border-border border-b pb-2">
                                {categories.map((cat) => (
                                    <button className={`rounded-md px-4 py-2 font-medium text-sm transition-colors ${activeCategory === cat.id ? "bg-primary text-primary-foreground" : "hover:bg-muted"}`} key={cat.id} onClick={() => setActiveCategory(cat.id)}>
                                        {cat.name}
                                    </button>
                                ))}
                            </div>
                        </ScrollArea>
                    </div>

                    {/* Voice Lines */}
                    <ScrollArea className="h-[60vh]">
                        <div className="space-y-2 pr-4">
                            {currentCategory?.lines.map((line) => (
                                <motion.div
                                    animate={{ opacity: 1, y: 0 }}
                                    className={`cursor-pointer rounded-lg border p-4 transition-colors ${activeLine === line.id ? "border-primary bg-primary/10" : "border-border bg-card/50 hover:border-primary/50"}`}
                                    initial={{ opacity: 0, y: 10 }}
                                    key={line.id}
                                    onClick={() => handleLineSelect(line.id, line.url)}
                                >
                                    <div className="flex items-start justify-between gap-3">
                                        <div className="flex-1">
                                            <div className="flex items-center gap-2">
                                                <button className="rounded-full bg-primary/20 p-1.5">{activeLine === line.id && isPlaying ? <Pause className="h-4 w-4 text-primary" /> : <Play className="h-4 w-4 text-primary" />}</button>
                                                <h4 className="font-medium">{line.name}</h4>
                                            </div>
                                            <p className="mt-2 text-muted-foreground text-sm">{line.transcript}</p>
                                        </div>
                                        <div className="flex gap-1">
                                            <Button onClick={toggleMute} size="icon" variant="ghost">
                                                {isMuted ? <VolumeX className="h-4 w-4" /> : <Volume2 className="h-4 w-4" />}
                                            </Button>
                                            <Button onClick={(e) => handleDownload(e, line)} size="icon" variant="ghost">
                                                <Download className="h-4 w-4" />
                                            </Button>
                                        </div>
                                    </div>
                                </motion.div>
                            ))}
                        </div>
                    </ScrollArea>
                </AnimatedGroup>
            ) : (
                <p className="text-muted-foreground">No voice lines available for this operator.</p>
            )}
        </div>
    );
}

function getCategoryName(placeType: string): string {
    const mapping: Record<string, string> = {
        HOME_PLACE: "Greetings",
        NEW_YEAR: "Greetings",
        GREETING: "Greetings",
        ANNIVERSARY: "Greetings",
        BIRTHDAY: "Greetings",
        HOME_SHOW: "Interaction",
        HOME_WAIT: "Interaction",
        GACHA: "Recruitment",
        LEVEL_UP: "Level Up",
        EVOLVE_ONE: "Level Up",
        EVOLVE_TWO: "Level Up",
        SQUAD: "Squad",
        SQUAD_FIRST: "Squad",
        BATTLE_START: "Combat",
        BATTLE_FACE_ENEMY: "Combat",
        BATTLE_SELECT: "Combat",
        BATTLE_PLACE: "Combat",
        BATTLE_SKILL_1: "Combat",
        BATTLE_SKILL_2: "Combat",
        BATTLE_SKILL_3: "Combat",
        BATTLE_SKILL_4: "Combat",
        FOUR_STAR: "Combat",
        THREE_STAR: "Combat",
        TWO_STAR: "Combat",
        LOSE: "Combat",
        BUILDING_PLACE: "Base",
        BUILDING_TOUCHING: "Base",
        BUILDING_FAVOR_BUBBLE: "Base",
        LOADING_PANEL: "Other",
    };
    return mapping[placeType] ?? "Other";
}

function getLangLabel(lang: LangType): string {
    const labels: Record<string, string> = {
        JP: "Japanese",
        CN_MANDARIN: "Chinese",
        EN: "English",
        KR: "Korean",
        RUS: "Russian",
        ITA: "Italian",
        CN_TOPOLECT: "CN Dialect",
        LINKAGE: "Collaboration",
        GER: "German",
        FRE: "French",
    };
    return labels[lang] ?? lang;
}
