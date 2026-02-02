"use client";

import { memo } from "react";
import type { Enemy } from "~/types/api";

interface EnemyCardListProps {
    enemy: Enemy;
    listColumns?: number;
    isHovered?: boolean;
    shouldGrayscale?: boolean;
}

export const EnemyCardList = memo(function EnemyCardList({ enemy, listColumns = 1, isHovered = false, shouldGrayscale = false }: EnemyCardListProps) {
    return <></>;
});
