"use client";

import * as SliderPrimitive from "@radix-ui/react-slider";
import { hueToPreviewColor } from "~/lib/color-utils";
import { cn } from "~/lib/utils";

interface HueSliderProps {
    value: number;
    onChange: (value: number) => void;
    className?: string;
}

/**
 * A slider component specifically for selecting hue values (0-360).
 * Displays a rainbow gradient background to help users visualize their selection.
 */
export function HueSlider({ value, onChange, className }: HueSliderProps) {
    const previewColor = hueToPreviewColor(value);

    return (
        <div className={cn("flex items-center gap-3", className)}>
            <SliderPrimitive.Root
                className="relative flex h-5 w-full touch-none select-none items-center"
                max={360}
                min={0}
                onValueChange={([newValue]) => {
                    if (newValue !== undefined) {
                        onChange(newValue);
                    }
                }}
                step={1}
                value={[value]}
            >
                <SliderPrimitive.Track
                    className="relative h-3 w-full overflow-hidden rounded-full"
                    style={{
                        background: `linear-gradient(to right,
                            oklch(0.7 0.15 0),
                            oklch(0.7 0.15 30),
                            oklch(0.7 0.15 60),
                            oklch(0.7 0.15 90),
                            oklch(0.7 0.15 120),
                            oklch(0.7 0.15 150),
                            oklch(0.7 0.15 180),
                            oklch(0.7 0.15 210),
                            oklch(0.7 0.15 240),
                            oklch(0.7 0.15 270),
                            oklch(0.7 0.15 300),
                            oklch(0.7 0.15 330),
                            oklch(0.7 0.15 360)
                        )`,
                    }}
                >
                    <SliderPrimitive.Range className="absolute h-full" />
                </SliderPrimitive.Track>
                <SliderPrimitive.Thumb className="block size-5 cursor-grab rounded-full border-2 border-white shadow-md ring-ring/50 transition-shadow hover:ring-4 focus-visible:outline-none focus-visible:ring-4 active:cursor-grabbing" style={{ backgroundColor: previewColor }} />
            </SliderPrimitive.Root>
            <div className="size-6 shrink-0 rounded-md border border-border shadow-sm" style={{ backgroundColor: previewColor }} title={`Hue: ${value}°`} />
        </div>
    );
}
