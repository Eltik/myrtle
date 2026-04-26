import { useSuspenseQuery } from "@tanstack/react-query";
import { useParams } from "@tanstack/react-router";
import { operatorQueryOptions } from "#/lib/api/operators";
import { useState } from "react";
import { OperatorHero } from "./impl/OperatorHero";
import { OperatorTabs } from "./impl/OperatorTabs";

export function OperatorDetail() {
    const { id } = useParams({ from: "/operators_/$id" });
    const { data: operator } = useSuspenseQuery(operatorQueryOptions(id));

    const [activeTab, setActiveTab] = useState<"info" | "skills" | "levelup" | "skins" | "audio" | "lore">("info");

    return operator ? (
        <div className="relative min-h-screen w-full min-w-0 overflow-x-hidden">
            <OperatorHero operator={operator} />
            <div className="relative z-10 mx-auto box-border w-full min-w-0 max-w-6xl px-3 pb-16 sm:px-4 md:px-8">
                <OperatorTabs activeTab={activeTab} onTabChange={setActiveTab} operator={operator} />
            </div>
        </div>
    ) : null;
}
