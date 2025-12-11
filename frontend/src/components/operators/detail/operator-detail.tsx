"use client";

import { useState } from "react";
import { motion, AnimatePresence } from "motion/react";
import type { Operator } from "~/types/api";
import { OperatorHero } from "./operator-hero";
import { OperatorNavigation, type TabType } from "./operator-navigation";
import { InfoTab } from "./tabs/info-tab";
import { SkillsTab } from "./tabs/skills-tab";
import { LevelUpTab } from "./tabs/levelup-tab";
import { SkinsTab } from "./tabs/skins-tab";
import { AudioTab } from "./tabs/audio-tab";

interface OperatorDetailProps {
    operator: Operator;
}

const tabVariants = {
    enter: { opacity: 0, y: 20 },
    center: { opacity: 1, y: 0 },
    exit: { opacity: 0, y: -20 },
};

export function OperatorDetail({ operator }: OperatorDetailProps) {
    const [activeTab, setActiveTab] = useState<TabType>("info");

    const renderTabContent = () => {
        switch (activeTab) {
            case "info":
                return <InfoTab key="info" operator={operator} />;
            case "skills":
                return <SkillsTab key="skills" operator={operator} />;
            case "levelup":
                return <LevelUpTab key="levelup" operator={operator} />;
            case "skins":
                return <SkinsTab key="skins" operator={operator} />;
            case "audio":
                return <AudioTab key="audio" operator={operator} />;
            default:
                return <InfoTab key="info" operator={operator} />;
        }
    };

    return (
        <div className="-mx-4 -mt-8 md:-mx-8 md:-mt-12">
            <OperatorHero operator={operator} />

            <div className="mx-auto max-w-6xl px-4 md:px-8">
                <div className="flex flex-col gap-6 lg:flex-row">
                    <OperatorNavigation activeTab={activeTab} onTabChange={setActiveTab} />

                    <div className="min-w-0 flex-1">
                        <AnimatePresence mode="wait">
                            <motion.div key={activeTab} variants={tabVariants} initial="enter" animate="center" exit="exit" transition={{ duration: 0.2, ease: "easeInOut" }}>
                                {renderTabContent()}
                            </motion.div>
                        </AnimatePresence>
                    </div>
                </div>
            </div>
        </div>
    );
}
