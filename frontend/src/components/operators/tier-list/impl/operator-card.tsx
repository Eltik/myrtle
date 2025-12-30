"use client";

import Image from "next/image";
import Link from "next/link";
import { memo } from "react";
import { cn, rarityToNumber } from "~/lib/utils";
import type { OperatorFromList } from "~/types/api/operators";
import { RARITY_BLUR_COLORS, RARITY_COLORS } from "../../list/constants";
import { OperatorTooltip } from "./operator-tooltip";

interface TierOperatorCardProps {
    operator: OperatorFromList;
    isHovered?: boolean;
    shouldGrayscale?: boolean;
    onHoverChange?: (isHovered: boolean) => void;
}

export const TierOperatorCard = memo(function TierOperatorCard({ operator, isHovered = false, shouldGrayscale = false, onHoverChange }: TierOperatorCardProps) {
    const rarityNum = rarityToNumber(operator.rarity);
    const rarityColor = RARITY_COLORS[rarityNum] ?? "#ffffff";
    const rarityBlurColor = RARITY_BLUR_COLORS[rarityNum] ?? "#aaaaaa";
    const operatorId = operator.id ?? "";

    const cardContent = (
        <Link aria-label={`View details for ${operator.name}`} className="group card-hover-transition relative flex aspect-3/4 overflow-clip rounded-md border border-muted/50 bg-card contain-content hover:rounded-lg" href={`/operators?id=${operatorId}`}>
            {/* Portrait */}
            <div className={cn("absolute inset-0 origin-center transform-gpu transition-all duration-200 ease-out group-hover:scale-110", shouldGrayscale && "grayscale", isHovered && "grayscale-0")}>
                <Image alt={`${operator.name} Portrait`} className="h-full w-full rounded-lg object-cover object-top" decoding="async" fill loading="lazy" src={`/api/cdn${operator.portrait}`} />
            </div>

            {/* Rarity indicator */}
            <div className={cn("absolute bottom-0 h-1 w-full grayscale-transition", shouldGrayscale && "grayscale", isHovered && "grayscale-0")} style={{ backgroundColor: rarityColor }} />
            <div className={cn("-bottom-0.5 absolute h-1 w-full blur-sm grayscale-transition", shouldGrayscale && "grayscale", isHovered && "grayscale-0")} style={{ backgroundColor: rarityBlurColor }} />

            {/* Hover overlay with name */}
            <div className="absolute inset-0 flex items-end bg-linear-to-t from-background/50 to-transparent opacity-0 transition-opacity duration-200 group-hover:opacity-100">
                <p className="w-full truncate px-1 pb-2 text-center font-medium text-foreground text-xs">{operator.name}</p>
            </div>
        </Link>
    );

    return (
        <OperatorTooltip onHoverChange={onHoverChange} operator={operator}>
            {cardContent}
        </OperatorTooltip>
    );
});
