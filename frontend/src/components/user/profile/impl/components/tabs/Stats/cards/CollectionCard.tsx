import { Users } from "lucide-react";
import { cn } from "#/lib/utils";
import { PALETTE } from "../palette";
import { CARD_PADDING, KICKER_TEXT, Kicker, MetricRow, StatCard } from "../primitives";

interface ICollectionCardProps {
    totalOwned: number;
    totalAvailable: number;
    collectionPercentage: number;
}

export function CollectionCard({ totalOwned, totalAvailable, collectionPercentage }: ICollectionCardProps) {
    const pctLabel = `${(Math.round(collectionPercentage * 100) / 100).toFixed(2)}%`;

    return (
        <StatCard color={PALETTE.collection}>
            <div className={cn("flex h-full flex-col gap-5", CARD_PADDING)}>
                <Kicker icon={Users} label="Operator Collection" />
                <div className="flex flex-1 flex-col justify-between gap-5">
                    <div className="flex flex-col items-center gap-1.5 py-1">
                        <div className="flex items-baseline gap-1.5">
                            <span
                                className="font-bold tabular-nums leading-none"
                                style={{
                                    fontSize: "clamp(2.25rem, 3vw + 1rem, 3rem)",
                                    letterSpacing: "-0.03em",
                                    color: PALETTE.collection,
                                }}
                            >
                                {totalOwned}
                            </span>
                            <span className="font-medium font-mono text-lg text-muted-foreground/50 tabular-nums">/ {totalAvailable}</span>
                        </div>
                        <span className={KICKER_TEXT}>operators collected</span>
                    </div>
                    <MetricRow color={PALETTE.collection} label="Completion" pct={collectionPercentage} value={pctLabel} />
                </div>
            </div>
        </StatCard>
    );
}
