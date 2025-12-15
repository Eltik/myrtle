"use client";

import { useState } from "react";
import type { Operator } from "~/types/api";
import { OperatorHero } from "./sections/operator-hero";
import { OperatorTabs } from "./sections/operator-tabs";

interface OperatorDetailProps {
    operator: Operator;
}

export function OperatorDetail({ operator }: OperatorDetailProps) {
    const [activeTab, setActiveTab] = useState<"info" | "skills" | "levelup" | "skins" | "audio" | "lore">("info");

    return (
        <div className="relative min-h-screen w-full min-w-0 overflow-x-hidden">
            {/* Hero Section with parallax background */}
            <OperatorHero operator={operator} />

            {/* Content Section */}
            <div className="relative z-10 mx-auto box-border w-full min-w-0 max-w-6xl overflow-hidden px-3 pb-16 sm:px-4 md:px-8">
                <OperatorTabs activeTab={activeTab} onTabChange={setActiveTab} operator={operator} />
            </div>
        </div>
    );
}
