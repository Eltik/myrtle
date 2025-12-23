"use client";

import { motion } from "motion/react";
import { TransitionPanel } from "~/components/ui/motion-primitives/transition-panel";
import { cn } from "~/lib/utils";
import type { Operator } from "~/types/api";
import { AudioContent } from "../tabs/audio-content";
import { InfoContent } from "../tabs/info-content";
import { LevelUpContent } from "../tabs/levelup-content";
import { LoreContent } from "../tabs/lore-content";
import { SkillsContent } from "../tabs/skills-content";
import { SkinsContent } from "../tabs/skins-content";
import { TABS, type TabType } from "./impl/constants";

interface OperatorTabsProps {
    operator: Operator;
    activeTab: TabType;
    onTabChange: (tab: TabType) => void;
}

export type { TabType };

export function OperatorTabs({ operator, activeTab, onTabChange }: OperatorTabsProps) {
    const activeIndex = TABS.findIndex((t) => t.type === activeTab);

    return (
        <div className="flex min-w-0 flex-col lg:flex-row lg:gap-6">
            {/* Tab Navigation - Sidebar on desktop, horizontal scroll on mobile */}
            <nav className="min-w-0 shrink-0 lg:w-48">
                {/* Mobile: Horizontal scroll */}
                <div className="-mx-3 sm:-mx-4 flex gap-2 overflow-x-auto px-3 pb-4 sm:px-4 lg:hidden">
                    {TABS.map((tab) => {
                        const Icon = tab.icon;
                        const isActive = activeTab === tab.type;
                        return (
                            <button
                                className={cn("flex shrink-0 items-center gap-2 rounded-lg border px-4 py-2 font-medium text-sm transition-all", isActive ? "border-primary bg-primary/10 text-primary" : "border-border bg-card/50 text-muted-foreground hover:border-primary/50 hover:text-foreground")}
                                key={tab.type}
                                onClick={() => onTabChange(tab.type)}
                                type="button"
                            >
                                <Icon className="h-4 w-4" />
                                {tab.label}
                            </button>
                        );
                    })}
                </div>

                {/* Desktop: Vertical sidebar */}
                <div className="sticky top-20 hidden flex-col gap-1 lg:flex">
                    {TABS.map((tab) => {
                        const Icon = tab.icon;
                        const isActive = activeTab === tab.type;
                        return (
                            <motion.button
                                className={cn("relative flex items-center gap-3 rounded-lg px-4 py-3 text-left font-medium text-sm transition-colors", isActive ? "text-foreground" : "text-muted-foreground hover:text-foreground")}
                                key={tab.type}
                                onClick={() => onTabChange(tab.type)}
                                type="button"
                                whileHover={{ x: 2 }}
                                whileTap={{ scale: 0.98 }}
                            >
                                {isActive && <motion.div className="absolute inset-0 rounded-lg border border-primary/50 bg-primary/10" layoutId="activeTab" transition={{ type: "spring", bounce: 0.2, duration: 0.4 }} />}
                                <Icon className="relative z-10 h-4 w-4" />
                                <span className="relative z-10">{tab.label}</span>
                            </motion.button>
                        );
                    })}
                </div>
            </nav>

            {/* Tab Content */}
            <div className="min-w-0 flex-1">
                <div className="rounded-xl border border-border bg-card/80 backdrop-blur-md">
                    <TransitionPanel
                        activeIndex={activeIndex}
                        animateHeight
                        heightTransition={{ type: "spring", bounce: 0.1, duration: 0.4 }}
                        transition={{ duration: 0.2, ease: "easeOut" }}
                        variants={{
                            enter: { opacity: 0, y: 8 },
                            center: { opacity: 1, y: 0 },
                            exit: { opacity: 0, y: -8 },
                        }}
                    >
                        <InfoContent operator={operator} />
                        <SkillsContent operator={operator} />
                        <LevelUpContent operator={operator} />
                        <SkinsContent operator={operator} />
                        <AudioContent operator={operator} />
                        <LoreContent operator={operator} />
                    </TransitionPanel>
                </div>
            </div>
        </div>
    );
}
