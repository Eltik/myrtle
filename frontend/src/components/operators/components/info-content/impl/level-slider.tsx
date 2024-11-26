"use client";

import { useState, useEffect } from "react";
import { Slider } from "~/components/ui/slider";

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

    return (
        <div className="w-full max-w-md">
            <Slider min={1} max={maxLevel} step={1} value={[level]} onValueChange={handleSliderChange} />
            <div className="mt-2 text-sm text-muted-foreground">
                Level: {level} / {maxLevel}
            </div>
        </div>
    );
}
