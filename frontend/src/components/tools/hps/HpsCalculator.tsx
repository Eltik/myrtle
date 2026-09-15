import { useQuery } from "@tanstack/react-query";
import { ChevronRight, ChevronsDownUp, ChevronsUpDown, Download, HeartPulse, HelpCircle, RefreshCw } from "lucide-react";
import * as React from "react";
import { AxisControls } from "#/components/tools/shared/AxisControls";
import { CalcChart } from "#/components/tools/shared/CalcChart";
import { buildSweepPoints, formatLargeNumber } from "#/components/tools/shared/constants";
import { exportSvgAsPng } from "#/components/tools/shared/exportChart";
import { InstanceCard } from "#/components/tools/shared/InstanceCard";
import { KpiPanel } from "#/components/tools/shared/KpiPanel";
import { OperatorPicker } from "#/components/tools/shared/OperatorPicker";
import { moduleShortLabel } from "#/components/tools/shared/useOperatorDetail";
import type { messages as detailMessages } from "#/components/tools/shared/useOperatorDetail.messages";
import { AlertDialog, AlertDialogClose, AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogPopup, AlertDialogTitle, AlertDialogTrigger } from "#/components/ui/alert-dialog";
import { Button } from "#/components/ui/button";
import { Card, CardHeader, CardPanel, CardTitle } from "#/components/ui/card";
import { Popover, PopoverPopup, PopoverTrigger } from "#/components/ui/popover";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import { hpsOperatorsQueryOptions, type IHpsOperatorListEntry } from "#/lib/api/hps";
import { operatorsListQueryOptions } from "#/lib/api/operators";
import { type TypedRichT, useGamedataServer, useLocale, useRichT, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { messages } from "./HpsCalculator.messages";
import { BuffsPanel } from "./impl/components/BuffsPanel";
import { METRIC_COLUMNS, X_AXIS_INPUT, X_AXIS_LABEL_KEYS, X_AXIS_SHORT_KEYS, Y_METRIC_HINT_KEYS, Y_METRIC_LABEL_KEYS } from "./impl/constants";
import type { messages as hpsConstantsMessages } from "./impl/constants.messages";
import type { HpsXAxis, HpsYMetric } from "./impl/types";
import { useHpsCurves } from "./impl/useHpsCurves";
import { useHpsResults } from "./impl/useHpsResults";
import { useHpsState } from "./impl/useHpsState";

/** This page renders its own chrome, the axis/metric labels from `constants.ts`, and the build-summary labels `useOperatorDetail.ts` derives. */
type HpsT = TypedT<typeof messages & typeof hpsConstantsMessages & typeof detailMessages>;
type HpsRichT = TypedRichT<typeof messages>;

export function HpsCalculator(): React.ReactElement {
    const t: HpsT = useT("tools");
    const rt: HpsRichT = useRichT("tools");
    const locale = useLocale();
    const { state, dispatch, hydrationToken } = useHpsState();

    const axes = React.useMemo(() => (Object.keys(X_AXIS_LABEL_KEYS) as HpsXAxis[]).map((k) => ({ value: k, label: t(X_AXIS_LABEL_KEYS[k]), short: t(X_AXIS_SHORT_KEYS[k]) })), [t]);
    const metrics = React.useMemo(() => (Object.keys(Y_METRIC_LABEL_KEYS) as HpsYMetric[]).map((k) => ({ value: k, label: t(Y_METRIC_LABEL_KEYS[k]) })), [t]);
    const columns = React.useMemo(
        () =>
            METRIC_COLUMNS.map((c) => ({
                key: c.key,
                label: t(c.labelKey),
                hint: c.key === "base_hps" ? (d: Record<string, number>) => (d.base_hps === 0 ? t("hps.baseHps.burstHint") : undefined) : undefined,
            })),
        [t],
    );

    const snapshots = useHpsResults(state.instances, state.buffs);
    const curves = useHpsCurves(state, state.buffs);

    const hasInstances = state.instances.length > 0;
    const allCollapsed = hasInstances && state.instances.every((i) => i.collapsed);
    const visibleCount = state.instances.filter((i) => i.visible).length;

    const atkPct = Math.round((state.buffs.buffs.atk ?? 0) * 100);
    const snapshotX = state.xAxis === "targets" ? state.buffs.targets : state.xAxis === "atk" ? atkPct : (state.buffs.buffs.aspd ?? 0);
    const pointCount = buildSweepPoints(state.sweep[state.xAxis], state.xAxis === "targets").length;

    const onResetAll = React.useCallback(() => dispatch({ type: "RESET_INSTANCES" }), [dispatch]);
    const onToggleCollapseAll = React.useCallback(() => dispatch({ type: "SET_ALL_COLLAPSED", collapsed: !allCollapsed }), [allCollapsed, dispatch]);

    const { data: staticOps } = useQuery(operatorsListQueryOptions(useGamedataServer()));
    const chartContainerRef = React.useRef<HTMLDivElement>(null);
    const [isExporting, setIsExporting] = React.useState(false);
    const onExportChart = React.useCallback(async () => {
        const svg = findChartSvg(chartContainerRef.current);
        if (!svg) return;
        const stamp = new Date().toISOString().slice(0, 10);
        const filename = `hps-${state.xAxis}-${state.yMetric}-${stamp}.png`;

        const sameOpCounts = new Map<string, number>();
        for (const i of state.instances) sameOpCounts.set(i.op.id, (sameOpCounts.get(i.op.id) ?? 0) + 1);

        const visibleInstances = state.instances.filter((i) => i.visible);
        const legend = visibleInstances.map((inst, idx) => {
            const dupSuffix = (sameOpCounts.get(inst.op.id) ?? 1) > 1 ? ` #${idx + 1}` : "";
            const moduleSummary =
                inst.config.moduleIndex > 0
                    ? t("hps.export.moduleSummary", {
                          module: moduleShortLabel(
                              staticOps?.find((o) => o.id === inst.op.id),
                              inst.op,
                              inst.config.moduleIndex,
                              t,
                          ),
                          level: inst.config.moduleLevel,
                      })
                    : t("calc.detail.noModuleLower");
            return { color: inst.color, label: `${inst.op.name}${dupSuffix}`, sublabel: t("hps.export.buildSummary", { skill: inst.config.skillIndex, module: moduleSummary }) };
        });

        const buffsDesc = t("hps.export.buffs", { atk: atkPct, aspd: state.buffs.buffs.aspd ?? 0, targets: state.buffs.targets });
        const title = t("hps.export.title", { metric: t(Y_METRIC_LABEL_KEYS[state.yMetric]), axis: t(X_AXIS_LABEL_KEYS[state.xAxis]), buffs: buffsDesc });

        const snapshotEntries = visibleInstances
            .map((inst, idx) => {
                const dupSuffix = (sameOpCounts.get(inst.op.id) ?? 1) > 1 ? ` #${idx + 1}` : "";
                const value = snapshots.find((s) => s.uid === inst.uid)?.data?.[state.yMetric];
                if (typeof value !== "number") return null;
                return { color: inst.color, name: `${inst.op.name}${dupSuffix}`, value: formatLargeNumber(value, locale) };
            })
            .filter((e): e is { color: string; name: string; value: string } => e !== null);

        const snapshotHeading = state.xAxis === "targets" ? t("hps.export.snapshotTargets", { count: state.buffs.targets }) : state.xAxis === "atk" ? t("hps.export.snapshotAtk", { value: atkPct }) : t("hps.export.snapshotAspd", { value: state.buffs.buffs.aspd ?? 0 });
        const snapshot = snapshotEntries.length > 0 ? { heading: snapshotHeading, entries: snapshotEntries } : undefined;

        setIsExporting(true);
        try {
            await exportSvgAsPng(svg, { filename, title, legend, snapshot });
        } catch (err) {
            console.error("HPS chart export failed:", err);
            if (typeof window !== "undefined") window.alert(t("hps.export.failed", { error: err instanceof Error ? err.message : String(err) }));
        } finally {
            setIsExporting(false);
        }
    }, [state.xAxis, state.yMetric, state.instances, state.buffs, atkPct, snapshots, staticOps, t, locale]);

    const { data: latestOps } = useQuery(hpsOperatorsQueryOptions());
    React.useEffect(() => {
        if (!latestOps || state.instances.length === 0) return;
        const map = new Map(latestOps.map((op) => [op.id, op]));
        const stale = state.instances.some((inst) => {
            const fresh = map.get(inst.op.id);
            return fresh && fresh !== inst.op;
        });
        if (stale) dispatch({ type: "REFRESH_OPS", freshOps: map });
    }, [latestOps, state.instances, dispatch]);

    return (
        <div className="relative z-1 mx-auto w-[min(1400px,calc(100%-2rem))] py-5 pb-20">
            <nav aria-label="breadcrumb" className="mb-2.5 flex items-center gap-1.5 font-medium font-sans text-[12px] text-muted-foreground leading-none">
                <span>{t("hps.breadcrumb.tools")}</span>
                <ChevronRight className="size-2.5" />
                <span className="text-foreground">{t("hps.title")}</span>
            </nav>
            <div className="flex flex-wrap items-end justify-between gap-3">
                <div className="min-w-0 flex-1">
                    <div className="flex items-center gap-2">
                        <h1 className="m-0 font-bold font-sans text-[24px] text-foreground leading-[1.1] tracking-tight sm:text-[30px]">{t("hps.title")}</h1>
                        <Popover>
                            <PopoverTrigger
                                render={(p) => (
                                    <Button {...p} aria-label={t("hps.help.open")} variant="ghost" size="icon-sm" className="text-muted-foreground hover:text-foreground">
                                        <HelpCircle />
                                    </Button>
                                )}
                            />
                            <PopoverPopup className="w-[min(380px,calc(100vw-2rem))]">
                                <h2 className="mb-2 font-semibold text-[14px] text-foreground">{t("hps.help.title")}</h2>
                                <ul className="space-y-1.5 text-[12.5px] text-muted-foreground leading-relaxed">
                                    <li>
                                        <span className="font-medium text-foreground">{t("hps.help.skillHps.term")}</span> {t("hps.help.skillHps.desc")}
                                    </li>
                                    <li>
                                        <span className="font-medium text-foreground">{t("hps.help.baseHps.term")}</span> {t("hps.help.baseHps.desc")}
                                    </li>
                                    <li>
                                        <span className="font-medium text-foreground">{t("hps.help.averageHps.term")}</span> {t("hps.help.averageHps.desc")}
                                    </li>
                                    <li>{t("hps.help.sweep")}</li>
                                </ul>
                            </PopoverPopup>
                        </Popover>
                    </div>
                    <p className="mt-1.5 font-sans text-[13.5px] text-muted-foreground leading-normal">{t("hps.intro")}</p>
                    <p className="max-w-xl font-sans text-[13.5px] text-muted-foreground leading-normal">
                        <b>{t("hps.note.label")}</b>{" "}
                        {rt("hps.note.credit", {
                            link: (
                                <a className="text-blue-500 hover:underline" href="https://github.com/WhoAteMyCQQkie/ArknightsDpsCompare" target="_blank" rel="noopener">
                                    {t("hps.note.creditLink")}
                                </a>
                            ),
                        })}
                    </p>
                </div>
                {hasInstances && (
                    <AlertDialog>
                        <AlertDialogTrigger
                            render={(p) => (
                                <Button {...p} variant="outline" size="sm">
                                    <RefreshCw />
                                    {t("hps.clearAll")}
                                </Button>
                            )}
                        />
                        <AlertDialogPopup>
                            <AlertDialogHeader>
                                <AlertDialogTitle>{t("hps.clearAll.title")}</AlertDialogTitle>
                                <AlertDialogDescription>{t("hps.clearAll.desc", { count: state.instances.length })}</AlertDialogDescription>
                            </AlertDialogHeader>
                            <AlertDialogFooter>
                                <AlertDialogClose render={(p) => <Button {...p} variant="outline" />}>{t("hps.cancel")}</AlertDialogClose>
                                <AlertDialogClose render={(p) => <Button {...p} variant="destructive" onClick={onResetAll} />}>{t("hps.clearAll")}</AlertDialogClose>
                            </AlertDialogFooter>
                        </AlertDialogPopup>
                    </AlertDialog>
                )}
            </div>

            <div className="mt-6 grid grid-cols-1 items-start gap-4 xl:grid-cols-[420px_1fr]">
                <aside className="flex min-w-0 flex-col gap-4 xl:col-start-1 xl:row-start-1">
                    <Card>
                        <CardHeader className="flex grid-rows-1 flex-row items-center justify-between gap-2">
                            <CardTitle className="min-w-0 flex-1 truncate text-[15px]">
                                {t("hps.healers")}
                                {hasInstances && (
                                    <span className="ml-1.5 font-medium font-mono text-[11px] text-muted-foreground">{visibleCount === state.instances.length ? t("hps.healers.count", { count: state.instances.length }) : t("hps.healers.countVisible", { visible: visibleCount, total: state.instances.length })}</span>
                                )}
                            </CardTitle>
                            {state.instances.length > 1 && (
                                <Tooltip>
                                    <TooltipTrigger
                                        render={(p) => (
                                            <Button {...p} aria-label={allCollapsed ? t("hps.expandAll.aria") : t("hps.collapseAll.aria")} variant="ghost" size="icon-sm" className="shrink-0" onClick={onToggleCollapseAll}>
                                                {allCollapsed ? <ChevronsUpDown /> : <ChevronsDownUp />}
                                            </Button>
                                        )}
                                    />
                                    <TooltipPopup>{allCollapsed ? t("hps.expandAll") : t("hps.collapseAll")}</TooltipPopup>
                                </Tooltip>
                            )}
                        </CardHeader>
                        <CardPanel className="pt-0">
                            <HpsOperatorPicker existingCount={state.instances.length} onAdd={(op) => dispatch({ type: "ADD_INSTANCE", op })} />
                        </CardPanel>
                    </Card>

                    <BuffsPanel hydrationToken={hydrationToken} buffs={state.buffs} onChangeBuffs={(patch) => dispatch({ type: "SET_BUFFS", patch })} onChangeBuffValues={(patch) => dispatch({ type: "SET_BUFF_VALUES", patch })} />
                </aside>

                <main className="flex min-w-0 flex-col gap-4 xl:sticky xl:top-20 xl:col-start-2 xl:row-span-2 xl:row-start-1 xl:max-h-[calc(100vh-6rem)] xl:overflow-y-auto xl:pr-1">
                    <Card>
                        <CardHeader className="pb-3">
                            <AxisControls
                                axes={axes}
                                xAxis={state.xAxis}
                                onChangeAxis={(axis) => dispatch({ type: "SET_AXIS", axis: axis as HpsXAxis })}
                                metrics={metrics}
                                yMetric={state.yMetric}
                                onChangeMetric={(metric) => dispatch({ type: "SET_METRIC", metric: metric as HpsYMetric })}
                                sweep={state.sweep[state.xAxis]}
                                axisInput={X_AXIS_INPUT[state.xAxis]}
                                pointCount={pointCount}
                                hydrationToken={hydrationToken}
                                metricHint={t(Y_METRIC_HINT_KEYS[state.yMetric])}
                                onChangeSweep={(patch) => dispatch({ type: "SET_SWEEP", axis: state.xAxis, patch })}
                                rangeAction={
                                    hasInstances ? (
                                        <Tooltip>
                                            <TooltipTrigger
                                                render={(p) => (
                                                    <Button {...p} aria-label={t("hps.download")} variant="outline" size="icon-sm" loading={isExporting} onClick={onExportChart} className="mb-1">
                                                        <Download />
                                                    </Button>
                                                )}
                                            />
                                            <TooltipPopup>{t("hps.download")}</TooltipPopup>
                                        </Tooltip>
                                    ) : undefined
                                }
                            />
                        </CardHeader>
                        <CardPanel className="pt-0">
                            <CalcChart
                                instances={state.instances}
                                rows={curves.rows}
                                xLabel={t(X_AXIS_LABEL_KEYS[state.xAxis])}
                                yLabel={t(Y_METRIC_LABEL_KEYS[state.yMetric])}
                                allowDecimals={state.xAxis !== "targets"}
                                formatTooltipX={(x) => (state.xAxis === "targets" ? t("hps.tooltip.targets", { count: x }) : state.xAxis === "atk" ? t("hps.tooltip.atk", { value: x }) : t("hps.tooltip.aspd", { value: x }))}
                                snapshotX={snapshotX}
                                isLoading={curves.isPending}
                                onLegendClick={(uid) => dispatch({ type: "TOGGLE_VISIBILITY", uid })}
                                containerRef={chartContainerRef}
                                emptyIcon={<HeartPulse />}
                                emptyTitle={t("hps.empty.title")}
                                emptyDescription={rt("hps.empty.desc", { axis: <span className="font-medium text-foreground">{t(X_AXIS_LABEL_KEYS[state.xAxis])}</span> })}
                            />
                        </CardPanel>
                    </Card>

                    <KpiPanel instances={state.instances} snapshots={snapshots} leaderKey={state.yMetric} leaderLabel={t(Y_METRIC_LABEL_KEYS[state.yMetric])} columns={columns} />
                </main>

                <section aria-label={t("hps.configured")} className="flex min-w-0 flex-col gap-4 xl:col-start-1 xl:row-start-2">
                    {state.instances.map((inst, idx) => (
                        <InstanceCard
                            key={inst.uid}
                            inst={inst}
                            index={idx}
                            isFirst={idx === 0}
                            isLast={idx === state.instances.length - 1}
                            onUpdate={(patch) => dispatch({ type: "UPDATE_CONFIG", uid: inst.uid, patch })}
                            onToggleConditional={(key, value) => dispatch({ type: "TOGGLE_CONDITIONAL", uid: inst.uid, key, value })}
                            onToggleVisibility={() => dispatch({ type: "TOGGLE_VISIBILITY", uid: inst.uid })}
                            onToggleCollapsed={() => dispatch({ type: "TOGGLE_COLLAPSED", uid: inst.uid })}
                            onMoveUp={() => dispatch({ type: "REORDER_INSTANCE", uid: inst.uid, direction: "up" })}
                            onMoveDown={() => dispatch({ type: "REORDER_INSTANCE", uid: inst.uid, direction: "down" })}
                            onDuplicate={() => dispatch({ type: "DUPLICATE_INSTANCE", uid: inst.uid })}
                            onRemove={() => dispatch({ type: "REMOVE_INSTANCE", uid: inst.uid })}
                        />
                    ))}
                </section>
            </div>
        </div>
    );
}

function HpsOperatorPicker({ existingCount, onAdd }: { existingCount: number; onAdd: (op: IHpsOperatorListEntry) => void }): React.ReactElement {
    const { data: operators = [], isLoading, isError, error } = useQuery(hpsOperatorsQueryOptions());
    return <OperatorPicker operators={operators} isLoading={isLoading} isError={isError} error={error} existingCount={existingCount} onAdd={onAdd} noun="healer" />;
}

function findChartSvg(container: HTMLDivElement | null): SVGSVGElement | null {
    if (!container) return null;
    const wrapperChild = container.querySelector<SVGSVGElement>(".recharts-wrapper > svg");
    if (wrapperChild) return wrapperChild;
    const candidates = Array.from(container.querySelectorAll<SVGSVGElement>("svg"));
    let best: SVGSVGElement | null = null;
    let bestArea = 0;
    for (const svg of candidates) {
        const r = svg.getBoundingClientRect();
        const area = r.width * r.height;
        if (area > bestArea) {
            best = svg;
            bestArea = area;
        }
    }
    return best;
}
