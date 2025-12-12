"use client";

import { useState } from "react";
import type { Operator } from "~/types/api";
import { OperatorHero } from "./sections/operator-hero";
import { OperatorTabs } from "./sections/operator-tabs";

interface OperatorDetailProps {
    operator: Operator;
}

export function OperatorDetail({ operator }: OperatorDetailProps) {
    const [activeTab, setActiveTab] = useState<"info" | "skills" | "levelup" | "skins" | "audio">("info");

    return (
        <div className="relative min-h-screen w-full">
            {/* Hero Section with parallax background */}
            <OperatorHero operator={operator} />

            {/* Content Section */}
            <div className="relative z-10 mx-auto w-full max-w-6xl px-4 pb-16 md:px-8">
                <OperatorTabs activeTab={activeTab} onTabChange={setActiveTab} operator={operator} />
            </div>
        </div>
    );
}
