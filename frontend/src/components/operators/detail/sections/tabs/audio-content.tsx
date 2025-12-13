"use client";

import { Pause, Play, Volume2, VolumeX } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { Button } from "~/components/ui/shadcn/button";
import { ScrollArea } from "~/components/ui/shadcn/scroll-area";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "~/components/ui/shadcn/select";
import { Skeleton } from "~/components/ui/shadcn/skeleton";
import { Slider } from "~/components/ui/shadcn/slider";
import { cn } from "~/lib/utils";
import type { Operator } from "~/types/api";
import type { Voice, Voices } from "~/types/api/impl/voice";

interface AudioContentProps {
    operator: Operator;
}

interface VoiceLine {
    id: string;
    title: string;
    text: string;
    audioUrl?: string;
}

const VOICE_LANGUAGES = [
    { id: "JP", label: "Japanese" },
    { id: "CN", label: "Chinese" },
    { id: "EN", label: "English" },
    { id: "KR", label: "Korean" },
];

export function AudioContent({ operator }: AudioContentProps) {
    const [voices, setVoices] = useState<VoiceLine[]>([]);
    const [isLoading, setIsLoading] = useState(true);
    const [selectedLanguage, setSelectedLanguage] = useState("JP");
    const [playingId, setPlayingId] = useState<string | null>(null);
    const [volume, setVolume] = useState(80);
    const [isMuted, setIsMuted] = useState(false);

    const audioRef = useRef<HTMLAudioElement | null>(null);

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
                    const formattedVoices = formatVoices(data.voices, operator.id ?? "");
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
            setPlayingId(null);
            return;
        }

        // Play new
        if (audioRef.current) {
            audioRef.current.pause();
        }

        // Construct audio URL based on language using local CDN
        // Voice files are stored in the assets folder organized by language and operator ID
        const audioUrl = `/api/cdn/upk/audio/voice_${selectedLanguage.toLowerCase()}/${operator.id}/${voice.id}.wav`;

        const audio = new Audio(audioUrl);
        audio.volume = isMuted ? 0 : volume / 100;
        audio.onended = () => setPlayingId(null);
        audio.onerror = () => {
            console.error("Failed to load audio");
            setPlayingId(null);
        };

        audioRef.current = audio;
        audio.play().catch(() => setPlayingId(null));
        setPlayingId(voice.id);
    };

    // Update volume when changed
    useEffect(() => {
        if (audioRef.current) {
            audioRef.current.volume = isMuted ? 0 : volume / 100;
        }
    }, [volume, isMuted]);

    // Cleanup audio on unmount
    useEffect(() => {
        return () => {
            audioRef.current?.pause();
        };
    }, []);

    return (
        <div className="p-4 md:p-6">
            {/* Header */}
            <div className="mb-6">
                <h2 className="font-semibold text-foreground text-xl">Voice Lines</h2>
                <p className="text-muted-foreground text-sm">Listen to operator voice lines and audio</p>
            </div>

            {/* Controls */}
            <div className="mb-6 flex flex-wrap items-center gap-4">
                <Select onValueChange={setSelectedLanguage} value={selectedLanguage}>
                    <SelectTrigger className="w-40">
                        <SelectValue placeholder="Language" />
                    </SelectTrigger>
                    <SelectContent>
                        {VOICE_LANGUAGES.map((lang) => (
                            <SelectItem key={lang.id} value={lang.id}>
                                {lang.label}
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
                            <div className={cn("group flex items-start gap-3 rounded-lg border border-border p-3 transition-colors", playingId === voice.id ? "border-primary bg-primary/10" : "bg-card/30 hover:bg-secondary/30")} key={voice.id}>
                                <Button className="shrink-0" onClick={() => playVoice(voice)} size="icon" variant={playingId === voice.id ? "default" : "secondary"}>
                                    {playingId === voice.id ? <Pause className="h-4 w-4" /> : <Play className="h-4 w-4" />}
                                </Button>
                                <div className="min-w-0 flex-1">
                                    <h4 className="mb-1 font-medium text-foreground text-sm">{voice.title}</h4>
                                    <p className="line-clamp-2 text-muted-foreground text-xs">{voice.text}</p>
                                </div>
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

function formatVoices(voiceData: Voices | Voice[], _operatorId: string): VoiceLine[] {
    const voiceLines: VoiceLine[] = [];

    // Common voice line categories
    const defaultVoiceLines = [
        { id: "CN_001", title: "Appointment" },
        { id: "CN_002", title: "Talk 1" },
        { id: "CN_003", title: "Talk 2" },
        { id: "CN_004", title: "Talk 3" },
        { id: "CN_005", title: "Talk after Promotion 1" },
        { id: "CN_006", title: "Talk after Promotion 2" },
        { id: "CN_007", title: "Talk after Trust Increase 1" },
        { id: "CN_008", title: "Talk after Trust Increase 2" },
        { id: "CN_009", title: "Talk after Trust Increase 3" },
        { id: "CN_010", title: "Idle" },
        { id: "CN_011", title: "Onboard" },
        { id: "CN_012", title: "Watch Battlerecord" },
        { id: "CN_013", title: "Promotion 1" },
        { id: "CN_014", title: "Promotion 2" },
        { id: "CN_017", title: "Added to Squad" },
        { id: "CN_018", title: "Appointed as Squad Leader" },
        { id: "CN_019", title: "Depart" },
        { id: "CN_020", title: "Begin Operation" },
        { id: "CN_021", title: "Selecting Operator 1" },
        { id: "CN_022", title: "Selecting Operator 2" },
        { id: "CN_023", title: "Deployment 1" },
        { id: "CN_024", title: "Deployment 2" },
        { id: "CN_025", title: "In Battle 1" },
        { id: "CN_026", title: "In Battle 2" },
        { id: "CN_027", title: "In Battle 3" },
        { id: "CN_028", title: "In Battle 4" },
        { id: "CN_029", title: "4-star Result" },
        { id: "CN_030", title: "3-star Result" },
        { id: "CN_031", title: "Operation Failure" },
        { id: "CN_032", title: "Assigned to Facility" },
        { id: "CN_033", title: "Tap" },
        { id: "CN_034", title: "Trust Tap" },
        { id: "CN_036", title: "Greeting" },
    ];

    // If we have actual voice data, use it
    if (Array.isArray(voiceData)) {
        voiceData.forEach((voice) => {
            voiceLines.push({
                id: voice.id ?? "",
                title: voice.voiceTitle ?? "Voice Line",
                text: voice.voiceText ?? "",
            });
        });
    } else if (voiceData && typeof voiceData === "object") {
        // Handle object format
        Object.entries(voiceData).forEach(([key, value]) => {
            if (value && typeof value === "object") {
                voiceLines.push({
                    id: key,
                    title: (value as Voice).voiceTitle ?? key,
                    text: (value as Voice).voiceText ?? "",
                });
            }
        });
    }

    // If no voice data, return default structure
    if (voiceLines.length === 0) {
        return defaultVoiceLines.map((v) => ({
            id: v.id,
            title: v.title,
            text: "Voice line text not available",
        }));
    }

    return voiceLines;
}
