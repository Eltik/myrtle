"use client";

import { useState, useEffect } from "react";
import { Input } from "~/components/ui/input";
import { Slider } from "~/components/ui/slider";
import { Info } from "lucide-react";
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
    }, [phaseIndex]);

    const handleSliderChange = (value: number[]) => {
        const newLevel = value[0];
        setLevel(newLevel ?? 1);
        onLevelChange(newLevel ?? 1);
    };

    const handleInputChange = (event: React.ChangeEvent<HTMLInputElement>) => {
        const newLevel = parseInt(event.target.value, 10);
        if (!isNaN(newLevel)) {
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
                    <span className="text-sm font-medium">Level</span>
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
                <div className="text-xs text-muted-foreground">
                    {level} / {maxLevel}
                </div>
            </div>

            <Slider min={1} max={maxLevel} step={1} value={[level]} onValueChange={handleSliderChange} className="mb-2" />

            <div className="flex flex-row items-center gap-3 text-sm">
                <div className="flex items-center gap-2">
                    <Input type="number" min={1} max={maxLevel} value={level} onChange={handleInputChange} className="w-16" />
                    <span className="text-muted-foreground">/ {maxLevel}</span>
                </div>

                <div className="ml-auto flex gap-2">
                    <button onClick={() => updateLevel(1)} className="rounded-md bg-muted px-2 py-1 text-xs hover:bg-muted/80">
                        Min
                    </button>
                    <button onClick={() => updateLevel(maxLevel)} className="rounded-md bg-muted px-2 py-1 text-xs hover:bg-muted/80">
                        Max
                    </button>
                </div>
            </div>
        </div>
    );
}
