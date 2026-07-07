import { useQuery } from "@tanstack/react-query";
import { useMemo } from "react";
import { skinsIndexQueryOptions, userSkinsQueryOptions } from "#/lib/api/skins";
import { type IRosterEntry, userCheckinQueryOptions } from "#/lib/api/user";
import type { IOperatorListItem } from "#/types/operators";
import { ClassBreakdownCard } from "./cards/ClassBreakdownCard";
import { CollectionCard } from "./cards/CollectionCard";
import { ElitePromotionCard } from "./cards/ElitePromotionCard";
import { MasteryCard } from "./cards/MasteryCard";
import { ModulesSkinsCard } from "./cards/ModulesSkinsCard";
import { SignInCalendarCard, SignInOverviewCard } from "./cards/SignInCard";
import { TopOperatorsCard } from "./cards/TopOperatorsCard";
import { computeUserStats } from "./helpers";
import { StatsTabSkeleton } from "./StatsTabSkeleton";

interface IStatsTabProps {
    uid: string;
    server: string;
    roster: IRosterEntry[];
    operatorsStatic: IOperatorListItem[];
    nonDefaultSkinCount: number | null;
}

const EMPTY_OWNED_SKINS = new Set<string>();

export function StatsTab({ uid, server, roster, operatorsStatic, nonDefaultSkinCount }: IStatsTabProps) {
    const { data: charSkins } = useQuery(skinsIndexQueryOptions());
    const { data: ownedSkins } = useQuery(userSkinsQueryOptions(uid));
    const { data: checkin } = useQuery(userCheckinQueryOptions(uid));

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
            <SignInOverviewCard checkin={checkin} server={server} />
            <SignInCalendarCard checkin={checkin} />
        </div>
    );
}
