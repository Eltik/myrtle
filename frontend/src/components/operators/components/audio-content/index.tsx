import { useState, useRef, useEffect, useCallback } from "react";
import { Card, CardContent } from "~/components/ui/card";
import { ScrollArea } from "~/components/ui/scroll-area";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "~/components/ui/tabs";
import { Button } from "~/components/ui/button";
import { Separator } from "~/components/ui/separator";
import { PlayCircle, PauseCircle, Download, Volume2, VolumeX } from "lucide-react";
import type { Operator } from "~/types/impl/api/static/operator";
import { type Voice, PlaceType, LangType } from "~/types/impl/api/static/voices";

// Voice line types
type VoiceLine = {
    id: string;
    name: string;
    description: string;
    transcript: string;
    url: string;
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

function AudioContent({ operator }: { operator: Operator }) {
    const [activeCategory, setActiveCategory] = useState<string>("Greetings");
    const [activeLine, setActiveLine] = useState<string | null>(null);
    const [isPlaying, setIsPlaying] = useState<boolean>(false);
    const [isMuted, setIsMuted] = useState<boolean>(false);
    const [volume, setVolume] = useState<number>(0.7);
    const [isLoading, setIsLoading] = useState<boolean>(true);
    const audioRef = useRef<HTMLAudioElement | null>(null);

    const [voiceCategories, setVoiceCategories] = useState<VoiceCategory[]>([]);
    const [voiceActor, setVoiceActor] = useState<{ name: string; language: string }>({
        name: "Unknown",
        language: "Unknown",
    });

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
                    },
                    {
                        id: "greeting-2",
                        name: "Appointed as Secretary",
                        description: "When appointed as secretary",
                        transcript: `You need me, Doctor? I'll do my best to assist you.`,
                        url: "#",
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
                    },
                    {
                        id: "battle-2",
                        name: "Skill Activation",
                        description: "When activating skill",
                        transcript: `Skill activated! Stand back!`,
                        url: "#",
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

            // Get voice actor info from the first voice if available
            if (fetchedVoices[0]?.data?.[0]?.language !== undefined) {
                const language = getLangTypeName(fetchedVoices[0].data[0].language);
                const cvName = "Unknown"; // We don't have access to cvName in the current data structure
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
                    url: voice.data?.[0]?.voiceURL ?? "#",
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
    }, [fetchVoices, setupFallbackData]);

    // Helper function to get language name from LangType
    const getLangTypeName = (langType: LangType): string => {
        switch (langType) {
            case LangType.JP:
                return "Japanese";
            case LangType.CN_MANDARIN:
            case LangType.CN_TOPOLECT:
                return "Chinese";
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

        if (audioRef.current) {
            audioRef.current.addEventListener("ended", handleAudioEnd);
        }

        return () => {
            if (audioRef.current) {
                audioRef.current.removeEventListener("ended", handleAudioEnd);
            }
        };
    }, [activeLine]);

    return (
        <div className="flex flex-col gap-4 p-4">
            {/* Voice actor information */}
            <Card className="bg-card/50 backdrop-blur-sm">
                <CardContent className="pt-6">
                    <div className="mb-4">
                        <h2 className="text-xl font-semibold">Voice Information</h2>
                        <Separator className="my-2" />
                        <div className="grid grid-cols-[120px_1fr] gap-2">
                            <span className="text-muted-foreground">Voice Actor:</span>
                            <span>{voiceActor.name}</span>

                            <span className="text-muted-foreground">Language:</span>
                            <span>{voiceActor.language}</span>
                        </div>
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
                        <Tabs defaultValue={activeCategory} onValueChange={setActiveCategory} className="w-full">
                            <div className="border-b px-4 pt-2">
                                <ScrollArea className="pb-2">
                                    <TabsList className="bg-transparent">
                                        {voiceCategories.map((category) => (
                                            <TabsTrigger key={category.id} value={category.id} className="data-[state=active]:bg-secondary">
                                                {category.name}
                                            </TabsTrigger>
                                        ))}
                                    </TabsList>
                                </ScrollArea>
                            </div>

                            {voiceCategories.map((category) => (
                                <TabsContent key={category.id} value={category.id} className="mt-0">
                                    <div className="p-4">
                                        <h3 className="mb-4 text-lg font-medium">{category.name} Voice Lines</h3>

                                        <div className="space-y-3">
                                            {category.lines.map((line) => (
                                                <div key={line.id} className={`rounded-lg border p-3 transition-colors ${activeLine === line.id ? "border-primary bg-secondary/20" : "hover:bg-accent/50"}`}>
                                                    <div className="flex items-center justify-between">
                                                        <div className="flex flex-1 items-center gap-2">
                                                            <Button variant="ghost" size="icon" className="h-8 w-8 rounded-full" onClick={() => handleLineSelect(line.id)}>
                                                                {isPlaying && activeLine === line.id ? <PauseCircle className="h-6 w-6" /> : <PlayCircle className="h-6 w-6" />}
                                                            </Button>
                                                            <div>
                                                                <div className="font-medium">{line.name}</div>
                                                                <div className="text-xs text-muted-foreground">{line.description}</div>
                                                            </div>
                                                        </div>
                                                        <Button
                                                            variant="ghost"
                                                            size="icon"
                                                            className="h-8 w-8"
                                                            disabled={line.url === "#"}
                                                            onClick={() => {
                                                                if (line.url !== "#") {
                                                                    const a = document.createElement("a");
                                                                    a.href = line.url;
                                                                    a.download = `${operator.name}-${line.name}.mp3`;
                                                                    document.body.appendChild(a);
                                                                    a.click();
                                                                    document.body.removeChild(a);
                                                                }
                                                            }}
                                                        >
                                                            <Download className="h-4 w-4" />
                                                        </Button>
                                                    </div>

                                                    {activeLine === line.id && (
                                                        <div className="mt-3 rounded-md bg-secondary/30 p-3">
                                                            <p className="text-sm italic">{line.transcript}</p>
                                                        </div>
                                                    )}
                                                </div>
                                            ))}
                                        </div>
                                    </div>
                                </TabsContent>
                            ))}
                        </Tabs>
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
                    <div className="container mx-auto flex items-center justify-between">
                        <div className="flex items-center gap-2">
                            <Button variant="ghost" size="icon" className="h-10 w-10" onClick={togglePlayPause}>
                                {isPlaying ? <PauseCircle className="h-6 w-6" /> : <PlayCircle className="h-6 w-6" />}
                            </Button>

                            <div className="flex flex-col">
                                <span className="text-sm font-medium">{currentLine?.name}</span>
                                <span className="text-xs text-muted-foreground">{currentCategory?.name}</span>
                            </div>
                        </div>

                        <div className="flex items-center gap-2">
                            <Button variant="ghost" size="icon" onClick={toggleMute}>
                                {isMuted ? <VolumeX className="h-5 w-5" /> : <Volume2 className="h-5 w-5" />}
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
                                className="h-2 w-24 appearance-none rounded-full bg-secondary"
                            />
                        </div>
                    </div>
                </div>
            )}

            {/* Hidden audio element */}
            <audio ref={audioRef} src={currentLine?.url} style={{ display: "none" }} controls />
        </div>
    );
}

export default AudioContent;
