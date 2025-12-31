"use client";
import { Info } from "lucide-react";
import { useState } from "react";
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from "~/components/ui/shadcn/dialog";
import { getContrastTextColor } from "~/lib/utils";
import type { Tier } from "~/types/api/impl/tier-list";
import type { OperatorFromList } from "~/types/api/operators";
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
    const [isDialogOpen, setIsDialogOpen] = useState(false);

    return (
        <>
            <div className="flex flex-col gap-2 overflow-hidden rounded-lg border border-border bg-card/50 md:flex-row">
                {/* Tier Label */}
                <button
                    aria-label={tier.description ? `View details for ${tier.name} tier` : tier.name}
                    className="flex shrink-0 items-center justify-center px-6 py-4 font-bold text-xl leading-tight transition-opacity hover:opacity-90 focus:outline-none focus:ring-2 focus:ring-primary focus:ring-offset-2 disabled:cursor-default disabled:opacity-100 md:w-40 md:flex-col md:text-2xl md:leading-snug"
                    disabled={!tier.description}
                    onClick={() => tier.description && setIsDialogOpen(true)}
                    style={{
                        backgroundColor: tierColor,
                        color: textColor,
                    }}
                    type="button"
                >
                    <span className="wrap-break-word hyphens-auto text-center drop-shadow-md">{tier.name}</span>
                    {tier.description && <Info aria-hidden="true" className="mt-1 h-4 w-4 opacity-60" />}
                </button>

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

            {/* Tier Description Dialog */}
            <Dialog onOpenChange={setIsDialogOpen} open={isDialogOpen}>
                <DialogContent className="max-w-md">
                    <DialogHeader>
                        <DialogTitle className="flex items-center gap-2">
                            <div
                                className="flex h-8 w-8 items-center justify-center rounded font-bold text-sm"
                                style={{
                                    backgroundColor: tierColor,
                                    color: textColor,
                                }}
                            >
                                {tier.name}
                            </div>
                            Tier Information
                        </DialogTitle>
                        <DialogDescription>Details about the {tier.name} tier</DialogDescription>
                    </DialogHeader>
                    <div className="space-y-4">
                        <div className="rounded-md border border-border/50 bg-muted/30 p-4">
                            <p className="text-foreground text-sm leading-relaxed">{tier.description}</p>
                        </div>
                    </div>
                </DialogContent>
            </Dialog>
        </>
    );
}
