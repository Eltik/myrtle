"use client";

import * as TooltipPrimitive from "@radix-ui/react-tooltip";
import type * as React from "react";

import { cn } from "~/lib/utils";

function TooltipProvider({ delayDuration = 0, ...props }: React.ComponentProps<typeof TooltipPrimitive.Provider>) {
    return <TooltipPrimitive.Provider data-slot="tooltip-provider" delayDuration={delayDuration} {...props} />;
}

function Tooltip({ ...props }: React.ComponentProps<typeof TooltipPrimitive.Root>) {
    return (
        <TooltipProvider>
            <TooltipPrimitive.Root data-slot="tooltip" {...props} />
        </TooltipProvider>
    );
}

function TooltipTrigger({ ...props }: React.ComponentProps<typeof TooltipPrimitive.Trigger>) {
    return <TooltipPrimitive.Trigger data-slot="tooltip-trigger" {...props} />;
}

type TooltipVariant = "light" | "dark";

interface TooltipContentProps extends React.ComponentProps<typeof TooltipPrimitive.Content> {
    variant?: TooltipVariant;
    hideArrow?: boolean;
}

const variantStyles: Record<TooltipVariant, { content: string; arrow: string }> = {
    light: {
        content: "bg-foreground text-background",
        arrow: "bg-foreground fill-foreground",
    },
    dark: {
        content: "bg-popover text-popover-foreground border border-border",
        arrow: "bg-popover fill-popover",
    },
};

function TooltipContent({ className, sideOffset = 0, children, variant = "light", hideArrow = false, ...props }: TooltipContentProps) {
    const styles = variantStyles[variant];

    return (
        <TooltipPrimitive.Portal>
            <TooltipPrimitive.Content
                className={cn(
                    "fade-in-0 zoom-in-95 data-[state=closed]:fade-out-0 data-[state=closed]:zoom-out-95 data-[side=bottom]:slide-in-from-top-2 data-[side=left]:slide-in-from-right-2 data-[side=right]:slide-in-from-left-2 data-[side=top]:slide-in-from-bottom-2 z-50 w-fit origin-(--radix-tooltip-content-transform-origin) animate-in text-balance rounded-md px-3 py-1.5 text-xs data-[state=closed]:animate-out",
                    styles.content,
                    className,
                )}
                data-slot="tooltip-content"
                sideOffset={sideOffset}
                {...props}
            >
                {children}
                {!hideArrow && <TooltipPrimitive.Arrow className={cn("z-50 size-2.5 translate-y-[calc(-50%-2px)] rotate-45 rounded-[2px]", styles.arrow)} />}
            </TooltipPrimitive.Content>
        </TooltipPrimitive.Portal>
    );
}

export { Tooltip, TooltipTrigger, TooltipContent, TooltipProvider };
export type { TooltipVariant };
