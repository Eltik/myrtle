import { Link } from "@tanstack/react-router";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { HoverCard, HoverCardContent, HoverCardTrigger } from "#/components/ui/preview-card";
import type { ITierOperator } from "#/lib/api/tier-lists";
import { RARITY_HEX_MUTED } from "#/lib/utils";
import styles from "./TierListDetail.module.css";
import { TierOperatorPreview } from "./TierOperatorPreview";

interface IOperatorTileProps {
    operator: ITierOperator;
}

export function OperatorTile({ operator }: IOperatorTileProps) {
    const color = RARITY_HEX_MUTED[operator.rarity] ?? RARITY_HEX_MUTED[1];

    return (
        <HoverCard>
            <HoverCardTrigger
                render={
                    <Link to="/operators/$id" params={{ id: operator.id }} className={styles.opTile} style={{ ["--rarity-color" as string]: color }} aria-label={`${operator.name} (${operator.rarity}★)`}>
                        <OperatorAvatar charId={operator.id} name={operator.name} />
                        <span className={styles.opRarity} aria-hidden="true" />
                    </Link>
                }
            />
            <HoverCardContent className="w-max max-w-[calc(100vw-2rem)] p-0" sideOffset={6}>
                <TierOperatorPreview operator={operator} />
            </HoverCardContent>
        </HoverCard>
    );
}
