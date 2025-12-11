"use client";

import { Info } from "lucide-react";
import { useEffect, useState } from "react";
import { Input } from "~/components/ui/input";
import { Slider } from "~/components/ui/slider";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "~/components/ui/tooltip";

interface LevelSliderProps {
    phaseIndex: number;
    maxLevels: number[];
    onLevelChange: (level: number) => void;
}

export function LevelSlider({ phaseIndex, maxLevels, onLevelChange }: LevelSliderProps) {
    const [level, setLevel] = useState(1);
    const maxLevel = maxLevels[phaseIndex] ?? 1;

    useEffect(() => {
        setLevel(maxLevel);
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [maxLevel]);

    const handleSliderChange = (value: number[]) => {
        const newLevel = value[0];
        setLevel(newLevel ?? 1);
        onLevelChange(newLevel ?? 1);
    };

    const handleInputChange = (event: React.ChangeEvent<HTMLInputElement>) => {
        const newLevel = parseInt(event.target.value, 10);
        if (!Number.isNaN(newLevel)) {
            updateLevel(newLevel);
        }
    };

    const updateLevel = (newLevel: number) => {
        const clampedLevel = Math.max(1, Math.min(newLevel, maxLevel));
        setLevel(clampedLevel);
        onLevelChange(clampedLevel);
    };

    return (
        <div className="w-full max-w-md">
            <div className="mb-2 flex items-center justify-between">
                <div className="flex items-center gap-1">
                    <span className="font-medium text-sm">Level</span>
                    <TooltipProvider>
                        <Tooltip>
                            <TooltipTrigger asChild>
                                <Info className="h-3 w-3 cursor-help text-muted-foreground" />
                            </TooltipTrigger>
                            <TooltipContent>
                                <p>Adjust the operator&apos;s level to see how their stats change</p>
                            </TooltipContent>
                        </Tooltip>
                    </TooltipProvider>
                </div>
                <div className="text-muted-foreground text-xs">
                    {level} / {maxLevel}
                </div>
            </div>

            <Slider className="mb-2" max={maxLevel} min={1} onValueChange={handleSliderChange} step={1} value={[level]} />

            <div className="flex flex-row items-center gap-3 text-sm">
                <div className="flex items-center gap-2">
                    <Input className="w-16" max={maxLevel} min={1} onChange={handleInputChange} type="number" value={level} />
                    <span className="text-muted-foreground">/ {maxLevel}</span>
                </div>

                <div className="ml-auto flex gap-2">
                    <button className="rounded-md bg-muted px-2 py-1 text-xs hover:bg-muted/80" onClick={() => updateLevel(1)}>
                        Min
                    </button>
                    <button className="rounded-md bg-muted px-2 py-1 text-xs hover:bg-muted/80" onClick={() => updateLevel(maxLevel)}>
                        Max
                    </button>
                </div>
            </div>
        </div>
    );
}
