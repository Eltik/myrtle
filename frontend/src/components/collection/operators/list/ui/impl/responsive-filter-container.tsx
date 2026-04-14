"use client";

import { SlidersHorizontal, X } from "lucide-react";
import React from "react";
import { MorphingDialog, MorphingDialogContainer, MorphingDialogContent, MorphingDialogTrigger } from "~/components/ui/motion-primitives/morphing-dialog";
import { MorphingPopover, MorphingPopoverContent, MorphingPopoverTrigger } from "~/components/ui/motion-primitives/morphing-popover";
import { useIsMobile } from "~/hooks/use-mobile";
import { cn } from "~/lib/utils";

interface ResponsiveFilterContainerProps {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    hasActiveFilters: boolean;
    activeFilterCount: number;
    onClearFilters: () => void;
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

export function ResponsiveFilterContainer({ open, onOpenChange, hasActiveFilters, activeFilterCount, onClearFilters, children }: ResponsiveFilterContainerProps) {
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
                    <MorphingDialogContent className="relative max-h-[85vh] w-[calc(100vw-2rem)] overflow-hidden rounded-xl border border-border bg-card/90 backdrop-blur-md">
                        {/* Mobile Header */}
                        <div className="flex items-center justify-between border-border border-b bg-card/95 px-4 py-3">
                            <h3 className="font-semibold text-foreground">Filters</h3>
                            <div className="flex items-center gap-2">
                                {hasActiveFilters && (
                                    <button className="flex items-center gap-1 text-muted-foreground text-sm transition-colors hover:text-foreground" onClick={onClearFilters} type="button">
                                        <X className="h-3 w-3" />
                                        Clear all
                                    </button>
                                )}
                                <button className="flex h-8 w-8 items-center justify-center rounded-lg border border-border bg-secondary/50 text-muted-foreground transition-colors hover:text-foreground" onClick={() => onOpenChange(false)} type="button">
                                    <X className="h-4 w-4" />
                                </button>
                            </div>
                        </div>
                        {/* Scrollable content */}
                        <div className="max-h-[calc(85vh-52px)] overflow-y-auto">{React.isValidElement(children) ? React.cloneElement(children as React.ReactElement<{ hideHeader?: boolean }>, { hideHeader: true }) : children}</div>
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
            <MorphingPopoverContent className="w-[calc(100vw-2rem)] max-w-4xl bg-card/95 p-0 drop-shadow-2xl backdrop-blur-md sm:w-150 md:w-150 lg:w-225">{children}</MorphingPopoverContent>
        </MorphingPopover>
    );
}
