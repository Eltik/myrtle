import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import { CARD_PADDING, CardKicker, KICKER_TEXT, MetricRow, StatCard } from "../primitives";
import type { messages } from "./CollectionCard.messages";

interface ICollectionCardProps {
    totalOwned: number;
    totalAvailable: number;
    collectionPercentage: number;
}

export function CollectionCard({ totalOwned, totalAvailable, collectionPercentage }: ICollectionCardProps) {
    const t: TypedT<typeof messages> = useT("user");
    const pctLabel = `${(Math.round(collectionPercentage * 100) / 100).toFixed(2)}%`;

    return (
        <StatCard>
            <div className={cn("flex h-full flex-col gap-5", CARD_PADDING)}>
                <CardKicker label={t("profile.stats.collection.title")} />
                <div className="flex flex-1 flex-col justify-between gap-5">
                    <div className="flex flex-col items-center gap-1.5 py-1">
                        <div className="flex items-baseline gap-1.5">
                            <span
                                className="font-bold text-foreground tabular-nums leading-none"
                                style={{
                                    fontSize: "clamp(2.25rem, 3vw + 1rem, 3rem)",
                                    letterSpacing: "-0.03em",
                                }}
                            >
                                {totalOwned}
                            </span>
                            <span className="font-medium font-mono text-lg text-subtle-foreground tabular-nums">/ {totalAvailable}</span>
                        </div>
                        <span className={KICKER_TEXT}>{t("profile.stats.collection.caption")}</span>
                    </div>
                    <MetricRow label={t("profile.stats.collection.completion")} pct={collectionPercentage} value={pctLabel} />
                </div>
            </div>
        </StatCard>
    );
}
