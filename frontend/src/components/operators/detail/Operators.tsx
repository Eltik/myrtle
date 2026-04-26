import { useSuspenseQuery } from "@tanstack/react-query";
import { useParams } from "@tanstack/react-router";
import { useState } from "react";
import { operatorQueryOptions } from "#/lib/api/operators";
import { OperatorHero } from "./impl/components/OperatorHero";
import { OperatorTabs } from "./impl/components/OperatorTabs";
import type { TabType } from "./impl/constants";

export function OperatorDetail() {
    const { id } = useParams({ from: "/operators_/$id" });
    const { data: operator } = useSuspenseQuery(operatorQueryOptions(id));
    const [activeTab, setActiveTab] = useState<TabType>("info");

    if (!operator) return null;
    return (
        <div className="relative min-h-screen w-full min-w-0 overflow-x-clip">
            <OperatorHero operator={operator} />
            <div className="relative z-10 mx-auto box-border w-full min-w-0 max-w-6xl px-3 pb-16 sm:px-4 md:px-8">
                <OperatorTabs activeTab={activeTab} onTabChange={setActiveTab} operator={operator} />
            </div>
        </div>
    );
}
