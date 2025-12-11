"use client";

import { useState } from "react";
import { motion } from "motion/react";
import type { Operator } from "~/types/api";
import { OperatorHero } from "./operator-hero";
import { OperatorNav } from "./operator-nav";
import { TransitionPanel } from "~/components/ui/motion-primitives/transition-panel";
import { InfoTab } from "./tabs/info-tab";
import { SkillsTab } from "./tabs/skills-tab";
import { LevelCostTab } from "./tabs/level-cost-tab";
import { SkinsTab } from "./tabs/skins-tab";
import { AudioTab } from "./tabs/audio-tab";
import { ScrollArea } from "~/components/ui/shadcn/scroll-area";

export type TabType = "info" | "skills" | "levelup" | "skins" | "audio";

interface OperatorDetailProps {
    operator: Operator;
}

const tabVariants = {
    enter: { opacity: 0, x: 20 },
    center: { opacity: 1, x: 0 },
    exit: { opacity: 0, x: -20 },
};

export function OperatorDetail({ operator }: OperatorDetailProps) {
    const [activeTab, setActiveTab] = useState<TabType>("info");

    const tabs: { type: TabType; label: string }[] = [
        { type: "info", label: "Information" },
        { type: "skills", label: "Skills & Talents" },
        { type: "levelup", label: "Level-Up Cost" },
        { type: "skins", label: "Skins" },
        { type: "audio", label: "Audio/SFX" },
    ];

    const tabIndex = tabs.findIndex((t) => t.type === activeTab);

    const tabContent = [<InfoTab key="info" operator={operator} />, <SkillsTab key="skills" operator={operator} />, <LevelCostTab key="levelup" operator={operator} />, <SkinsTab key="skins" operator={operator} />, <AudioTab key="audio" operator={operator} />];

    return (
        <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} transition={{ duration: 0.5 }} className="min-h-screen w-full">
            {/* Hero Section */}
            <OperatorHero operator={operator} />

            {/* Main Content */}
            <div className="container relative z-20 mx-auto -mt-32 px-4 pb-16">
                <div className="flex w-full flex-col gap-4 md:flex-row md:gap-6">
                    {/* Navigation */}
                    <OperatorNav tabs={tabs} activeTab={activeTab} onTabChange={setActiveTab} />

                    {/* Content Area */}
                    <div className="flex-1 overflow-hidden rounded-lg border border-border bg-card/80 backdrop-blur-md">
                        <ScrollArea className="h-full max-h-[calc(100vh-200px)]">
                            <TransitionPanel activeIndex={tabIndex} variants={tabVariants} transition={{ duration: 0.3, ease: "easeInOut" }} className="p-4 md:p-6">
                                {tabContent}
                            </TransitionPanel>
                        </ScrollArea>
                    </div>
                </div>
            </div>
        </motion.div>
    );
}
