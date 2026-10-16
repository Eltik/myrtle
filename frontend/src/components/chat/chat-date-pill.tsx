"use client";

import type { ComponentProps, ReactElement } from "react";
import { cn } from "#/lib/utils";

export function ChatDatePill({ className, ...props }: ComponentProps<"div">): ReactElement {
    return <div data-slot="chat-date-pill" className={cn("pointer-events-none sticky top-4 z-20 mx-auto flex h-6 w-fit items-center justify-center rounded-full border bg-popover/90 px-2 font-medium text-foreground text-xs backdrop-blur-xs", className)} {...props} />;
}
