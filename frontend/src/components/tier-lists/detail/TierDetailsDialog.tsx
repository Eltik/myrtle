import { Link } from "@tanstack/react-router";
import { ArrowUpRight, MessageSquareText } from "lucide-react";
import { ClassIcon } from "#/components/operators/list/impl/components/Icons";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import type { ITierEntryFull } from "#/lib/api/tier-lists";
import { formatProfession, formatRelative, formatSubProfession, RARITY_LABELS } from "#/lib/utils";
import type { OperatorRarity } from "#/types/operators";
import { type ReadableTextColor, readableTextColor } from "./contrast";
import { computeTierStats } from "./tierStats";

interface ITierDetailsDialogProps {
    tier: ITierEntryFull;
    color: string;
}

const RARITY_VAR: Record<OperatorRarity, string> = {
    6: "var(--rarity-6)",
    5: "var(--rarity-5)",
    4: "var(--rarity-4)",
    3: "var(--rarity-3)",
    2: "var(--rarity-2)",
    1: "var(--rarity-1)",
};

function rarityShadow(textColor: ReadableTextColor) {
    return textColor === "white" ? "0 1px 0 oklch(0 0 0 / 0.25)" : "0 1px 0 oklch(1 0 0 / 0.5)";
}

export function TierDetailsDialog({ tier, color }: ITierDetailsDialogProps) {
    const textColor = readableTextColor(color);
    const stats = computeTierStats(tier);
    const ops = tier.operators;
    const total = stats.total;

    return (
        <div className="flex max-h-[88dvh] flex-col sm:max-h-[80vh]">
            <header className="relative flex items-start gap-3 border-border border-b px-4 pt-5 pb-4 sm:gap-4 sm:px-6 sm:pt-7 sm:pb-5" style={{ background: `linear-gradient(135deg, color-mix(in srgb, ${color} 16%, transparent), transparent 70%)` }}>
                <span className="absolute inset-x-0 top-0 h-1" style={{ background: color }} aria-hidden="true" />
                <span
                    className="flex h-12 w-12 shrink-0 items-center justify-center rounded-xl font-extrabold font-features-['ss01','cv11'] font-sans text-[26px] leading-none tracking-tight sm:h-16 sm:w-16 sm:rounded-2xl sm:text-[34px]"
                    style={{
                        background: color,
                        color: textColor,
                        textShadow: rarityShadow(textColor),
                        boxShadow: `0 6px 24px color-mix(in srgb, ${color} 30%, transparent)`,
                    }}
                    aria-hidden="true"
                >
                    {tier.name.length <= 2 ? tier.name : tier.name.charAt(0)}
                </span>
                <div className="min-w-0 flex-1 pt-0.5 pr-10 sm:pr-0">
                    <p className="m-0 font-bold font-mono text-[10px] text-muted-foreground/80 uppercase leading-none tracking-[0.2em] sm:text-[10.5px]">Tier</p>
                    <h2 className="wrap-break-word m-0 mt-1.5 font-bold font-features-['ss01','cv11'] font-sans text-[22px] text-foreground leading-[1.1] tracking-[-0.015em] sm:mt-2 sm:text-[28px]">{tier.name}</h2>
                    <p className="m-0 mt-1.5 font-mono font-semibold text-[10.5px] text-muted-foreground uppercase tabular-nums leading-none tracking-[0.14em] sm:mt-2 sm:text-[11px]">
                        <span className="text-foreground/90">{total}</span> op{total === 1 ? "" : "s"}
                        {stats.lastUpdatedAt && (
                            <>
                                <span className="mx-1.5 opacity-50 sm:mx-2">·</span>
                                <span>Updated {formatRelative(stats.lastUpdatedAt)}</span>
                            </>
                        )}
                    </p>
                </div>
            </header>

            <div className="flex-1 space-y-5 overflow-y-auto px-4 py-5 sm:space-y-6 sm:px-6 sm:py-6">
                {tier.description ? (
                    <p className="wrap-break-word m-0 max-w-prose whitespace-pre-line font-sans text-[14.5px] text-foreground leading-[1.65] tracking-[-0.005em]">{tier.description}</p>
                ) : (
                    <p className="m-0 font-sans text-[13px] text-muted-foreground italic leading-relaxed">The author hasn't written a description for this tier.</p>
                )}

                {total > 0 && (
                    <section aria-label="Tier overview" className="grid grid-cols-3 gap-2">
                        <Stat label="Operators" value={String(total)} />
                        <Stat label="Avg ★" value={stats.averageRarity ? stats.averageRarity.toFixed(1) : "-"} />
                        <Stat label="Melee / Ranged" value={`${stats.position.melee} / ${stats.position.ranged}`} />
                    </section>
                )}

                {stats.rarity.length > 0 && (
                    <section aria-label="Rarity breakdown" className="space-y-2">
                        <SectionHeading>Rarity</SectionHeading>
                        <div className="flex flex-wrap gap-1.5">
                            {stats.rarity.map(({ rarity, count }) => (
                                <span key={rarity} className="inline-flex items-center gap-1.5 rounded-md border bg-card px-2.5 py-1.5 font-sans font-semibold text-[12.5px] text-foreground leading-none" style={{ borderColor: `color-mix(in srgb, ${RARITY_VAR[rarity]} 38%, var(--border))` }}>
                                    <span className="h-1.5 w-1.5 rounded-full" style={{ background: RARITY_VAR[rarity] }} aria-hidden="true" />
                                    <span className="tabular-nums" style={{ color: RARITY_VAR[rarity] }}>
                                        {RARITY_LABELS[rarity]}
                                    </span>
                                    <span className="font-medium font-mono text-[11.5px] text-muted-foreground tabular-nums">{count}</span>
                                </span>
                            ))}
                        </div>
                    </section>
                )}

                {stats.profession.length > 0 && (
                    <section aria-label="Class breakdown" className="space-y-2">
                        <SectionHeading>Classes</SectionHeading>
                        <div className="flex flex-wrap gap-1.5">
                            {stats.profession.map(({ profession, count }) => (
                                <span key={profession} className="inline-flex items-center gap-1.5 rounded-md border border-border bg-card px-2.5 py-1.5 font-sans font-semibold text-[12.5px] text-foreground leading-none">
                                    <ClassIcon profession={profession} size={14} />
                                    <span>{formatProfession(profession)}</span>
                                    <span className="font-medium font-mono text-[11.5px] text-muted-foreground tabular-nums">{count}</span>
                                </span>
                            ))}
                        </div>
                    </section>
                )}

                <section aria-label="Operators in this tier" className="space-y-2.5">
                    <div className="flex items-baseline justify-between gap-3">
                        <SectionHeading>Operators</SectionHeading>
                        <span className="font-mono text-[10.5px] text-muted-foreground uppercase tabular-nums tracking-[0.12em]">{total}</span>
                    </div>
                    {ops.length === 0 ? (
                        <p className="rounded-lg border border-border border-dashed bg-muted/20 px-4 py-6 text-center font-sans text-[12.5px] text-muted-foreground italic">No operators have been placed in this tier yet.</p>
                    ) : (
                        <ul className="m-0 grid grid-cols-1 gap-1 p-0 sm:grid-cols-2">
                            {ops.map((op) => (
                                <li key={op.id} className="list-none">
                                    <Link to="/operators/$id" params={{ id: op.id }} className="group flex items-center gap-3 rounded-lg border border-transparent px-2 py-2 no-underline transition-colors hover:border-border hover:bg-muted/40">
                                        <span className="flex h-10 w-10 shrink-0 items-center justify-center overflow-hidden rounded-md bg-muted font-semibold text-[11px] text-foreground" style={{ borderBottom: `2px solid ${RARITY_VAR[op.rarity]}` }}>
                                            <OperatorAvatar charId={op.id} name={op.name} />
                                        </span>
                                        <span className="flex min-w-0 flex-1 flex-col gap-0.5">
                                            <span className="flex min-w-0 items-center gap-1.5">
                                                <span className="truncate font-sans font-semibold text-[13.5px] text-foreground tracking-[-0.005em] transition-colors group-hover:text-primary">{op.name}</span>
                                                {op.notes?.trim() && (
                                                    <MessageSquareText className="h-3 w-3 shrink-0 text-muted-foreground/70" aria-label={`${op.name} has a note`}>
                                                        <title>Has note</title>
                                                    </MessageSquareText>
                                                )}
                                            </span>
                                            <span className="truncate font-mono text-[10.5px] text-muted-foreground uppercase leading-none tracking-[0.08em]">
                                                <span style={{ color: RARITY_VAR[op.rarity] }} className="font-semibold">
                                                    {op.rarity}★
                                                </span>
                                                <span className="mx-1 opacity-50">·</span>
                                                <span>{formatSubProfession(op.subProfessionId).replace(formatProfession(op.profession), "").trim() || formatProfession(op.profession)}</span>
                                            </span>
                                        </span>
                                        <ArrowUpRight className="h-3.5 w-3.5 shrink-0 text-muted-foreground/0 transition-all group-hover:translate-x-0.5 group-hover:text-muted-foreground" aria-hidden="true" />
                                    </Link>
                                </li>
                            ))}
                        </ul>
                    )}
                </section>
            </div>
        </div>
    );
}

function SectionHeading({ children }: { children: React.ReactNode }) {
    return <h3 className="m-0 font-bold font-mono text-[10.5px] text-muted-foreground/80 uppercase leading-none tracking-[0.2em]">{children}</h3>;
}

interface IStatProps {
    label: string;
    value: string;
    muted?: boolean;
}

function Stat({ label, value, muted = false }: IStatProps) {
    return (
        <div className="rounded-lg border border-border bg-card/60 px-3 py-2.5">
            <p className="m-0 font-bold font-mono text-[9.5px] text-muted-foreground/80 uppercase leading-none tracking-[0.18em]">{label}</p>
            <p className={`m-0 mt-2 font-extrabold font-features-['ss01','cv11'] font-sans text-[20px] tabular-nums leading-none tracking-[-0.015em] ${muted ? "text-muted-foreground/60" : "text-foreground"}`}>{value}</p>
        </div>
    );
}
