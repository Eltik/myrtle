"use client";

import Image from "next/image";
import Link from "next/link";
import { memo } from "react";
import { HoverCard, HoverCardContent, HoverCardTrigger } from "~/components/ui/shadcn/hover-card";
import { capitalize, cn, formatSubProfession, rarityToNumber } from "~/lib/utils";
import type { OperatorFromList } from "~/types/api/operators";
import { HOVER_DELAY, RARITY_BLUR_COLORS, RARITY_COLORS } from "../../list/constants";
import { FactionLogo } from "../../list/ui/impl/faction-logo";

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
        <Link aria-label={`View details for ${operator.name}`} className="group card-hover-transition relative flex aspect-square overflow-clip rounded-md border border-muted/50 bg-card contain-content hover:rounded-lg" href={`/operators?id=${operatorId}`}>
            {/* Portrait */}
            <div className={cn("absolute inset-0 origin-center transform-gpu transition-all duration-200 ease-out group-hover:scale-110", shouldGrayscale && "grayscale", isHovered && "grayscale-0")}>
                <Image alt={`${operator.name} Portrait`} className="h-full w-full rounded-lg object-cover" decoding="async" fill loading="lazy" src={`/api/cdn${operator.portrait}`} />
            </div>

            {/* Rarity indicator */}
            <div className={cn("absolute bottom-0 h-1 w-full grayscale-transition", shouldGrayscale && "grayscale", isHovered && "grayscale-0")} style={{ backgroundColor: rarityColor }} />
            <div className={cn("-bottom-0.5 absolute h-1 w-full blur-sm grayscale-transition", shouldGrayscale && "grayscale", isHovered && "grayscale-0")} style={{ backgroundColor: rarityBlurColor }} />

            {/* Hover overlay with name */}
            <div className="absolute inset-0 flex items-end bg-linear-to-t from-background/90 to-transparent opacity-0 transition-opacity duration-200 group-hover:opacity-100">
                <p className="w-full truncate px-1 pb-2 text-center font-medium text-foreground text-xs">{operator.name}</p>
            </div>
        </Link>
    );

    return (
        <HoverCard closeDelay={50} onOpenChange={onHoverChange} openDelay={HOVER_DELAY}>
            <HoverCardTrigger asChild>{cardContent}</HoverCardTrigger>
            <HoverCardContent className="w-80 p-4" side="top">
                <div className="flex items-start space-x-4">
                    {/* Avatar */}
                    <div className="relative h-16 w-16 shrink-0">
                        <Image alt={`${operator.name} Avatar`} className="rounded-md object-cover" fill src={`/api/cdn${operator.portrait}`} />
                    </div>
                    <div className="grow space-y-1">
                        {/* Name and faction */}
                        <div className="flex items-center justify-between">
                            <h4 className="font-semibold text-base">{operator.name}</h4>
                            <div className="h-6 w-6 shrink-0">
                                <FactionLogo className="object-contain" groupId={operator.groupId} nationId={operator.nationId} size={24} teamId={operator.teamId} />
                            </div>
                        </div>
                        {/* Rarity and class info */}
                        <p className="font-semibold text-xs" style={{ color: rarityColor }}>
                            {`${rarityNum}★ ${formatSubProfession(operator.subProfessionId.toLowerCase())}`}
                        </p>
                        {/* Position and race */}
                        <div className="flex space-x-2 pt-1 text-muted-foreground text-xs">
                            <span>{capitalize(operator.position?.toLowerCase() ?? "Unknown")}</span>
                            {operator.profile?.basicInfo?.race && (
                                <>
                                    <span>•</span>
                                    <span>{operator.profile.basicInfo.race}</span>
                                </>
                            )}
                        </div>
                    </div>
                </div>
            </HoverCardContent>
        </HoverCard>
    );
});
