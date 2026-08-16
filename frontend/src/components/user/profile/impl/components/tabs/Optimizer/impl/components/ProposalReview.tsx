import { ArrowRight, Check } from "lucide-react";
import { Button } from "#/components/ui/button";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import type { IRoomDiff } from "#/lib/api/base";
import { cn } from "#/lib/utils";
import { roomLabel } from "../layout";
import type { RosterOption } from "../roster";
import type { OptimizerApi } from "../use-optimizer";

interface ProposalReviewProps {
    api: OptimizerApi;
    roster: RosterOption[];
}

/**
 * Room-by-room review of what the optimizer wants to change.
 *
 * Nothing is applied until the player says so, and each room can be taken on
 * its own - a plan you only half-agree with is still worth half of.
 */
export function ProposalReview({ api, roster }: ProposalReviewProps) {
    const proposal = api.proposal;
    if (!proposal) {
        return <p className="text-muted-foreground text-sm">Run the optimizer to see proposed changes.</p>;
    }

    if (proposal.room_diffs.length === 0) {
        return (
            <div className="flex flex-col gap-3">
                <p className="text-foreground text-sm">Your layout is already the best arrangement the solver found.</p>
                <Button variant="outline" size="sm" onClick={api.discardProposal}>
                    Dismiss
                </Button>
            </div>
        );
    }

    const nameById = new Map(roster.map((o) => [o.id, o.name]));
    const gain = proposal.proposal.yield_total_value - proposal.baseline.yield_total_value;

    return (
        <div className="flex h-full flex-col gap-3 overflow-hidden">
            <header className="flex items-center justify-between gap-2">
                <div>
                    <p className="text-muted-foreground text-xs">Projected daily value</p>
                    <p className={cn("font-mono font-semibold text-sm", gain >= 0 ? "text-emerald-500" : "text-destructive")}>
                        {gain >= 0 ? "+" : ""}
                        {Math.round(gain).toLocaleString()}
                    </p>
                </div>
                <div className="flex gap-1.5">
                    <Button variant="ghost" size="sm" onClick={api.discardProposal}>
                        Discard
                    </Button>
                    <Button size="sm" onClick={api.acceptAll}>
                        Accept all
                    </Button>
                </div>
            </header>

            <ul className="flex min-h-0 flex-1 flex-col gap-2 overflow-y-auto">
                {proposal.room_diffs.map((diff) => (
                    <DiffCard key={diff.slot_id} api={api} diff={diff} nameById={nameById} />
                ))}
            </ul>
        </div>
    );
}

function DiffCard({ api, diff, nameById }: { api: OptimizerApi; diff: IRoomDiff; nameById: Map<string, string> }) {
    const delta = diff.yield_after - diff.yield_before;

    return (
        <li className="rounded-md border border-border bg-background p-2">
            <div className="mb-2 flex items-center justify-between gap-2">
                <div className="min-w-0">
                    <p className="truncate font-semibold text-foreground text-xs">{roomLabel(diff.room_type, api.catalog)}</p>
                    <p className="font-mono text-[10px] text-muted-foreground">{diff.slot_id}</p>
                </div>
                <span className={cn("shrink-0 font-mono text-[11px]", delta >= 0 ? "text-emerald-500" : "text-destructive")}>
                    {delta >= 0 ? "+" : ""}
                    {Math.round(delta).toLocaleString()}/d
                </span>
            </div>

            <div className="flex items-center gap-2">
                <CrewRow ids={diff.before} nameById={nameById} muted />
                <ArrowRight className="h-3 w-3 shrink-0 text-muted-foreground" />
                <CrewRow ids={diff.after} nameById={nameById} />
            </div>

            <div className="mt-2 flex items-center justify-between gap-2">
                <span className="font-mono text-[10px] text-muted-foreground">
                    {Math.round(diff.efficiency_before)}% → {Math.round(diff.efficiency_after)}%
                </span>
                <div className="flex gap-1">
                    <Button variant="ghost" size="icon" aria-label="Apply this room" onClick={() => api.acceptRoom(diff.slot_id)}>
                        <Check className="h-3.5 w-3.5" />
                    </Button>
                </div>
            </div>
        </li>
    );
}

function CrewRow({ ids, nameById, muted = false }: { ids: string[]; nameById: Map<string, string>; muted?: boolean }) {
    if (ids.length === 0) {
        return <span className="flex-1 text-[10px] text-muted-foreground italic">empty</span>;
    }
    return (
        <div className={cn("flex flex-1 flex-wrap gap-1", muted && "opacity-50")}>
            {ids.map((id) => (
                <span key={id} className="grid h-6 w-6 place-items-center overflow-hidden rounded-sm bg-muted" title={nameById.get(id) ?? id}>
                    <OperatorAvatar charId={id} name={nameById.get(id) ?? id} />
                </span>
            ))}
        </div>
    );
}
