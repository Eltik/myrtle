import { ChevronDown } from "lucide-react";
import { memo, useMemo, useState } from "react";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import type { IImprovementsResponse, IOperatorGap, IUpgradeDelta } from "#/lib/api/user";
import { cn } from "#/lib/utils";
import { EmptyHint, operatorRarityColor, PANEL_PADDING, SectionHeader, ShowMoreButton, StatTile, TEXT_BADGE, TEXT_BODY, TEXT_KICKER } from "./shared";

interface IProps {
    improvements: IImprovementsResponse;
    accent: string;
}

const TAG_ORDER = ["ELITE", "MAX_LEVEL", "M3", "SL7", "MOD3", "POT6", "TRUST"] as const;
type Tag = (typeof TAG_ORDER)[number];

const TAG_LABEL: Record<Tag, string> = {
    ELITE: "E↑",
    MAX_LEVEL: "Lvl",
    M3: "M3",
    SL7: "SL7",
    MOD3: "Mod",
    POT6: "Pot",
    TRUST: "Trust",
};

const TAG_DESC: Record<Tag, string> = {
    ELITE: "Elite promotion",
    MAX_LEVEL: "Level cap",
    M3: "Mastery 3",
    SL7: "Skill 7",
    MOD3: "Module L3",
    POT6: "Potential 6",
    TRUST: "Max trust",
};

const TAG_TOOLTIP_DETAIL: Record<Tag, string> = {
    ELITE: "promoting to max elite and re-leveling to its new cap",
    MAX_LEVEL: "leveling to the cap of this operator's current elite phase",
    M3: "raising one skill to Mastery 3",
    SL7: "raising the shared skill level to 7",
    MOD3: "raising one advanced module to Level 3",
    POT6: "reaching Potential 6",
    TRUST: "reaching the trust target (200% for published support units, 100% otherwise)",
};

const INITIAL_VISIBLE = 18;

// Below this we treat the delta as effectively zero and render a muted "·" in
// place of a number. Avoids cluttering the row with "+0.00%" stamps from tiny
// ops that don't move the needle.
const SHOW_DELTA_THRESHOLD_PCT = 0.005;

export function OperatorPanel({ improvements, accent }: IProps) {
    const ops = improvements.operators.below_milestone;
    const [activeFilter, setActiveFilter] = useState<Tag | null>(null);

    // Counts per upgrade type - drives the breakdown summary and the filter chips.
    const tagCounts = useMemo(() => {
        const counts: Record<Tag, number> = { ELITE: 0, MAX_LEVEL: 0, M3: 0, SL7: 0, MOD3: 0, POT6: 0, TRUST: 0 };
        for (const op of ops) {
            for (const tag of op.missing) {
                if (tag in counts) counts[tag as Tag] += 1;
            }
        }
        return counts;
    }, [ops]);

    // Aggregate subscore-points available per tag across the whole roster. Lets
    // the user spot e.g. "M3 alone is +6.2pts to my Operators score" at a glance.
    const tagGains = useMemo(() => {
        const gains: Record<Tag, number> = { ELITE: 0, MAX_LEVEL: 0, M3: 0, SL7: 0, MOD3: 0, POT6: 0, TRUST: 0 };
        for (const op of ops) {
            for (const d of op.deltas) {
                if (d.tag in gains) gains[d.tag as Tag] += d.operator_grade_delta * 100;
            }
        }
        return gains;
    }, [ops]);

    // Apply the active filter (if any), then group what's left by rarity.
    const buckets = useMemo(() => {
        const filtered = activeFilter ? ops.filter((op) => op.missing.includes(activeFilter)) : ops;
        const byRarity = new Map<number, IOperatorGap[]>();
        for (const op of filtered) {
            const bucket = byRarity.get(op.rarity);
            if (bucket) bucket.push(op);
            else byRarity.set(op.rarity, [op]);
        }
        return [...byRarity.entries()].sort(([a], [b]) => b - a);
    }, [ops, activeFilter]);

    // Two roll-ups across every owned op (ELITE/MAX_LEVEL deduped by the backend):
    //   - subscore headroom: how far this section's % could climb (sums toward 100%).
    //   - overall-grade gain: what completing everything adds to the headline score,
    //     i.e. the subscore headroom already scaled by this section's weight. This is
    //     the figure tied to the user's actual grade, so it leads the header.
    const totalSubscoreGainPct = useMemo(() => ops.reduce((acc, op) => acc + op.subscore_potential_gain * 100, 0), [ops]);
    const totalOverallGainPct = useMemo(() => ops.reduce((acc, op) => acc + op.total_potential_gain * 100, 0), [ops]);

    if (ops.length === 0) {
        return (
            <div className={PANEL_PADDING}>
                <EmptyHint>Every owned operator is at their last milestone - nothing to upgrade here.</EmptyHint>
            </div>
        );
    }

    const totalShown = buckets.reduce((acc, [, list]) => acc + list.length, 0);

    return (
        <div className={`${PANEL_PADDING} flex flex-col gap-4`}>
            <SectionHeader title="Operators below milestone" count={`${ops.length} total · +${totalOverallGainPct.toFixed(1)} to overall grade`} accent={accent} />

            {/* By upgrade type - also acts as a filter for the rarity buckets below. */}
            <div className="flex flex-col gap-2">
                <span className={cn(TEXT_KICKER, "text-muted-foreground/70")}>By upgrade type · % gained in this section</span>
                <div className="grid grid-cols-3 gap-1.5 sm:grid-cols-6">
                    {TAG_ORDER.map((tag) => {
                        const gain = tagGains[tag];
                        const sub = gain > 0.01 ? `+${gain.toFixed(1)}%` : TAG_DESC[tag];
                        return <StatTile key={tag} value={tagCounts[tag]} label={TAG_LABEL[tag]} sub={sub} accent={accent} active={activeFilter === tag} onClick={tagCounts[tag] > 0 ? () => setActiveFilter((cur) => (cur === tag ? null : tag)) : undefined} />;
                    })}
                </div>
                {activeFilter && (
                    <button type="button" onClick={() => setActiveFilter(null)} className={cn("self-start", TEXT_KICKER, "text-muted-foreground transition-colors hover:text-foreground")}>
                        Clear filter ({totalShown} shown)
                    </button>
                )}
            </div>

            {/* Per-rarity collapsible buckets. */}
            <div className="flex flex-col gap-1.5">{buckets.length === 0 ? <EmptyHint>No operators match this filter.</EmptyHint> : buckets.map(([rarity, list], idx) => <RarityBucket key={rarity} rarity={rarity} ops={list} defaultOpen={idx === 0} accent={accent} />)}</div>

            {/* Footnote: explain the units the user is reading so they don't
                have to hunt down the math. */}
            <p className={cn(TEXT_BADGE, "text-muted-foreground/60 leading-relaxed")}>
                The header's <span className="text-foreground/80">+{totalOverallGainPct.toFixed(1)} to overall grade</span> is what finishing every upgrade here would add to your headline score. The per-tag, per-rarity, and per-operator figures below are <span className="text-foreground/80">% of this section</span>{" "}
                (the percentage shown at the top of the card) - there's +{totalSubscoreGainPct.toFixed(1)}% of room left here to climb toward 100%. ELITE includes a full re-level at the new phase, so it overlaps MAX_LEVEL; the per-op total deduplicates the pair.
            </p>
        </div>
    );
}

interface IBucketProps {
    rarity: number;
    ops: IOperatorGap[];
    defaultOpen: boolean;
    accent: string;
}

function RarityBucket({ rarity, ops, defaultOpen, accent }: IBucketProps) {
    const [open, setOpen] = useState(defaultOpen);
    const [showAll, setShowAll] = useState(false);
    const color = operatorRarityColor(rarity);

    const visible = useMemo(() => (showAll ? ops : ops.slice(0, INITIAL_VISIBLE)), [showAll, ops]);
    const remaining = ops.length - visible.length;

    // Roll-up of subscore pts available in this rarity bucket. Lets the user
    // see at a glance e.g. "6★s alone hold +4.1 pts of room to grow."
    const bucketGain = useMemo(() => ops.reduce((acc, op) => acc + op.subscore_potential_gain * 100, 0), [ops]);

    return (
        <div className="overflow-hidden rounded-md border border-border/35 bg-muted/8">
            <button type="button" onClick={() => setOpen((o) => !o)} className="group flex w-full items-center gap-2 px-2.5 py-1.5 transition-colors hover:bg-muted/20" aria-expanded={open}>
                <span className={cn("rounded-sm border border-border/40 px-1.5 py-0.5 font-bold", TEXT_BADGE)} style={{ color, background: `color-mix(in oklch, ${color} 10%, transparent)` }}>
                    {rarity}★
                </span>
                <span className={cn(TEXT_BADGE, "text-foreground/85")}>{ops.length}</span>
                <span className={cn(TEXT_KICKER, "text-muted-foreground/65")}>to upgrade</span>
                {bucketGain > 0.01 && (
                    <span
                        className={cn(TEXT_BADGE, "rounded-sm border border-border/40 bg-background px-1 py-px font-semibold tabular-nums")}
                        style={{ color: `color-mix(in oklch, ${accent} 70%, var(--foreground))` }}
                        title={`Maxing every ${rarity}★ below milestone adds ~${bucketGain.toFixed(1)}% to this section's score.`}
                    >
                        +{bucketGain.toFixed(1)}%
                    </span>
                )}
                <span className="ml-auto flex items-center gap-2">
                    <PreviewStrip ops={ops.slice(0, 5)} color={color} dim={open} />
                    <ChevronDown aria-hidden className={cn("size-3.5 text-muted-foreground/70 transition-transform duration-200", open && "rotate-180")} />
                </span>
            </button>

            {open && (
                <div className="border-border/30 border-t bg-background/40 p-2 sm:p-2.5">
                    {/* 1 col by default, 2 at sm+. We don't push to 3 cols - each card
                       holds avatar + name + meta + a wrapping tag row, and at 3 cols
                       there isn't enough width to keep the meta line on one line. */}
                    <div className="grid grid-cols-1 gap-1.5 sm:grid-cols-2">
                        {visible.map((op) => (
                            <OperatorRow key={op.operator_id} op={op} color={color} accent={accent} />
                        ))}
                    </div>
                    {remaining > 0 && (
                        <div className="mt-2">
                            <ShowMoreButton onClick={() => setShowAll(true)} label={`Show ${remaining} more`} />
                        </div>
                    )}
                    {showAll && ops.length > INITIAL_VISIBLE && (
                        <div className="mt-2">
                            <ShowMoreButton onClick={() => setShowAll(false)} label="Show less" />
                        </div>
                    )}
                </div>
            )}
        </div>
    );
}

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

const OperatorRow = memo(function OperatorRow({ op, color, accent }: { op: IOperatorGap; color: string; accent: string }) {
    const sortedTags = TAG_ORDER.filter((t) => op.missing.includes(t));

    // Map tag → delta so the tag chip can show the projected gain inline. The
    // backend guarantees one delta per tag in `missing`, but build the lookup
    // defensively in case of mismatch.
    const deltaByTag = useMemo(() => {
        const map: Record<string, IUpgradeDelta> = {};
        for (const d of op.deltas) map[d.tag] = d;
        return map;
    }, [op.deltas]);

    const subscoreGainPct = op.subscore_potential_gain * 100;
    const totalScoreGainPct = op.total_potential_gain * 100;

    return (
        <div className="flex items-start gap-2.5 rounded-md border border-border/30 bg-card/60 px-2.5 py-2">
            {/* Avatar - slightly larger (size-9) to balance with the multi-line content beside it. */}
            <span className="relative size-9 shrink-0 overflow-hidden rounded-md border border-border/40" style={{ background: `color-mix(in oklch, ${color} 12%, transparent)` }}>
                <OperatorAvatar charId={op.operator_id} name={op.name} />
            </span>

            {/* Stack: name + total gain, meta, tags. `min-w-0` is required so the truncate/whitespace-nowrap below actually clips. */}
            <div className="flex min-w-0 flex-1 flex-col gap-1">
                <div className="flex min-w-0 items-baseline gap-1.5">
                    <span className={cn(TEXT_BODY, "min-w-0 flex-1 truncate font-medium")} title={op.name}>
                        {op.name}
                    </span>
                    {subscoreGainPct >= SHOW_DELTA_THRESHOLD_PCT && (
                        <span
                            className={cn(TEXT_BADGE, "shrink-0 rounded-sm border border-border/40 bg-background px-1 py-px font-semibold tabular-nums")}
                            style={{ color: `color-mix(in oklch, ${accent} 70%, var(--foreground))` }}
                            title={`Completing every upgrade on ${op.name} would add ~${subscoreGainPct.toFixed(2)}% to this section (~${totalScoreGainPct.toFixed(2)} to your overall grade). ELITE/MAX_LEVEL overlap is deduped.`}
                        >
                            +{subscoreGainPct.toFixed(2)}
                        </span>
                    )}
                </div>
                {/* `whitespace-nowrap truncate` together prevent the char-by-char wrap and let ellipsis kick in when the card is narrow. */}
                <span className={cn(TEXT_BADGE, "truncate whitespace-nowrap text-muted-foreground/80")}>
                    E{op.current_elite}·L{op.current_level}
                    {op.max_mastery >= 0 && `·M${op.max_mastery}`}
                    {op.max_module_level >= 0 && `·Mod${op.max_module_level}`}
                    {op.current_trust > 0 && `·T${Math.round(op.current_trust)}`}
                </span>
                {/* Tags live on their own row so they wrap freely instead of fighting the name for horizontal space. */}
                {sortedTags.length > 0 && (
                    <span className="-ml-0.5 flex flex-wrap items-center gap-1 pt-0.5">
                        {sortedTags.map((tag) => {
                            const d = deltaByTag[tag];
                            const tagDeltaPct = d ? d.operator_grade_delta * 100 : 0;
                            const showNumber = tagDeltaPct >= SHOW_DELTA_THRESHOLD_PCT;
                            const tooltip = d ? `${TAG_DESC[tag]} - ${TAG_TOOLTIP_DETAIL[tag]}: ${showNumber ? `+${tagDeltaPct.toFixed(2)}% to this section (+${(d.total_score_delta * 100).toFixed(2)} to overall grade)` : "less than 0.01% - negligible"}` : TAG_DESC[tag];
                            return (
                                <span key={tag} className={cn("rounded-sm border border-border/40 bg-background px-1 py-px font-semibold", TEXT_BADGE)} style={{ color: `color-mix(in oklch, ${color} 65%, var(--foreground))` }} title={tooltip}>
                                    {TAG_LABEL[tag]}
                                    {showNumber ? <span className="ml-1 font-mono text-muted-foreground/85">+{tagDeltaPct.toFixed(2)}</span> : <span className="ml-1 text-muted-foreground/45">·</span>}
                                </span>
                            );
                        })}
                    </span>
                )}
            </div>
        </div>
    );
});
