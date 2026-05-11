import { Check, Search, X } from "lucide-react";
import * as React from "react";
import { Button } from "#/components/ui/button";
import { Input } from "#/components/ui/input";
import { cn, getAvatarById } from "#/lib/utils";
import type { OperatorRarity } from "#/types/operators";
import type { IRandomizerOperator, IRosterIndex } from "../types";

interface IRosterPickerProps {
    operators: IRandomizerOperator[];
    selected: Set<string>;
    onChange: (next: Set<string>) => void;
    /** Profile-owned operators — when present, "Sync from profile" sets selection to this. */
    rosterIndex: IRosterIndex;
    hasProfile: boolean;
}

const RARITY_SORT: Record<OperatorRarity, number> = { 6: 0, 5: 1, 4: 2, 3: 3, 2: 4, 1: 5 };

export function RosterPicker({ operators, selected, onChange, rosterIndex, hasProfile }: IRosterPickerProps): React.ReactElement {
    const [query, setQuery] = React.useState("");

    const sorted = React.useMemo(() => {
        const list = operators.slice().sort((a, b) => {
            const r = RARITY_SORT[a.rarity] - RARITY_SORT[b.rarity];
            if (r !== 0) return r;
            return a.name.localeCompare(b.name);
        });
        if (!query.trim()) return list;
        const q = query.toLowerCase();
        return list.filter((op) => op.name.toLowerCase().includes(q));
    }, [operators, query]);

    const onToggle = React.useCallback(
        (id: string) => {
            const next = new Set(selected);
            if (next.has(id)) next.delete(id);
            else next.add(id);
            onChange(next);
        },
        [selected, onChange],
    );

    const onSelectAll = () => onChange(new Set(operators.map((op) => op.id)));
    const onDeselectAll = () => onChange(new Set());
    const onSyncProfile = () => onChange(new Set(rosterIndex.owned));

    return (
        <div className="flex h-full flex-col gap-3">
            <div className="flex items-center gap-2">
                <div className="relative min-w-0 flex-1">
                    <Search aria-hidden="true" className="pointer-events-none absolute left-2.5 top-1/2 size-3.5 -translate-y-1/2 text-muted-foreground" />
                    <Input value={query} onChange={(e) => setQuery(e.currentTarget.value)} placeholder="Search operators…" size="sm" className="rounded-md border-input pl-7" />
                    {query && (
                        <button type="button" onClick={() => setQuery("")} className="absolute right-1.5 top-1/2 inline-flex size-5 -translate-y-1/2 items-center justify-center rounded-md text-muted-foreground hover:bg-accent hover:text-foreground" aria-label="Clear search">
                            <X className="size-3" />
                        </button>
                    )}
                </div>
            </div>

            <div className="flex flex-wrap items-center justify-between gap-2 font-mono text-[11px] uppercase tracking-[0.16em] text-muted-foreground">
                <span>
                    <span className="text-foreground">{selected.size}</span> / {operators.length} selected
                </span>
                <div className="flex flex-wrap gap-1.5">
                    <Button onClick={onSelectAll} size="xs" variant="ghost">
                        All
                    </Button>
                    <Button onClick={onDeselectAll} size="xs" variant="ghost">
                        None
                    </Button>
                    {hasProfile && rosterIndex.owned.size > 0 && (
                        <Button onClick={onSyncProfile} size="xs" variant="outline">
                            Sync profile ({rosterIndex.owned.size})
                        </Button>
                    )}
                </div>
            </div>

            <div className="-mx-1 grid grid-cols-4 gap-1.5 overflow-y-auto px-1 pb-2 sm:grid-cols-5 md:grid-cols-6 lg:grid-cols-5 xl:grid-cols-6">
                {sorted.map((op) => (
                    <RosterChip key={op.id} op={op} selected={selected.has(op.id)} onToggle={() => onToggle(op.id)} />
                ))}
                {sorted.length === 0 && <p className="col-span-full py-8 text-center text-sm text-muted-foreground">No matches.</p>}
            </div>
        </div>
    );
}

function RosterChip({ op, selected, onToggle }: { op: IRandomizerOperator; selected: boolean; onToggle: () => void }) {
    return (
        <button
            type="button"
            onClick={onToggle}
            aria-pressed={selected}
            className={cn("group relative aspect-square overflow-hidden rounded-md border bg-secondary/40 outline-none transition-all duration-150", "focus-visible:ring-2 focus-visible:ring-ring", selected ? "border-primary/80 shadow-sm shadow-primary/20" : "border-border/50 opacity-60 hover:opacity-95")}
            title={op.name}
        >
            <img
                src={getAvatarById(op.id)}
                alt=""
                aria-hidden="true"
                loading="lazy"
                decoding="async"
                className="absolute inset-0 h-full w-full object-cover"
                onError={(e) => {
                    e.currentTarget.style.display = "none";
                }}
            />
            <div className="absolute inset-x-0 bottom-0 h-1" style={{ background: `var(--rarity-${op.rarity})` }} />
            {selected && (
                <div className="absolute right-0.5 top-0.5 inline-flex size-4 items-center justify-center rounded-full bg-primary text-primary-foreground">
                    <Check className="size-2.5" />
                </div>
            )}
            <span className="sr-only">{op.name}</span>
        </button>
    );
}
