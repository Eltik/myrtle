"use client";

import type React from "react";

import { motion } from "motion/react";
import { Info, Sparkles, TrendingUp, Palette, Volume2 } from "lucide-react";
import { Button } from "~/components/ui/shadcn/button";
import { ScrollArea, ScrollBar } from "~/components/ui/shadcn/scroll-area";
import { cn } from "~/lib/utils";

export type TabType = "info" | "skills" | "levelup" | "skins" | "audio";

interface Tab {
    type: TabType;
    label: string;
    icon: React.ComponentType<{ className?: string }>;
}

const tabs: Tab[] = [
    { type: "info", label: "Information", icon: Info },
    { type: "skills", label: "Skills & Talents", icon: Sparkles },
    { type: "levelup", label: "Level-Up Cost", icon: TrendingUp },
    { type: "skins", label: "Skins", icon: Palette },
    { type: "audio", label: "Voice Lines", icon: Volume2 },
];

interface OperatorNavigationProps {
    activeTab: TabType;
    onTabChange: (tab: TabType) => void;
}

export function OperatorNavigation({ activeTab, onTabChange }: OperatorNavigationProps) {
    return (
        <>
            {/* Mobile horizontal tabs */}
            <div className="w-full lg:hidden">
                <ScrollArea className="w-full whitespace-nowrap">
                    <div className="flex gap-2 pb-3">
                        {tabs.map((tab) => {
                            const Icon = tab.icon;
                            const isActive = activeTab === tab.type;

                            return (
                                <Button key={tab.type} variant={isActive ? "default" : "outline"} size="sm" onClick={() => onTabChange(tab.type)} className={cn("relative flex-shrink-0 gap-2 transition-all", isActive && "shadow-md")}>
                                    <Icon className="h-4 w-4" />
                                    <span>{tab.label}</span>
                                    {isActive && <motion.div layoutId="activeTabMobile" className="absolute inset-0 -z-10 rounded-md bg-primary" transition={{ type: "spring", bounce: 0.2, duration: 0.6 }} />}
                                </Button>
                            );
                        })}
                    </div>
                    <ScrollBar orientation="horizontal" className="h-0" />
                </ScrollArea>
            </div>

            {/* Desktop vertical sidebar */}
            <div className="hidden w-52 flex-shrink-0 lg:block">
                <div className="sticky top-24 space-y-1">
                    {tabs.map((tab) => {
                        const Icon = tab.icon;
                        const isActive = activeTab === tab.type;

                        return (
                            <button key={tab.type} onClick={() => onTabChange(tab.type)} className={cn("relative flex w-full items-center gap-3 rounded-lg px-4 py-3 text-left text-sm font-medium transition-all", isActive ? "text-foreground" : "text-muted-foreground hover:bg-muted hover:text-foreground")}>
                                {isActive && <motion.div layoutId="activeTabDesktop" className="absolute inset-0 rounded-lg bg-muted" transition={{ type: "spring", bounce: 0.2, duration: 0.6 }} />}
                                <Icon className="relative z-10 h-4 w-4" />
                                <span className="relative z-10">{tab.label}</span>
                            </button>
                        );
                    })}
                </div>
            </div>
        </>
    );
}
