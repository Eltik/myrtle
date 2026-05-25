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
import { AlertDialog, AlertDialogClose, AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogPopup, AlertDialogTitle, AlertDialogTrigger } from "#/components/ui/alert-dialog";
import { Button } from "#/components/ui/button";
import { Card, CardHeader, CardPanel, CardTitle } from "#/components/ui/card";
import { Popover, PopoverPopup, PopoverTrigger } from "#/components/ui/popover";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import { hpsOperatorsQueryOptions, type IHpsOperatorListEntry } from "#/lib/api/hps";
import { BuffsPanel } from "./impl/components/BuffsPanel";
import { X_AXIS_INPUT, X_AXIS_LABELS, X_AXIS_SHORT, Y_METRIC_HINTS, Y_METRIC_LABELS } from "./impl/constants";
import type { HpsXAxis, HpsYMetric } from "./impl/types";
import { useHpsCurves } from "./impl/useHpsCurves";
import { useHpsResults } from "./impl/useHpsResults";
import { useHpsState } from "./impl/useHpsState";

const AXES = (Object.keys(X_AXIS_LABELS) as HpsXAxis[]).map((k) => ({ value: k, label: X_AXIS_LABELS[k], short: X_AXIS_SHORT[k] }));
const METRICS = (Object.keys(Y_METRIC_LABELS) as HpsYMetric[]).map((k) => ({ value: k, label: Y_METRIC_LABELS[k] }));
const COLUMNS = [
    { key: "skill_hps", label: "Skill HPS" },
    { key: "base_hps", label: "Base HPS", hint: (d: Record<string, number>) => (d.base_hps === 0 ? "Burst healer - heals only on skill activation, so there's no off-skill (base) healing." : undefined) },
    { key: "avg_hps", label: "Avg HPS" },
];

export function HpsCalculator(): React.ReactElement {
    const { state, dispatch, hydrationToken } = useHpsState();

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
            const moduleSummary = inst.config.moduleIndex > 0 ? `Mod${inst.config.moduleIndex} L${inst.config.moduleLevel}` : "no module";
            return { color: inst.color, label: `${inst.op.name}${dupSuffix}`, sublabel: `S${inst.config.skillIndex} · ${moduleSummary}` };
        });

        const buffsDesc = `ATK +${atkPct}% · ASPD +${state.buffs.buffs.aspd ?? 0} · ${state.buffs.targets} target${state.buffs.targets === 1 ? "" : "s"}`;
        const title = `${Y_METRIC_LABELS[state.yMetric]} vs ${X_AXIS_LABELS[state.xAxis]} - ${buffsDesc}`;

        const snapshotEntries = visibleInstances
            .map((inst, idx) => {
                const dupSuffix = (sameOpCounts.get(inst.op.id) ?? 1) > 1 ? ` #${idx + 1}` : "";
                const value = snapshots.find((s) => s.uid === inst.uid)?.data?.[state.yMetric];
                if (typeof value !== "number") return null;
                return { color: inst.color, name: `${inst.op.name}${dupSuffix}`, value: formatLargeNumber(value) };
            })
            .filter((e): e is { color: string; name: string; value: string } => e !== null);

        const snapshotHeading = state.xAxis === "targets" ? `@ ${state.buffs.targets} target${state.buffs.targets === 1 ? "" : "s"}` : state.xAxis === "atk" ? `@ ATK +${atkPct}%` : `@ ASPD +${state.buffs.buffs.aspd ?? 0}`;
        const snapshot = snapshotEntries.length > 0 ? { heading: snapshotHeading, entries: snapshotEntries } : undefined;

        setIsExporting(true);
        try {
            await exportSvgAsPng(svg, { filename, title, legend, snapshot });
        } catch (err) {
            console.error("HPS chart export failed:", err);
            if (typeof window !== "undefined") window.alert(`Couldn't export chart: ${err instanceof Error ? err.message : String(err)}`);
        } finally {
            setIsExporting(false);
        }
    }, [state.xAxis, state.yMetric, state.instances, state.buffs, atkPct, snapshots]);

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
                <span>Tools</span>
                <ChevronRight className="size-2.5" />
                <span className="text-foreground">HPS Calculator</span>
            </nav>
            <div className="flex flex-wrap items-end justify-between gap-3">
                <div className="min-w-0 flex-1">
                    <div className="flex items-center gap-2">
                        <h1 className="m-0 font-bold font-sans text-[24px] text-foreground leading-[1.1] tracking-tight sm:text-[30px]">HPS Calculator</h1>
                        <Popover>
                            <PopoverTrigger
                                render={(p) => (
                                    <Button {...p} aria-label="How does this work?" variant="ghost" size="icon-sm" className="text-muted-foreground hover:text-foreground">
                                        <HelpCircle />
                                    </Button>
                                )}
                            />
                            <PopoverPopup className="w-[min(380px,calc(100vw-2rem))]">
                                <h2 className="mb-2 font-semibold text-[14px] text-foreground">How this works</h2>
                                <ul className="space-y-1.5 text-[12.5px] text-muted-foreground leading-relaxed">
                                    <li>
                                        <span className="font-medium text-foreground">Skill HPS</span> is healing per second while the skill is active.
                                    </li>
                                    <li>
                                        <span className="font-medium text-foreground">Base HPS</span> is always-on healing during SP recharge - it's 0 for burst healers that only heal on activation.
                                    </li>
                                    <li>
                                        <span className="font-medium text-foreground">Average HPS</span> averages skill uptime against SP recharge - the long-run sustainable rate.
                                    </li>
                                    <li>Healing ignores enemy DEF/RES, so the chart sweeps a team variable instead: target count, ATK buff, or ASPD. Single-target vs. AoE healers diverge most along the Targets axis.</li>
                                </ul>
                            </PopoverPopup>
                        </Popover>
                    </div>
                    <p className="mt-1.5 font-sans text-[13.5px] text-muted-foreground leading-normal">Compare healer output across target counts and team buffs. Add multiple instances of the same operator to compare masteries, modules, and conditional setups side-by-side.</p>
                    <p className="max-w-xl font-sans text-[13.5px] text-muted-foreground leading-normal">
                        <b>Note:</b> All calculations go to the credit of{" "}
                        <a className="text-blue-500 hover:underline" href="https://github.com/WhoAteMyCQQkie/ArknightsDpsCompare" target="_blank" rel="noopener">
                            WhoAteMyCQQkie's
                        </a>{" "}
                        GitHub repository.
                    </p>
                </div>
                {hasInstances && (
                    <AlertDialog>
                        <AlertDialogTrigger
                            render={(p) => (
                                <Button {...p} variant="outline" size="sm">
                                    <RefreshCw />
                                    Clear all
                                </Button>
                            )}
                        />
                        <AlertDialogPopup>
                            <AlertDialogHeader>
                                <AlertDialogTitle>Remove all healers?</AlertDialogTitle>
                                <AlertDialogDescription>
                                    This removes all {state.instances.length} configured healer{state.instances.length === 1 ? "" : "s"} from the calculator. Buff and chart settings stay.
                                </AlertDialogDescription>
                            </AlertDialogHeader>
                            <AlertDialogFooter>
                                <AlertDialogClose render={(p) => <Button {...p} variant="outline" />}>Cancel</AlertDialogClose>
                                <AlertDialogClose render={(p) => <Button {...p} variant="destructive" onClick={onResetAll} />}>Clear all</AlertDialogClose>
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
                                Healers
                                {hasInstances && <span className="ml-1.5 font-medium font-mono text-[11px] text-muted-foreground">{visibleCount === state.instances.length ? `(${state.instances.length})` : `(${visibleCount}/${state.instances.length} visible)`}</span>}
                            </CardTitle>
                            {state.instances.length > 1 && (
                                <Tooltip>
                                    <TooltipTrigger
                                        render={(p) => (
                                            <Button {...p} aria-label={allCollapsed ? "Expand all healer cards" : "Collapse all healer cards"} variant="ghost" size="icon-sm" className="shrink-0" onClick={onToggleCollapseAll}>
                                                {allCollapsed ? <ChevronsUpDown /> : <ChevronsDownUp />}
                                            </Button>
                                        )}
                                    />
                                    <TooltipPopup>{allCollapsed ? "Expand all" : "Collapse all"}</TooltipPopup>
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
                                axes={AXES}
                                xAxis={state.xAxis}
                                onChangeAxis={(axis) => dispatch({ type: "SET_AXIS", axis: axis as HpsXAxis })}
                                metrics={METRICS}
                                yMetric={state.yMetric}
                                onChangeMetric={(metric) => dispatch({ type: "SET_METRIC", metric: metric as HpsYMetric })}
                                sweep={state.sweep[state.xAxis]}
                                axisInput={X_AXIS_INPUT[state.xAxis]}
                                pointCount={pointCount}
                                hydrationToken={hydrationToken}
                                metricHint={Y_METRIC_HINTS[state.yMetric]}
                                onChangeSweep={(patch) => dispatch({ type: "SET_SWEEP", axis: state.xAxis, patch })}
                                rangeAction={
                                    hasInstances ? (
                                        <Tooltip>
                                            <TooltipTrigger
                                                render={(p) => (
                                                    <Button {...p} aria-label="Download chart as PNG" variant="outline" size="icon-sm" loading={isExporting} onClick={onExportChart} className="mb-1">
                                                        <Download />
                                                    </Button>
                                                )}
                                            />
                                            <TooltipPopup>Download chart as PNG</TooltipPopup>
                                        </Tooltip>
                                    ) : undefined
                                }
                            />
                        </CardHeader>
                        <CardPanel className="pt-0">
                            <CalcChart
                                instances={state.instances}
                                rows={curves.rows}
                                xLabel={X_AXIS_LABELS[state.xAxis]}
                                yLabel={Y_METRIC_LABELS[state.yMetric]}
                                allowDecimals={state.xAxis !== "targets"}
                                formatTooltipX={(x) => (state.xAxis === "targets" ? `${x} target${x === 1 ? "" : "s"}` : state.xAxis === "atk" ? `ATK +${x}%` : `ASPD +${x}`)}
                                snapshotX={snapshotX}
                                isLoading={curves.isPending}
                                onLegendClick={(uid) => dispatch({ type: "TOGGLE_VISIBILITY", uid })}
                                containerRef={chartContainerRef}
                                emptyIcon={<HeartPulse />}
                                emptyTitle="No healers yet"
                                emptyDescription={
                                    <>
                                        Pick any healer from the picker to plot an HPS curve. The chart will show how healing scales as you sweep across <span className="font-medium text-foreground">{X_AXIS_LABELS[state.xAxis]}</span>.
                                    </>
                                }
                            />
                        </CardPanel>
                    </Card>

                    <KpiPanel instances={state.instances} snapshots={snapshots} leaderKey={state.yMetric} leaderLabel={Y_METRIC_LABELS[state.yMetric]} columns={COLUMNS} />
                </main>

                <section aria-label="Configured healers" className="flex min-w-0 flex-col gap-4 xl:col-start-1 xl:row-start-2">
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
