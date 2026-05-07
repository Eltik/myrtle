import { useQuery } from "@tanstack/react-query";
import { useMemo } from "react";
import { skinsQueryOptions } from "#/lib/api/skins";
import type { IRosterEntry } from "#/lib/api/user";
import type { IOperatorListItem } from "#/types/operators";
import { ClassBreakdownCard } from "./cards/ClassBreakdownCard";
import { CollectionCard } from "./cards/CollectionCard";
import { ElitePromotionCard } from "./cards/ElitePromotionCard";
import { MasteryCard } from "./cards/MasteryCard";
import { ModulesSkinsCard } from "./cards/ModulesSkinsCard";
import { TopOperatorsCard } from "./cards/TopOperatorsCard";
import { computeUserStats } from "./helpers";
import { StatsTabSkeleton } from "./StatsTabSkeleton";

interface IStatsTabProps {
    roster: IRosterEntry[];
    operatorsStatic: IOperatorListItem[];
}

export function StatsTab({ roster, operatorsStatic }: IStatsTabProps) {
    const { data: skinData } = useQuery(skinsQueryOptions());
    const charSkins = skinData?.charSkins;

    const stats = useMemo(() => computeUserStats(roster, operatorsStatic, charSkins), [roster, operatorsStatic, charSkins]);

    if (!charSkins) return <StatsTabSkeleton />;

    return (
        <div className="grid gap-3 pb-8 sm:grid-cols-2">
            <CollectionCard collectionPercentage={stats.collectionPercentage} totalAvailable={stats.totalAvailable} totalOwned={stats.totalOwned} />
            <ElitePromotionCard eliteBreakdown={stats.eliteBreakdown} />
            <ClassBreakdownCard professions={stats.professions} />
            <MasteryCard masteries={stats.masteries} />
            <ModulesSkinsCard modules={stats.modules} skins={stats.skins} />
            <TopOperatorsCard operatorsStatic={operatorsStatic} roster={roster} />
        </div>
    );
}
