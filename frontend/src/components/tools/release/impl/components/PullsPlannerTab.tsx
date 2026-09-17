import { useQuery } from "@tanstack/react-query";
import * as React from "react";
import { useAuth } from "#/hooks/use-auth";
import { operatorsIndexQueryOptions } from "#/lib/api/operators";
import { releaseBannersQueryOptions } from "#/lib/api/release";
import { useGamedataServer, useLocale, useT } from "#/lib/i18n";
import { formatDate } from "../helpers";
import { projectIncome } from "../pulls/income";
import { MAX_POT_COPIES, pullsToGoal } from "../pulls/odds";
import { buildPlan, planToCsv } from "../pulls/plan";
import { useSkinCommitment } from "../pulls/skins";
import { usePullsSettings } from "../pulls/store";
import { PullsBanners } from "./PullsBanners";
import { PullsBudget } from "./PullsBudget";
import { PullsOdds } from "./PullsOdds";
import type { PullsT } from "./PullsShared";
import { PullsSimulator } from "./PullsSimulator";
import { buildOperatorLookup, ReleaseError, ReleaseLoading } from "./shared";

interface IPullsPlannerTabProps {
    today: Date;
}

export function PullsPlannerTab({ today }: IPullsPlannerTabProps): React.ReactElement {
    const t: PullsT = useT("tools");
    const locale = useLocale();
    const [settings, setSettings, ready] = usePullsSettings();
    const { user } = useAuth();
    const skins = useSkinCommitment(user?.uid ?? null);
    const banners = useQuery(releaseBannersQueryOptions());
    const index = useQuery(operatorsIndexQueryOptions(useGamedataServer()));
    const lookup = React.useMemo(() => buildOperatorLookup(index.data), [index.data]);

    /**
     * The panels below are cheap except for the plan, which runs an exact DP per
     * banner row: forty rows measures about 50 ms, and both a keystroke in the
     * resources fields and a click on a plus button change `settings` identity.
     * Deferring the settings that feed the derived work keeps the inputs painting at
     * once and lets the tables catch up a frame later.
     */
    const deferred = React.useDeferredValue(settings);

    const horizon = React.useMemo(() => {
        const end = new Date(today);
        end.setUTCDate(end.getUTCDate() + deferred.horizonDays);
        return end;
    }, [today, deferred.horizonDays]);

    const days = React.useMemo(() => projectIncome(deferred, today, horizon), [deferred, today, horizon]);

    const plan = React.useMemo(
        () =>
            buildPlan({
                banners: banners.data?.banners ?? [],
                days,
                model: banners.data?.model ?? null,
                today,
                pity: deferred.pity,
                allocations: deferred.allocations,
                targets: deferred.targets,
                spendOriginite: deferred.spendOriginite,
                countFreePulls: deferred.countFreePulls,
            }),
        [banners.data, days, today, deferred.pity, deferred.allocations, deferred.targets, deferred.spendOriginite, deferred.countFreePulls],
    );

    const allocate = React.useCallback(
        (key: string, pulls: number) =>
            setSettings((s) => {
                const allocations = { ...s.allocations };
                // Zero is "not planned", not "planned zero", so the key leaves rather
                // than accumulating an entry per banner the user ever touched.
                if (pulls > 0) allocations[key] = pulls;
                else delete allocations[key];
                return { ...s, allocations };
            }),
        [setSettings],
    );
    /**
     * Setting a potential is the fast path into a plan: it says what the user wants
     * out of the banner, and the pull count follows from that rather than being typed.
     *
     * The count is SET to the joint estimate rather than summed per operator, because
     * medians DO NOT add, and they fail to add in both directions. Two copies of one
     * rate-up is 171 rolls against twice-70 of 140, dearer, because the median of a
     * sum is pulled toward the mean of a right-skewed wait. One copy of each is 126,
     * cheaper than 140, because the two operators share the six-star pool. Only the
     * joint walk gets either case right.
     *
     * The walk runs HERE, on the click, rather than in an effect watching the plan.
     * The effect version rewrote the count on every rebuild, so pressing Spark or
     * Guarantee set the number and then had it snatched back a frame later. Pricing
     * it once on the click leaves every other way of setting the count alone.
     */
    const setTarget = React.useCallback(
        (key: string, charId: string, copies: number) =>
            setSettings((s) => {
                const clamped = Math.max(0, Math.min(MAX_POT_COPIES, Math.floor(copies)));
                const current = { ...(s.targets[key] ?? {}) };
                if (clamped > 0) current[charId] = clamped;
                else delete current[charId];

                const targets = { ...s.targets };
                const allocations = { ...s.allocations };
                const row = plan.rows.find((r) => r.key === key);

                if (Object.keys(current).length > 0) {
                    targets[key] = current;
                    if (row) {
                        const featured = row.featured;
                        allocations[key] = pullsToGoal(
                            row.model,
                            {
                                copiesA: current[featured[0]] ?? 0,
                                copiesB: featured.length > 1 ? (current[featured[1]] ?? 0) : 0,
                            },
                            { startPityDist: row.pityDist, startPity: 0 },
                        ).p50;
                    }
                } else {
                    delete targets[key];
                    delete allocations[key];
                }
                return { ...s, targets, allocations };
            }),
        [setSettings, plan.rows],
    );

    const resetPlan = React.useCallback(() => setSettings((s) => ({ ...s, allocations: {}, targets: {} })), [setSettings]);

    const exportPlan = React.useCallback(() => {
        const csv = planToCsv(plan.rows, (at) => formatDate(at, locale));
        const url = URL.createObjectURL(new Blob([csv], { type: "text/csv;charset=utf-8" }));
        const link = document.createElement("a");
        link.href = url;
        link.download = "arknights-pull-plan.csv";
        link.click();
        URL.revokeObjectURL(url);
    }, [plan.rows, locale]);

    // What the odds and simulator panels should reason about is what the plan has
    // NOT already committed, otherwise both would quote a budget the user has spent.
    const uncommitted = plan.totals.remaining;

    /**
     * Where the plan takes rolls out of the bank, for the projection chart above.
     * `spent` rather than `allocated`: an allocation the bank cannot pay for does not
     * leave it, and drawing the overrun as a withdrawal would show a balance falling
     * by rolls the user never had.
     */
    const spend = React.useMemo(() => plan.rows.filter((r) => r.spent > 0).map((r) => ({ at: r.enStart, pulls: r.spent })), [plan.rows]);

    if (!ready) return <ReleaseLoading />;
    if (banners.isError) return <ReleaseError error={banners.error} onRetry={() => banners.refetch()} />;

    return (
        <div className="flex flex-col gap-3">
            <p className="m-0 font-sans text-[12.5px] text-muted-foreground leading-normal">{t("release.pulls.modelNote")}</p>

            <PullsBudget settings={settings} setSettings={setSettings} days={days} committed={plan.totals.spent} freePulls={plan.totals.freePulls} spend={spend} skins={skins} />

            {banners.isPending ? <ReleaseLoading /> : <PullsBanners rows={plan.rows} totals={plan.totals} lookup={lookup} charNames={banners.data?.charNames ?? {}} today={today} onAllocate={allocate} onSetTarget={setTarget} onReset={resetPlan} onExport={exportPlan} />}

            <div className="grid grid-cols-1 gap-3 xl:grid-cols-2 xl:items-start">
                <PullsOdds budget={uncommitted} pity={deferred.pity} />
                <PullsSimulator budget={uncommitted} pity={deferred.pity} />
            </div>
        </div>
    );
}
