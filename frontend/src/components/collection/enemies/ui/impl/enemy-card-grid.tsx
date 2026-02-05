"use client";

import Image from "next/image";
import { useTheme } from "next-themes";
import { memo, useState } from "react";
import { MorphingDialog, MorphingDialogContainer, MorphingDialogContent, MorphingDialogTrigger } from "~/components/ui/motion-primitives/morphing-dialog";
import { HoverCard, HoverCardContent, HoverCardTrigger } from "~/components/ui/shadcn/hover-card";
import { capitalize, cn } from "~/lib/utils";
import type { Enemy } from "~/types/api";
import { HOVER_DELAY, LEVEL_BAR_COLORS, LEVEL_BLUR_COLORS, LEVEL_TEXT_COLORS, LEVEL_TEXT_COLORS_LIGHT } from "../../constants";
import { formatDamageType } from "../../enemy-list/impl/helpers";
import { EnemyDetailDialog } from "./enemy-detail-dialog";
import { EnemyLevelLogo } from "./enemy-level-logo";
import { EnemyPlaceholder } from "./enemy-placeholder";

interface EnemyCardGridProps {
    enemy: Enemy;
    isHovered?: boolean;
    shouldGrayscale?: boolean;
    onHoverChange?: (isOpen: boolean) => void;
}

export const EnemyCardGrid = memo(function EnemyCardGrid({ enemy, isHovered = false, shouldGrayscale = false, onHoverChange }: EnemyCardGridProps) {
    const { resolvedTheme } = useTheme();
    const [imgError, setImgError] = useState(false);
    const levelColor = LEVEL_BAR_COLORS[enemy.enemyLevel] ?? "#71717a";
    const levelBlurColor = LEVEL_BLUR_COLORS[enemy.enemyLevel] ?? "#a1a1aa";
    const levelTextColor = (resolvedTheme === "light" ? LEVEL_TEXT_COLORS_LIGHT : LEVEL_TEXT_COLORS)[enemy.enemyLevel] ?? "#a1a1aa";

    const cardVisual = (
        <div className="group card-hover-transition relative flex aspect-square overflow-clip rounded-md border border-muted/50 bg-card contain-content hover:rounded-lg">
            {/* Enemy image */}
            <div className={cn("absolute inset-0 origin-center transform-gpu transition-all duration-200 ease-out group-hover:scale-105", shouldGrayscale && "grayscale", isHovered && "grayscale-0")}>
                {imgError || !enemy.portrait ? <EnemyPlaceholder className="h-full w-full p-4" /> : <Image alt={`${enemy.name} Portrait`} className="h-full w-full rounded-lg object-contain" decoding="async" fill loading="lazy" onError={() => setImgError(true)} src={`/api/cdn${enemy.portrait}`} />}
            </div>

            {/* Bottom info bar */}
            <div className="absolute inset-x-0 bottom-0 z-10">
                <div className="relative">
                    <div className="h-12 w-full bg-background/80 backdrop-blur-sm" />
                    <h2 className="absolute bottom-1 left-1 line-clamp-2 max-w-[92%] font-bold text-xs uppercase opacity-60 opacity-transition group-hover:opacity-100 sm:text-sm md:text-sm">{enemy.name}</h2>

                    {/* Icon placeholder */}
                    <div className="card-hover-transition absolute right-1 bottom-1 flex scale-75 items-center opacity-0 group-hover:scale-100 group-hover:opacity-100">
                        <div className="h-4 w-4 md:h-6 md:w-6">{enemy.enemyLevel !== "NORMAL" ? <EnemyLevelLogo level={enemy.enemyLevel} size={50} /> : null}</div>
                    </div>

                    {/* Level color bar */}
                    <div className={cn("absolute bottom-0 h-0.5 w-full grayscale-transition", shouldGrayscale && "grayscale", isHovered && "grayscale-0")} style={{ backgroundColor: levelColor }} />
                    <div className={cn("absolute -bottom-0.5 h-1 w-full blur-sm grayscale-transition", shouldGrayscale && "grayscale", isHovered && "grayscale-0")} style={{ backgroundColor: levelBlurColor }} />
                </div>
            </div>
        </div>
    );

    return (
        <MorphingDialog
            transition={{
                type: "spring",
                bounce: 0.1,
                duration: 0.4,
            }}
        >
            <MorphingDialogTrigger className="block w-full">
                <HoverCard closeDelay={50} onOpenChange={onHoverChange} openDelay={HOVER_DELAY}>
                    <HoverCardTrigger asChild>
                        <div>{cardVisual}</div>
                    </HoverCardTrigger>
                    <HoverCardContent className="w-80 p-4" side="top">
                        <div className="flex items-start space-x-4">
                            {/* Avatar */}
                            <div className="relative h-16 w-16 shrink-0">{imgError || !enemy.portrait ? <EnemyPlaceholder className="h-full w-full" /> : <Image alt={`${enemy.name} Avatar`} className="rounded-md object-cover" fill src={`/api/cdn${enemy.portrait}`} />}</div>
                            <div className="grow space-y-1">
                                {/* Name and level */}
                                <div className="flex items-center justify-between">
                                    <h4 className="font-semibold text-base">{enemy.name}</h4>
                                    <div className="h-6 w-6 shrink-0">{enemy.enemyLevel !== "NORMAL" ? <EnemyLevelLogo className="object-contain" level={enemy.enemyLevel} size={24} /> : null}</div>
                                </div>
                                {/* Level info */}
                                <p className="font-semibold text-xs" style={{ color: levelTextColor }}>
                                    {capitalize(enemy.enemyLevel)}
                                </p>
                                {/* Damage type and index */}
                                <div className="flex space-x-2 pt-1 text-muted-foreground text-xs">
                                    <span>{enemy.damageType.length > 0 ? enemy.damageType.map((damageType) => formatDamageType(damageType)).join(", ") : "Unknown"}</span>
                                    <span>•</span>
                                    <span>{enemy.enemyIndex}</span>
                                </div>
                            </div>
                        </div>
                    </HoverCardContent>
                </HoverCard>
            </MorphingDialogTrigger>
            <MorphingDialogContainer>
                <MorphingDialogContent>
                    <EnemyDetailDialog enemy={enemy} />
                </MorphingDialogContent>
            </MorphingDialogContainer>
        </MorphingDialog>
    );
});
