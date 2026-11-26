"use client";

import { Toggle as TogglePrimitive } from "@base-ui/react/toggle";
import { cva, type VariantProps } from "class-variance-authority";
import type React from "react";
import { cn } from "#/lib/utils";

/*
 * Pressed is an accent tint: a `--primary` fill at 12% (16% in dark, a step
 * more under the pointer) inside a solid `--ring` inset ring, with foreground
 * text. Hover is the neutral `bg-accent`, and the two fills sit within 1.1:1
 * of each other in luminance, so pressed is told from hover by the red hue and
 * the ring, not by lightness. Computed from the tokens (oklab mix, not measured
 * in a browser): the ring is 3.9:1 against its own fill in light and 7.2:1 in
 * dark, and foreground text on the fill stays above 15:1. Text stays
 * foreground because `--primary-ink` on the light fill computes to 3.9:1,
 * under AA for body text.
 */
export const toggleVariants = cva(
    "relative inline-flex shrink-0 cursor-pointer select-none items-center justify-center gap-2 whitespace-nowrap rounded-lg border font-medium text-base text-foreground outline-none transition-shadow before:pointer-events-none before:absolute before:inset-0 before:rounded-[calc(var(--radius-lg)-1px)] pointer-coarse:after:absolute pointer-coarse:after:size-full pointer-coarse:after:min-h-11 pointer-coarse:after:min-w-11 hover:bg-accent focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background disabled:cursor-not-allowed disabled:opacity-64 data-pressed:inset-ring data-pressed:inset-ring-ring data-pressed:bg-primary/12 data-pressed:text-foreground data-pressed:hover:bg-primary/16 sm:text-sm dark:data-pressed:bg-primary/16 dark:data-pressed:hover:bg-primary/22 [&_svg:not([class*='opacity-'])]:opacity-80 [&_svg:not([class*='size-'])]:size-4.5 sm:[&_svg:not([class*='size-'])]:size-4 [&_svg]:pointer-events-none [&_svg]:-mx-0.5 [&_svg]:shrink-0",
    {
        defaultVariants: {
            size: "default",
            variant: "default",
        },
        variants: {
            size: {
                default: "h-9 min-w-9 px-[calc(--spacing(2)-1px)] sm:h-8 sm:min-w-8",
                lg: "h-10 min-w-10 px-[calc(--spacing(2.5)-1px)] sm:h-9 sm:min-w-9",
                sm: "h-8 min-w-8 px-[calc(--spacing(1.5)-1px)] sm:h-7 sm:min-w-7",
            },
            variant: {
                default: "border-transparent",
                outline:
                    "border-input bg-background not-dark:bg-clip-padding shadow-xs/5 not-disabled:not-active:not-data-pressed:before:shadow-[0_1px_--theme(--color-black/4%)] dark:bg-input/32 dark:hover:bg-input/64 dark:not-disabled:not-active:not-data-pressed:before:shadow-[0_-1px_--theme(--color-white/6%)] dark:not-disabled:not-data-pressed:before:shadow-[0_-1px_--theme(--color-white/2%)] [:disabled,:active,[data-pressed]]:shadow-none",
            },
        },
    },
);

export function Toggle({ className, variant, size, ...props }: TogglePrimitive.Props & VariantProps<typeof toggleVariants>): React.ReactElement {
    return <TogglePrimitive className={cn(toggleVariants({ className, size, variant }))} data-slot="toggle" {...props} />;
}

export { TogglePrimitive };
