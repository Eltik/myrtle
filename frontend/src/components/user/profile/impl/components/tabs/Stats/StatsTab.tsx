import { useQuery } from "@tanstack/react-query";
import { useMemo } from "react";
import { skinsQueryOptions, userSkinsQueryOptions } from "#/lib/api/skins";
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
    uid: string;
    roster: IRosterEntry[];
    operatorsStatic: IOperatorListItem[];
    nonDefaultSkinCount: number | null;
}

const EMPTY_OWNED_SKINS = new Set<string>();

export function StatsTab({ uid, roster, operatorsStatic, nonDefaultSkinCount }: IStatsTabProps) {
    const { data: skinData } = useQuery(skinsQueryOptions());
    const { data: ownedSkins } = useQuery(userSkinsQueryOptions(uid));
    const charSkins = skinData?.charSkins;

    const stats = useMemo(() => computeUserStats(roster, operatorsStatic, charSkins, nonDefaultSkinCount), [roster, operatorsStatic, charSkins, nonDefaultSkinCount]);

    const ownedSkinIds = useMemo(() => (ownedSkins ? new Set(ownedSkins.map((s) => s.skin_id)) : EMPTY_OWNED_SKINS), [ownedSkins]);

    if (!charSkins) return <StatsTabSkeleton />;

    return (
        <div className="grid gap-3 pb-8 sm:grid-cols-2">
            <CollectionCard collectionPercentage={stats.collectionPercentage} totalAvailable={stats.totalAvailable} totalOwned={stats.totalOwned} />
            <ElitePromotionCard eliteBreakdown={stats.eliteBreakdown} />
            <ClassBreakdownCard professions={stats.professions} />
            <MasteryCard masteries={stats.masteries} />
            <ModulesSkinsCard charSkins={charSkins} modules={stats.modules} operatorsStatic={operatorsStatic} ownedSkinIds={ownedSkinIds} skins={stats.skins} />
            <TopOperatorsCard operatorsStatic={operatorsStatic} roster={roster} />
        </div>
    );
}
