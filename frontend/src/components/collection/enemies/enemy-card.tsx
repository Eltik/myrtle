"use client";

import { memo } from "react";
import type { Enemy } from "~/types/api";
import { EnemyCardGrid, EnemyCardList } from "./ui";

interface EnemyCardProps {
    enemy: Enemy;
    viewMode: "grid" | "list";
    listColumns?: number;
    isHovered?: boolean;
    shouldGrayscale?: boolean;
    onHoverChange?: (isOpen: boolean) => void;
}

export const EnemyCard = memo(function EnemyCard({ enemy, viewMode, listColumns = 1, isHovered = false, shouldGrayscale = false, onHoverChange }: EnemyCardProps) {
    if (viewMode === "list") {
        return <EnemyCardList enemy={enemy} isHovered={isHovered} listColumns={listColumns} shouldGrayscale={shouldGrayscale} />;
    }

    return <EnemyCardGrid enemy={enemy} isHovered={isHovered} onHoverChange={onHoverChange} shouldGrayscale={shouldGrayscale} />;
});
