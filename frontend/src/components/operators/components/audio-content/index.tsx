import { useState, useRef, useEffect, useCallback } from "react";
import { Card, CardContent } from "~/components/ui/card";
import { ScrollArea, ScrollBar } from "~/components/ui/scroll-area";
import { Tabs, TabsList, TabsTrigger } from "~/components/ui/tabs";
import { Button } from "~/components/ui/button";
import { Separator } from "~/components/ui/separator";
import { PlayCircle, PauseCircle, Download, Volume2, VolumeX, Volume1, ChevronDown } from "lucide-react";
import type { Operator } from "~/types/impl/api/static/operator";
import { type Voice, PlaceType, LangType } from "~/types/impl/api/static/voices";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "~/components/ui/select";
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from "~/components/ui/dropdown-menu";

// Voice line types
type VoiceLine = {
    id: string;
    name: string;
    description: string;
    transcript: string;
    url: string;
    cvName: string[];
};

type VoiceCategory = {
    id: string;
    name: string;
    lines: VoiceLine[];
};

// Helper function to convert PlaceType to user-friendly category name
const getCategoryName = (placeType: PlaceType): string => {
    switch (placeType) {
        case PlaceType.GREETING:
            return "Greetings";
        case PlaceType.BATTLE_START:
        case PlaceType.BATTLE_FACE_ENEMY:
        case PlaceType.BATTLE_SELECT:
        case PlaceType.BATTLE_PLACE:
        case PlaceType.BATTLE_SKILL_1:
        case PlaceType.BATTLE_SKILL_2:
        case PlaceType.BATTLE_SKILL_3:
        case PlaceType.BATTLE_SKILL_4:
            return "Combat";
        case PlaceType.HOME_PLACE:
        case PlaceType.HOME_SHOW:
        case PlaceType.HOME_WAIT:
            return "Interaction";
        case PlaceType.BUILDING_PLACE:
        case PlaceType.BUILDING_TOUCHING:
        case PlaceType.BUILDING_FAVOR_BUBBLE:
            return "Base";
        case PlaceType.LEVEL_UP:
            return "Level Up";
        case PlaceType.GACHA:
            return "Recruitment";
        case PlaceType.SQUAD:
        case PlaceType.SQUAD_FIRST:
            return "Squad";
        default:
            return "Other";
    }
};

// Helper to get description for voice line
const getVoiceDescription = (placeType: PlaceType): string => {
    switch (placeType) {
        case PlaceType.GREETING:
            return "Greeting";
        case PlaceType.BATTLE_START:
            return "Battle Start";
        case PlaceType.BATTLE_FACE_ENEMY:
            return "Facing Enemy";
        case PlaceType.BATTLE_SELECT:
            return "Selected in Battle";
        case PlaceType.BATTLE_PLACE:
            return "Deployment";
        case PlaceType.BATTLE_SKILL_1:
            return "Activating Skill 1";
        case PlaceType.BATTLE_SKILL_2:
            return "Activating Skill 2";
        case PlaceType.BATTLE_SKILL_3:
            return "Activating Skill 3";
        case PlaceType.HOME_PLACE:
            return "Assigned as Assistant";
        case PlaceType.HOME_SHOW:
            return "Conversation 1";
        case PlaceType.HOME_WAIT:
            return "Conversation 2";
        case PlaceType.BUILDING_PLACE:
            return "Assigned to Facility";
        case PlaceType.BUILDING_TOUCHING:
            return "Interacting in Base";
        case PlaceType.BUILDING_FAVOR_BUBBLE:
            return "Trust Tap";
        case PlaceType.LEVEL_UP:
            return "Level Up";
        case PlaceType.GACHA:
            return "Recruitment";
        case PlaceType.SQUAD:
            return "Added to Squad";
        case PlaceType.SQUAD_FIRST:
            return "First Time in Squad";
        default:
            return placeType.toString();
    }
};

// Waveform component for audio visualization
const Waveform = ({ audioUrl, isPlaying, onSeek, audioRef }: { audioUrl: string; isPlaying: boolean; onSeek: (time: number) => void; audioRef: React.RefObject<HTMLAudioElement> }) => {
    const canvasRef = useRef<HTMLCanvasElement | null>(null);
    const [audioBuffer, setAudioBuffer] = useState<AudioBuffer | null>(null);
    const [duration, setDuration] = useState<number>(0);
    const [currentTime, setCurrentTime] = useState<number>(0);
    const audioContextRef = useRef<AudioContext | null>(null);
    const animationRef = useRef<number>(0);

    // Initialize audio context and load audio data
    useEffect(() => {
        if (audioUrl === "#") return;

        const initAudio = async () => {
            try {
                // Create audio context if it doesn't exist
                if (!audioContextRef.current) {
                    // Properly type the AudioContext
                    const AudioContextClass = window.AudioContext || (window as unknown as { webkitAudioContext?: AudioContext["constructor"] }).webkitAudioContext;

                    if (AudioContextClass) {
                        audioContextRef.current = new AudioContextClass();
                    } else {
                        console.error("AudioContext not supported in this browser");
                        return;
                    }
                }

                const response = await fetch(audioUrl);
                const arrayBuffer = await response.arrayBuffer();
                if (audioContextRef.current) {
                    const buffer = await audioContextRef.current.decodeAudioData(arrayBuffer);
                    setAudioBuffer(buffer);
                    setDuration(buffer.duration);
                }
            } catch (error) {
                console.error("Error loading audio for waveform:", error);
            }
        };

        void initAudio();

        // Store animation frame ID for cleanup
        const animationFrameId = animationRef.current;

        return () => {
            cancelAnimationFrame(animationFrameId);
        };
    }, [audioUrl]);

    // Draw waveform on canvas
    useEffect(() => {
        if (!canvasRef.current || !audioBuffer) return;

        const canvas = canvasRef.current;
        const ctx = canvas.getContext("2d");
        if (!ctx) return;

        // Get the canvas dimensions
        const width = canvas.width;
        const height = canvas.height;

        // Clear the canvas
        ctx.clearRect(0, 0, width, height);

        // Get audio data
        const channelData = audioBuffer.getChannelData(0);
        const step = Math.ceil(channelData.length / width);

        // Draw background
        ctx.fillStyle = "#1a1a1a";
        ctx.fillRect(0, 0, width, height);

        // Draw waveform
        ctx.beginPath();
        ctx.lineWidth = 1;
        ctx.strokeStyle = "#4a4a4a";

        for (let i = 0; i < width; i++) {
            const startIdx = i * step;
            let min = 1.0;
            let max = -1.0;

            for (let j = 0; j < step; j++) {
                if (startIdx + j < channelData.length) {
                    const datum = channelData[startIdx + j];
                    if (typeof datum === "number") {
                        if (datum < min) min = datum;
                        if (datum > max) max = datum;
                    }
                }
            }

            const y = height / 2;
            const lineHeight = Math.max(1, (max - min) * height * 0.8);

            ctx.moveTo(i, y - lineHeight / 2);
            ctx.lineTo(i, y + lineHeight / 2);
        }
        ctx.stroke();

        // Draw playback position
        if (duration > 0) {
            const position = (currentTime / duration) * width;
            ctx.beginPath();
            ctx.lineWidth = 2;
            ctx.strokeStyle = "#a885ff";
            ctx.moveTo(position, 0);
            ctx.lineTo(position, height);
            ctx.stroke();
        }
    }, [audioBuffer, currentTime, duration]);

    // Update playback position
    useEffect(() => {
        let frameId = 0;

        const updatePlaybackPosition = () => {
            if (isPlaying) {
                const audioElement = audioRef.current;
                if (audioElement instanceof HTMLAudioElement) {
                    setCurrentTime(audioElement.currentTime);
                    frameId = requestAnimationFrame(updatePlaybackPosition);
                }
            }
        };

        if (isPlaying) {
            frameId = requestAnimationFrame(updatePlaybackPosition);
        }

        return () => {
            cancelAnimationFrame(frameId);
        };
    }, [isPlaying, audioRef]);

    // Handle click for seeking
    const handleCanvasClick = (e: React.MouseEvent<HTMLCanvasElement>) => {
        if (!canvasRef.current || !duration) return;

        const rect = canvasRef.current.getBoundingClientRect();
        const clickX = e.clientX - rect.left;
        const canvasWidth = canvasRef.current.width;
        const clickPercent = clickX / canvasWidth;
        const seekTime = duration * clickPercent;

        setCurrentTime(seekTime);
        onSeek(seekTime);
    };

    return (
        <div className="relative my-2 h-20 w-full">
            <canvas ref={canvasRef} width={800} height={80} className="h-full w-full cursor-pointer rounded-sm" onClick={handleCanvasClick} />
            {!audioBuffer && audioUrl !== "#" && (
                <div className="absolute inset-0 flex items-center justify-center bg-black/20">
                    <div className="h-4 w-4 animate-spin rounded-full border-2 border-primary border-t-transparent"></div>
                </div>
            )}
        </div>
    );
};

function AudioContent({ operator }: { operator: Operator }) {
    const [activeCategory, setActiveCategory] = useState<string>("Greetings");
    const [activeLine, setActiveLine] = useState<string | null>(null);
    const [isPlaying, setIsPlaying] = useState<boolean>(false);
    const [isMuted, setIsMuted] = useState<boolean>(false);
    const [volume, setVolume] = useState<number>(0.7);
    const [isLoading, setIsLoading] = useState<boolean>(true);
    const audioRef = useRef<HTMLAudioElement | null>(null);

    const [voiceCategories, setVoiceCategories] = useState<VoiceCategory[]>([]);
    const [voiceActor, setVoiceActor] = useState<{ name: string[]; language: string }>({
        name: ["Unknown"],
        language: "Unknown",
    });

    const [availableLanguages, setAvailableLanguages] = useState<Array<{ langType: LangType; label: string }>>([]);
    const [selectedLanguageIndex, setSelectedLanguageIndex] = useState<number>(0);

    const fetchVoices = useCallback(async () => {
        try {
            const data = (await (
                await fetch("/api/static", {
                    method: "POST",
                    headers: {
                        "Content-Type": "application/json",
                    },
                    body: JSON.stringify({
                        type: "voices",
                        id: operator.id,
                    }),
                })
            ).json()) as { voices: Voice[] };

            return data.voices;
        } catch (error) {
            console.error("Error fetching voices:", error);
            return [];
        }
    }, [operator.id]);

    // Setup fallback data if API fails or returns no data
    const setupFallbackData = useCallback(() => {
        const fallbackCategories: VoiceCategory[] = [
            {
                id: "greetings",
                name: "Greetings",
                lines: [
                    {
                        id: "greeting-1",
                        name: "Self Introduction",
                        description: "Operator's self introduction",
                        transcript: `Hello, Doctor. I am ${operator.name}. It's a pleasure to meet you.`,
                        url: "#", // This would be a real URL in production
                        cvName: ["Unknown"],
                    },
                    {
                        id: "greeting-2",
                        name: "Appointed as Secretary",
                        description: "When appointed as secretary",
                        transcript: `You need me, Doctor? I'll do my best to assist you.`,
                        url: "#",
                        cvName: ["Unknown"],
                    },
                ],
            },
            {
                id: "combat",
                name: "Combat",
                lines: [
                    {
                        id: "battle-1",
                        name: "Battle Start",
                        description: "When starting a battle",
                        transcript: `Let's begin the operation!`,
                        url: "#",
                        cvName: ["Unknown"],
                    },
                    {
                        id: "battle-2",
                        name: "Skill Activation",
                        description: "When activating skill",
                        transcript: `Skill activated! Stand back!`,
                        url: "#",
                        cvName: ["Unknown"],
                    },
                ],
            },
        ];

        setVoiceCategories(fallbackCategories);
        setActiveCategory("greetings");
        setIsLoading(false);
    }, [operator.name]);

    // Transform API voice data into our VoiceCategory format
    useEffect(() => {
        setIsLoading(true);
        void fetchVoices().then((fetchedVoices) => {
            if (fetchedVoices.length === 0) {
                // If no voices returned from API, use fallback data
                setupFallbackData();
                return;
            }

            const availableLanguages: Array<{ langType: LangType; label: string }> = [];
            for (const voice of fetchedVoices) {
                for (const data of voice.data ?? []) {
                    if (data.language !== undefined) {
                        const language = getLangTypeName(data.language);
                        if (!availableLanguages.some((lang) => lang.langType === data.language)) {
                            availableLanguages.push({ langType: data.language, label: language });
                        }
                    }
                }
            }

            setAvailableLanguages(availableLanguages);

            const langIndex = availableLanguages.findIndex((lang) => lang.langType === fetchedVoices[0]?.data?.[selectedLanguageIndex]?.language);
            setSelectedLanguageIndex(langIndex);

            // Get voice actor info from the first voice if available
            if (fetchedVoices[0]?.data?.[langIndex]?.language !== undefined) {
                const language = getLangTypeName(fetchedVoices[0].data[langIndex]?.language ?? LangType.JP);
                const cvName = fetchedVoices[0].data[langIndex]?.cvName ?? ["Unknown"];
                setVoiceActor({
                    name: cvName,
                    language,
                });
            }

            // Group voices by category
            const categoriesMap = new Map<string, VoiceLine[]>();

            for (const voice of fetchedVoices) {
                // Get category name from place type
                const categoryName = getCategoryName(voice.placeType);

                // Create voice line object
                const voiceLine: VoiceLine = {
                    id: voice.charWordId ?? voice.id ?? `${voice.voiceId}-${voice.voiceIndex}`,
                    name: voice.voiceTitle ?? getVoiceDescription(voice.placeType),
                    description: voice.lockDescription ?? getVoiceDescription(voice.placeType),
                    transcript: voice.voiceText ?? "No transcript available",
                    url: voice.data?.[langIndex]?.voiceURL ?? "#",
                    cvName: voice.data?.[langIndex]?.cvName ?? ["Unknown"],
                };

                // Add to category map
                if (!categoriesMap.has(categoryName)) {
                    categoriesMap.set(categoryName, []);
                }
                const lines = categoriesMap.get(categoryName);
                if (lines) {
                    lines.push(voiceLine);
                }
            }

            // Convert map to array of VoiceCategory objects
            const newCategories: VoiceCategory[] = [];
            categoriesMap.forEach((lines, name) => {
                if (lines) {
                    newCategories.push({
                        id: name.toLowerCase().replace(/\s+/g, "-"),
                        name,
                        lines,
                    });
                }
            });

            // Sort categories in a specific order
            const categoryOrder = ["Greetings", "Combat", "Interaction", "Base", "Level Up", "Recruitment", "Squad", "Other"];
            newCategories.sort((a, b) => {
                const indexA = categoryOrder.indexOf(a.name);
                const indexB = categoryOrder.indexOf(b.name);
                return (indexA === -1 ? 999 : indexA) - (indexB === -1 ? 999 : indexB);
            });

            setVoiceCategories(newCategories);

            // Set active category to first category if it exists
            if (newCategories.length > 0) {
                setActiveCategory(newCategories[0]?.id ?? "greetings");
            }

            setIsLoading(false);
        });
    }, [fetchVoices, selectedLanguageIndex, setupFallbackData]);

    // Helper function to get language name from LangType
    const getLangTypeName = (langType: LangType): string => {
        switch (langType) {
            case LangType.JP:
                return "Japanese";
            case LangType.CN_MANDARIN:
                return "Chinese";
            case LangType.CN_TOPOLECT:
                return "Chinese (Regional)";
            case LangType.KR:
                return "Korean";
            case LangType.EN:
                return "English";
            case LangType.RUS:
                return "Russian";
            case LangType.ITA:
                return "Italian";
            case LangType.GER:
                return "German";
            default:
                return "Unknown";
        }
    };

    const currentCategory = voiceCategories.find((category) => category.id === activeCategory) ?? voiceCategories[0];
    const currentLine = currentCategory?.lines.find((line) => line.id === activeLine) ?? null;

    // Handle play/pause
    const togglePlayPause = () => {
        if (!audioRef.current || !currentLine) return;

        if (isPlaying) {
            audioRef.current.pause();
        } else {
            void audioRef.current.play();
        }

        setIsPlaying(!isPlaying);
    };

    // Handle line selection
    const handleLineSelect = (lineId: string) => {
        if (activeLine === lineId && isPlaying) {
            // If clicking the active line and it's playing, pause it
            if (audioRef.current) {
                audioRef.current.pause();
                setIsPlaying(false);
            }
        } else {
            // Otherwise, set the new line and play it
            setActiveLine(lineId);
            setIsPlaying(true);

            // Need to wait for the ref to update with the new source
            setTimeout(() => {
                if (audioRef.current) {
                    audioRef.current.volume = volume;
                    void audioRef.current.play();
                }
            }, 50);
        }
    };

    // Toggle mute
    const toggleMute = () => {
        if (!audioRef.current) return;

        audioRef.current.muted = !isMuted;
        setIsMuted(!isMuted);
    };

    // Handle audio end
    useEffect(() => {
        const handleAudioEnd = () => {
            setIsPlaying(false);
        };

        const audioElement = audioRef.current;

        if (audioElement) {
            audioElement.addEventListener("ended", handleAudioEnd);
        }

        return () => {
            if (audioElement) {
                audioElement.removeEventListener("ended", handleAudioEnd);
            }
        };
    }, [activeLine]);

    const handleLanguageChange = (index: number) => {
        setSelectedLanguageIndex(index);
        setActiveLine(null);
        setIsPlaying(false);
    };

    // Add new seek function
    const handleSeek = useCallback(
        (time: number) => {
            if (!audioRef.current) return;
            audioRef.current.currentTime = time;
            if (!isPlaying) {
                void audioRef.current.play();
                setIsPlaying(true);
            }
        },
        [isPlaying],
    );

    // Handle download without triggering playback
    const handleDownload = useCallback(
        (e: React.MouseEvent, line: VoiceLine) => {
            e.stopPropagation();
            e.preventDefault();

            if (line.url !== "#") {
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

    return (
        <>
            <div className="p-2 px-4 backdrop-blur-2xl">
                <span className="text-lg font-bold sm:text-xl md:text-3xl">Voice Lines</span>
            </div>

            <Separator />
            <div className="flex flex-col gap-4 p-2 pb-40">
                {/* Voice actor information with language selector */}
                <Card className="bg-card/50 backdrop-blur-sm">
                    <CardContent className="pt-6">
                        <div>
                            <div className="flex items-center justify-between">
                                <h2 className="text-lg font-semibold sm:text-xl">Voice Information</h2>
                            </div>
                            <Separator className="my-2" />
                            <div className="grid grid-cols-[100px_1fr] gap-2 pb-2 sm:grid-cols-[120px_1fr]">
                                <span className="text-sm text-muted-foreground sm:text-base">Voice Actor:</span>
                                <span className="text-sm sm:text-base">{voiceActor.name.join(", ")}</span>
                                <span className="text-sm text-muted-foreground sm:text-base">Language:</span>
                                <span className="text-sm sm:text-base">{voiceActor.language}</span>
                            </div>

                            {/* Language selector - responsive version */}
                            {availableLanguages.length > 1 && (
                                <div className="mt-2 flex flex-col space-y-2 sm:flex-row sm:items-center sm:space-x-2 sm:space-y-0">
                                    <span className="text-sm text-muted-foreground">Language:</span>

                                    {/* Mobile dropdown selector */}
                                    <div className="w-full sm:hidden">
                                        <Select value={selectedLanguageIndex.toString()} onValueChange={(value) => handleLanguageChange(parseInt(value))}>
                                            <SelectTrigger className="w-full">
                                                <SelectValue placeholder="Select Language" />
                                            </SelectTrigger>
                                            <SelectContent>
                                                {availableLanguages.map((lang, index) => (
                                                    <SelectItem key={lang.langType} value={index.toString()}>
                                                        {lang.label}
                                                    </SelectItem>
                                                ))}
                                            </SelectContent>
                                        </Select>
                                    </div>

                                    {/* Desktop button group */}
                                    <div className="hidden space-x-1 sm:flex">
                                        {availableLanguages.map((lang, index) => (
                                            <Button key={lang.langType} variant={index === selectedLanguageIndex ? "secondary" : "outline"} size="sm" onClick={() => handleLanguageChange(index)} className="h-8 text-xs">
                                                {lang.label}
                                            </Button>
                                        ))}
                                    </div>
                                </div>
                            )}
                        </div>
                    </CardContent>
                </Card>

                {/* Voice line player */}
                <Card className="bg-card/50 backdrop-blur-sm">
                    <CardContent className="p-0">
                        {isLoading ? (
                            <div className="flex h-40 items-center justify-center">
                                <div className="text-center">
                                    <div className="mb-2 h-6 w-6 animate-spin rounded-full border-2 border-primary border-t-transparent"></div>
                                    <p>Loading voice lines...</p>
                                </div>
                            </div>
                        ) : voiceCategories.length > 0 ? (
                            <div className="flex w-full flex-col flex-wrap lg:flex-nowrap lg:p-[0_24px]">
                                {/* Desktop tabs */}
                                <div className="hidden w-full flex-1 lg:block">
                                    <div className="w-full flex-1 border-b px-4 pt-2">
                                        <ScrollArea className="w-full whitespace-nowrap rounded-md pb-2">
                                            <Tabs defaultValue={activeCategory} onValueChange={setActiveCategory} className="w-full">
                                                <TabsList className="inline-flex h-10 w-full items-center justify-center rounded-md bg-transparent p-1 text-muted-foreground">
                                                    {voiceCategories.map((category) => (
                                                        <TabsTrigger key={category.id} value={category.id} className="hover:bg-accent/50 data-[state=active]:bg-secondary">
                                                            {category.name}
                                                        </TabsTrigger>
                                                    ))}
                                                </TabsList>
                                            </Tabs>
                                            <ScrollBar orientation="horizontal" className="h-0" />
                                        </ScrollArea>
                                    </div>
                                </div>

                                {/* Mobile dropdown category selector */}
                                <div className="px-4 py-3 lg:hidden">
                                    <DropdownMenu>
                                        <DropdownMenuTrigger asChild>
                                            <Button variant="outline" className="w-full justify-between">
                                                {currentCategory?.name ?? "Select Category"}
                                                <ChevronDown className="ml-2 h-4 w-4" />
                                            </Button>
                                        </DropdownMenuTrigger>
                                        <DropdownMenuContent className="w-full" align="start">
                                            {voiceCategories.map((category) => (
                                                <DropdownMenuItem key={category.id} onClick={() => setActiveCategory(category.id)} className={category.id === activeCategory ? "bg-secondary/20" : ""}>
                                                    {category.name}
                                                </DropdownMenuItem>
                                            ))}
                                        </DropdownMenuContent>
                                    </DropdownMenu>
                                </div>

                                <div className="flex-1">
                                    {voiceCategories.map((category) => (
                                        <div key={category.id} className={`mt-0 ${category.id === activeCategory ? "block" : "hidden"}`}>
                                            <div className="p-2 sm:p-4">
                                                <h3 className="mb-2 text-base font-medium sm:mb-4 sm:text-lg">{category.name} Voice Lines</h3>
                                                <ScrollArea className="h-[40vh] pr-2 sm:h-[50vh] sm:pr-4 md:h-[60vh]">
                                                    <div className="space-y-2 sm:space-y-3">
                                                        {category.lines.map((line) => {
                                                            if (line.id.includes("CN_TOPOLECT") || line.id.includes("ITA") || line.id.includes("GER") || line.id.includes("RUS")) {
                                                                return null;
                                                            }
                                                            return (
                                                                <div key={line.id} className={`cursor-pointer rounded-lg border p-2 transition-colors sm:p-3 ${activeLine === line.id ? "bg-secondary/20" : "hover:bg-accent/50"}`} onClick={() => handleLineSelect(line.id)}>
                                                                    <div className="flex items-center justify-between">
                                                                        <div className="flex flex-1 items-center gap-1 sm:gap-2">
                                                                            <Button variant="ghost" size="icon" className="h-6 w-6 rounded-full sm:h-8 sm:w-8">
                                                                                {isPlaying && activeLine === line.id ? <PauseCircle className="h-4 w-4 sm:h-6 sm:w-6" /> : <PlayCircle className="h-4 w-4 sm:h-6 sm:w-6" />}
                                                                            </Button>
                                                                            <div>
                                                                                <div className="text-sm font-medium sm:text-base">{line.name}</div>
                                                                                <div className="text-xs text-muted-foreground">{line.description}</div>
                                                                            </div>
                                                                        </div>
                                                                        <div onClick={(e) => e.stopPropagation()} className="pointer-events-auto">
                                                                            <Button variant="ghost" size="icon" className="h-6 w-6 sm:h-8 sm:w-8" disabled={line.url === "#"} onClick={(e) => handleDownload(e, line)}>
                                                                                <Download className="h-3 w-3 sm:h-4 sm:w-4" />
                                                                            </Button>
                                                                        </div>
                                                                    </div>
                                                                    <div className={`mt-2 rounded-md bg-secondary/30 p-2 sm:mt-3 sm:p-3 ${activeLine === line.id ? "block" : "block"}`}>
                                                                        <p className="text-xs italic sm:text-sm">{line.transcript}</p>
                                                                    </div>
                                                                </div>
                                                            );
                                                        })}
                                                    </div>
                                                </ScrollArea>
                                            </div>
                                        </div>
                                    ))}
                                </div>
                            </div>
                        ) : (
                            <div className="flex h-40 items-center justify-center">
                                <p className="text-muted-foreground">No voice lines available for this operator.</p>
                            </div>
                        )}
                    </CardContent>
                </Card>

                {/* Audio playback controls - fixed at bottom */}
                {activeLine && (
                    <div className="fixed bottom-0 left-0 right-0 z-10 border-t bg-background/80 p-2 backdrop-blur-md">
                        <div className="container mx-auto flex max-w-full flex-col px-2 sm:px-4">
                            {/* Waveform visualization */}
                            <Waveform audioUrl={currentLine?.url ?? "#"} isPlaying={isPlaying} onSeek={handleSeek} audioRef={audioRef} />

                            <div className="flex items-center justify-between">
                                <div className="flex items-center gap-2">
                                    <Button variant="ghost" size="icon" className="h-8 w-8 sm:h-10 sm:w-10" onClick={togglePlayPause}>
                                        {isPlaying ? <PauseCircle className="h-5 w-5 sm:h-6 sm:w-6" /> : <PlayCircle className="h-5 w-5 sm:h-6 sm:w-6" />}
                                    </Button>

                                    <div className="flex max-w-[120px] flex-col sm:max-w-none">
                                        <span className="truncate text-xs font-medium sm:text-sm">{currentLine?.name}</span>
                                        <span className="truncate text-xs text-muted-foreground">{currentCategory?.name}</span>
                                    </div>
                                </div>

                                <div className="flex items-center gap-1 sm:gap-2">
                                    <Button variant="ghost" size="icon" onClick={toggleMute} className="h-8 w-8 sm:h-auto sm:w-auto">
                                        {isMuted || volume === 0 ? <VolumeX className="h-4 w-4 sm:h-5 sm:w-5" /> : volume > 0.5 ? <Volume2 className="h-4 w-4 sm:h-5 sm:w-5" /> : <Volume1 className="h-4 w-4 sm:h-5 sm:w-5" />}
                                    </Button>

                                    <input
                                        type="range"
                                        min="0"
                                        max="1"
                                        step="0.01"
                                        value={volume}
                                        onChange={(e) => {
                                            const newVolume = parseFloat(e.target.value);
                                            setVolume(newVolume);
                                            if (audioRef.current) {
                                                audioRef.current.volume = newVolume;
                                            }
                                        }}
                                        className="h-1.5 w-16 appearance-none rounded-full bg-secondary sm:h-2 sm:w-24"
                                    />
                                </div>
                            </div>
                        </div>
                    </div>
                )}

                {/* Hidden audio element */}
                <audio ref={audioRef} src={currentLine?.url} style={{ display: "none" }} controls />
            </div>
        </>
    );
}

export default AudioContent;
