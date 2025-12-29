"use client";

import { Info } from "lucide-react";
import { useState } from "react";
import { Card } from "~/components/ui/shadcn/card";
import type { TierListResponse } from "~/types/api/impl/tier-list";
import type { OperatorFromList } from "~/types/api/operators";
import { TierRow } from "./impl/tier-row";

interface TierListViewProps {
    tierListData: TierListResponse;
    operatorsData: Record<string, OperatorFromList>;
}

export function TierListView({ tierListData, operatorsData }: TierListViewProps) {
    const [hoveredOperator, setHoveredOperator] = useState<string | null>(null);
    const [isGrayscaleActive, setIsGrayscaleActive] = useState(false);

    const handleOperatorHover = (operatorId: string | null, isHovered: boolean) => {
        if (isHovered) {
            setHoveredOperator(operatorId);
            setIsGrayscaleActive(true);
        } else {
            setHoveredOperator(null);
            setIsGrayscaleActive(false);
        }
    };

    return (
        <div className="min-w-0 space-y-6">
            {/* Header */}
            <div className="space-y-2">
                <h1 className="font-bold text-3xl text-foreground md:text-4xl">{tierListData.tier_list.name}</h1>
                {tierListData.tier_list.description && <p className="text-muted-foreground">{tierListData.tier_list.description}</p>}
            </div>

            {/* Info Banner */}
            <Card className="border-primary/20 bg-primary/5">
                <div className="flex items-start gap-3 p-4">
                    <Info className="mt-0.5 h-5 w-5 shrink-0 text-primary" />
                    <div className="space-y-1">
                        <p className="font-medium text-foreground text-sm">Tier List Information</p>
                        <p className="text-muted-foreground text-sm">Operators are ranked based on their overall performance, versatility, and impact in various game modes. Rankings may vary based on team composition and specific use cases.</p>
                    </div>
                </div>
            </Card>

            {/* Tier List */}
            <div className="space-y-4">
                {tierListData.tiers.map((tier) => {
                    const operators = tier.placements
                        .sort((a, b) => a.sub_order - b.sub_order)
                        .map((placement) => operatorsData[placement.operator_id])
                        .filter((op): op is OperatorFromList => op !== undefined);

                    return <TierRow hoveredOperator={hoveredOperator} isGrayscaleActive={isGrayscaleActive} key={tier.id} onOperatorHover={handleOperatorHover} operators={operators} tier={tier} />;
                })}
            </div>

            {/* Last Updated */}
            <div className="flex items-center justify-center pt-4 text-muted-foreground text-sm">
                <p>
                    Last updated:{" "}
                    {new Date(tierListData.tier_list.updated_at).toLocaleDateString("en-US", {
                        year: "numeric",
                        month: "long",
                        day: "numeric",
                    })}
                </p>
            </div>
        </div>
    );
}
