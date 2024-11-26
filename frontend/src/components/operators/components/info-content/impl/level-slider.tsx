"use client";

import { useState, useEffect } from "react";
import { Input } from "~/components/ui/input";
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
            <Slider min={1} max={maxLevel} step={1} value={[level]} onValueChange={handleSliderChange} />
            <div className="mt-2 text-sm flex flex-row gap-3 items-center">
                <Input type="number" min={1} max={maxLevel} value={level} onChange={handleInputChange} className="w-16" />
                <span className="text-muted-foreground"> / {maxLevel}</span>
            </div>
        </div>
    );
}
