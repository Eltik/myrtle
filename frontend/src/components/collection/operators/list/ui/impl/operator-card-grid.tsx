"use client";

import Image from "next/image";
import Link from "next/link";
import { useTheme } from "next-themes";
import { memo } from "react";
import { HoverCard, HoverCardContent, HoverCardTrigger } from "~/components/ui/shadcn/hover-card";
import { capitalize, cn, formatSubProfession, rarityToNumber } from "~/lib/utils";
import type { OperatorFromList } from "~/types/api";
import { HOVER_DELAY, RARITY_BLUR_COLORS, RARITY_COLORS, RARITY_COLORS_LIGHT } from "../../constants";
import { ClassIcon } from "./class-icon";
import { FactionLogo } from "./faction-logo";

interface OperatorCardGridProps {
    operator: OperatorFromList;
    isHovered?: boolean;
    shouldGrayscale?: boolean;
    onHoverChange?: (isOpen: boolean) => void;
}

export const OperatorCardGrid = memo(function OperatorCardGrid({ operator, isHovered = false, shouldGrayscale = false, onHoverChange }: OperatorCardGridProps) {
    const { resolvedTheme } = useTheme();
    const rarityNum = rarityToNumber(operator.rarity);
    const rarityColor = RARITY_COLORS[rarityNum] ?? "#ffffff";
    const rarityTextColor = (resolvedTheme === "light" ? RARITY_COLORS_LIGHT : RARITY_COLORS)[rarityNum] ?? "#ffffff";
    const rarityBlurColor = RARITY_BLUR_COLORS[rarityNum] ?? "#aaaaaa";
    const operatorId = operator.id ?? "";

    const cardContent = (
        <Link aria-label={`View details for ${operator.name}`} className="group card-hover-transition relative flex aspect-2/3 overflow-clip rounded-md border border-muted/50 bg-card contain-content hover:rounded-lg" href={`/collection/operators?id=${operatorId}`}>
            {/* Faction background */}
            <div className="absolute -translate-x-8 -translate-y-4">
                <FactionLogo className="opacity-5 opacity-transition group-hover:opacity-10" nationId={operator.nationId} size={360} teamId={operator.teamId} />
            </div>

            {/* Portrait */}
            <div className={cn("absolute inset-0 origin-center transform-gpu transition-all duration-200 ease-out group-hover:scale-105", shouldGrayscale && "grayscale", isHovered && "grayscale-0")}>
                <Image alt={`${operator.name} Portrait`} className="h-full w-full rounded-lg object-contain" decoding="async" fill loading="lazy" src={`/api/cdn${operator.portrait}`} />
            </div>

            {/* Bottom info bar */}
            <div className="absolute inset-x-0 bottom-0 z-10">
                <div className="relative">
                    <div className="h-12 w-full bg-background/80 backdrop-blur-sm" />
                    <h2 className="absolute bottom-1 left-1 line-clamp-2 max-w-[92%] font-bold text-xs uppercase opacity-60 opacity-transition group-hover:opacity-100 sm:text-sm md:text-sm">{operator.name}</h2>
                    {/* Class icon */}
                    <div className="card-hover-transition absolute right-1 bottom-1 flex scale-75 items-center opacity-0 group-hover:scale-100 group-hover:opacity-100">
                        <div className="h-4 w-4 md:h-6 md:w-6">
                            <ClassIcon profession={operator.profession} size={160} />
                        </div>
                    </div>
                    {/* Rarity color bar */}
                    <div className={cn("absolute bottom-0 h-0.5 w-full grayscale-transition", shouldGrayscale && "grayscale", isHovered && "grayscale-0")} style={{ backgroundColor: rarityColor }} />
                    <div className={cn("absolute -bottom-0.5 h-1 w-full blur-sm grayscale-transition", shouldGrayscale && "grayscale", isHovered && "grayscale-0")} style={{ backgroundColor: rarityBlurColor }} />
                </div>
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
                        <p className="font-semibold text-xs" style={{ color: rarityTextColor }}>
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
