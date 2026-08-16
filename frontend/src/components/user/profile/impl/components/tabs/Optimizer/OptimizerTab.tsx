import { RotateCcw, Sparkles } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { Button } from "#/components/ui/button";
import { Skeleton } from "#/components/ui/skeleton";
import { Tabs, TabsList, TabsPanel, TabsTab } from "#/components/ui/tabs";
import type { IRosterEntry } from "#/lib/api/user";
import type { IOperatorListItem } from "#/types/operators";
import { AssignPicker } from "./impl/components/AssignPicker";
import { BaseBoard } from "./impl/components/BaseBoard";
import { OptimizeDialog } from "./impl/components/OptimizeDialog";
import { ProposalReview } from "./impl/components/ProposalReview";
import { ReportPanel } from "./impl/components/ReportPanel";
import { RoomInspector } from "./impl/components/RoomInspector";
import { ShiftBar } from "./impl/components/ShiftBar";
import { toRosterOptions } from "./impl/roster";
import { useOptimizer } from "./impl/use-optimizer";

interface OptimizerTabProps {
    uid: string;
    roster: IRosterEntry[];
    operatorsStatic: IOperatorListItem[];
}

/**
 * The interactive base planner.
 *
 * Opens on the player's real stationed base, lets them move operators around,
 * and asks the backend what each arrangement is worth. Every figure on screen
 * comes from the same clause engine as the read-only plan in the Score tab.
 */
export function OptimizerTab({ uid, roster, operatorsStatic }: OptimizerTabProps) {
    const api = useOptimizer(uid);
    const rosterOptions = useMemo(() => toRosterOptions(roster, operatorsStatic), [roster, operatorsStatic]);

    const [assignOpen, setAssignOpen] = useState(false);
    const [optimizeOpen, setOptimizeOpen] = useState(false);
    const [sideTab, setSideTab] = useState("inspector");

    // A fresh proposal is the thing the player just asked for - show it.
    useEffect(() => {
        if (api.proposal) setSideTab("proposal");
    }, [api.proposal]);

    const proposedSlotIds = useMemo(() => new Set(api.proposal?.room_diffs.map((d) => d.slot_id) ?? []), [api.proposal]);

    if (api.layoutLoading) {
        return (
            <div className="flex flex-col gap-3">
                <Skeleton className="h-10 w-full" />
                <Skeleton className="h-64 w-full" />
            </div>
        );
    }

    if (api.layout.length === 0) {
        return (
            <div className="rounded-xl border border-border border-dashed p-8 text-center">
                <p className="text-muted-foreground text-sm">This profile has no base data to plan against yet.</p>
            </div>
        );
    }

    return (
        <div className="flex flex-col gap-4">
            <header className="flex flex-wrap items-end justify-between gap-3">
                <div>
                    <p className="font-mono text-primary text-xs uppercase tracking-widest">RIIC · Infrastructure</p>
                    <h2 className="font-heading font-semibold text-foreground text-xl">Base Optimizer</h2>
                    <p className="mt-1 max-w-2xl text-pretty text-muted-foreground text-sm">Move operators between rooms and see what each arrangement is actually worth. The solver accounts for cross-room buffs, so review its proposals room by room before applying them.</p>
                </div>
                <div className="flex flex-wrap items-center gap-2">
                    {api.evaluating ? <span className="text-muted-foreground text-xs">Scoring…</span> : null}
                    <Button variant="outline" onClick={api.reset} disabled={!api.dirty}>
                        <RotateCcw className="h-4 w-4" /> Reset to my base
                    </Button>
                    <Button onClick={() => setOptimizeOpen(true)} disabled={api.optimizing}>
                        <Sparkles className="h-4 w-4" /> {api.optimizing ? "Solving…" : "Optimize"}
                    </Button>
                </div>
            </header>

            {api.evaluationError ? <ErrorNote message={api.evaluationError.message} /> : null}
            {api.optimizeError ? <ErrorNote message={api.optimizeError.message} /> : null}
            {api.rotationError ? <ErrorNote message={api.rotationError.message} /> : null}

            <div className="grid gap-4 lg:grid-cols-[minmax(0,1fr)_380px]">
                <div className="flex min-w-0 flex-col gap-4">
                    <ShiftBar api={api} />
                    <BaseBoard api={api} proposedSlotIds={proposedSlotIds} />
                    <ReportPanel api={api} />
                </div>

                <aside className="lg:sticky lg:top-4 lg:h-[calc(100vh-2rem)]">
                    <Tabs value={sideTab} onValueChange={setSideTab} className="flex h-full flex-col">
                        <TabsList>
                            <TabsTab value="inspector">Inspector</TabsTab>
                            <TabsTab value="proposal" disabled={!api.proposal}>
                                Proposal{api.proposal ? ` (${api.proposal.room_diffs.length})` : ""}
                            </TabsTab>
                        </TabsList>
                        <div className="min-h-0 flex-1 overflow-hidden rounded-xl border border-border bg-card p-4">
                            <TabsPanel value="inspector" className="h-full">
                                <RoomInspector api={api} onOpenAssign={() => setAssignOpen(true)} />
                            </TabsPanel>
                            <TabsPanel value="proposal" className="h-full">
                                <ProposalReview api={api} roster={rosterOptions} />
                            </TabsPanel>
                        </div>
                    </Tabs>
                </aside>
            </div>

            <AssignPicker api={api} roster={rosterOptions} open={assignOpen} onOpenChange={setAssignOpen} />
            <OptimizeDialog api={api} open={optimizeOpen} onOpenChange={setOptimizeOpen} />
        </div>
    );
}

function ErrorNote({ message }: { message: string }) {
    return <p className="rounded-md border border-destructive/40 bg-destructive/10 p-2 text-destructive text-xs">{message}</p>;
}
