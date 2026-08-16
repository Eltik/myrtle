import { CircleCheck, TriangleAlert } from "lucide-react";
import { Badge } from "#/components/ui/badge";
import { cn } from "#/lib/utils";
import type { OptimizerApi } from "../use-optimizer";

/**
 * The shift selector that sits above the board.
 *
 * A rotation is read one shift at a time - "who works this room right now" -
 * so the board itself switches between shifts rather than duplicating the whole
 * base in a side table. `Draft` is the layout being edited; each numbered shift
 * is the rotation's staffing for it, read-only.
 *
 * Shift count comes from the solver, so nothing here assumes three.
 */
export function ShiftBar({ api }: { api: OptimizerApi }) {
    const shiftCount = api.rotation?.shift_count ?? 0;
    const shiftIndices = Array.from({ length: shiftCount }, (_, i) => i + 1);
    const sustainability = api.rotation?.rotation.sustainability;
    const sustained = api.rotation?.rotation.sustained ?? [];

    return (
        <div className="flex flex-col gap-2">
            <div className="flex flex-wrap items-center justify-between gap-2">
                <div className="inline-flex rounded-lg border border-border bg-card p-0.5">
                    <ShiftButton label="Draft" active={api.viewShift == null} onClick={() => api.setViewShift(null)} />
                    {shiftIndices.map((index) => (
                        <ShiftButton key={index} label={`Shift ${index}`} active={api.viewShift === index} onClick={() => api.setViewShift(index)} />
                    ))}
                    {shiftCount === 0 ? <span className="px-3 py-1.5 text-muted-foreground text-xs">{api.rotating ? "Planning rotation…" : "Optimize to plan shifts"}</span> : null}
                </div>

                {sustainability ? <SustainabilityBadge verdict={sustainability.verdict} depletedCount={sustainability.depleted.length} dormOverflow={sustainability.dorm_overflow} horizonHours={sustainability.horizon_hours} /> : null}
            </div>

            {api.viewShift != null ? (
                <p className="text-[11px] text-muted-foreground">
                    Showing the rotation's crews for shift {api.viewShift} — read-only. Switch to <span className="font-medium text-foreground">Draft</span> to edit.
                </p>
            ) : null}

            {api.viewShift == null && sustained.length > 0 ? (
                <p className="text-[11px] text-muted-foreground">
                    <span className="font-medium text-foreground">{sustained.map((o) => o.name).join(", ")}</span> run 24/7 via a morale-swap manager — they never take the rest shift.
                </p>
            ) : null}
        </div>
    );
}

function ShiftButton({ label, active, onClick }: { label: string; active: boolean; onClick: () => void }) {
    return (
        <button type="button" onClick={onClick} aria-pressed={active} className={cn("rounded-md px-3 py-1.5 font-medium text-sm transition-colors", active ? "bg-primary text-primary-foreground" : "text-muted-foreground hover:text-foreground")}>
            {label}
        </button>
    );
}

function SustainabilityBadge({ verdict, depletedCount, dormOverflow, horizonHours }: { verdict: string; depletedCount: number; dormOverflow: number; horizonHours: number }) {
    const holds = verdict === "holds_up";
    const days = Math.round(horizonHours / 24);

    const detail = holds ? `Nobody runs dry over ${days} simulated day${days === 1 ? "" : "s"}.` : `${depletedCount} operator${depletedCount === 1 ? "" : "s"} hit zero morale within ${days} day${days === 1 ? "" : "s"}.`;

    return (
        <div className="flex items-center gap-2">
            <Badge variant={holds ? "secondary" : "destructive"} className="gap-1">
                {holds ? <CircleCheck className="h-3 w-3" /> : <TriangleAlert className="h-3 w-3" />}
                {holds ? "Holds up" : "Depletes"}
            </Badge>
            <p className="text-[11px] text-muted-foreground">
                {detail}
                {dormOverflow > 0 ? ` Dorms are ${dormOverflow} bed${dormOverflow === 1 ? "" : "s"} short at peak.` : ""}
            </p>
        </div>
    );
}
