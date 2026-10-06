import { useQuery } from "@tanstack/react-query";
import { useMemo } from "react";
import { skinsIndexQueryOptions, userSkinsQueryOptions } from "#/lib/api/skins";
import { type IRosterEntry, userCheckinQueryOptions } from "#/lib/api/user";
import { useGamedataServer, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { IOperatorIndexEntry } from "#/types/operators";
import { ClassBreakdownCard } from "./cards/ClassBreakdownCard";
import { CollectionCard } from "./cards/CollectionCard";
import { ElitePromotionCard } from "./cards/ElitePromotionCard";
import { MasteryCard } from "./cards/MasteryCard";
import { ModulesSkinsCard } from "./cards/ModulesSkinsCard";
import { SignInCalendarCard, SignInOverviewCard } from "./cards/SignInCard";
import { TopOperatorsCard } from "./cards/TopOperatorsCard";
import { computeUserStats } from "./helpers";
import type { messages } from "./StatsTab.messages";
import { StatsTabSkeleton } from "./StatsTabSkeleton";

interface IStatsTabProps {
    uid: string;
    server: string;
    roster: IRosterEntry[];
    operatorsIndex: IOperatorIndexEntry[];
    nonDefaultSkinCount: number | null;
    /**
     * The owner hid their Roster tab from this visitor, so `/roster` refused it and
     * `roster` is empty. Every card computed from the roster is left out rather than
     * drawn as an empty account; the sign-in cards do not read it and stay.
     */
    rosterPrivate?: boolean;
}

const EMPTY_OWNED_SKINS = new Set<string>();

export function StatsTab({ uid, server, roster, operatorsIndex, nonDefaultSkinCount, rosterPrivate = false }: IStatsTabProps) {
    const t: TypedT<typeof messages> = useT("user");
    const { data: charSkins } = useQuery(skinsIndexQueryOptions(useGamedataServer()));
    const { data: ownedSkins } = useQuery({ ...userSkinsQueryOptions(uid), enabled: !rosterPrivate });
    const { data: checkin } = useQuery(userCheckinQueryOptions(uid));

    const stats = useMemo(() => computeUserStats(roster, operatorsIndex, charSkins, nonDefaultSkinCount), [roster, operatorsIndex, charSkins, nonDefaultSkinCount]);

    const ownedSkinIds = useMemo(() => (ownedSkins ? new Set(ownedSkins.map((s) => s.skin_id)) : EMPTY_OWNED_SKINS), [ownedSkins]);

    if (rosterPrivate) {
        return (
            <div className="flex flex-col gap-3 pb-8">
                <p className="text-muted-foreground text-sm">{t("profile.stats.rosterPrivate")}</p>
                <div className="grid gap-3 sm:grid-cols-2">
                    <SignInOverviewCard checkin={checkin} server={server} />
                    <SignInCalendarCard checkin={checkin} server={server} />
                </div>
            </div>
        );
    }

    if (!charSkins) return <StatsTabSkeleton />;

    return (
        <div className="grid gap-3 pb-8 sm:grid-cols-2">
            <CollectionCard collectionPercentage={stats.collectionPercentage} totalAvailable={stats.totalAvailable} totalOwned={stats.totalOwned} />
            <ElitePromotionCard eliteBreakdown={stats.eliteBreakdown} />
            <ClassBreakdownCard professions={stats.professions} />
            <MasteryCard masteries={stats.masteries} />
            <ModulesSkinsCard charSkins={charSkins} modules={stats.modules} operatorsIndex={operatorsIndex} ownedSkinIds={ownedSkinIds} skins={stats.skins} />
            <TopOperatorsCard operatorsIndex={operatorsIndex} roster={roster} />
            <SignInOverviewCard checkin={checkin} server={server} />
            <SignInCalendarCard checkin={checkin} server={server} />
        </div>
    );
}
