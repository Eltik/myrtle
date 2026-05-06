import { Medal } from "lucide-react";
import { useMemo } from "react";
import { eliteIcon } from "#/components/operators/detail/impl/assets";
import { RARITY_COLORS } from "#/components/operators/list/impl/constants";
import type { IRosterEntry } from "#/lib/api/user";
import { rarityToNumber } from "#/lib/utils";
import type { IOperatorListItem } from "#/types/operators";
import { MAX_LEVEL_BY_RARITY, ownedAvatar, parseOperatorName, specializedIcon } from "../../Roster/helpers.card";
import { PALETTE } from "../palette";
import { Kicker, StatCard } from "../primitives";

const TOP_OPERATOR_LIMIT = 8;
const EXCLUDED_MODULE_KEYS = ["uniequip_000", "uniequip_001"];

/**
 * Investment-weighted score for ranking the top operators.
 * Weights are intentionally rough — they only have to produce a sensible
 * ordering for the showcase.
 */
const SCORE_WEIGHTS = {
    elitePromotion: 300,
    levelProgressMax: 100,
    masteryLevel: 15,
    moduleUnlocked: 25,
    moduleMaxed: 15,
} as const;

function operatorScore(entry: IRosterEntry, rarity: number): number {
    const maxLevel = MAX_LEVEL_BY_RARITY[rarity]?.[entry.elite] ?? 90;
    const usableModules = entry.modules.filter((m) => !EXCLUDED_MODULE_KEYS.some((k) => m.id.startsWith(k)) && !m.locked && m.level > 0);
    const masteryTotal = entry.masteries.reduce((s, m) => s + m.mastery, 0);
    const moduleMaxed = usableModules.filter((m) => m.level === 3).length;

    return entry.elite * SCORE_WEIGHTS.elitePromotion + (entry.level / maxLevel) * SCORE_WEIGHTS.levelProgressMax + masteryTotal * SCORE_WEIGHTS.masteryLevel + usableModules.length * SCORE_WEIGHTS.moduleUnlocked + moduleMaxed * SCORE_WEIGHTS.moduleMaxed;
}

interface IRanked {
    entry: IRosterEntry;
    op: IOperatorListItem;
    rarity: number;
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
            ranked.push({ entry, op, rarity, score: operatorScore(entry, rarity) });
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
    const { displayName } = parseOperatorName(op.name);
    const avatarUrl = ownedAvatar(entry.operator_id, entry.skin_id);
    const hasMasteries = entry.masteries.some((m) => m.mastery > 0);

    return (
        <div className="relative flex flex-col gap-2.5 bg-card p-3 sm:p-4">
            <div aria-hidden className="absolute inset-x-0 top-0 h-0.5" style={{ background: rarityColor, opacity: 0.55 }} />
            <div className="absolute top-2.5 right-2.5 font-mono text-[10px] font-bold tabular-nums text-muted-foreground/35">#{rank}</div>

            <div className="flex items-center gap-2.5 pt-0.5">
                <div className="relative shrink-0">
                    <img alt={displayName} className="h-10 w-10 rounded-lg object-cover" src={avatarUrl} style={{ background: `${rarityColor}18` }} />
                    <img alt={`E${entry.elite}`} className="absolute -right-1.5 -bottom-1.5 h-4 w-4" src={eliteIcon(entry.elite)} />
                </div>
                <div className="min-w-0 flex-1">
                    <p className="truncate font-semibold text-sm leading-tight" style={{ maxWidth: "9rem" }}>
                        {displayName}
                    </p>
                    <p className="font-mono text-[10.5px] text-muted-foreground/70">Lv {entry.level}</p>
                </div>
            </div>

            {hasMasteries && <div className="flex gap-1">{entry.masteries.map((m) => (m.mastery > 0 ? <img alt={`M${m.mastery}`} className="h-4 w-4 opacity-90" key={m.index} src={specializedIcon(m.mastery)} /> : <div className="h-4 w-4 rounded-sm bg-muted/40" key={m.index} />))}</div>}
        </div>
    );
}
