import { useMemo, useState } from "react";
import { Kicker } from "#/components/ui/kicker";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import type { IOperatorPopularity } from "#/lib/api/gacha";
import { formatNumber, formatProfession } from "#/lib/utils";
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
        <section className="flex flex-col gap-4 rounded-[14px] border border-border bg-card p-[22px_24px]">
            <header className="flex items-start justify-between gap-4">
                <div>
                    <Kicker className="mb-1.5">Most pulled · per rarity</Kicker>
                    <h2 className="m-0 font-sans text-[22px] font-semibold leading-[1.15] tracking-[-0.02em] text-foreground text-balance">The top of the pile.</h2>
                </div>
                <div className="inline-flex rounded-[9px] border border-border bg-muted p-0.75">
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
                        <th className="border-b border-border bg-card px-2 py-2 text-left font-mono text-[9.5px] font-medium uppercase tracking-[0.14em] text-muted-foreground w-10">#</th>
                        <th className="border-b border-border bg-card px-2 py-2 text-left font-mono text-[9.5px] font-medium uppercase tracking-[0.14em] text-muted-foreground">Operator</th>
                        <th className="border-b border-border bg-card px-2 py-2 text-left font-mono text-[9.5px] font-medium uppercase tracking-[0.14em] text-muted-foreground">Pulls</th>
                        <th className="border-b border-border bg-card px-2 py-2 text-left font-mono text-[9.5px] font-medium uppercase tracking-[0.14em] text-muted-foreground">Share</th>
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
                                <td className="px-2 py-2.5 font-mono text-[11px] font-medium text-muted-foreground w-10 whitespace-nowrap align-middle tabular-nums">{String(i + 1).padStart(2, "0")}</td>
                                <td className="px-2 py-2.5 align-middle">
                                    <div className="flex items-center gap-3">
                                        <span
                                            className={`relative inline-flex h-9 w-9 shrink-0 items-center justify-center overflow-hidden rounded-lg font-bold text-[11px] tracking-[-0.04em] text-[oklch(0.18_0.04_30/0.85)] shadow-[inset_0_1px_0_oklch(1_0_0/0.4),0_0_0_1px_oklch(0_0_0/0.1)] rarity-${op.rarity}`}
                                            style={{ background: rarityGradient(op.rarity) }}
                                        >
                                            <OperatorAvatar charId={op.charId} name={name} className="block h-full w-full object-cover" />
                                        </span>
                                        <div className="min-w-0">
                                            <div className="font-sans text-sm font-semibold text-foreground truncate">{name}</div>
                                            <div className="mt-0.5 flex flex-wrap items-center gap-2 font-mono text-[10.5px] text-muted-foreground">
                                                <span style={{ color: rarityStarColor(op.rarity) }} className="text-[11px] tracking-wider">
                                                    {"★".repeat(op.rarity)}
                                                </span>
                                                {role ? <span>· {role}</span> : null}
                                                <span className="text-[9.5px] opacity-60">{op.charId}</span>
                                            </div>
                                        </div>
                                    </div>
                                </td>
                                <td className="px-2 py-2.5 text-left pl-4 font-mono text-[13px] font-medium tabular-nums text-foreground w-25 align-middle">{formatNumber(op.pullCount)}</td>
                                <td className="px-2 py-2.5 align-middle w-45 pr-2">
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

function rarityGradient(rarity: number): string {
    switch (rarity) {
        case 6:
            return "linear-gradient(155deg,#f7d166,#f59e0b)";
        case 5:
            return "linear-gradient(155deg,#f7e79e,#d4b94a)";
        case 4:
            return "linear-gradient(155deg,#bcabdb,#8a72ad)";
        case 3:
            return "linear-gradient(155deg,#88c8e3,#5a9bbf)";
        case 2:
            return "linear-gradient(155deg,#7ef2a3,#4fc97a)";
        default:
            return "linear-gradient(155deg,#cfcfcf,#9a9a9a)";
    }
}

function rarityStarColor(rarity: number): string {
    switch (rarity) {
        case 6:
            return "#f7a452";
        case 5:
            return "#f7e79e";
        case 4:
            return "#bcabdb";
        case 3:
            return "#88c8e3";
        case 2:
            return "#7ef2a3";
        default:
            return "#b5b5b5";
    }
}
