"use client";
import type { Tier } from "~/types/api/impl/tier-list";
import type { OperatorFromList } from "~/types/api/operators";
import { getContrastTextColor } from "~/lib/utils";
import { TierOperatorCard } from "./operator-card";

interface TierRowProps {
    tier: Tier;
    operators: OperatorFromList[];
    hoveredOperator: string | null;
    isGrayscaleActive: boolean;
    onOperatorHover: (operatorId: string | null, isHovered: boolean) => void;
}

// Default tier colors if not specified in the database
const DEFAULT_TIER_COLORS: Record<string, string> = {
    "S+": "#ff7f7f",
    S: "#ff9f7f",
    "A+": "#ffbf7f",
    A: "#ffdf7f",
    "B+": "#ffff7f",
    B: "#bfff7f",
    C: "#7fff7f",
    D: "#7fffff",
};

export function TierRow({ tier, operators, hoveredOperator, isGrayscaleActive, onOperatorHover }: TierRowProps) {
    // Use tier color from database or fall back to default colors
    const tierColor = tier.color || DEFAULT_TIER_COLORS[tier.name] || "#888888";
    // Calculate optimal text color based on background luminance
    const textColor = getContrastTextColor(tierColor);

    return (
        <div className="flex flex-col gap-2 overflow-hidden rounded-lg border border-border bg-card/50 md:flex-row">
            {/* Tier Label */}
            <div
                className="flex shrink-0 items-center justify-center px-6 py-4 font-bold text-2xl md:w-24 md:flex-col md:text-3xl"
                style={{
                    backgroundColor: tierColor,
                    color: textColor,
                }}
            >
                <span className="drop-shadow-md">{tier.name}</span>
                {tier.description && <span className="mt-1 hidden text-center font-normal text-xs md:block" style={{ color: `${textColor}cc` }}>{tier.description}</span>}
            </div>

            {/* Operators Grid */}
            <div className="min-w-0 flex-1 p-3">
                {operators.length > 0 ? (
                    <div className="grid grid-cols-4 gap-2 sm:grid-cols-6 md:grid-cols-7 lg:grid-cols-8 xl:grid-cols-10">
                        {operators.map((operator) => {
                            const operatorId = operator.id ?? "";
                            const isCurrentlyHovered = hoveredOperator === operatorId;
                            const shouldGrayscale = isGrayscaleActive && !isCurrentlyHovered;

                            return <TierOperatorCard isHovered={isCurrentlyHovered} key={operatorId} onHoverChange={(isHovered) => onOperatorHover(operatorId, isHovered)} operator={operator} shouldGrayscale={shouldGrayscale} />;
                        })}
                    </div>
                ) : (
                    <div className="flex h-24 items-center justify-center text-muted-foreground text-sm">No operators in this tier</div>
                )}
            </div>
        </div>
    );
}
