"use client";

import { AnimatedBackground } from "~/components/ui/motion-primitives/animated-background";
import { ScrollArea, ScrollBar } from "~/components/ui/shadcn/scroll-area";
import type { TabType } from "./operator-detail";

interface OperatorNavProps {
    tabs: { type: TabType; label: string }[];
    activeTab: TabType;
    onTabChange: (tab: TabType) => void;
}

export function OperatorNav({ tabs, activeTab, onTabChange }: OperatorNavProps) {
    const handleValueChange = (newId: string | null) => {
        if (newId) {
            onTabChange(newId as TabType);
        }
    };

    return (
        <>
            {/* Mobile: Horizontal scrollable tabs */}
            <div className="w-full md:hidden">
                <ScrollArea className="w-full whitespace-nowrap">
                    <div className="flex gap-1 p-1 rounded-lg bg-muted/50 backdrop-blur-sm">
                        <AnimatedBackground defaultValue={activeTab} onValueChange={handleValueChange} className="rounded-md bg-primary" transition={{ type: "spring", bounce: 0.2, duration: 0.4 }}>
                            {tabs.map((tab) => (
                                <button key={tab.type} data-id={tab.type} type="button" className="relative z-10 px-4 py-2 text-sm font-medium transition-colors data-[checked=true]:text-primary-foreground">
                                    {tab.label}
                                </button>
                            ))}
                        </AnimatedBackground>
                    </div>
                    <ScrollBar orientation="horizontal" className="h-0" />
                </ScrollArea>
            </div>

            {/* Desktop: Vertical sidebar navigation */}
            <div className="hidden w-52 flex-shrink-0 md:block">
                <div className="sticky top-4 rounded-lg border border-border bg-card/80 p-2 backdrop-blur-md">
                    <AnimatedBackground defaultValue={activeTab} onValueChange={handleValueChange} className="rounded-md bg-primary" transition={{ type: "spring", bounce: 0.2, duration: 0.4 }}>
                        {tabs.map((tab) => (
                            <button key={tab.type} data-id={tab.type} type="button" className="relative z-10 flex w-full items-center justify-start rounded-md px-4 py-3 text-sm font-medium transition-colors data-[checked=true]:text-primary-foreground">
                                {tab.label}
                            </button>
                        ))}
                    </AnimatedBackground>
                </div>
            </div>
        </>
    );
}
