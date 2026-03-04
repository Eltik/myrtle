"use client";

import { memo } from "react";
import type { OperatorFromList } from "~/types/api";
import { OperatorCardCompact } from "./ui/impl/operator-card-compact";
import { OperatorCardGrid } from "./ui/impl/operator-card-grid";
import { OperatorCardList } from "./ui/impl/operator-card-list";

interface OperatorCardProps {
    operator: OperatorFromList;
    viewMode: "grid" | "list" | "compact";
    listColumns?: number;
    isHovered?: boolean;
    shouldGrayscale?: boolean;
    onHoverChange?: (isOpen: boolean) => void;
}

export const OperatorCard = memo(function OperatorCard({ operator, viewMode, listColumns = 1, isHovered = false, shouldGrayscale = false, onHoverChange }: OperatorCardProps) {
    if (viewMode === "compact") {
        return <OperatorCardCompact operator={operator} />;
    }

    if (viewMode === "list") {
        return <OperatorCardList isHovered={isHovered} listColumns={listColumns} operator={operator} shouldGrayscale={shouldGrayscale} />;
    }

    return <OperatorCardGrid isHovered={isHovered} onHoverChange={onHoverChange} operator={operator} shouldGrayscale={shouldGrayscale} />;
});
