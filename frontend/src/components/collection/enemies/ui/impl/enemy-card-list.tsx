"use client";

import Image from "next/image";
import { useTheme } from "next-themes";
import { memo, useState } from "react";
import { MorphingDialog, MorphingDialogContainer, MorphingDialogContent, MorphingDialogTrigger } from "~/components/ui/motion-primitives/morphing-dialog";
import { capitalize, cn } from "~/lib/utils";
import type { Enemy } from "~/types/api";
import { APPLY_WAY_DISPLAY, LEVEL_BAR_COLORS, LEVEL_TEXT_COLORS, LEVEL_TEXT_COLORS_LIGHT } from "../../constants";
import { formatDamageType } from "../../enemy-list/impl/helpers";
import { EnemyDetailDialog } from "./enemy-detail-dialog";
import { EnemyLevelLogo } from "./enemy-level-logo";
import { EnemyPlaceholder } from "./enemy-placeholder";

interface EnemyCardListProps {
    enemy: Enemy;
    listColumns?: number;
    isHovered?: boolean;
    shouldGrayscale?: boolean;
}

export const EnemyCardList = memo(function EnemyCardList({ enemy, listColumns = 1, isHovered = false, shouldGrayscale = false }: EnemyCardListProps) {
    const { resolvedTheme } = useTheme();
    const [imgError, setImgError] = useState(false);
    const useCompactLayout = listColumns > 1;

    // Color calculations
    const levelColor = LEVEL_BAR_COLORS[enemy.enemyLevel] ?? "#71717a";
    const levelTextColor = (resolvedTheme === "light" ? LEVEL_TEXT_COLORS_LIGHT : LEVEL_TEXT_COLORS)[enemy.enemyLevel] ?? "#a1a1aa";

    // Get first level stats for attack type
    const firstLevelStats = enemy.stats?.levels?.[0];
    const attackType = firstLevelStats?.applyWay ? APPLY_WAY_DISPLAY[firstLevelStats.applyWay] : "None";

    // Format damage types
    const damageTypesDisplay = enemy.damageType.length > 0 ? enemy.damageType.map((dt) => formatDamageType(dt)).join(", ") : "Unknown";

    const cardContent = (
        <div className={cn("group card-hover-transition relative flex items-center gap-3 rounded-lg border border-transparent bg-card/50 px-3 py-2.5 grayscale-transition contain-layout hover:border-border hover:bg-card", shouldGrayscale && "grayscale", isHovered && "grayscale-0")}>
            {/* Level indicator bar (left) */}
            <div className="absolute top-1/2 left-0 h-8 w-0.5 -translate-y-1/2 rounded-full opacity-60 opacity-transition group-hover:opacity-100" style={{ backgroundColor: levelColor }} />

            {/* Portrait */}
            <div className="relative h-12 w-12 shrink-0 overflow-hidden rounded-md border border-border/50 bg-background">
                <div className="absolute inset-0 transform-gpu transition-transform duration-200 ease-out group-hover:scale-110">
                    {imgError || !enemy.portrait ? <EnemyPlaceholder className="h-full w-full" /> : <Image alt={enemy.name} className="object-contain" fill onError={() => setImgError(true)} src={`/api/cdn${enemy.portrait}`} />}
                </div>
            </div>

            {/* Desktop Layout: Name column */}
            {!useCompactLayout && (
                <div className="hidden min-w-0 flex-1 md:block">
                    <span className="truncate font-semibold text-foreground text-sm uppercase tracking-wide">{enemy.name}</span>
                </div>
            )}

            {/* Desktop: Level with logo */}
            {!useCompactLayout && (
                <div className="hidden w-24 shrink-0 items-center gap-2 md:flex">
                    {enemy.enemyLevel !== "NORMAL" && (
                        <div className="flex h-5 w-5 items-center justify-center">
                            <EnemyLevelLogo className="opacity-60 transition-opacity group-hover:opacity-100" level={enemy.enemyLevel} size={20} />
                        </div>
                    )}
                    <span className="font-medium text-sm" style={{ color: levelTextColor }}>
                        {capitalize(enemy.enemyLevel)}
                    </span>
                </div>
            )}

            {/* Desktop: Damage Types */}
            {!useCompactLayout && (
                <div className="hidden w-32 shrink-0 md:block">
                    <span className="truncate text-muted-foreground text-sm">{damageTypesDisplay}</span>
                </div>
            )}

            {/* Desktop: Attack Type (lg+) */}
            {!useCompactLayout && (
                <div className="hidden w-24 shrink-0 lg:block">
                    <span className="text-muted-foreground text-sm">{attackType}</span>
                </div>
            )}

            {/* Compact/Mobile Layout */}
            <div className={cn("flex min-w-0 flex-1 flex-col gap-1", useCompactLayout ? "flex" : "md:hidden")}>
                {/* Row 1: Name + Level Logo */}
                <div className="flex items-center gap-2">
                    <span className="truncate font-semibold text-foreground text-sm uppercase tracking-wide">{enemy.name}</span>
                    {enemy.enemyLevel !== "NORMAL" && (
                        <div className="flex h-4 w-4 shrink-0 items-center justify-center opacity-60">
                            <EnemyLevelLogo level={enemy.enemyLevel} size={16} />
                        </div>
                    )}
                </div>

                {/* Row 2: Level + Damage Types + Attack Type */}
                <div className="flex items-center gap-2 text-xs">
                    <span className="font-medium" style={{ color: levelTextColor }}>
                        {capitalize(enemy.enemyLevel)}
                    </span>

                    <span className="text-muted-foreground/50">·</span>

                    <span className="text-muted-foreground">{damageTypesDisplay}</span>

                    <span className="text-muted-foreground/50">·</span>

                    <span className="truncate text-muted-foreground">{attackType}</span>
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
            <MorphingDialogTrigger className="block w-full text-left">{cardContent}</MorphingDialogTrigger>
            <MorphingDialogContainer>
                <MorphingDialogContent>
                    <EnemyDetailDialog enemy={enemy} />
                </MorphingDialogContent>
            </MorphingDialogContainer>
        </MorphingDialog>
    );
});
