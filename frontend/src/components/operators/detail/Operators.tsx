import { useSuspenseQuery } from "@tanstack/react-query";
import { useParams } from "@tanstack/react-router";
import { useState } from "react";
import { operatorQueryOptions } from "#/lib/api/operators";
import { useGamedataServer } from "#/lib/i18n";
import { CnUpdatesContext, useCnUpdatesFor } from "./impl/cnUpdates";
import { CnChangeSummary } from "./impl/components/CnMarkers";
import { OperatorFormSwitcher } from "./impl/components/OperatorFormSwitcher";
import { OperatorHero } from "./impl/components/OperatorHero";
import { OperatorTabs } from "./impl/components/OperatorTabs";
import type { TabType } from "./impl/constants";

export function OperatorDetail() {
    const { id } = useParams({ from: "/operators_/$id" });
    const server = useGamedataServer();
    const { data: operator } = useSuspenseQuery(operatorQueryOptions(id, server));
    const [activeTab, setActiveTab] = useState<TabType>("info");
    // The tabs read CN's added modules and skills as their own, and mark CN's changed values in
    // place; with the CN updates setting off they get `operator` untouched and nothing is fetched.
    const { operator: shown, updates } = useCnUpdatesFor(operator, server);

    if (!operator || !shown) return null;
    return (
        <div className="relative mx-auto min-h-screen w-full min-w-0 max-w-6xl overflow-x-clip">
            <OperatorHero operator={operator} />
            <div className="relative z-10 mx-auto box-border w-full min-w-0 max-w-6xl px-3 pb-16 sm:px-4 md:px-8">
                <OperatorFormSwitcher operator={operator} />
                {updates && <CnChangeSummary changes={updates.changes} className="mb-4" />}
                <CnUpdatesContext value={updates}>
                    <OperatorTabs activeTab={activeTab} onTabChange={setActiveTab} operator={shown} />
                </CnUpdatesContext>
            </div>
        </div>
    );
}
