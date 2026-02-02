"use client";

import Image from "next/image";
import { memo } from "react";
import { MorphingDialog, MorphingDialogContainer, MorphingDialogContent, MorphingDialogTrigger } from "~/components/ui/motion-primitives/morphing-dialog";
import { HoverCard, HoverCardContent, HoverCardTrigger } from "~/components/ui/shadcn/hover-card";
import { cn } from "~/lib/utils";
import type { Enemy } from "~/types/api";
import { HOVER_DELAY, LEVEL_BAR_COLORS, LEVEL_BLUR_COLORS } from "../../constants";

interface EnemyCardGridProps {
    enemy: Enemy;
    isHovered?: boolean;
    shouldGrayscale?: boolean;
    onHoverChange?: (isOpen: boolean) => void;
}

export const EnemyCardGrid = memo(function EnemyCardGrid({ enemy, isHovered = false, shouldGrayscale = false, onHoverChange }: EnemyCardGridProps) {
    const levelColor = LEVEL_BAR_COLORS[enemy.enemyLevel] ?? "#71717a";
    const levelBlurColor = LEVEL_BLUR_COLORS[enemy.enemyLevel] ?? "#a1a1aa";

    const cardContent = (
        <MorphingDialogTrigger className="block w-full">
            {/* Card trigger content goes here */}
            <div className="group card-hover-transition relative flex aspect-2/3 overflow-clip rounded-md border border-muted/50 bg-card contain-content hover:rounded-lg">
                <div className={cn("absolute inset-0 origin-center transform-gpu transition-all duration-200 ease-out group-hover:scale-105", shouldGrayscale && "grayscale", isHovered && "grayscale-0")}>
                    <Image alt={`${enemy.name} Portrait`} className="h-full w-full rounded-lg object-contain" decoding="async" fill loading="lazy" src={`/api/cdn${enemy.portrait}`} />
                </div>
                <div className="absolute inset-x-0 bottom-0 z-10">
                    <div className="relative">
                        <div className="h-12 w-full bg-background/80 backdrop-blur-sm" />
                        <h2 className="absolute bottom-1 left-1 line-clamp-2 max-w-[92%] font-bold text-xs uppercase opacity-60 opacity-transition group-hover:opacity-100 sm:text-sm md:text-sm">{enemy.name}</h2>
                        {/* Class icon */}
                        <div className="card-hover-transition absolute right-1 bottom-1 flex scale-75 items-center opacity-0 group-hover:scale-100 group-hover:opacity-100">
                            <div className="h-4 w-4 md:h-6 md:w-6">{/* todo */}</div>
                        </div>
                        {/* Level color bar */}
                        <div className={cn("absolute bottom-0 h-0.5 w-full grayscale-transition", shouldGrayscale && "grayscale", isHovered && "grayscale-0")} style={{ backgroundColor: levelColor }} />
                        <div className={cn("absolute -bottom-0.5 h-1 w-full blur-sm grayscale-transition", shouldGrayscale && "grayscale", isHovered && "grayscale-0")} style={{ backgroundColor: levelBlurColor }} />
                    </div>
                </div>
            </div>
        </MorphingDialogTrigger>
    );

    return (
        <MorphingDialog
            transition={{
                type: "spring",
                bounce: 0.1,
                duration: 0.4,
            }}
        >
            <HoverCard closeDelay={50} onOpenChange={onHoverChange} openDelay={HOVER_DELAY}>
                <HoverCardTrigger asChild>{cardContent}</HoverCardTrigger>
                <HoverCardContent className="w-80 p-4" side="top">
                    {/* Hover card content goes here */}
                </HoverCardContent>
            </HoverCard>
            <MorphingDialogContainer>
                <MorphingDialogContent>
                    {/* Dialog content goes here */}
                    <div />
                </MorphingDialogContent>
            </MorphingDialogContainer>
        </MorphingDialog>
    );
});
