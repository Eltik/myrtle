import { useSuspenseQuery } from "@tanstack/react-query";
import { useParams } from "@tanstack/react-router";
import { useSelector } from "@tanstack/react-store";
import { useState } from "react";
import { operatorQueryOptions } from "#/lib/api/operators";
import { useGamedataServer } from "#/lib/i18n";
import { themeStore } from "#/lib/theme/store";
import { CnUpdatesSection } from "./impl/components/CnUpdatesSection";
import { OperatorFormSwitcher } from "./impl/components/OperatorFormSwitcher";
import { OperatorHero } from "./impl/components/OperatorHero";
import { OperatorTabs } from "./impl/components/OperatorTabs";
import type { TabType } from "./impl/constants";

export function OperatorDetail() {
    const { id } = useParams({ from: "/operators_/$id" });
    const server = useGamedataServer();
    const { data: operator } = useSuspenseQuery(operatorQueryOptions(id, server));
    const [activeTab, setActiveTab] = useState<TabType>("info");
    // Off by default and false during SSR, so the section and its two requests exist only for a
    // reader who opted in; a CN page has nothing newer to show.
    const cnUpdates = useSelector(themeStore, (s) => s.cnUpdates);
    const showCnUpdates = cnUpdates && server !== "cn" && operator?.server !== "cn";

    if (!operator) return null;
    return (
        <div className="relative mx-auto min-h-screen w-full min-w-0 max-w-6xl overflow-x-clip">
            <OperatorHero operator={operator} />
            <div className="relative z-10 mx-auto box-border w-full min-w-0 max-w-6xl px-3 pb-16 sm:px-4 md:px-8">
                <OperatorFormSwitcher operator={operator} />
                {showCnUpdates && <CnUpdatesSection operator={operator} server={server} />}
                <OperatorTabs activeTab={activeTab} onTabChange={setActiveTab} operator={operator} />
            </div>
        </div>
    );
}
