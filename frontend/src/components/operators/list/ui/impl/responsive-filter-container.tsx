"use client";

import { SlidersHorizontal, X } from "lucide-react";
import { MorphingDialog, MorphingDialogClose, MorphingDialogContainer, MorphingDialogContent, MorphingDialogTrigger } from "~/components/ui/motion-primitives/morphing-dialog";
import { MorphingPopover, MorphingPopoverContent, MorphingPopoverTrigger } from "~/components/ui/motion-primitives/morphing-popover";
import { useIsMobile } from "~/hooks/use-mobile";
import { cn } from "~/lib/utils";

interface ResponsiveFilterContainerProps {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    hasActiveFilters: boolean;
    activeFilterCount: number;
    children: React.ReactNode;
}

function FilterTriggerButton({ isOpen, hasActiveFilters, activeFilterCount }: { isOpen: boolean; hasActiveFilters: boolean; activeFilterCount: number }) {
    return (
        <button className={cn("flex h-10 items-center gap-2 rounded-lg border px-3 transition-colors", isOpen || hasActiveFilters ? "border-primary bg-primary/10 text-primary" : "border-border bg-secondary/50 text-muted-foreground hover:text-foreground")} type="button">
            <SlidersHorizontal className="h-4 w-4" />
            <span className="font-medium text-sm">Filters</span>
            {hasActiveFilters && <span className="flex h-5 w-5 items-center justify-center rounded-full bg-primary text-primary-foreground text-xs">{activeFilterCount}</span>}
        </button>
    );
}

export function ResponsiveFilterContainer({ open, onOpenChange, hasActiveFilters, activeFilterCount, children }: ResponsiveFilterContainerProps) {
    const isMobile = useIsMobile();

    // Mobile: Use MorphingDialog for full-screen modal experience
    if (isMobile) {
        return (
            <MorphingDialog
                onOpenChange={onOpenChange}
                open={open}
                transition={{
                    type: "spring",
                    bounce: 0.1,
                    duration: 0.4,
                }}
            >
                <MorphingDialogTrigger className="inline-flex">
                    <FilterTriggerButton activeFilterCount={activeFilterCount} hasActiveFilters={hasActiveFilters} isOpen={open} />
                </MorphingDialogTrigger>
                <MorphingDialogContainer>
                    <MorphingDialogContent className="relative max-h-[85vh] w-[calc(100vw-2rem)] overflow-y-auto rounded-xl border border-border bg-card/95 backdrop-blur-sm">
                        <MorphingDialogClose className="absolute top-3 right-3 z-10 flex h-8 w-8 items-center justify-center rounded-lg border border-border bg-secondary/50 text-muted-foreground transition-colors hover:text-foreground">
                            <X className="h-4 w-4" />
                        </MorphingDialogClose>
                        {children}
                    </MorphingDialogContent>
                </MorphingDialogContainer>
            </MorphingDialog>
        );
    }

    // Desktop: Use MorphingPopover for dropdown experience
    return (
        <MorphingPopover onOpenChange={onOpenChange} open={open}>
            <MorphingPopoverTrigger>
                <FilterTriggerButton activeFilterCount={activeFilterCount} hasActiveFilters={hasActiveFilters} isOpen={open} />
            </MorphingPopoverTrigger>
            <MorphingPopoverContent className="w-[calc(100vw-2rem)] max-w-4xl bg-card/95 p-0 drop-shadow-2xl backdrop-blur-sm sm:w-[600px] md:w-[700px] lg:w-[900px]">{children}</MorphingPopoverContent>
        </MorphingPopover>
    );
}
