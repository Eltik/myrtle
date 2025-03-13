import { useState, useRef, useEffect } from "react";
import { Card, CardContent } from "~/components/ui/card";
import { ScrollArea } from "~/components/ui/scroll-area";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "~/components/ui/tabs";
import { Button } from "~/components/ui/button";
import { Separator } from "~/components/ui/separator";
import { Badge } from "~/components/ui/badge";
import { PlayCircle, PauseCircle, Download, Volume2, VolumeX } from "lucide-react";
import type { Operator } from "~/types/impl/api/static/operator";

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

function AudioContent({ operator }: { operator: Operator }) {
    const [activeCategory, setActiveCategory] = useState<string>("greeting");
    const [activeLine, setActiveLine] = useState<string | null>(null);
    const [isPlaying, setIsPlaying] = useState<boolean>(false);
    const [isMuted, setIsMuted] = useState<boolean>(false);
    const [volume, setVolume] = useState<number>(0.7);
    const audioRef = useRef<HTMLAudioElement | null>(null);

    // Example voice categories and lines
    // In a real implementation, these would come from an API or data file
    const voiceCategories: VoiceCategory[] = [
        {
            id: "greeting",
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
                {
                    id: "greeting-3",
                    name: "Login",
                    description: "Daily login greeting",
                    transcript: `Welcome back, Doctor. Ready to get to work?`,
                    url: "#",
                },
            ],
        },
        {
            id: "battle",
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
                {
                    id: "battle-3",
                    name: "Deployment",
                    description: "Upon deployment",
                    transcript: `Reporting for duty!`,
                    url: "#",
                },
                {
                    id: "battle-4",
                    name: "4x Speed",
                    description: "When 4x speed is toggled",
                    transcript: `Double time!`,
                    url: "#",
                },
            ],
        },
        {
            id: "interaction",
            name: "Interaction",
            lines: [
                {
                    id: "interaction-1",
                    name: "Talk 1",
                    description: "General conversation",
                    transcript: `Rhodes Island is quite the interesting place, wouldn't you agree, Doctor?`,
                    url: "#",
                },
                {
                    id: "interaction-2",
                    name: "Talk 2",
                    description: "General conversation",
                    transcript: `You should get some rest, Doctor. You're working too hard.`,
                    url: "#",
                },
                {
                    id: "interaction-3",
                    name: "Trust Increase",
                    description: "When trust increases",
                    transcript: `I appreciate our time working together, Doctor. I believe we've grown closer.`,
                    url: "#",
                },
            ],
        },
        {
            id: "facility",
            name: "Base",
            lines: [
                {
                    id: "facility-1",
                    name: "Assigned to Factory",
                    description: "When assigned to factory",
                    transcript: `Factory duty? Leave it to me.`,
                    url: "#",
                },
                {
                    id: "facility-2",
                    name: "Assigned to Trading Post",
                    description: "When assigned to trading post",
                    transcript: `I'll negotiate the best deals for Rhodes Island.`,
                    url: "#",
                },
                {
                    id: "facility-3",
                    name: "Assigned to Power Plant",
                    description: "When assigned to power plant",
                    transcript: `I'll make sure everything runs smoothly.`,
                    url: "#",
                },
            ],
        },
    ];

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

    // Voice actor info (in a real implementation this would come from the operator data)
    const voiceActor = {
        name: "Voice Actor", // This would be the actual name
        language: operator.name ? (operator.name.includes("Nian") ? "Chinese" : "Japanese") : "Japanese", // Example logic
    };

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
                                                    <Button variant="ghost" size="icon" className="h-8 w-8">
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
