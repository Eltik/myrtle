import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import { Bar, CARD_PADDING, CardKicker, KICKER_TEXT, StatCard } from "../primitives";
import type { messages } from "./ElitePromotionCard.messages";

interface IElitePromotionCardProps {
    eliteBreakdown: { e0: number; e1: number; e2: number; total: number };
}

/** A key in `ElitePromotionCard.messages.ts`; resolved by the card below. */
type MessageKey = keyof typeof messages & string;

const ELITE_ROWS = [
    { key: "e2", labelKey: "profile.stats.elite.e2" },
    { key: "e1", labelKey: "profile.stats.elite.e1" },
    { key: "e0", labelKey: "profile.stats.elite.e0" },
] as const satisfies ReadonlyArray<{ key: string; labelKey: MessageKey }>;

export function ElitePromotionCard({ eliteBreakdown }: IElitePromotionCardProps) {
    const t: TypedT<typeof messages> = useT("user");
    return (
        <StatCard>
            <div className={cn("flex h-full flex-col gap-5", CARD_PADDING)}>
                <CardKicker label={t("profile.stats.elite.title")} />
                <div className="flex flex-1 flex-col justify-center gap-4">
                    {ELITE_ROWS.map(({ key, labelKey }, i) => {
                        const count = eliteBreakdown[key];
                        const pct = eliteBreakdown.total > 0 ? (count / eliteBreakdown.total) * 100 : 0;
                        const isPrimary = i === 0;
                        return (
                            <div className="space-y-1.5" key={key}>
                                <div className="flex items-center justify-between">
                                    <span className={KICKER_TEXT}>{t(labelKey)}</span>
                                    <div className="flex items-baseline gap-1.5">
                                        <span className="font-bold text-sm tabular-nums">{count}</span>
                                        <span className="font-mono text-[10px] text-muted-foreground tabular-nums">{pct.toFixed(0)}%</span>
                                    </div>
                                </div>
                                <Bar dim={!isPrimary} pct={pct} />
                            </div>
                        );
                    })}
                </div>
            </div>
        </StatCard>
    );
}
