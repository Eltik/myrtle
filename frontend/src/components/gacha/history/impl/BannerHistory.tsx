import { useMemo, useState } from "react";
import { Kicker } from "#/components/ui/kicker";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import type { ClientGachaGroup, IClientGachaRecords, IGachaItem } from "#/lib/api/gacha";
import { formatNumber } from "#/lib/utils";
import type { IOperatorIndexEntry } from "#/types/operators";

interface IBannerHistoryProps {
    records: IClientGachaRecords | null;
    operatorsById: Map<string, IOperatorIndexEntry>;
    isLoading: boolean;
}

const TABS: { key: ClientGachaGroup; label: string }[] = [
    { key: "limited", label: "Limited" },
    { key: "linkage", label: "Collab" },
    { key: "regular", label: "Standard" },
    { key: "special", label: "Kernel" },
];

const PAGE_SIZE = 50;

function rarityGradient(star: number): string {
    switch (star) {
        case 6:
            return "linear-gradient(155deg,#f7d166,#f59e0b)";
        case 5:
            return "linear-gradient(155deg,#f7e79e,#d4b94a)";
        case 4:
            return "linear-gradient(155deg,#bcabdb,#8a72ad)";
        case 3:
            return "linear-gradient(155deg,#88c8e3,#5a9bbf)";
        default:
            return "linear-gradient(155deg,#cfcfcf,#9a9a9a)";
    }
}

function rarityStarColor(star: number): string {
    switch (star) {
        case 6:
            return "#f7a452";
        case 5:
            return "#f7e79e";
        case 4:
            return "#bcabdb";
        case 3:
            return "#88c8e3";
        default:
            return "#b5b5b5";
    }
}

function fmtDateTime(ts: number): string {
    if (!ts) return "-";
    return new Date(ts).toLocaleString("en-US", {
        month: "short",
        day: "numeric",
        year: "numeric",
        hour: "2-digit",
        minute: "2-digit",
    });
}

function PullTable({ items, operatorsById, total }: { items: IGachaItem[]; operatorsById: Map<string, IOperatorIndexEntry>; total: number }) {
    const [page, setPage] = useState(0);
    const sorted = useMemo(() => [...items].sort((a, b) => b.at - a.at), [items]);
    const pageCount = Math.ceil(sorted.length / PAGE_SIZE);
    const pageItems = sorted.slice(page * PAGE_SIZE, (page + 1) * PAGE_SIZE);

    if (sorted.length === 0) {
        return <div className="py-10 text-center font-sans text-sm text-muted-foreground">No pulls recorded for this banner type.</div>;
    }

    return (
        <div className="flex flex-col gap-3">
            <div className="-mx-1 overflow-y-auto px-1 [scrollbar-color:var(--border)_transparent] [scrollbar-width:thin]">
                <table className="w-full border-collapse">
                    <thead className="sticky top-0 z-10 bg-card">
                        <tr>
                            <th className="border-b border-border bg-card px-1.5 py-2 text-left font-mono text-[9.5px] font-medium uppercase tracking-[0.14em] text-muted-foreground w-10 sm:px-2">#</th>
                            <th className="border-b border-border bg-card px-1.5 py-2 text-left font-mono text-[9.5px] font-medium uppercase tracking-[0.14em] text-muted-foreground sm:px-2">Operator</th>
                            <th className="hidden border-b border-border bg-card px-2 py-2 text-left font-mono text-[9.5px] font-medium uppercase tracking-[0.14em] text-muted-foreground sm:table-cell">Banner</th>
                            <th className="hidden border-b border-border bg-card px-2 py-2 text-right font-mono text-[9.5px] font-medium uppercase tracking-[0.14em] text-muted-foreground md:table-cell">Date</th>
                        </tr>
                    </thead>
                    <tbody>
                        {pageItems.map((item, i) => {
                            const star = Number(item.star);
                            const indexEntry = operatorsById.get(item.charId);
                            const name = item.charName || indexEntry?.name || item.charId;
                            const isSixStar = star === 6;
                            return (
                                // biome-ignore lint/suspicious/noArrayIndexKey: order is stable
                                <tr key={`${item.charId}-${item.at}-${i}`} className={`not-last:border-b not-last:border-border/50 ${isSixStar ? "bg-amber-500/5" : ""}`}>
                                    <td className="px-1.5 py-2 font-mono text-[11px] font-medium text-muted-foreground align-middle tabular-nums sm:px-2">{total - (page * PAGE_SIZE + i)}</td>
                                    <td className="px-1.5 py-2 align-middle sm:px-2">
                                        <div className="flex items-center gap-2 sm:gap-3">
                                            <span
                                                className="relative inline-flex h-7 w-7 shrink-0 items-center justify-center overflow-hidden rounded-md font-bold text-[10px] text-[oklch(0.18_0.04_30/0.85)] shadow-[inset_0_1px_0_oklch(1_0_0/0.4),0_0_0_1px_oklch(0_0_0/0.1)] sm:h-8 sm:w-8"
                                                style={{ background: rarityGradient(star) }}
                                            >
                                                <OperatorAvatar charId={item.charId} name={name} className="block h-full w-full object-cover" />
                                            </span>
                                            <div className="min-w-0">
                                                <div className={`font-sans text-[12.5px] font-semibold leading-snug truncate sm:text-sm ${isSixStar ? "text-amber-600 dark:text-amber-400" : "text-foreground"}`}>{name}</div>
                                                <div className="flex items-center gap-1.5 font-mono text-[10px] text-muted-foreground mt-0.5">
                                                    <span style={{ color: rarityStarColor(star) }} className="tracking-wider">
                                                        {"★".repeat(star)}
                                                    </span>
                                                </div>
                                            </div>
                                        </div>
                                    </td>
                                    <td className="hidden px-2 py-2 align-middle sm:table-cell">
                                        <span className="font-sans text-[12px] text-muted-foreground truncate max-w-45 block">{item.poolName || item.poolId}</span>
                                    </td>
                                    <td className="hidden px-2 py-2 text-right align-middle font-mono text-[11px] text-muted-foreground tabular-nums whitespace-nowrap md:table-cell">{fmtDateTime(item.at)}</td>
                                </tr>
                            );
                        })}
                    </tbody>
                </table>
            </div>

            {pageCount > 1 ? (
                <div className="flex items-center justify-between border-t border-border pt-3">
                    <span className="font-mono text-[11px] text-muted-foreground tabular-nums">
                        {page * PAGE_SIZE + 1}–{Math.min((page + 1) * PAGE_SIZE, sorted.length)} of {formatNumber(sorted.length)} pulls
                    </span>
                    <div className="flex items-center gap-1.5">
                        <button
                            type="button"
                            onClick={() => setPage((p) => Math.max(0, p - 1))}
                            disabled={page === 0}
                            className="flex h-7 items-center rounded-md border border-border bg-card px-2.5 font-mono text-[11px] text-muted-foreground transition-colors hover:text-foreground disabled:pointer-events-none disabled:opacity-40"
                        >
                            Prev
                        </button>
                        <span className="font-mono text-[11px] text-muted-foreground tabular-nums px-1">
                            {page + 1} / {pageCount}
                        </span>
                        <button
                            type="button"
                            onClick={() => setPage((p) => Math.min(pageCount - 1, p + 1))}
                            disabled={page >= pageCount - 1}
                            className="flex h-7 items-center rounded-md border border-border bg-card px-2.5 font-mono text-[11px] text-muted-foreground transition-colors hover:text-foreground disabled:pointer-events-none disabled:opacity-40"
                        >
                            Next
                        </button>
                    </div>
                </div>
            ) : null}
        </div>
    );
}

export function BannerHistory({ records, operatorsById, isLoading }: IBannerHistoryProps) {
    const [activeTab, setActiveTab] = useState<ClientGachaGroup>("limited");

    if (isLoading) {
        return (
            <section className="flex flex-col gap-4 rounded-[14px] border border-border bg-card p-4.5 sm:p-[22px_24px]">
                <div className="h-4 w-28 rounded bg-muted animate-pulse" />
                <div className="flex gap-2 border-b border-border pb-0">
                    {Array.from({ length: 3 }).map((_, i) => (
                        // biome-ignore lint/suspicious/noArrayIndexKey: skeleton
                        <div key={i} className="h-8 w-20 rounded-t-md bg-muted animate-pulse" />
                    ))}
                </div>
                <div className="flex flex-col gap-2.5">
                    {Array.from({ length: 8 }).map((_, i) => (
                        // biome-ignore lint/suspicious/noArrayIndexKey: skeleton
                        <div key={i} className="flex items-center gap-3 py-1 not-last:border-b not-last:border-border/50">
                            <div className="h-7 w-7 rounded-md bg-muted animate-pulse" />
                            <div className="flex flex-1 flex-col gap-1.5">
                                <div className="h-3 w-28 rounded bg-muted animate-pulse" />
                                <div className="h-2 w-16 rounded bg-muted animate-pulse" />
                            </div>
                            <div className="h-3 w-24 rounded bg-muted animate-pulse hidden sm:block" />
                        </div>
                    ))}
                </div>
            </section>
        );
    }

    if (!records) return null;

    return (
        <section className="flex flex-col gap-4 rounded-[14px] border border-border bg-card p-4.5 sm:p-[22px_24px]">
            <header>
                <Kicker className="mb-1.5">Pull history</Kicker>
                <h2 className="m-0 font-sans text-[20px] font-semibold leading-[1.15] tracking-[-0.02em] text-foreground sm:text-[22px]">Every pull, sorted newest first.</h2>
            </header>

            <div className="-mx-1 flex gap-0.5 overflow-x-auto border-b border-border px-1 sm:gap-1 [scrollbar-width:none] [&::-webkit-scrollbar]:hidden overflow-y-hidden">
                {TABS.map((tab) => {
                    const count = records[tab.key].total;
                    const isActive = activeTab === tab.key;
                    return (
                        <button
                            key={tab.key}
                            type="button"
                            onClick={() => setActiveTab(tab.key)}
                            className={`relative flex shrink-0 items-center gap-1 cursor-pointer border-none bg-none px-2 py-2 font-sans text-[12px] font-medium whitespace-nowrap transition-colors sm:gap-1.5 sm:px-3 sm:text-[13px] ${isActive ? "text-foreground" : "text-muted-foreground hover:text-foreground"}`}
                        >
                            {tab.label}
                            <span className={`inline-flex h-4.5 min-w-5 shrink-0 items-center justify-center rounded-full px-1 font-mono text-[10px] tabular-nums ${isActive ? "bg-primary/15 text-primary" : "bg-muted text-muted-foreground"}`}>{formatNumber(count)}</span>
                            {isActive ? <span className="absolute -bottom-px left-2 right-2 h-0.5 rounded-full bg-primary sm:left-3 sm:right-3" /> : null}
                        </button>
                    );
                })}
            </div>

            <PullTable key={activeTab} items={records[activeTab].records} operatorsById={operatorsById} total={records[activeTab].total} />
        </section>
    );
}
