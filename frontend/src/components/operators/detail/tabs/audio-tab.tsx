"use client";

import { useState, useEffect, useRef, useCallback } from "react";
import { motion } from "motion/react";
import { PlayCircle, PauseCircle, Download, ChevronDown } from "lucide-react";
import type { Operator, Voice, LangType } from "~/types/api";
import { InView } from "~/components/ui/motion-primitives/in-view";
import { Button } from "~/components/ui/shadcn/button";
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from "~/components/ui/shadcn/dropdown-menu";
import { ScrollArea } from "~/components/ui/shadcn/scroll-area";
import { cn } from "~/lib/utils";

interface AudioTabProps {
    operator: Operator;
}

interface VoiceCategory {
    id: string;
    name: string;
    lines: VoiceLine[];
}

interface VoiceLine {
    id: string;
    name: string;
    description: string;
    transcript: string;
    url: string;
    cvName: string[];
}

const LANG_DISPLAY: Record<string, string> = {
    JP: "Japanese",
    CN_MANDARIN: "Chinese",
    EN: "English",
    KR: "Korean",
    RUS: "Russian",
    ITA: "Italian",
    GER: "German",
    FRE: "French",
};

const CATEGORY_MAP: Record<string, string> = {
    HOME_PLACE: "Greetings",
    HOME_SHOW: "Greetings",
    HOME_WAIT: "Greetings",
    GREETING: "Greetings",
    NEW_YEAR: "Greetings",
    ANNIVERSARY: "Greetings",
    BIRTHDAY: "Greetings",
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
    GACHA: "Recruitment",
    LEVEL_UP: "Level Up",
    EVOLVE_ONE: "Level Up",
    EVOLVE_TWO: "Level Up",
    SQUAD: "Squad",
    SQUAD_FIRST: "Squad",
    BUILDING_PLACE: "Base",
    BUILDING_TOUCHING: "Base",
    BUILDING_FAVOR_BUBBLE: "Base",
    LOADING_PANEL: "Other",
};

export function AudioTab({ operator }: AudioTabProps) {
    const [voices, setVoices] = useState<Voice[]>([]);
    const [categories, setCategories] = useState<VoiceCategory[]>([]);
    const [activeCategory, setActiveCategory] = useState("");
    const [activeLine, setActiveLine] = useState<string | null>(null);
    const [isPlaying, setIsPlaying] = useState(false);
    const [isMuted, setIsMuted] = useState(false);
    const [selectedLang, setSelectedLang] = useState(0);
    const [availableLangs, setAvailableLangs] = useState<LangType[]>([]);
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
                if (response.ok) {
                    const data = (await response.json()) as { voices: Voice[] };
                    setVoices(data.voices ?? []);
                }
            } catch (error) {
                console.error("Failed to fetch voices:", error);
            } finally {
                setIsLoading(false);
            }
        };
        void fetchVoices();
    }, [operator.id]);

    // Process voices into categories
    useEffect(() => {
        if (voices.length === 0) return;

        // Get available languages
        const langs = new Set<LangType>();
        for (const voice of voices) {
            for (const data of voice.data ?? []) {
                if (data.language) {
                    langs.add(data.language);
                }
            }
        }
        setAvailableLangs(Array.from(langs));

        // Group voices by category
        const categoryMap = new Map<string, VoiceLine[]>();
        for (const voice of voices) {
            const categoryName = CATEGORY_MAP[voice.placeType] ?? "Other";
            const voiceData = voice.data?.[selectedLang];

            const line: VoiceLine = {
                id: voice.charWordId ?? voice.voiceId,
                name: voice.voiceTitle,
                description: voice.lockDescription ?? "",
                transcript: voice.voiceText,
                url: voiceData?.voiceUrl ?? "",
                cvName: voiceData?.cvName ?? [],
            };

            if (!categoryMap.has(categoryName)) {
                categoryMap.set(categoryName, []);
            }
            categoryMap.get(categoryName)?.push(line);
        }

        const cats: VoiceCategory[] = [];
        categoryMap.forEach((lines, name) => {
            cats.push({
                id: name.toLowerCase().replace(/\s+/g, "-"),
                name,
                lines,
            });
        });

        // Sort categories
        const order = ["Greetings", "Combat", "Base", "Level Up", "Recruitment", "Squad", "Other"];
        cats.sort((a, b) => {
            const idxA = order.indexOf(a.name);
            const idxB = order.indexOf(b.name);
            return (idxA === -1 ? 999 : idxA) - (idxB === -1 ? 999 : idxB);
        });

        setCategories(cats);
        if (cats.length > 0 && !activeCategory) {
            setActiveCategory(cats[0]?.id ?? "");
        }
    }, [voices, selectedLang, activeCategory]);

    const currentCategory = categories.find((c) => c.id === activeCategory);
    const currentLine = currentCategory?.lines.find((l) => l.id === activeLine);

    const handlePlay = useCallback(
        (lineId: string) => {
            if (activeLine === lineId && isPlaying) {
                audioRef.current?.pause();
                setIsPlaying(false);
            } else {
                setActiveLine(lineId);
                setIsPlaying(true);
                setTimeout(() => {
                    void audioRef.current?.play();
                }, 50);
            }
        },
        [activeLine, isPlaying],
    );

    const handleDownload = useCallback(
        (url: string, name: string) => {
            if (url) {
                const a = document.createElement("a");
                a.href = url;
                a.download = `${operator.name}-${name}.mp3`;
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
    }, [activeLine]);

    if (isLoading) {
        return (
            <div className="flex h-64 items-center justify-center">
                <div className="h-8 w-8 animate-spin rounded-full border-2 border-primary border-t-transparent" />
            </div>
        );
    }

    const voiceActor = voices[0]?.data?.[selectedLang]?.cvName ?? [];

    return (
        <div className="space-y-6">
            {/* Voice info */}
            <InView
                variants={{
                    hidden: { opacity: 0, y: 20 },
                    visible: { opacity: 1, y: 0 },
                }}
                transition={{ duration: 0.4 }}
            >
                <div className="rounded-xl border bg-card/50 p-6 backdrop-blur-sm">
                    <h2 className="mb-4 text-xl font-semibold">Voice Information</h2>

                    <div className="mb-4 grid gap-2 text-sm">
                        <div className="flex gap-2">
                            <span className="text-muted-foreground">Voice Actor:</span>
                            <span>{voiceActor.join(", ") || "Unknown"}</span>
                        </div>
                        <div className="flex gap-2">
                            <span className="text-muted-foreground">Language:</span>
                            <span>{LANG_DISPLAY[availableLangs[selectedLang] ?? ""] ?? "Unknown"}</span>
                        </div>
                    </div>

                    {availableLangs.length > 1 && (
                        <div className="flex flex-wrap gap-2">
                            {availableLangs.map((lang, idx) => (
                                <Button
                                    key={lang}
                                    variant={selectedLang === idx ? "secondary" : "outline"}
                                    size="sm"
                                    onClick={() => {
                                        setSelectedLang(idx);
                                        setActiveLine(null);
                                        setIsPlaying(false);
                                    }}
                                >
                                    {LANG_DISPLAY[lang] ?? lang}
                                </Button>
                            ))}
                        </div>
                    )}
                </div>
            </InView>

            {/* Voice lines */}
            <InView
                variants={{
                    hidden: { opacity: 0, y: 20 },
                    visible: { opacity: 1, y: 0 },
                }}
                transition={{ duration: 0.4, delay: 0.1 }}
            >
                <div className="rounded-xl border bg-card/50 p-6 backdrop-blur-sm">
                    <h2 className="mb-4 text-xl font-semibold">Voice Lines</h2>

                    {/* Category selector - desktop */}
                    <div className="mb-4 hidden gap-2 overflow-x-auto lg:flex">
                        {categories.map((cat) => (
                            <button key={cat.id} onClick={() => setActiveCategory(cat.id)} className={cn("whitespace-nowrap rounded-lg px-4 py-2 text-sm transition-colors", activeCategory === cat.id ? "bg-secondary text-secondary-foreground" : "text-muted-foreground hover:bg-muted hover:text-foreground")}>
                                {cat.name}
                            </button>
                        ))}
                    </div>

                    {/* Category selector - mobile */}
                    <div className="mb-4 lg:hidden">
                        <DropdownMenu>
                            <DropdownMenuTrigger asChild>
                                <Button variant="outline" className="w-full justify-between bg-transparent">
                                    {currentCategory?.name ?? "Select Category"}
                                    <ChevronDown className="h-4 w-4" />
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

                    {/* Voice lines list */}
                    <ScrollArea className="h-[400px]">
                        <div className="space-y-2 pr-4">
                            {currentCategory?.lines.map((line) => (
                                <motion.div key={line.id} className={cn("rounded-lg border p-4 transition-colors", activeLine === line.id ? "border-primary bg-primary/5" : "border-border hover:bg-muted/50")}>
                                    <div className="mb-2 flex items-start justify-between">
                                        <div>
                                            <h4 className="font-medium">{line.name}</h4>
                                            {line.description && <p className="text-xs text-muted-foreground">{line.description}</p>}
                                        </div>
                                        <div className="flex gap-1">
                                            <Button variant="ghost" size="icon" onClick={() => handlePlay(line.id)} disabled={!line.url}>
                                                {activeLine === line.id && isPlaying ? <PauseCircle className="h-5 w-5" /> : <PlayCircle className="h-5 w-5" />}
                                            </Button>
                                            <Button variant="ghost" size="icon" onClick={() => handleDownload(line.url, line.name)} disabled={!line.url}>
                                                <Download className="h-4 w-4" />
                                            </Button>
                                        </div>
                                    </div>
                                    <p className="text-sm text-muted-foreground">{line.transcript}</p>
                                </motion.div>
                            ))}
                        </div>
                    </ScrollArea>

                    {/* Audio element */}
                    {currentLine?.url && <audio ref={audioRef} src={currentLine.url} muted={isMuted} onEnded={() => setIsPlaying(false)} />}
                </div>
            </InView>
        </div>
    );
}
