import { ChevronDown } from "lucide-react";
import { memo, useMemo, useState } from "react";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import type { IImprovementsResponse, IOperatorGap } from "#/lib/api/user";
import { cn } from "#/lib/utils";
import { EmptyHint, PANEL_PADDING, SectionHeader } from "./shared";

interface IProps {
    improvements: IImprovementsResponse;
    accent: string;
}

const TAG_ORDER = ["ELITE", "MAX_LEVEL", "M3", "SL7", "MOD3", "POT6"] as const;
type Tag = (typeof TAG_ORDER)[number];

const TAG_LABEL: Record<Tag, string> = {
    ELITE: "E↑",
    MAX_LEVEL: "Lvl",
    M3: "M3",
    SL7: "SL7",
    MOD3: "Mod",
    POT6: "Pot",
};

const TAG_DESC: Record<Tag, string> = {
    ELITE: "Elite promotion",
    MAX_LEVEL: "Level cap",
    M3: "Mastery 3",
    SL7: "Skill 7",
    MOD3: "Module L3",
    POT6: "Potential 6",
};

const INITIAL_VISIBLE = 18;

function rarityColor(r: number): string {
    if (r >= 6) return "oklch(0.78 0.18 75)";
    if (r === 5) return "oklch(0.74 0.17 75)";
    if (r === 4) return "oklch(0.67 0.18 295)";
    if (r === 3) return "oklch(0.65 0.13 220)";
    return "oklch(0.55 0.02 285)";
}

export function OperatorPanel({ improvements, accent }: IProps) {
    const ops = improvements.operators.below_milestone;

    // Group up front so each bucket only re-renders when its own state flips.
    const buckets = useMemo(() => {
        const byRarity = new Map<number, IOperatorGap[]>();
        for (const op of ops) {
            const bucket = byRarity.get(op.rarity);
            if (bucket) bucket.push(op);
            else byRarity.set(op.rarity, [op]);
        }
        return [...byRarity.entries()].sort(([a], [b]) => b - a);
    }, [ops]);

    if (ops.length === 0) {
        return (
            <div className={PANEL_PADDING}>
                <EmptyHint>Every owned operator is at their last milestone — nothing to upgrade here.</EmptyHint>
            </div>
        );
    }

    return (
        <div className={`${PANEL_PADDING} flex flex-col gap-4`}>
            <SectionHeader title="Operators below milestone" count={`${ops.length} total`} accent={accent} />

            {/* Static legend: one source of truth for what each tag means, no per-row tooltips needed. */}
            <Legend />

            <div className="flex flex-col gap-1.5">
                {buckets.map(([rarity, list], idx) => (
                    <RarityBucket key={rarity} rarity={rarity} ops={list} defaultOpen={idx === 0} />
                ))}
            </div>
        </div>
    );
}

function Legend() {
    return (
        <div className="flex flex-wrap items-center gap-1.5 rounded-md border border-border/35 bg-muted/15 px-2.5 py-1.5">
            <span className="font-mono text-[9.5px] uppercase tracking-wider text-muted-foreground/70">Tags</span>
            {TAG_ORDER.map((tag) => (
                <span key={tag} className="flex items-center gap-1 text-[10.5px]">
                    <span className="rounded-sm border border-border/50 bg-background px-1 py-px font-mono font-semibold tabular-nums text-foreground/80">{TAG_LABEL[tag]}</span>
                    <span className="text-muted-foreground/75">{TAG_DESC[tag]}</span>
                </span>
            ))}
        </div>
    );
}

interface IBucketProps {
    rarity: number;
    ops: IOperatorGap[];
    defaultOpen: boolean;
}

function RarityBucket({ rarity, ops, defaultOpen }: IBucketProps) {
    const [open, setOpen] = useState(defaultOpen);
    const [showAll, setShowAll] = useState(false);
    const color = rarityColor(rarity);

    const visible = useMemo(() => (showAll ? ops : ops.slice(0, INITIAL_VISIBLE)), [showAll, ops]);
    const remaining = ops.length - visible.length;

    return (
        <div className="overflow-hidden rounded-md border border-border/35 bg-muted/8">
            <button type="button" onClick={() => setOpen((o) => !o)} className="group flex w-full items-center gap-2 px-2.5 py-1.5 transition-colors hover:bg-muted/20" aria-expanded={open}>
                <span className="rounded-sm border border-border/40 px-1.5 py-0.5 font-mono text-[10px] font-bold tabular-nums" style={{ color, background: `color-mix(in oklch, ${color} 10%, transparent)` }}>
                    {rarity}★
                </span>
                <span className="font-mono text-[10.5px] tabular-nums text-foreground/85">{ops.length}</span>
                <span className="font-mono text-[10px] uppercase tracking-wider text-muted-foreground/65">to upgrade</span>
                <span className="ml-auto flex items-center gap-2">
                    <PreviewStrip ops={ops.slice(0, 5)} color={color} dim={open} />
                    <ChevronDown aria-hidden className={cn("size-3.5 text-muted-foreground/70 transition-transform duration-200", open && "rotate-180")} />
                </span>
            </button>

            {open && (
                <div className="border-t border-border/30 bg-background/40 p-2 sm:p-2.5">
                    <div className="grid grid-cols-1 gap-1 sm:grid-cols-2 lg:grid-cols-3">
                        {visible.map((op) => (
                            <OperatorRow key={op.operator_id} op={op} color={color} />
                        ))}
                    </div>
                    {remaining > 0 && (
                        <button type="button" onClick={() => setShowAll(true)} className="mt-2 w-full rounded-md border border-dashed border-border/40 py-1.5 font-mono text-[10.5px] uppercase tracking-wider text-muted-foreground transition-colors hover:border-border/70 hover:text-foreground">
                            Show {remaining} more
                        </button>
                    )}
                    {showAll && ops.length > INITIAL_VISIBLE && (
                        <button type="button" onClick={() => setShowAll(false)} className="mt-2 w-full rounded-md border border-dashed border-border/40 py-1.5 font-mono text-[10.5px] uppercase tracking-wider text-muted-foreground transition-colors hover:border-border/70 hover:text-foreground">
                            Show less
                        </button>
                    )}
                </div>
            )}
        </div>
    );
}

/** Small avatar preview shown beside the bucket header when collapsed. */
function PreviewStrip({ ops, color, dim }: { ops: IOperatorGap[]; color: string; dim: boolean }) {
    if (dim) return null;
    return (
        <span className="hidden items-center sm:flex">
            {ops.map((op, i) => (
                <span key={op.operator_id} className="-ml-1.5 size-5 overflow-hidden rounded-full border border-background ring-1 ring-border/50 first:ml-0" style={{ background: `color-mix(in oklch, ${color} 12%, transparent)`, zIndex: ops.length - i }} aria-hidden>
                    <OperatorAvatar charId={op.operator_id} name={op.name} />
                </span>
            ))}
        </span>
    );
}

/** Pure-presentational row, memoized so re-renders of the bucket don't redraw every row. */
const OperatorRow = memo(function OperatorRow({ op, color }: { op: IOperatorGap; color: string }) {
    // Sort tags into a canonical order so the eye sweeps consistently across rows.
    const sortedTags = TAG_ORDER.filter((t) => op.missing.includes(t));
    return (
        <div className="flex items-center gap-2 rounded-md border border-border/30 bg-card/60 px-2 py-1.5">
            <span className="relative size-7 shrink-0 overflow-hidden rounded-md border border-border/40" style={{ background: `color-mix(in oklch, ${color} 12%, transparent)` }}>
                <OperatorAvatar charId={op.operator_id} name={op.name} />
            </span>
            <div className="flex min-w-0 flex-1 flex-col">
                <span className="truncate text-[11.5px] font-medium leading-tight">{op.name}</span>
                <span className="font-mono text-[9.5px] tabular-nums text-muted-foreground/85">
                    E{op.current_elite}·L{op.current_level}
                    {op.max_mastery >= 0 && `·M${op.max_mastery}`}
                    {op.max_module_level >= 0 && `·Mod${op.max_module_level}`}
                </span>
            </div>
            <span className="flex shrink-0 flex-wrap items-center gap-0.5">
                {sortedTags.map((tag) => (
                    <span key={tag} className="rounded-sm border border-border/40 bg-background px-1 py-px font-mono text-[9px] font-semibold tabular-nums" style={{ color: `color-mix(in oklch, ${color} 65%, var(--foreground))` }} title={TAG_DESC[tag]}>
                        {TAG_LABEL[tag]}
                    </span>
                ))}
            </span>
        </div>
    );
});
