import { AlertTriangle, Crown } from "lucide-react";
import type * as React from "react";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { Spinner } from "#/components/ui/spinner";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import type { IDpsCalculateResponse } from "#/lib/api/dps";
import { formatLargeNumber, Y_METRIC_LABELS } from "../constants";
import type { IDpsInstance, YMetric } from "../types";
import type { IInstanceSnapshot } from "../useDpsResults";
import { useOperatorDetail } from "../useOperatorDetail";

interface IKpiPanelProps {
    instances: IDpsInstance[];
    snapshots: IInstanceSnapshot[];
    yMetric: YMetric;
}

export function KpiPanel({ instances, snapshots, yMetric }: IKpiPanelProps): React.ReactElement | null {
    if (instances.length === 0) return null;

    const leaderUid = pickLeader(snapshots, yMetric);
    const leaderValue = leaderUid ? snapshots.find((s) => s.uid === leaderUid)?.data?.[yMetric] : undefined;

    return (
        <div className="space-y-2" aria-live="polite">
            <div className="flex items-center justify-between">
                <h2 className="font-semibold text-[14px] text-foreground">Snapshot</h2>
                <span className="text-[11px] text-muted-foreground">
                    Leader by <span className="text-foreground">{Y_METRIC_LABELS[yMetric]}</span>
                </span>
            </div>
            <div className="space-y-2">
                {instances.map((inst, idx) => {
                    const snap = snapshots.find((s) => s.uid === inst.uid);
                    const isLeader = inst.uid === leaderUid;
                    return <KpiRow key={inst.uid} inst={inst} snap={snap} index={idx} isLeader={isLeader} leaderValue={leaderValue} yMetric={yMetric} sameOpCount={instances.filter((i) => i.op.id === inst.op.id).length} />;
                })}
            </div>
        </div>
    );
}

interface IKpiRowProps {
    inst: IDpsInstance;
    snap: IInstanceSnapshot | undefined;
    index: number;
    isLeader: boolean;
    leaderValue: number | undefined;
    yMetric: YMetric;
    sameOpCount: number;
}

function KpiRow({ inst, snap, index, isLeader, leaderValue, yMetric, sameOpCount }: IKpiRowProps): React.ReactElement {
    const data: IDpsCalculateResponse | undefined = snap?.data;
    const dim = !inst.visible;
    const detail = useOperatorDetail(inst.op.id);
    const skillSummary = detail.skillName(inst.config.skillIndex);
    const moduleSummary = inst.config.moduleIndex > 0 ? detail.moduleName(inst.config.moduleIndex) : "no module";
    const value = data ? data[yMetric] : undefined;
    const relative = !isLeader && typeof leaderValue === "number" && leaderValue > 0 && typeof value === "number" ? (value / leaderValue - 1) * 100 : null;
    return (
        <div className={`flex flex-col gap-2 rounded-lg border border-border bg-card px-3 py-2.5 transition-opacity sm:flex-row sm:items-center sm:gap-3 ${dim ? "opacity-50" : ""}`}>
            <div className="flex min-w-0 items-center gap-3">
                <span aria-hidden="true" className="relative inline-flex size-9 shrink-0 items-center justify-center overflow-hidden rounded-lg bg-muted font-semibold text-[10px]" style={{ boxShadow: `inset 0 0 0 2px ${inst.color}` }}>
                    <OperatorAvatar charId={inst.op.id} name={inst.op.name} />
                </span>
                <div className="min-w-0 flex-1">
                    <div className="flex items-center gap-1.5">
                        <span className="truncate font-semibold text-[13px]">
                            {inst.op.name}
                            {sameOpCount > 1 && <span className="ml-1 font-mono text-[10px] text-muted-foreground">#{index + 1}</span>}
                        </span>
                        {snap?.error ? (
                            <Tooltip>
                                <TooltipTrigger render={(p) => <AlertTriangle {...p} aria-label="Calculation failed" className="size-3.5 cursor-help text-destructive" />} />
                                <TooltipPopup className="max-w-64">{snap.error.message || "Calculation failed"}</TooltipPopup>
                            </Tooltip>
                        ) : isLeader ? (
                            <span className="inline-flex items-center gap-0.5 rounded bg-primary/12 px-1.5 py-0.5 font-medium text-[10px] text-primary" title="Leader">
                                <Crown className="size-3" />
                                top
                            </span>
                        ) : (
                            relative !== null && (
                                <span className="rounded bg-muted px-1.5 py-0.5 font-mono text-[10px] text-muted-foreground">
                                    {relative >= 0 ? "+" : ""}
                                    {relative.toFixed(1)}%
                                </span>
                            )
                        )}
                    </div>
                    <div className="mt-0.5 truncate text-[10.5px] text-muted-foreground">
                        {skillSummary} · M{inst.config.masteryLevel} · {moduleSummary}
                    </div>
                </div>
            </div>
            <div className="grid grid-cols-3 gap-x-3 sm:gap-x-4 sm:text-right">
                <Stat label="Skill DPS" value={data?.skill_dps} highlight={yMetric === "skill_dps" && isLeader} loading={!!snap?.isPending} />
                <Stat label="Avg DPS" value={data?.average_dps} highlight={yMetric === "average_dps" && isLeader} loading={!!snap?.isPending} />
                <Stat label="Total" value={data?.total_damage} highlight={yMetric === "total_damage" && isLeader} loading={!!snap?.isPending} />
            </div>
        </div>
    );
}

function Stat({ label, value, highlight, loading }: { label: string; value: number | undefined; highlight: boolean; loading: boolean }): React.ReactElement {
    return (
        <div className="min-w-14">
            <div className="font-medium text-[10px] text-muted-foreground uppercase tracking-wide">{label}</div>
            <div className={`flex items-center gap-1 font-mono text-[14px] tabular-nums sm:justify-end ${highlight ? "text-primary" : "text-foreground"}`}>{loading && value === undefined ? <Spinner className="size-3 text-muted-foreground" /> : formatLargeNumber(value)}</div>
        </div>
    );
}

function pickLeader(snapshots: IInstanceSnapshot[], metric: YMetric): string | null {
    let best: { uid: string; v: number } | null = null;
    for (const s of snapshots) {
        const v = s.data?.[metric];
        if (typeof v !== "number") continue;
        if (!best || v > best.v) best = { uid: s.uid, v };
    }
    return best?.uid ?? null;
}
