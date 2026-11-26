"use client";

import { Slider as SliderPrimitive } from "@base-ui/react/slider";
import * as React from "react";
import { cn } from "#/lib/utils";

export function Slider({ className, children, defaultValue, value, min = 0, max = 100, ...props }: SliderPrimitive.Root.Props): React.ReactElement {
    const _values = React.useMemo(() => {
        if (value !== undefined) {
            return Array.isArray(value) ? value : [value];
        }
        if (defaultValue !== undefined) {
            return Array.isArray(defaultValue) ? defaultValue : [defaultValue];
        }
        return [min];
    }, [value, defaultValue, min]);

    return (
        <SliderPrimitive.Root className={cn("data-[orientation=horizontal]:w-full", className)} defaultValue={defaultValue} max={max} min={min} thumbAlignment="edge" value={value} {...props}>
            {children}
            <SliderPrimitive.Control
                className="flex touch-none select-none data-[orientation=vertical]:h-full data-[orientation=vertical]:min-h-44 data-[orientation=horizontal]:w-full data-[orientation=horizontal]:min-w-44 data-disabled:cursor-not-allowed data-[orientation=vertical]:flex-col data-disabled:opacity-64"
                data-slot="slider-control"
            >
                <SliderPrimitive.Track
                    className="relative grow select-none before:absolute before:rounded-full before:bg-input data-[orientation=horizontal]:h-1 data-[orientation=vertical]:h-full data-[orientation=horizontal]:w-full data-[orientation=vertical]:w-1 data-[orientation=horizontal]:before:inset-x-0.5 data-[orientation=vertical]:before:inset-x-0 data-[orientation=horizontal]:before:inset-y-0 data-[orientation=vertical]:before:inset-y-0.5"
                    data-slot="slider-track"
                >
                    <SliderPrimitive.Indicator className="select-none rounded-full bg-primary data-[orientation=horizontal]:ms-0.5 data-[orientation=vertical]:mb-0.5" data-slot="slider-indicator" />
                    {Array.from({ length: _values.length }, (_, index) => (
                        <SliderPrimitive.Thumb
                            className="pointer-coarse:after:-translate-1/2 block pointer-coarse:size-6 size-5 shrink-0 select-none rounded-full border border-input bg-background not-dark:bg-clip-padding shadow-xs/5 outline-none transition-[box-shadow,scale] before:absolute before:inset-0 before:rounded-full before:shadow-[0_1px_--theme(--color-black/4%)] pointer-coarse:after:absolute pointer-coarse:after:top-1/2 pointer-coarse:after:left-1/2 pointer-coarse:after:size-11 has-focus-visible:ring-2 has-focus-visible:ring-ring has-focus-visible:ring-offset-2 has-focus-visible:ring-offset-background data-dragging:scale-120 sm:size-4 dark:border-background dark:bg-foreground [:has(*:focus-visible),[data-dragging]]:shadow-none"
                            data-slot="slider-thumb"
                            index={index}
                            key={String(index)}
                        />
                    ))}
                </SliderPrimitive.Track>
            </SliderPrimitive.Control>
        </SliderPrimitive.Root>
    );
}

export function SliderValue({ className, ...props }: SliderPrimitive.Value.Props): React.ReactElement {
    return <SliderPrimitive.Value className={cn("flex justify-end text-sm", className)} data-slot="slider-value" {...props} />;
}

export { SliderPrimitive };
