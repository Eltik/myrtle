import { useMemo, useState } from "react";
import { Kicker } from "#/components/ui/kicker";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import type { IClientGachaRecords, IGachaItem } from "#/lib/api/gacha";
import { formatNumber, formatProfession, rarityGradient, rarityStarColor } from "#/lib/utils";
import type { IOperatorIndexEntry } from "#/types/operators";

interface ITopOperatorsProps {
    records: IClientGachaRecords | null;
    operatorsById: Map<string, IOperatorIndexEntry>;
    isLoading: boolean;
}

interface IOperatorTally {
    charId: string;
    charName: string;
    star: number;
    count: number;
}

const STAR_TABS = [6, 5, 4] as const;
type StarTab = (typeof STAR_TABS)[number];

function tallyOperators(items: IGachaItem[]): IOperatorTally[] {
    const map = new Map<string, IOperatorTally>();
    for (const item of items) {
        const star = Number(item.star);
        const existing = map.get(item.charId);
        if (existing) {
            existing.count++;
        } else {
            map.set(item.charId, { charId: item.charId, charName: item.charName, star, count: 1 });
        }
    }
    return Array.from(map.values()).sort((a, b) => b.count - a.count);
}

export function TopOperators({ records, operatorsById, isLoading }: ITopOperatorsProps) {
    const [activeStar, setActiveStar] = useState<StarTab>(6);

    const allItems = useMemo<IGachaItem[]>(() => {
        if (!records) return [];
        return [...records.limited.records, ...records.linkage.records, ...records.regular.records, ...records.special.records];
    }, [records]);

    const tallies = useMemo(() => tallyOperators(allItems), [allItems]);
    const filtered = useMemo(() => tallies.filter((t) => t.star === activeStar).slice(0, 20), [tallies, activeStar]);

    if (isLoading) {
        return (
            <section className="flex flex-col gap-4 rounded-[14px] border border-border bg-card p-4.5 sm:p-[22px_24px]">
                <div className="flex items-start justify-between gap-4">
                    <div className="flex flex-col gap-1.5">
                        <div className="h-2.5 w-24 animate-pulse rounded-full bg-muted" />
                        <div className="h-5 w-40 animate-pulse rounded bg-muted" />
                    </div>
                    <div className="h-7 w-28 animate-pulse rounded-lg bg-muted" />
                </div>
                <div className="flex flex-col gap-2.5">
                    {Array.from({ length: 5 }).map((_, i) => (
                        // biome-ignore lint/suspicious/noArrayIndexKey: skeleton
                        <div key={i} className="flex items-center gap-3">
                            <div className="h-9 w-9 animate-pulse rounded-lg bg-muted" />
                            <div className="flex flex-1 flex-col gap-1.5">
                                <div className="h-3 w-32 animate-pulse rounded bg-muted" />
                                <div className="h-2 w-20 animate-pulse rounded bg-muted" />
                            </div>
                            <div className="h-3 w-12 animate-pulse rounded bg-muted" />
                        </div>
                    ))}
                </div>
            </section>
        );
    }

    if (!records) return null;

    const max = filtered[0]?.count ?? 1;

    return (
        <section className="flex flex-col gap-4 rounded-[14px] border border-border bg-card p-4.5 sm:p-[22px_24px]">
            <header className="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between sm:gap-4">
                <div>
                    <Kicker className="mb-1.5">Most pulled · per rarity</Kicker>
                    <h2 className="m-0 font-sans font-semibold text-[20px] text-foreground leading-[1.15] tracking-[-0.02em] sm:text-[22px]">Your top operators.</h2>
                </div>
                <div className="inline-flex self-start rounded-[9px] border border-border bg-muted p-0.75 sm:self-auto">
                    {STAR_TABS.map((s) => (
                        <button
                            key={s}
                            type="button"
                            onClick={() => setActiveStar(s)}
                            className={`h-6.5 cursor-pointer rounded-md px-3 font-medium font-sans text-xs transition-colors ${activeStar === s ? "bg-card text-foreground shadow-[0_1px_2px_oklch(0_0_0/0.4)]" : "bg-transparent text-muted-foreground hover:text-foreground"}`}
                        >
                            {s}★
                        </button>
                    ))}
                </div>
            </header>

            {filtered.length === 0 ? (
                <div className="py-8 text-center font-sans text-muted-foreground text-sm">No {activeStar}★ operators pulled yet.</div>
            ) : (
                <div className="-mx-1 max-h-105 overflow-y-auto pr-1 pl-1 [scrollbar-color:var(--border)_transparent] [scrollbar-width:thin]">
                    <table className="w-full border-collapse">
                        <thead className="sticky top-0 z-10 bg-card">
                            <tr>
                                <th className="w-8 border-border border-b bg-card px-1.5 py-2 text-left font-medium font-mono text-[9.5px] text-muted-foreground uppercase tracking-[0.14em] sm:w-10 sm:px-2">#</th>
                                <th className="border-border border-b bg-card px-1.5 py-2 text-left font-medium font-mono text-[9.5px] text-muted-foreground uppercase tracking-[0.14em] sm:px-2">Operator</th>
                                <th className="border-border border-b bg-card px-1.5 py-2 text-right font-medium font-mono text-[9.5px] text-muted-foreground uppercase tracking-[0.14em] sm:px-2 sm:text-left">Copies</th>
                                <th className="hidden w-40 border-border border-b bg-card px-2 py-2 text-left font-medium font-mono text-[9.5px] text-muted-foreground uppercase tracking-[0.14em] sm:table-cell">Frequency</th>
                            </tr>
                        </thead>
                        <tbody>
                            {filtered.map((op, i) => {
                                const indexEntry = operatorsById.get(op.charId);
                                const name = op.charName || indexEntry?.name || op.charId;
                                const role = indexEntry ? formatProfession(indexEntry.profession) : "";
                                const bar = (op.count / max) * 100;
                                return (
                                    <tr key={op.charId} className="not-last:border-border/50 not-last:border-b">
                                        <td className="w-8 whitespace-nowrap px-1.5 py-2.5 align-middle font-medium font-mono text-[11px] text-muted-foreground tabular-nums sm:w-10 sm:px-2">{String(i + 1).padStart(2, "0")}</td>
                                        <td className="px-1.5 py-2.5 align-middle sm:px-2">
                                            <div className="flex items-center gap-2 sm:gap-3">
                                                <span
                                                    className="relative inline-flex h-8 w-8 shrink-0 items-center justify-center overflow-hidden rounded-lg font-bold text-[11px] text-[oklch(0.18_0.04_30/0.85)] tracking-[-0.04em] shadow-[inset_0_1px_0_oklch(1_0_0/0.4),0_0_0_1px_oklch(0_0_0/0.1)] sm:h-9 sm:w-9"
                                                    style={{ background: rarityGradient(op.star) }}
                                                >
                                                    <OperatorAvatar charId={op.charId} name={name} className="block h-full w-full object-cover" />
                                                </span>
                                                <div className="min-w-0 flex-1">
                                                    <div className="truncate font-sans font-semibold text-[13px] text-foreground sm:text-sm">{name}</div>
                                                    <div className="mt-0.5 flex flex-wrap items-center gap-1.5 font-mono text-[10.5px] text-muted-foreground sm:gap-2">
                                                        <span style={{ color: rarityStarColor(op.star) }} className="text-[11px] tracking-wider">
                                                            {"★".repeat(op.star)}
                                                        </span>
                                                        {role ? <span className="hidden sm:inline">· {role}</span> : null}
                                                    </div>
                                                </div>
                                            </div>
                                        </td>
                                        <td className="whitespace-nowrap px-1.5 py-2.5 text-right align-middle font-medium font-mono text-[12px] text-foreground tabular-nums sm:w-20 sm:px-2 sm:pl-4 sm:text-left sm:text-[13px]">{formatNumber(op.count)}</td>
                                        <td className="hidden w-40 px-2 py-2.5 pr-2 align-middle sm:table-cell">
                                            <div className="flex items-center gap-2.5">
                                                <div className="h-1 flex-1 overflow-hidden rounded-full bg-muted">
                                                    <div className="h-full rounded-full" style={{ width: `${bar}%`, background: "linear-gradient(90deg, var(--primary), oklch(0.78 0.16 25))" }} />
                                                </div>
                                                <span className="min-w-10 text-right font-mono text-[11px] text-muted-foreground tabular-nums">{op.count}×</span>
                                            </div>
                                        </td>
                                    </tr>
                                );
                            })}
                        </tbody>
                    </table>
                </div>
            )}
        </section>
    );
}
