import { useQuery } from "@tanstack/react-query";
import * as React from "react";
import { useAuth } from "#/hooks/use-auth";
import { operatorsIndexQueryOptions } from "#/lib/api/operators";
import { releaseBannersQueryOptions } from "#/lib/api/release";
import { useGamedataServer, useLocale, useT } from "#/lib/i18n";
import { formatDate } from "../helpers";
import { projectIncome } from "../pulls/income";
import { MAX_POT_COPIES } from "../pulls/odds";
import { buildPlan, goalEstimateFor, planToCsv } from "../pulls/plan";
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

    // The outfit picks from the Planner tab are reserved out of the Originite balance
    // before any of it is converted, so a pick there is a step down in pulls here.
    const days = React.useMemo(() => projectIncome(deferred, today, horizon, skins.originite), [deferred, today, horizon, skins.originite]);

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
        (key: string, charId: string, copies: number) => {
            const row = plan.rows.find((r) => r.key === key);
            const clamped = Math.max(0, Math.min(MAX_POT_COPIES, Math.floor(copies)));
            // From the LIVE settings, not from `row`, which is built off the deferred
            // copy: two quick presses would both read the pre-first-press picks and
            // the second would write P1 where P2 was asked for.
            const current = { ...(settings.targets[key] ?? {}) };
            if (clamped > 0) current[charId] = clamped;
            else delete current[charId];

            /**
             * The walk runs HERE, before the state update, not inside the updater.
             * React may call an updater more than once for one dispatch, and this walk
             * is up to 297 ms, which is the delay reported between pressing a plus and
             * seeing the number move. It is also memoised now, so the rebuild that
             * follows asks the same question and gets the cached answer rather than
             * paying for the same walk a second time.
             */
            const wanted = Object.keys(current).length > 0;
            const priced =
                wanted && row
                    ? goalEstimateFor(
                          row.model,
                          {
                              copiesA: current[row.featured[0]] ?? 0,
                              copiesB: row.featured.length > 1 ? (current[row.featured[1]] ?? 0) : 0,
                          },
                          row.pityDist,
                      ).p50
                    : null;

            setSettings((s) => {
                const targets = { ...s.targets };
                const allocations = { ...s.allocations };
                if (wanted) {
                    targets[key] = current;
                    if (priced !== null) allocations[key] = priced;
                } else {
                    delete targets[key];
                    delete allocations[key];
                }
                return { ...s, targets, allocations };
            });
        },
        [setSettings, plan.rows, settings.targets],
    );

    /**
     * A preset from the goal list or the Set-to row: commit that many pulls AND drop
     * this banner's potential picks.
     *
     * Clearing them is the point. A pick sets the count through `setTarget`, so a
     * banner carrying picks has a count that MEANS those picks; committing a different
     * figure on top of them left the row claiming two different goals at once, and the
     * "Your goal" line went on describing the operators while every other line
     * described the new count.
     */
    const preset = React.useCallback(
        (key: string, pulls: number) =>
            setSettings((s) => {
                const allocations = { ...s.allocations };
                const targets = { ...s.targets };
                if (pulls > 0) allocations[key] = pulls;
                else delete allocations[key];
                delete targets[key];
                return { ...s, allocations, targets };
            }),
        [setSettings],
    );

    /** One banner back to untouched: the committed pulls and the potentials both go. */
    const clearRow = React.useCallback(
        (key: string) =>
            setSettings((s) => {
                const allocations = { ...s.allocations };
                const targets = { ...s.targets };
                delete allocations[key];
                delete targets[key];
                return { ...s, allocations, targets };
            }),
        [setSettings],
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

            {banners.isPending ? <ReleaseLoading /> : <PullsBanners rows={plan.rows} totals={plan.totals} lookup={lookup} charNames={banners.data?.charNames ?? {}} today={today} onAllocate={allocate} onPreset={preset} onSetTarget={setTarget} onClearRow={clearRow} onReset={resetPlan} onExport={exportPlan} />}

            <div className="grid grid-cols-1 gap-3 xl:grid-cols-2 xl:items-start">
                <PullsOdds budget={uncommitted} pity={deferred.pity} />
                <PullsSimulator budget={uncommitted} pity={deferred.pity} />
            </div>
        </div>
    );
}
