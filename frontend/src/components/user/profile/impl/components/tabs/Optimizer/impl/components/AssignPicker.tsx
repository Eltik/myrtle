import { useMemo, useState } from "react";
import { Badge } from "#/components/ui/badge";
import { Dialog, DialogContent, DialogHeader, DialogTitle } from "#/components/ui/dialog";
import { Input } from "#/components/ui/input";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { searchAndRank } from "#/lib/search/fuzzy";
import { roomLabel } from "../layout";
import type { RosterOption } from "../roster";
import type { OptimizerApi } from "../use-optimizer";

interface AssignPickerProps {
    api: OptimizerApi;
    /** The profile's owned operators. */
    roster: RosterOption[];
    open: boolean;
    onOpenChange: (open: boolean) => void;
}

/** Enough to fill the scroll area without rendering a 700-row list. */
const MAX_RESULTS = 60;

/**
 * Pick an operator for the selected room.
 *
 * The candidate list is the player's real roster - hundreds of operators - so
 * it is searched, not scrolled. It deliberately does not rank candidates by
 * predicted gain: that ranking is the optimizer's job, and guessing at it here
 * would mean scoring on the client.
 */
export function AssignPicker({ api, roster, open, onOpenChange }: AssignPickerProps) {
    const [query, setQuery] = useState("");
    const room = api.selectedRoom;

    const candidates = useMemo(() => {
        const trimmed = query.trim();
        if (trimmed.length === 0) return roster.slice(0, MAX_RESULTS);
        return searchAndRank(trimmed, roster, (op) => ({ name: op.name }), MAX_RESULTS).map((r) => r.item);
    }, [roster, query]);

    if (!room) return null;

    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogContent className="max-w-lg">
                <DialogHeader>
                    <DialogTitle>Assign to {roomLabel(room.room_type, api.catalog)}</DialogTitle>
                </DialogHeader>

                <Input autoFocus placeholder="Search operators…" value={query} onChange={(e) => setQuery(e.target.value)} />

                <ul className="max-h-[50vh] overflow-y-auto">
                    {candidates.map((op) => {
                        const currentSlot = api.slotOf(op.id);
                        const excluded = api.excludedIds.includes(op.id);
                        return (
                            <li key={op.id}>
                                <button
                                    type="button"
                                    className="flex w-full items-center gap-2 rounded-md p-2 text-left transition-colors hover:bg-muted"
                                    onClick={() => {
                                        api.assign(room.slot_id, op.id);
                                        onOpenChange(false);
                                    }}
                                >
                                    <span className="grid h-8 w-8 shrink-0 place-items-center overflow-hidden rounded-sm bg-muted">
                                        <OperatorAvatar charId={op.id} name={op.name} />
                                    </span>
                                    <span className="min-w-0 flex-1 truncate text-foreground text-sm">{op.name}</span>
                                    {excluded ? <Badge variant="outline">excluded</Badge> : null}
                                    {currentSlot && currentSlot !== room.slot_id ? (
                                        <Badge variant="secondary" className="font-mono text-[10px]">
                                            in {currentSlot}
                                        </Badge>
                                    ) : null}
                                </button>
                            </li>
                        );
                    })}
                    {candidates.length === 0 ? <li className="p-4 text-center text-muted-foreground text-sm">No operators match.</li> : null}
                </ul>
            </DialogContent>
        </Dialog>
    );
}
