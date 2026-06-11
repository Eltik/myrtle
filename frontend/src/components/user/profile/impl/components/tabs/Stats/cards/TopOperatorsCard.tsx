import { Check, Medal } from "lucide-react";
import { useMemo } from "react";
import { eliteIcon } from "#/components/operators/detail/impl/assets";
import { RARITY_COLORS } from "#/components/operators/list/impl/constants";
import { PreviewCard, PreviewCardPopup, PreviewCardTrigger } from "#/components/ui/preview-card";
import type { IRosterEntry } from "#/lib/api/user";
import { rarityToNumber } from "#/lib/utils";
import type { IOperatorListItem } from "#/types/operators";
import { operatorCompleteness, operatorMissing, ownedAvatar, parseOperatorName, RARITY_WEIGHT, specializedIcon } from "../../Roster/helpers.card";
import { PALETTE } from "../palette";
import { Kicker, StatCard } from "../primitives";

const TOP_OPERATOR_LIMIT = 8;

interface IRanked {
    entry: IRosterEntry;
    op: IOperatorListItem;
    rarity: number;
    completeness: number;
    score: number;
}

interface ITopOperatorsCardProps {
    roster: IRosterEntry[];
    operatorsStatic: IOperatorListItem[];
}

export function TopOperatorsCard({ roster, operatorsStatic }: ITopOperatorsCardProps) {
    const top = useMemo<IRanked[]>(() => {
        const opMap = new Map<string, IOperatorListItem>();
        for (const op of operatorsStatic) if (op.id) opMap.set(op.id, op);

        const ranked: IRanked[] = [];
        for (const entry of roster) {
            const op = opMap.get(entry.operator_id);
            if (!op || op.isNotObtainable) continue;
            const rarity = rarityToNumber(op.rarity);
            const completeness = operatorCompleteness(entry, op, rarity);
            ranked.push({ entry, op, rarity, completeness, score: completeness * (RARITY_WEIGHT[rarity] ?? 0.15) });
        }
        ranked.sort((a, b) => b.score - a.score);
        return ranked.slice(0, TOP_OPERATOR_LIMIT);
    }, [roster, operatorsStatic]);

    if (top.length === 0) return null;

    return (
        <StatCard className="sm:col-span-2" color={PALETTE.top}>
            <div className="p-4 pb-3 sm:p-5 sm:pb-3">
                <Kicker icon={Medal} label="Top Operators" />
            </div>
            <div className="grid grid-cols-2 gap-px bg-border sm:grid-cols-4">
                {top.map((ranked, i) => (
                    <TopOperatorTile key={ranked.entry.operator_id} ranked={ranked} rank={i + 1} />
                ))}
            </div>
        </StatCard>
    );
}

function TopOperatorTile({ ranked, rank }: { ranked: IRanked; rank: number }) {
    const { entry, op, rarity } = ranked;
    const rarityColor = RARITY_COLORS[rarity] ?? "#ffffff";
    const { displayName, subtitle } = parseOperatorName(op.name);
    const avatarURL = ownedAvatar(entry.operator_id, entry.skin_id);
    const hasMasteries = entry.masteries.some((m) => m.mastery > 0);
    const completenessPct = Math.round(ranked.completeness * 100);
    const missing = useMemo(() => operatorMissing(entry, op), [entry, op]);

    return (
        <PreviewCard>
            <PreviewCardTrigger>
                <div className="relative flex flex-col gap-2.5 bg-card p-3 sm:p-4">
                    <div aria-hidden className="absolute inset-x-0 top-0 h-0.5" style={{ background: rarityColor, opacity: 0.55 }} />
                    <div className="absolute top-2.5 right-2.5 font-bold font-mono text-[10px] text-muted-foreground/35 tabular-nums">#{rank}</div>

                    <div className="flex items-center gap-2.5 pt-0.5">
                        <div className="relative shrink-0">
                            <img alt={displayName} className="h-10 w-10 rounded-lg object-cover" src={avatarURL} style={{ background: `${rarityColor}18` }} />
                            <img alt={`E${entry.elite}`} className="absolute -right-1.5 -bottom-1.5 h-4 w-4" src={eliteIcon(entry.elite)} />
                        </div>
                        <div className="min-w-0 flex-1">
                            <p className="truncate font-semibold text-sm leading-tight" style={{ maxWidth: "9rem" }}>
                                {displayName}
                            </p>
                            {subtitle && (
                                <p className="truncate text-[9px] text-muted-foreground/60 leading-tight" style={{ maxWidth: "9rem" }}>
                                    {subtitle}
                                </p>
                            )}
                            <p className="font-mono text-[10.5px] text-muted-foreground/70">Lv {entry.level}</p>
                        </div>
                    </div>

                    {hasMasteries && <div className="flex gap-1">{entry.masteries.map((m) => (m.mastery > 0 ? <img alt={`M${m.mastery}`} className="h-4 w-4 opacity-90" key={m.index} src={specializedIcon(m.mastery)} /> : <div className="h-4 w-4 rounded-sm bg-muted/40" key={m.index} />))}</div>}

                    <div className="mt-auto flex items-center gap-2 pt-0.5">
                        <div aria-label={`${completenessPct}% complete`} aria-valuemax={100} aria-valuemin={0} aria-valuenow={completenessPct} className="h-1 flex-1 overflow-hidden rounded-full bg-muted/40" role="progressbar">
                            <div className="h-full rounded-full" style={{ width: `${completenessPct}%`, background: rarityColor, opacity: 0.7 }} />
                        </div>
                        <span className="font-mono text-[10px] text-muted-foreground/70 tabular-nums">{completenessPct}%</span>
                    </div>
                </div>
            </PreviewCardTrigger>
            <PreviewCardPopup className="w-56">
                <div className="flex flex-col gap-2.5">
                    <div className="flex items-center justify-between gap-3">
                        <span className="truncate font-semibold text-sm leading-tight">
                            {displayName}
                            {subtitle && <span className="ml-1 font-normal text-muted-foreground text-xs">{subtitle}</span>}
                        </span>
                        <span className="shrink-0 font-mono text-muted-foreground text-xs tabular-nums">{completenessPct}%</span>
                    </div>
                    {missing.length === 0 ? (
                        <div className="flex items-center gap-1.5 font-medium text-emerald-500 text-xs">
                            <Check className="h-3.5 w-3.5" />
                            Fully built
                        </div>
                    ) : (
                        <div className="flex flex-col gap-1.5">
                            <span className="font-medium text-[10px] text-muted-foreground/60 uppercase tracking-wide">Remaining</span>
                            <ul className="flex flex-col gap-1">
                                {missing.map((gap) => (
                                    <li className="flex items-center gap-2 text-foreground/80 text-xs" key={gap.tag}>
                                        <span aria-hidden className="h-1 w-1 shrink-0 rounded-full" style={{ background: rarityColor }} />
                                        {gap.label}
                                    </li>
                                ))}
                            </ul>
                        </div>
                    )}
                </div>
            </PreviewCardPopup>
        </PreviewCard>
    );
}
