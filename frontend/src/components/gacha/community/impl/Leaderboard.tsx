import { useMemo, useState } from "react";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { Skeleton } from "#/components/ui/skeleton";
import { ToggleGroup, ToggleGroupItem } from "#/components/ui/toggle-group";
import type { IOperatorPopularity } from "#/lib/api/gacha";
import { useFormatters, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { professionLabel, rarityGradient, rarityInk } from "#/lib/utils";
import type { IOperatorIndexEntry } from "#/types/operators";
import { fmtPct } from "./format";
import type { messages } from "./Leaderboard.messages";

const RARITIES = [6, 5, 4] as const;
type Rarity = (typeof RARITIES)[number];

interface ILeaderboardProps {
    ops: IOperatorPopularity[];
    operatorsById: Map<string, IOperatorIndexEntry>;
    isLoading: boolean;
}

export function Leaderboard({ ops, operatorsById, isLoading }: ILeaderboardProps) {
    const t: TypedT<typeof messages> = useT("gacha");
    const [rarity, setRarity] = useState<Rarity>(6);
    const filtered = useMemo(() => ops.filter((o) => o.rarity === rarity), [ops, rarity]);

    return (
        <section className="flex flex-col gap-4 rounded-2xl border border-border bg-card p-[18px_18px] sm:p-[22px_24px] lg:absolute lg:inset-0">
            <header className="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between sm:gap-4">
                <div>
                    <h2 className="m-0 font-sans font-semibold text-[15px] text-foreground leading-tight">{t("community.leaderboard.kicker")}</h2>
                </div>
                <ToggleGroup
                    aria-label={t("community.leaderboard.rarityFilter")}
                    className="self-start sm:self-auto"
                    onValueChange={(next) => {
                        const picked = RARITIES.find((r) => String(r) === next[0]);
                        if (picked) setRarity(picked);
                    }}
                    size="sm"
                    value={[String(rarity)]}
                    variant="outline"
                >
                    {RARITIES.map((r) => (
                        <ToggleGroupItem key={r} className="px-3" value={String(r)}>
                            {r}★
                        </ToggleGroupItem>
                    ))}
                </ToggleGroup>
            </header>

            <LeaderTable ops={filtered} operatorsById={operatorsById} isLoading={isLoading} />
        </section>
    );
}

function LeaderTable({ ops, operatorsById, isLoading }: { ops: IOperatorPopularity[]; operatorsById: Map<string, IOperatorIndexEntry>; isLoading: boolean }) {
    const t: TypedT<typeof messages> = useT("gacha");
    const f = useFormatters();

    if (isLoading && ops.length === 0) {
        return (
            <div className="flex flex-col gap-2.5 py-2">
                {Array.from({ length: 6 }).map((_, i) => (
                    // biome-ignore lint/suspicious/noArrayIndexKey: loading skeleton
                    <div key={i} className="flex items-center gap-3">
                        <Skeleton className="h-9 w-9 shrink-0 rounded-lg" />
                        <div className="flex flex-1 flex-col gap-1.5">
                            <Skeleton className="h-3 w-32 rounded" />
                            <Skeleton className="h-2 w-20 rounded" />
                        </div>
                        <Skeleton className="h-3 w-16 rounded" />
                    </div>
                ))}
            </div>
        );
    }

    if (ops.length === 0) {
        return <div className="py-8 text-center font-sans text-muted-foreground text-sm">{t("community.leaderboard.empty")}</div>;
    }

    const max = Math.max(...ops.map((o) => o.pullCount));

    return (
        <div className="-mx-1 max-h-105 min-h-0 overflow-y-auto pr-1 pl-1 [scrollbar-color:var(--border)_transparent] [scrollbar-width:thin] lg:max-h-none lg:flex-1">
            <table className="w-full border-collapse">
                <thead className="sticky top-0 z-10 bg-card">
                    <tr>
                        <th className="kicker w-8 border-border border-b bg-card px-1.5 py-2 text-left sm:w-10 sm:px-2">#</th>
                        <th className="kicker border-border border-b bg-card px-1.5 py-2 text-left sm:px-2">{t("community.leaderboard.col.operator")}</th>
                        <th className="kicker border-border border-b bg-card px-1.5 py-2 text-right sm:px-2 sm:text-left">{t("community.leaderboard.col.pulls")}</th>
                        <th className="kicker hidden border-border border-b bg-card px-2 py-2 text-left sm:table-cell">{t("community.leaderboard.col.share")}</th>
                    </tr>
                </thead>
                <tbody>
                    {ops.map((op, i) => {
                        const indexEntry = operatorsById.get(op.charId);
                        const name = op.charName || indexEntry?.name || op.charId;
                        const role = indexEntry ? professionLabel(indexEntry) : "";
                        const bar = (op.pullCount / max) * 100;
                        return (
                            <tr key={op.charId} className="not-last:border-border/50 not-last:border-b">
                                <td className="w-8 whitespace-nowrap px-1.5 py-2.5 align-middle font-medium font-mono text-[11px] text-muted-foreground tabular-nums sm:w-10 sm:px-2">{String(i + 1).padStart(2, "0")}</td>
                                <td className="px-1.5 py-2.5 align-middle sm:px-2">
                                    <div className="flex items-center gap-2 sm:gap-3">
                                        <span
                                            className={`relative inline-flex h-8 w-8 shrink-0 items-center justify-center overflow-hidden rounded-lg font-bold text-[11px] text-[oklch(0.18_0.04_30/0.85)] tracking-[-0.04em] shadow-[inset_0_1px_0_oklch(1_0_0/0.4),0_0_0_1px_oklch(0_0_0/0.1)] sm:h-9 sm:w-9 rarity-${op.rarity}`}
                                            style={{ background: rarityGradient(op.rarity) }}
                                        >
                                            <OperatorAvatar charId={op.charId} name={name} className="block h-full w-full object-cover" />
                                        </span>
                                        <div className="min-w-0 flex-1">
                                            <div className="truncate font-sans font-semibold text-[13px] text-foreground sm:text-sm">{name}</div>
                                            <div className="mt-0.5 flex flex-wrap items-center gap-1.5 font-mono text-[10.5px] text-muted-foreground sm:gap-2">
                                                <span style={{ color: rarityInk(op.rarity) }} className="text-[11px] tracking-wider">
                                                    {"★".repeat(op.rarity)}
                                                </span>
                                                {role ? <span className="hidden sm:inline">· {role}</span> : null}
                                                <span className="font-mono text-[10px] text-muted-foreground tabular-nums sm:hidden">· {fmtPct(op.percentage, 2)}</span>
                                            </div>
                                        </div>
                                    </div>
                                </td>
                                <td className="whitespace-nowrap px-1.5 py-2.5 text-right align-middle font-medium font-mono text-[12px] text-foreground tabular-nums sm:w-25 sm:px-2 sm:pl-4 sm:text-left sm:text-[13px]">{f.number(op.pullCount)}</td>
                                <td className="hidden w-45 px-2 py-2.5 pr-2 align-middle sm:table-cell">
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
