import { useQuery } from "@tanstack/react-query";
import { ChevronRight, ChevronsDownUp, ChevronsUpDown, HelpCircle, RefreshCw } from "lucide-react";
import * as React from "react";
import { AlertDialog, AlertDialogClose, AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogPopup, AlertDialogTitle, AlertDialogTrigger } from "#/components/ui/alert-dialog";
import { Button } from "#/components/ui/button";
import { Card, CardHeader, CardPanel, CardTitle } from "#/components/ui/card";
import { Popover, PopoverPopup, PopoverTrigger } from "#/components/ui/popover";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import { dpsOperatorsQueryOptions } from "#/lib/api/dps";
import { AxisControls } from "./impl/components/AxisControls";
import { DpsChart } from "./impl/components/DpsChart";
import { EnemyPanel } from "./impl/components/EnemyPanel";
import { InstanceCard } from "./impl/components/InstanceCard";
import { KpiPanel } from "./impl/components/KpiPanel";
import { OperatorPicker } from "./impl/components/OperatorPicker";
import { useDpsCurves } from "./impl/useDpsCurves";
import { useDpsResults } from "./impl/useDpsResults";
import { useDpsState } from "./impl/useDpsState";

export function DpsCalculator(): React.ReactElement {
    const { state, dispatch, hydrationToken } = useDpsState();

    const snapshots = useDpsResults(state.instances, state.enemy);
    const curves = useDpsCurves(state, state.enemy);

    const hasInstances = state.instances.length > 0;
    const allCollapsed = hasInstances && state.instances.every((i) => i.collapsed);
    const visibleCount = state.instances.filter((i) => i.visible).length;

    const onResetAll = React.useCallback(() => {
        dispatch({ type: "RESET_INSTANCES" });
    }, [dispatch]);

    const onToggleCollapseAll = React.useCallback(() => {
        dispatch({ type: "SET_ALL_COLLAPSED", collapsed: !allCollapsed });
    }, [allCollapsed, dispatch]);

    // After hydrating from localStorage, swap each instance's `op` reference
    // for the latest entry from the engine — covers cases where the engine
    // adds modules / conditionals to an operator since the user's last visit.
    const { data: latestOps } = useQuery(dpsOperatorsQueryOptions());
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
        <div className="relative z-1 mx-auto w-[min(1400px,calc(100%-2rem))] pb-20">
            <nav aria-label="breadcrumb" className="mb-2.5 flex items-center gap-1.5 font-medium font-sans text-[12px] leading-none text-muted-foreground">
                <span>Tools</span>
                <ChevronRight className="size-2.5" />
                <span className="text-foreground">DPS Calculator</span>
            </nav>
            <div className="flex flex-wrap items-end justify-between gap-3">
                <div className="min-w-0 flex-1">
                    <div className="flex items-center gap-2">
                        <h1 className="m-0 font-bold font-sans text-[24px] leading-[1.1] tracking-tight text-foreground sm:text-[30px]">DPS Calculator</h1>
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
                                <ul className="space-y-1.5 text-[12.5px] leading-relaxed text-muted-foreground">
                                    <li>
                                        <span className="font-medium text-foreground">Skill DPS</span> is damage per second while the skill is active.
                                    </li>
                                    <li>
                                        <span className="font-medium text-foreground">Average DPS</span> averages skill uptime against SP recharge — the long-run sustainable rate.
                                    </li>
                                    <li>
                                        <span className="font-medium text-foreground">Total Damage</span> is skill DPS × duration (or just skill DPS for passive skills).
                                    </li>
                                    <li>The chart sweeps the chosen X axis (DEF or RES). Whichever axis isn't being swept is held at the value you set in the Enemy panel.</li>
                                </ul>
                            </PopoverPopup>
                        </Popover>
                    </div>
                    <p className="mt-1.5 font-sans text-[13.5px] leading-normal text-muted-foreground">Compare operator output across enemy DEF or RES sweeps. Add multiple instances of the same operator to compare masteries, modules, and conditional setups side-by-side.</p>
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
                                <AlertDialogTitle>Remove all operators?</AlertDialogTitle>
                                <AlertDialogDescription>
                                    This removes all {state.instances.length} configured operator{state.instances.length === 1 ? "" : "s"} from the calculator. Enemy and chart settings stay.
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
                        <CardHeader className="flex flex-row items-center justify-between gap-2 grid-rows-1">
                            <CardTitle className="min-w-0 flex-1 truncate text-[15px]">
                                Operators
                                {hasInstances && <span className="ml-1.5 font-mono text-[11px] font-medium text-muted-foreground">{visibleCount === state.instances.length ? `(${state.instances.length})` : `(${visibleCount}/${state.instances.length} visible)`}</span>}
                            </CardTitle>
                            {state.instances.length > 1 && (
                                <Tooltip>
                                    <TooltipTrigger
                                        render={(p) => (
                                            <Button {...p} aria-label={allCollapsed ? "Expand all operator cards" : "Collapse all operator cards"} variant="ghost" size="icon-sm" className="shrink-0" onClick={onToggleCollapseAll}>
                                                {allCollapsed ? <ChevronsUpDown /> : <ChevronsDownUp />}
                                            </Button>
                                        )}
                                    />
                                    <TooltipPopup>{allCollapsed ? "Expand all" : "Collapse all"}</TooltipPopup>
                                </Tooltip>
                            )}
                        </CardHeader>
                        <CardPanel className="pt-0">
                            <OperatorPicker existingCount={state.instances.length} onAdd={(op) => dispatch({ type: "ADD_INSTANCE", op })} />
                        </CardPanel>
                    </Card>

                    <EnemyPanel hydrationToken={hydrationToken} enemy={state.enemy} onChangeEnemy={(patch) => dispatch({ type: "SET_ENEMY", patch })} onChangeShred={(patch) => dispatch({ type: "SET_SHRED", patch })} />
                </aside>

                <main className="flex min-w-0 flex-col gap-4 xl:col-start-2 xl:row-span-2 xl:row-start-1 xl:sticky xl:top-20 xl:max-h-[calc(100vh-6rem)] xl:overflow-y-auto xl:pr-1">
                    <Card>
                        <CardHeader className="pb-3">
                            <AxisControls
                                xAxis={state.xAxis}
                                yMetric={state.yMetric}
                                sweep={state.sweep[state.xAxis]}
                                hydrationToken={hydrationToken}
                                onChangeAxis={(axis) => dispatch({ type: "SET_AXIS", axis })}
                                onChangeMetric={(metric) => dispatch({ type: "SET_METRIC", metric })}
                                onChangeSweep={(patch) => dispatch({ type: "SET_SWEEP", axis: state.xAxis, patch })}
                            />
                        </CardHeader>
                        <CardPanel className="pt-0">
                            <DpsChart instances={state.instances} rows={curves.rows} xAxis={state.xAxis} yMetric={state.yMetric} snapshotX={state.xAxis === "defense" ? state.enemy.defense : state.enemy.res} isLoading={curves.isPending} onLegendClick={(uid) => dispatch({ type: "TOGGLE_VISIBILITY", uid })} />
                        </CardPanel>
                    </Card>

                    <KpiPanel instances={state.instances} snapshots={snapshots} yMetric={state.yMetric} />
                </main>

                <section aria-label="Configured operators" className="flex min-w-0 flex-col gap-4 xl:col-start-1 xl:row-start-2">
                    {state.instances.map((inst, idx) => (
                        <InstanceCard
                            key={inst.uid}
                            inst={inst}
                            index={idx}
                            isFirst={idx === 0}
                            isLast={idx === state.instances.length - 1}
                            onUpdate={(patch) => dispatch({ type: "UPDATE_CONFIG", uid: inst.uid, patch })}
                            onUpdateBuffs={(patch) => dispatch({ type: "UPDATE_BUFFS", uid: inst.uid, patch })}
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
