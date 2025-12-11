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
                    <div className="flex gap-1 rounded-lg bg-muted/50 p-1 backdrop-blur-sm">
                        <AnimatedBackground className="rounded-md bg-primary" defaultValue={activeTab} onValueChange={handleValueChange} transition={{ type: "spring", bounce: 0.2, duration: 0.4 }}>
                            {tabs.map((tab) => (
                                <button className="relative z-10 px-4 py-2 font-medium text-sm transition-colors data-[checked=true]:text-primary-foreground" data-id={tab.type} key={tab.type} type="button">
                                    {tab.label}
                                </button>
                            ))}
                        </AnimatedBackground>
                    </div>
                    <ScrollBar className="h-0" orientation="horizontal" />
                </ScrollArea>
            </div>

            {/* Desktop: Vertical sidebar navigation */}
            <div className="hidden w-52 flex-shrink-0 md:block">
                <div className="sticky top-4 rounded-lg border border-border bg-card/80 p-2 backdrop-blur-md">
                    <AnimatedBackground className="rounded-md bg-primary" defaultValue={activeTab} onValueChange={handleValueChange} transition={{ type: "spring", bounce: 0.2, duration: 0.4 }}>
                        {tabs.map((tab) => (
                            <button className="relative z-10 flex w-full items-center justify-start rounded-md px-4 py-3 font-medium text-sm transition-colors data-[checked=true]:text-primary-foreground" data-id={tab.type} key={tab.type} type="button">
                                {tab.label}
                            </button>
                        ))}
                    </AnimatedBackground>
                </div>
            </div>
        </>
    );
}
