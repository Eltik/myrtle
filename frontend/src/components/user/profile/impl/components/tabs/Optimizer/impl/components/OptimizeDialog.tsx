import { Sparkles } from "lucide-react";
import { useMemo, useState } from "react";
import { Button } from "#/components/ui/button";
import { Dialog, DialogContent, DialogFooter, DialogHeader, DialogTitle } from "#/components/ui/dialog";
import type { RoomType } from "#/lib/api/base";
import { cn } from "#/lib/utils";
import { BOARD_BANDS, roomLabel } from "../layout";
import type { OptimizerApi } from "../use-optimizer";

interface OptimizeDialogProps {
    api: OptimizerApi;
    open: boolean;
    onOpenChange: (open: boolean) => void;
}

/**
 * Choose what the optimizer is allowed to touch.
 *
 * Scoping matters because cross-room buffs mean a "just fix my factories" run
 * still has to *see* the whole base. Rooms left out of the scope keep their
 * crews and stay in the scoring - they are frozen, not ignored.
 */
export function OptimizeDialog({ api, open, onOpenChange }: OptimizeDialogProps) {
    const presentTypes = useMemo(() => {
        const seen = new Set<RoomType>();
        for (const room of api.layout) seen.add(room.room_type);
        return BOARD_BANDS.flatMap((b) => b.types).filter((t) => seen.has(t));
    }, [api.layout]);

    const [selectedTypes, setSelectedTypes] = useState<Set<RoomType>>(new Set());

    const scopeSlotIds = useMemo(() => {
        if (selectedTypes.size === 0) return [];
        return api.layout.filter((r) => selectedTypes.has(r.room_type)).map((r) => r.slot_id);
    }, [api.layout, selectedTypes]);

    const wholeBase = selectedTypes.size === 0;

    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogContent className="max-w-md">
                <DialogHeader>
                    <DialogTitle>Optimize</DialogTitle>
                </DialogHeader>

                <p className="text-[11px] text-muted-foreground">Solves the best peak staffing and plans a shift rotation for it, so you can see both what the layout is worth and whether it survives a week.</p>

                <div className="flex flex-col gap-4">
                    <section className="flex flex-col gap-2">
                        <h4 className="font-semibold text-muted-foreground text-xs uppercase tracking-wide">Rooms to restaff</h4>
                        <div className="flex flex-wrap gap-1.5">
                            {presentTypes.map((type) => {
                                const on = selectedTypes.has(type);
                                return (
                                    <button
                                        key={type}
                                        type="button"
                                        aria-pressed={on}
                                        onClick={() =>
                                            setSelectedTypes((prev) => {
                                                const next = new Set(prev);
                                                if (next.has(type)) next.delete(type);
                                                else next.add(type);
                                                return next;
                                            })
                                        }
                                        className={cn("rounded-md border px-2 py-1 text-xs transition-colors", on ? "border-primary bg-primary text-primary-foreground" : "border-border text-muted-foreground hover:text-foreground")}
                                    >
                                        {roomLabel(type, api.catalog)}
                                    </button>
                                );
                            })}
                        </div>
                        <p className="text-[11px] text-muted-foreground">{wholeBase ? "Nothing selected — the whole base is up for restaffing." : `${scopeSlotIds.length} room${scopeSlotIds.length === 1 ? "" : "s"} in scope. The rest keep their crews and still count toward cross-room buffs.`}</p>
                    </section>

                    {api.lockedIds.length > 0 ? (
                        <section className="rounded-md border border-border bg-background p-2">
                            <p className="text-[11px] text-muted-foreground">
                                <span className="font-semibold text-foreground">{api.lockedIds.length}</span> pinned operator
                                {api.lockedIds.length === 1 ? "" : "s"} will stay exactly where you put them.
                            </p>
                        </section>
                    ) : null}

                    {api.excludedIds.length > 0 ? (
                        <section className="rounded-md border border-border bg-background p-2">
                            <p className="text-[11px] text-muted-foreground">
                                <span className="font-semibold text-foreground">{api.excludedIds.length}</span> operator
                                {api.excludedIds.length === 1 ? "" : "s"} excluded — the solver will not seat them anywhere.
                            </p>
                        </section>
                    ) : null}
                </div>

                <DialogFooter>
                    <Button variant="outline" onClick={() => onOpenChange(false)}>
                        Cancel
                    </Button>
                    <Button
                        disabled={api.optimizing}
                        onClick={() => {
                            api.runOptimize(scopeSlotIds);
                            onOpenChange(false);
                        }}
                    >
                        <Sparkles className="h-4 w-4" />
                        {api.optimizing ? "Solving…" : "Run"}
                    </Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    );
}
