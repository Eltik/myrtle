import { useMemo, useState } from "react";
import { Kicker } from "#/components/ui/kicker";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import type { IOperatorPopularity } from "#/lib/api/gacha";
import { formatNumber, formatProfession, rarityGradient, rarityStarColor } from "#/lib/utils";
import type { IOperatorIndexEntry } from "#/types/operators";
import { fmtPct } from "./format";

const RARITIES = [6, 5, 4] as const;
type Rarity = (typeof RARITIES)[number];

interface ILeaderboardProps {
    ops: IOperatorPopularity[];
    operatorsById: Map<string, IOperatorIndexEntry>;
    isLoading: boolean;
}

export function Leaderboard({ ops, operatorsById, isLoading }: ILeaderboardProps) {
    const [rarity, setRarity] = useState<Rarity>(6);
    const filtered = useMemo(() => ops.filter((o) => o.rarity === rarity), [ops, rarity]);

    return (
        <section className="flex flex-col gap-4 rounded-[14px] border border-border bg-card p-[18px_18px] sm:p-[22px_24px]">
            <header className="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between sm:gap-4">
                <div>
                    <Kicker className="mb-1.5">Most pulled · per rarity</Kicker>
                    <h2 className="m-0 font-sans text-[20px] font-semibold leading-[1.15] tracking-[-0.02em] text-foreground text-balance sm:text-[22px]">The top of the pile.</h2>
                </div>
                <div className="inline-flex self-start rounded-[9px] border border-border bg-muted p-0.75 sm:self-auto">
                    {RARITIES.map((r) => (
                        <button
                            key={r}
                            type="button"
                            onClick={() => setRarity(r)}
                            className={`h-6.5 cursor-pointer rounded-md px-3 font-sans text-xs font-medium transition-colors ${rarity === r ? "bg-card text-foreground shadow-[0_1px_2px_oklch(0_0_0/0.4)]" : "bg-transparent text-muted-foreground hover:text-foreground"}`}
                        >
                            {r}★
                        </button>
                    ))}
                </div>
            </header>

            <LeaderTable ops={filtered} operatorsById={operatorsById} isLoading={isLoading} />
        </section>
    );
}

function LeaderTable({ ops, operatorsById, isLoading }: { ops: IOperatorPopularity[]; operatorsById: Map<string, IOperatorIndexEntry>; isLoading: boolean }) {
    if (isLoading && ops.length === 0) {
        return (
            <div className="flex flex-col gap-2.5 py-2">
                {Array.from({ length: 6 }).map((_, i) => (
                    // biome-ignore lint/suspicious/noArrayIndexKey: loading skeleton
                    <div key={i} className="flex items-center gap-3">
                        <div className="h-9 w-9 shrink-0 rounded-lg bg-muted animate-pulse" />
                        <div className="flex flex-1 flex-col gap-1.5">
                            <div className="h-3 w-32 rounded bg-muted animate-pulse" />
                            <div className="h-2 w-20 rounded bg-muted animate-pulse" />
                        </div>
                        <div className="h-3 w-16 rounded bg-muted animate-pulse" />
                    </div>
                ))}
            </div>
        );
    }

    if (ops.length === 0) {
        return <div className="py-8 text-center font-sans text-sm text-muted-foreground">No data for this rarity yet.</div>;
    }

    const max = Math.max(...ops.map((o) => o.pullCount));

    return (
        <div className="-mx-1 max-h-105 overflow-y-auto pr-1 pl-1 [scrollbar-color:var(--border)_transparent] [scrollbar-width:thin]">
            <table className="w-full border-collapse">
                <thead className="sticky top-0 z-10 bg-card">
                    <tr>
                        <th className="border-b border-border bg-card px-1.5 py-2 text-left font-mono text-[9.5px] font-medium uppercase tracking-[0.14em] text-muted-foreground w-8 sm:w-10 sm:px-2">#</th>
                        <th className="border-b border-border bg-card px-1.5 py-2 text-left font-mono text-[9.5px] font-medium uppercase tracking-[0.14em] text-muted-foreground sm:px-2">Operator</th>
                        <th className="border-b border-border bg-card px-1.5 py-2 text-right font-mono text-[9.5px] font-medium uppercase tracking-[0.14em] text-muted-foreground sm:px-2 sm:text-left">Pulls</th>
                        <th className="hidden border-b border-border bg-card px-2 py-2 text-left font-mono text-[9.5px] font-medium uppercase tracking-[0.14em] text-muted-foreground sm:table-cell">Share</th>
                    </tr>
                </thead>
                <tbody>
                    {ops.map((op, i) => {
                        const indexEntry = operatorsById.get(op.charId);
                        const name = op.charName || indexEntry?.name || op.charId;
                        const role = indexEntry ? formatProfession(indexEntry.profession) : "";
                        const bar = (op.pullCount / max) * 100;
                        return (
                            <tr key={op.charId} className="not-last:border-b not-last:border-border/50">
                                <td className="px-1.5 py-2.5 font-mono text-[11px] font-medium text-muted-foreground w-8 whitespace-nowrap align-middle tabular-nums sm:w-10 sm:px-2">{String(i + 1).padStart(2, "0")}</td>
                                <td className="px-1.5 py-2.5 align-middle sm:px-2">
                                    <div className="flex items-center gap-2 sm:gap-3">
                                        <span
                                            className={`relative inline-flex h-8 w-8 shrink-0 items-center justify-center overflow-hidden rounded-lg font-bold text-[11px] tracking-[-0.04em] text-[oklch(0.18_0.04_30/0.85)] shadow-[inset_0_1px_0_oklch(1_0_0/0.4),0_0_0_1px_oklch(0_0_0/0.1)] sm:h-9 sm:w-9 rarity-${op.rarity}`}
                                            style={{ background: rarityGradient(op.rarity) }}
                                        >
                                            <OperatorAvatar charId={op.charId} name={name} className="block h-full w-full object-cover" />
                                        </span>
                                        <div className="min-w-0 flex-1">
                                            <div className="font-sans text-[13px] font-semibold text-foreground truncate sm:text-sm">{name}</div>
                                            <div className="mt-0.5 flex flex-wrap items-center gap-1.5 font-mono text-[10.5px] text-muted-foreground sm:gap-2">
                                                <span style={{ color: rarityStarColor(op.rarity) }} className="text-[11px] tracking-wider">
                                                    {"★".repeat(op.rarity)}
                                                </span>
                                                {role ? <span className="hidden sm:inline">· {role}</span> : null}
                                                <span className="hidden text-[9.5px] opacity-60 sm:inline">{op.charId}</span>
                                                <span className="font-mono text-[10px] tabular-nums text-muted-foreground sm:hidden">· {fmtPct(op.percentage, 2)}</span>
                                            </div>
                                        </div>
                                    </div>
                                </td>
                                <td className="px-1.5 py-2.5 text-right font-mono text-[12px] font-medium tabular-nums text-foreground align-middle whitespace-nowrap sm:px-2 sm:pl-4 sm:text-left sm:text-[13px] sm:w-25">{formatNumber(op.pullCount)}</td>
                                <td className="hidden px-2 py-2.5 align-middle pr-2 w-45 sm:table-cell">
                                    <div className="flex items-center gap-2.5">
                                        <div className="h-1 flex-1 overflow-hidden rounded-full bg-muted">
                                            <div className="h-full rounded-full" style={{ width: `${bar}%`, background: "linear-gradient(90deg, var(--primary), oklch(0.78 0.16 25))" }} />
                                        </div>
                                        <span className="min-w-14 text-right font-mono text-[11px] text-muted-foreground tabular-nums">{fmtPct(op.percentage, 3)}</span>
                                    </div>
                                </td>
                            </tr>
                        );
                    })}
                </tbody>
            </table>
        </div>
    );
}
