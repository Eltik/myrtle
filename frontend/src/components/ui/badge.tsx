"use client";

import { mergeProps } from "@base-ui/react/merge-props";
import { useRender } from "@base-ui/react/use-render";
import { cva, type VariantProps } from "class-variance-authority";
import type React from "react";
import { cn } from "#/lib/utils";

export const badgeVariants = cva(
    "relative inline-flex shrink-0 items-center justify-center gap-1 whitespace-nowrap rounded-sm border border-transparent font-medium outline-none transition-shadow focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background disabled:cursor-not-allowed disabled:opacity-64 [&_svg:not([class*='opacity-'])]:opacity-80 [&_svg:not([class*='size-'])]:size-3.5 sm:[&_svg:not([class*='size-'])]:size-3 [&_svg]:pointer-events-none [&_svg]:shrink-0 [button&,a&]:cursor-pointer [button&,a&]:pointer-coarse:after:absolute [button&,a&]:pointer-coarse:after:size-full [button&,a&]:pointer-coarse:after:min-h-11 [button&,a&]:pointer-coarse:after:min-w-11",
    {
        defaultVariants: {
            size: "default",
            variant: "default",
        },
        variants: {
            size: {
                default: "h-5.5 min-w-5.5 px-[calc(--spacing(1)-1px)] text-sm sm:h-4.5 sm:min-w-4.5 sm:text-xs",
                lg: "h-6.5 min-w-6.5 px-[calc(--spacing(1.5)-1px)] text-base sm:h-5.5 sm:min-w-5.5 sm:text-sm",
                sm: "h-5 min-w-5 rounded-sm px-[calc(--spacing(1)-1px)] text-xs sm:h-4 sm:min-w-4 sm:text-[.625rem]",
            },
            variant: {
                default: "bg-primary text-primary-foreground [button&,a&]:hover:bg-primary-hover",
                destructive: "bg-destructive text-white [button&,a&]:hover:bg-destructive/90",
                error: "bg-destructive/8 text-destructive-foreground dark:bg-destructive/16",
                info: "bg-info/8 text-info-foreground dark:bg-info/16",
                outline: "border-input bg-background text-foreground dark:bg-input/32 [button&,a&]:hover:bg-accent/50 dark:[button&,a&]:hover:bg-input/48",
                secondary: "bg-secondary text-secondary-foreground [button&,a&]:hover:bg-secondary/90",
                // A data tone (rarity, enemy level, stage group) as a badge: the tone tints the
                // fill and draws the border, the text stays foreground. Tone text on its own
                // tint measured 1.66 to 3.96 across audit-002; foreground on a 12% tint reads.
                // Set the hue with `style={{ "--tone": "var(--enemy-elite)" }}`; primary by default.
                tint: "border-(--tone)/45 bg-(--tone)/12 text-foreground [--tone:var(--primary)]",
                success: "bg-success/8 text-success-foreground dark:bg-success/16",
                warning: "bg-warning/8 text-warning-foreground dark:bg-warning/16",
            },
        },
    },
);

export interface IBadgeProps extends useRender.ComponentProps<"span"> {
    variant?: VariantProps<typeof badgeVariants>["variant"];
    size?: VariantProps<typeof badgeVariants>["size"];
}

export function Badge({ className, variant, size, render, ...props }: IBadgeProps): React.ReactElement {
    const defaultProps = {
        className: cn(badgeVariants({ className, size, variant })),
        "data-slot": "badge",
    };

    return useRender({
        defaultTagName: "span",
        props: mergeProps<"span">(defaultProps, props),
        render,
    });
}
